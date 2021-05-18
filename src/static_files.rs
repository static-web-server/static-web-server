// Static File handler
// -> Most of the file is borrowed from https://github.com/seanmonstar/warp/blob/master/src/filters/fs.rs

use bytes::{Bytes, BytesMut};
use futures::future::Either;
use futures::{future, ready, stream, FutureExt, Stream, StreamExt, TryFutureExt};
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt, HeaderValue,
    IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range,
};
use humansize::{file_size_opts, FileSize};
use hyper::{Body, Method, Response, StatusCode};
use percent_encoding::percent_decode_str;
use std::fs::Metadata;
use std::future::Future;
use std::io;
use std::ops::Bound;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{cmp, path::Path};
use tokio::fs::File as TkFile;
use tokio::io::AsyncSeekExt;
use tokio_util::io::poll_read_buf;

use crate::error::Result;

/// A small Arch `PathBuf` wrapper since Arc<PathBuf> doesn't implement AsRef<Path>.
#[derive(Clone, Debug)]
pub struct ArcPath(pub Arc<PathBuf>);

impl AsRef<Path> for ArcPath {
    fn as_ref(&self) -> &Path {
        (*self.0).as_ref()
    }
}

/// Entry point to handle web server requests which map to specific files
/// on file system and return a file response.
pub async fn handle_request(
    method: &Method,
    headers: &HeaderMap<HeaderValue>,
    base: &Path,
    uri_path: &str,
    dir_listing: bool,
) -> Result<Response<Body>, StatusCode> {
    // Reject requests for non HEAD or GET methods
    if !(method == Method::HEAD || method == Method::GET) {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    let base = Arc::new(base.into());
    let (path, meta, auto_index) = path_from_tail(base, uri_path).await?;

    // Directory listing
    // 1. Check if "directory listing" feature is enabled,
    // if current path is a valid directory and
    // if it does not contain an `index.html` file
    if dir_listing && auto_index && !path.as_ref().exists() {
        // Redirect if current path does not end with a slash
        let current_path = uri_path;
        if !current_path.ends_with('/') {
            let uri = [current_path, "/"].concat();
            let loc = HeaderValue::from_str(uri.as_str()).unwrap();
            let mut resp = Response::new(Body::empty());

            resp.headers_mut().insert(hyper::header::LOCATION, loc);
            *resp.status_mut() = StatusCode::PERMANENT_REDIRECT;

            return Ok(resp);
        }

        return directory_listing(method, (current_path.to_string(), path)).await;
    }

    file_reply(headers, (path, meta, auto_index)).await
}

fn path_from_tail(
    base: Arc<PathBuf>,
    tail: &str,
) -> impl Future<Output = Result<(ArcPath, Metadata, bool), StatusCode>> + Send {
    future::ready(sanitize_path(base.as_ref(), tail)).and_then(|mut buf| async {
        match tokio::fs::metadata(&buf).await {
            Ok(meta) => {
                let mut auto_index = false;
                if meta.is_dir() {
                    tracing::debug!("dir: appending index.html to directory path");
                    buf.push("index.html");
                    auto_index = true;
                }
                tracing::trace!("dir: {:?}", buf);
                Ok((ArcPath(Arc::new(buf)), meta, auto_index))
            }
            Err(err) => {
                tracing::debug!("file not found: {:?}", err);
                Err(StatusCode::NOT_FOUND)
            }
        }
    })
}

fn directory_listing(
    method: &Method,
    res: (String, ArcPath),
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send {
    let (current_path, path) = res;
    let is_head = method == Method::HEAD;
    let parent = path.as_ref().parent().unwrap();
    let parent = PathBuf::from(parent);

    tokio::fs::read_dir(parent).then(move |res| match res {
        Ok(entries) => Either::Left(async move {
            match read_directory_entries(entries, &current_path, is_head).await {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    tracing::error!(
                        "error during directory entries reading (path={:?}): {} ",
                        path.as_ref().parent().unwrap().display(),
                        err
                    );
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }),
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!("entry file not found: {:?}", path.as_ref().display());
                    StatusCode::NOT_FOUND
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::warn!(
                        "entry file permission denied: {:?}",
                        path.as_ref().display()
                    );
                    StatusCode::FORBIDDEN
                }
                _ => {
                    tracing::error!(
                        "directory entries error (path={:?}): {} ",
                        path.as_ref().display(),
                        err
                    );
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            };
            Either::Right(future::err(status))
        }
    })
}

// It reads current directory entries and create the index page content. Otherwise returns a status error.
async fn read_directory_entries(
    mut entries: tokio::fs::ReadDir,
    base_path: &str,
    is_head: bool,
) -> Result<Response<Body>> {
    let mut entries_str = String::new();
    if base_path != "/" {
        entries_str = String::from(r#"<tr><td colspan="3"><a href="../">../</a></td></tr>"#);
    }
    while let Some(entry) = entries.next_entry().await? {
        let meta = entry.metadata().await?;
        let mut filesize = meta
            .len()
            .file_size(file_size_opts::DECIMAL)
            .map_err(anyhow::Error::msg)?;

        let mut name = entry
            .file_name()
            .into_string()
            .map_err(|err| anyhow::anyhow!(err.into_string().unwrap_or_default()))?;

        if meta.is_dir() {
            name = format!("{}/", name);
            filesize = String::from("-")
        }

        let uri = format!("{}{}", base_path, name);
        let modified = parse_last_modified(meta.modified()?).unwrap();

        entries_str = format!(
            "{}<tr><td><a href=\"{}\" title=\"{}\">{}</a></td><td style=\"width: 160px;\">{}</td><td align=\"right\">{}</td></tr>",
            entries_str,
            uri,
            name,
            name,
            modified.to_local().strftime("%F %T").unwrap(),
            filesize
        );
    }

    let current_path = percent_decode_str(&base_path).decode_utf8()?.to_string();

    let style_str = r#"<style>html{background-color:#fff;-moz-osx-font-smoothing:grayscale;-webkit-font-smoothing:antialiased;min-width:20rem;text-rendering:optimizeLegibility;-webkit-text-size-adjust:100%;-moz-text-size-adjust:100%;text-size-adjust:100%}body{padding:1rem;font-family:Consolas,'Liberation Mono',Menlo,monospace;font-size:.875rem;max-width:70rem;margin:0 auto;color:#4a4a4a;font-weight:400;line-height:1.5}h1{margin:0;padding:0 .5rem;font-size:1.375rem;}table{width:100%}table td{padding:.2rem .5rem;white-space:nowrap;vertical-align:top}table td a{display:inline-block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:95%;vertical-align:top}table tr:hover td{background-color:#f5f5f5}footer{padding-top:0.5rem}</style>"#;
    let footer_str = r#"<footer>Powered by <a target="_blank" href="https://git.io/static-web-server">static-web-server</a> | MIT &amp; Apache 2.0</footer>"#;
    let page_str = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Index of {}</title>{}</head><body><h1>Index of {}</h1><table><tr><th colspan=\"3\"><hr></th></tr>{}<tr><th colspan=\"3\"><hr></th></tr></table>{}</body></html>", current_path, style_str, current_path, entries_str, footer_str
    );

    let mut resp = Response::new(Body::empty());
    resp.headers_mut()
        .typed_insert(ContentLength(page_str.len() as u64));

    // We skip the body for HEAD requests
    if is_head {
        return Ok(resp);
    }

    *resp.body_mut() = Body::from(page_str);

    Ok(resp)
}

fn parse_last_modified(modified: SystemTime) -> Result<time::Tm, Box<dyn std::error::Error>> {
    let since_epoch = modified.duration_since(UNIX_EPOCH)?;
    // HTTP times don't have nanosecond precision, so we truncate
    // the modification time.
    // Converting to i64 should be safe until we get beyond the
    // planned lifetime of the universe
    //
    // TODO: Investigate how to write a test for this. Changing
    // the modification time of a file with greater than second
    // precision appears to be something that only is possible to
    // do on Linux.
    let ts = time::Timespec::new(since_epoch.as_secs() as i64, 0);
    Ok(time::at_utc(ts))
}

/// Reply with a file content.
fn file_reply(
    headers: &HeaderMap<HeaderValue>,
    res: (ArcPath, Metadata, bool),
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send {
    let (path, meta, auto_index) = res;
    let conditionals = get_conditional_headers(headers);
    TkFile::open(path.clone()).then(move |res| match res {
        Ok(f) => Either::Left(file_conditional(f, path, meta, auto_index, conditionals)),
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!("file not found: {:?}", path.as_ref().display());
                    StatusCode::NOT_FOUND
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::warn!("file permission denied: {:?}", path.as_ref().display());
                    StatusCode::FORBIDDEN
                }
                _ => {
                    tracing::error!(
                        "file open error (path={:?}): {} ",
                        path.as_ref().display(),
                        err
                    );
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            };
            Either::Right(future::err(status))
        }
    })
}

fn get_conditional_headers(header_list: &HeaderMap<HeaderValue>) -> Conditionals {
    let if_modified_since = header_list.typed_get::<IfModifiedSince>();
    let if_unmodified_since = header_list.typed_get::<IfUnmodifiedSince>();
    let if_range = header_list.typed_get::<IfRange>();
    let range = header_list.typed_get::<Range>();

    Conditionals {
        if_modified_since,
        if_unmodified_since,
        if_range,
        range,
    }
}

fn sanitize_path(base: impl AsRef<Path>, tail: &str) -> Result<PathBuf, StatusCode> {
    let mut buf = PathBuf::from(base.as_ref());
    let p = match percent_decode_str(tail).decode_utf8() {
        Ok(p) => p,
        Err(err) => {
            tracing::debug!("dir: failed to decode route={:?}: {:?}", tail, err);
            return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
    };
    tracing::trace!("dir? base={:?}, route={:?}", base.as_ref(), p);
    for seg in p.split('/') {
        if seg.starts_with("..") {
            tracing::warn!("dir: rejecting segment starting with '..'");
            return Err(StatusCode::NOT_FOUND);
        } else if seg.contains('\\') {
            tracing::warn!("dir: rejecting segment containing with backslash (\\)");
            return Err(StatusCode::NOT_FOUND);
        } else {
            buf.push(seg);
        }
    }
    Ok(buf)
}

#[derive(Debug)]
struct Conditionals {
    if_modified_since: Option<IfModifiedSince>,
    if_unmodified_since: Option<IfUnmodifiedSince>,
    if_range: Option<IfRange>,
    range: Option<Range>,
}

enum Cond {
    NoBody(Response<Body>),
    WithBody(Option<Range>),
}

impl Conditionals {
    fn check(self, last_modified: Option<LastModified>) -> Cond {
        if let Some(since) = self.if_unmodified_since {
            let precondition = last_modified
                .map(|time| since.precondition_passes(time.into()))
                .unwrap_or(false);

            tracing::trace!(
                "if-unmodified-since? {:?} vs {:?} = {}",
                since,
                last_modified,
                precondition
            );
            if !precondition {
                let mut res = Response::new(Body::empty());
                *res.status_mut() = StatusCode::PRECONDITION_FAILED;
                return Cond::NoBody(res);
            }
        }

        if let Some(since) = self.if_modified_since {
            tracing::trace!(
                "if-modified-since? header = {:?}, file = {:?}",
                since,
                last_modified
            );
            let unmodified = last_modified
                .map(|time| !since.is_modified(time.into()))
                // no last_modified means its always modified
                .unwrap_or(false);
            if unmodified {
                let mut res = Response::new(Body::empty());
                *res.status_mut() = StatusCode::NOT_MODIFIED;
                return Cond::NoBody(res);
            }
        }

        if let Some(if_range) = self.if_range {
            tracing::trace!("if-range? {:?} vs {:?}", if_range, last_modified);
            let can_range = !if_range.is_modified(None, last_modified.as_ref());
            if !can_range {
                return Cond::WithBody(None);
            }
        }

        Cond::WithBody(self.range)
    }
}

fn file_conditional(
    f: TkFile,
    path: ArcPath,
    meta: Metadata,
    auto_index: bool,
    conditionals: Conditionals,
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send {
    file_metadata(f, meta, auto_index)
        .map_ok(|(file, meta)| response_body(file, meta, path, conditionals))
}

async fn file_metadata(
    f: TkFile,
    meta: Metadata,
    auto_index: bool,
) -> Result<(TkFile, Metadata), StatusCode> {
    if !auto_index {
        return Ok((f, meta));
    }
    match f.metadata().await {
        Ok(meta) => Ok((f, meta)),
        Err(err) => {
            tracing::debug!("file metadata error: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn response_body(
    file: TkFile,
    meta: Metadata,
    path: ArcPath,
    conditionals: Conditionals,
) -> Response<Body> {
    let mut len = meta.len();
    let modified = meta.modified().ok().map(LastModified::from);
    match conditionals.check(modified) {
        Cond::NoBody(resp) => resp,
        Cond::WithBody(range) => {
            bytes_range(range, len)
                .map(|(start, end)| {
                    let sub_len = end - start;
                    let buf_size = optimal_buf_size(&meta);
                    let stream = file_stream(file, buf_size, (start, end));
                    let body = Body::wrap_stream(stream);

                    let mut resp = Response::new(body);

                    if sub_len != len {
                        *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
                        resp.headers_mut().typed_insert(
                            ContentRange::bytes(start..end, len).expect("valid ContentRange"),
                        );

                        len = sub_len;
                    }

                    let mime = mime_guess::from_path(path.as_ref()).first_or_octet_stream();

                    resp.headers_mut().typed_insert(ContentLength(len));
                    resp.headers_mut().typed_insert(ContentType::from(mime));
                    resp.headers_mut().typed_insert(AcceptRanges::bytes());

                    if let Some(last_modified) = modified {
                        resp.headers_mut().typed_insert(last_modified);
                    }

                    resp
                })
                .unwrap_or_else(|BadRange| {
                    // bad byte range
                    let mut resp = Response::new(Body::empty());
                    *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                    resp.headers_mut()
                        .typed_insert(ContentRange::unsatisfied_bytes(len));
                    resp
                })
        }
    }
}

struct BadRange;

fn bytes_range(range: Option<Range>, max_len: u64) -> Result<(u64, u64), BadRange> {
    let range = if let Some(range) = range {
        range
    } else {
        return Ok((0, max_len));
    };

    let ret = range
        .iter()
        .map(|(start, end)| {
            let start = match start {
                Bound::Unbounded => 0,
                Bound::Included(s) => s,
                Bound::Excluded(s) => s + 1,
            };

            let end = match end {
                Bound::Unbounded => max_len,
                Bound::Included(s) => {
                    // For the special case where s == the file size
                    if s == max_len {
                        s
                    } else {
                        s + 1
                    }
                }
                Bound::Excluded(s) => s,
            };

            if start < end && end <= max_len {
                Ok((start, end))
            } else {
                tracing::trace!("unsatisfiable byte range: {}-{}/{}", start, end, max_len);
                Err(BadRange)
            }
        })
        .next()
        .unwrap_or(Ok((0, max_len)));
    ret
}

fn file_stream(
    mut file: TkFile,
    buf_size: usize,
    (start, end): (u64, u64),
) -> impl Stream<Item = Result<Bytes, io::Error>> + Send {
    let seek = async move {
        if start != 0 {
            file.seek(io::SeekFrom::Start(start)).await?;
        }
        Ok(file)
    };

    seek.into_stream()
        .map(move |result| {
            let mut buf = BytesMut::new();
            let mut len = end - start;
            let mut f = match result {
                Ok(f) => f,
                Err(f) => return Either::Left(stream::once(future::err(f))),
            };

            Either::Right(stream::poll_fn(move |cx| {
                if len == 0 {
                    return Poll::Ready(None);
                }
                reserve_at_least(&mut buf, buf_size);

                let n = match ready!(poll_read_buf(Pin::new(&mut f), cx, &mut buf)) {
                    Ok(n) => n as u64,
                    Err(err) => {
                        tracing::debug!("file read error: {}", err);
                        return Poll::Ready(Some(Err(err)));
                    }
                };

                if n == 0 {
                    tracing::debug!("file read found EOF before expected length");
                    return Poll::Ready(None);
                }

                let mut chunk = buf.split().freeze();
                if n > len {
                    chunk = chunk.split_to(len as usize);
                    len = 0;
                } else {
                    len -= n;
                }

                Poll::Ready(Some(Ok(chunk)))
            }))
        })
        .flatten()
}

fn reserve_at_least(buf: &mut BytesMut, cap: usize) {
    if buf.capacity() - buf.len() < cap {
        buf.reserve(cap);
    }
}

const DEFAULT_READ_BUF_SIZE: usize = 8_192;

fn optimal_buf_size(metadata: &Metadata) -> usize {
    let block_size = get_block_size(metadata);

    // If file length is smaller than block size, don't waste space
    // reserving a bigger-than-needed buffer.
    cmp::min(block_size as u64, metadata.len()) as usize
}

#[cfg(unix)]
fn get_block_size(metadata: &Metadata) -> usize {
    use std::os::unix::fs::MetadataExt;
    //TODO: blksize() returns u64, should handle bad cast...
    //(really, a block size bigger than 4gb?)

    // Use device blocksize unless it's really small.
    cmp::max(metadata.blksize() as usize, DEFAULT_READ_BUF_SIZE)
}

#[cfg(not(unix))]
fn get_block_size(_metadata: &Metadata) -> usize {
    DEFAULT_READ_BUF_SIZE
}

#[cfg(test)]
mod tests {
    use super::sanitize_path;
    use bytes::BytesMut;

    #[test]
    fn test_sanitize_path() {
        let base = "/var/www";

        fn p(s: &str) -> &::std::path::Path {
            s.as_ref()
        }

        assert_eq!(
            sanitize_path(base, "/foo.html").unwrap(),
            p("/var/www/foo.html")
        );

        // bad paths
        sanitize_path(base, "/../foo.html").expect_err("dot dot");

        sanitize_path(base, "/C:\\/foo.html").expect_err("C:\\");
    }

    #[test]
    fn test_reserve_at_least() {
        let mut buf = BytesMut::new();
        let cap = 8_192;

        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 0);

        super::reserve_at_least(&mut buf, cap);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), cap);
    }
}
