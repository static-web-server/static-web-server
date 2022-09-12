// Static File handler
// -> Most of the file is borrowed from https://github.com/seanmonstar/warp/blob/master/src/filters/fs.rs

use bytes::{Bytes, BytesMut};
use futures_util::future::Either;
use futures_util::{future, ready, stream, FutureExt, Stream, StreamExt, TryFutureExt};
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt, HeaderValue,
    IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range,
};
use humansize::{file_size_opts, FileSize};
use hyper::{Body, Method, Response, StatusCode};
use mime_guess::mime;
use percent_encoding::percent_decode_str;
use std::cmp::Ordering;
use std::fs::Metadata;
use std::future::Future;
use std::io;
use std::ops::Bound;
use std::path::{Component, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{cmp, path::Path};
use tokio::fs::File as TkFile;
use tokio::io::AsyncSeekExt;
use tokio_util::io::poll_read_buf;

use crate::Result;

/// Arc `PathBuf` reference wrapper since Arc<PathBuf> doesn't implement AsRef<Path>.
#[derive(Clone, Debug)]
pub struct ArcPath(pub Arc<PathBuf>);

impl AsRef<Path> for ArcPath {
    fn as_ref(&self) -> &Path {
        (*self.0).as_ref()
    }
}

/// Defines all options needed by the static-files handler.
pub struct HandleOpts<'a> {
    pub method: &'a Method,
    pub headers: &'a HeaderMap<HeaderValue>,
    pub base_path: &'a PathBuf,
    pub uri_path: &'a str,
    pub uri_query: Option<&'a str>,
    pub dir_listing: bool,
    pub dir_listing_order: u8,
    pub redirect_trailing_slash: bool,
}

/// Entry point to handle incoming requests which map to specific files
/// on file system and return a file response.
pub async fn handle<'a>(opts: &HandleOpts<'a>) -> Result<Response<Body>, StatusCode> {
    let method = opts.method;
    let uri_path = opts.uri_path;

    // Check for disallowed HTTP methods and reject request accordently
    if !(method == Method::GET || method == Method::HEAD || method == Method::OPTIONS) {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    let base = Arc::new(opts.base_path.into());
    let (filepath, meta, auto_index) = path_from_tail(base, uri_path).await?;

    // NOTE: `auto_index` appends an `index.html` to an `uri_path` of kind directory only.

    // Check for a trailing slash on the current directory path
    // and redirect if that path doesn't end with the slash char
    if opts.redirect_trailing_slash && auto_index && !uri_path.ends_with('/') {
        let uri = [uri_path, "/"].concat();
        let loc = match HeaderValue::from_str(uri.as_str()) {
            Ok(val) => val,
            Err(err) => {
                tracing::error!("invalid header value from current uri: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let mut resp = Response::new(Body::empty());
        resp.headers_mut().insert(hyper::header::LOCATION, loc);
        *resp.status_mut() = StatusCode::PERMANENT_REDIRECT;
        tracing::trace!("uri doesn't end with a slash so redirecting permanently");
        return Ok(resp);
    }

    // Respond the permitted communication options
    if method == Method::OPTIONS {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::NO_CONTENT;
        resp.headers_mut()
            .typed_insert(headers::Allow::from_iter(vec![
                Method::OPTIONS,
                Method::HEAD,
                Method::GET,
            ]));
        resp.headers_mut().typed_insert(AcceptRanges::bytes());
        return Ok(resp);
    }

    // Directory listing
    // 1. Check if "directory listing" feature is enabled
    // if current path is a valid directory and
    // if it does not contain an `index.html` file (if a proper auto index is generated)
    if opts.dir_listing && auto_index && !filepath.as_ref().exists() {
        return directory_listing(
            method,
            uri_path,
            opts.uri_query,
            filepath.as_ref(),
            opts.dir_listing_order,
        )
        .await;
    }

    file_reply(opts.headers, (filepath, &meta, auto_index)).await
}

/// Convert an incoming uri into a valid and sanitized path then returns a tuple
// with the path as well as its file metadata and an auto index check if it's a directory.
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
                tracing::debug!("file not found: {} {:?}", buf.display(), err);
                Err(StatusCode::NOT_FOUND)
            }
        }
    })
}

/// Provides directory listing support for the current request.
/// Note that this function highly depends on `path_from_tail()` function
/// which must be called first. See `handle()` for more details.
fn directory_listing<'a>(
    method: &'a Method,
    current_path: &'a str,
    uri_query: Option<&'a str>,
    filepath: &'a Path,
    dir_listing_order: u8,
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send + 'a {
    let is_head = method == Method::HEAD;

    // Note: it's safe to call `parent()` here since `filepath`
    // value always refer to a path with file ending and under
    // a root directory boundary.
    // See `path_from_tail()` function which sanitizes the requested
    // path before to be delegated here.
    let parent = filepath.parent().unwrap_or(filepath);

    tokio::fs::read_dir(parent).then(move |res| match res {
        Ok(entries) => Either::Left(async move {
            match read_directory_entries(
                entries,
                current_path,
                uri_query,
                is_head,
                dir_listing_order,
            )
            .await
            {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    tracing::error!(
                        "error during directory entries reading (path={:?}): {} ",
                        parent.display(),
                        err
                    );
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }),
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!("entry file not found: {:?}", filepath.display());
                    StatusCode::NOT_FOUND
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::warn!("entry file permission denied: {:?}", filepath.display());
                    StatusCode::FORBIDDEN
                }
                _ => {
                    tracing::error!(
                        "directory entries error (filepath={:?}): {} ",
                        filepath.display(),
                        err
                    );
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            };
            Either::Right(future::err(status))
        }
    })
}

/// It reads the current directory entries and create an index page content.
/// Otherwise it returns a status error.
async fn read_directory_entries(
    mut entries: tokio::fs::ReadDir,
    base_path: &str,
    uri_query: Option<&str>,
    is_head: bool,
    mut dir_listing_order: u8,
) -> Result<Response<Body>> {
    let mut dirs_count: usize = 0;
    let mut files_count: usize = 0;
    let mut files_found: Vec<(String, String, u64, Option<String>)> = vec![];

    while let Some(entry) = entries.next_entry().await? {
        let meta = entry.metadata().await?;

        let mut name = entry
            .file_name()
            .into_string()
            .map_err(|err| anyhow::anyhow!(err.into_string().unwrap_or_default()))?;

        let mut filesize = 0_u64;

        if meta.is_dir() {
            name += "/";
            dirs_count += 1;
        } else if meta.is_file() {
            filesize = meta.len();
            files_count += 1;
        } else if meta.file_type().is_symlink() {
            let m = tokio::fs::symlink_metadata(entry.path().canonicalize()?).await?;
            if m.is_dir() {
                name += "/";
                dirs_count += 1;
            } else {
                filesize = meta.len();
                files_count += 1;
            }
        } else {
            continue;
        }

        let mut uri = None;
        // NOTE: Use relative paths by default and absolute ones only
        // when "redirect trailing slash" feature is disabled and
        // `base_path` doesn't end with a slash char
        if !base_path.ends_with('/') {
            let base_path = Path::new(base_path);
            let parent_dir = base_path.parent().unwrap_or(base_path);
            let mut base_dir = base_path;
            if base_path != parent_dir {
                base_dir = base_path.strip_prefix(parent_dir)?;
            }
            let mut base_str = String::new();
            if !base_dir.starts_with("/") {
                let base_dir = base_dir.to_str().unwrap_or_default();
                if !base_dir.is_empty() {
                    base_str.push_str(base_dir);
                }
                base_str.push('/');
            }
            base_str.push_str(&name);
            uri = Some(base_str);
        }

        let modified = match parse_last_modified(meta.modified()?) {
            Ok(tm) => tm.to_local().strftime("%F %T")?.to_string(),
            Err(err) => {
                tracing::error!("error determining file last modified: {:?}", err);
                String::from("-")
            }
        };
        files_found.push((name, modified, filesize, uri));
    }

    // Check the query uri for a sorting type. E.g https://blah/?sort=5
    if let Some(q) = uri_query {
        let mut parts = form_urlencoded::parse(q.as_bytes());
        if parts.count() > 0 {
            // NOTE: we just pick up the first value (pairs)
            if let Some(sort) = parts.next() {
                if sort.0 == "sort" && !sort.1.trim().is_empty() {
                    match sort.1.parse::<u8>() {
                        Ok(n) => dir_listing_order = n,
                        Err(e) => {
                            tracing::debug!("sorting: query value to u8 error: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    // Default sorting type values
    let mut sort_name = "0".to_owned();
    let mut sort_last_modified = "2".to_owned();
    let mut sort_size = "4".to_owned();

    files_found.sort_by(|a, b| match dir_listing_order {
        // Name (asc, desc)
        0 => {
            sort_name = "1".to_owned();
            a.0.to_lowercase().cmp(&b.0.to_lowercase())
        }
        1 => {
            sort_name = "0".to_owned();
            b.0.to_lowercase().cmp(&a.0.to_lowercase())
        }

        // Modified (asc, desc)
        2 => {
            sort_last_modified = "3".to_owned();
            a.1.cmp(&b.1)
        }
        3 => {
            sort_last_modified = "2".to_owned();
            b.1.cmp(&a.1)
        }

        // File size (asc, desc)
        4 => {
            sort_size = "5".to_owned();
            a.2.cmp(&b.2)
        }
        5 => {
            sort_size = "4".to_owned();
            b.2.cmp(&a.2)
        }

        // Unordered
        _ => Ordering::Equal,
    });

    // Prepare table header with sorting support
    let table_header = format!(
        r#"<thead><tr><th><a href="?sort={}">Name</a></th><th style="width:160px;"><a href="?sort={}">Last modified</a></th><th style="width:120px;text-align:right;"><a href="?sort={}">Size</a></th></tr></thead>"#,
        sort_name, sort_last_modified, sort_size,
    );

    let mut entries_str = String::new();
    if base_path != "/" {
        entries_str = String::from(r#"<tr><td colspan="3"><a href="../">../</a></td></tr>"#);
    }

    for f in files_found {
        let (name, modified, filesize, uri) = f;
        let mut filesize_str = filesize
            .file_size(file_size_opts::DECIMAL)
            .map_err(anyhow::Error::msg)?;

        if filesize == 0 {
            filesize_str = String::from("-");
        }

        let entry_uri = uri.unwrap_or_else(|| name.to_owned());

        entries_str = format!(
            "{}<tr><td><a href=\"{}\">{}</a></td><td>{}</td><td align=\"right\">{}</td></tr>",
            entries_str, entry_uri, name, modified, filesize_str
        );
    }

    let current_path = percent_decode_str(base_path).decode_utf8()?.to_owned();
    let dirs_str = if dirs_count == 1 {
        "directory"
    } else {
        "directories"
    };
    let summary_str = format!(
        "<div>{} {}, {} {}</div>",
        dirs_count, dirs_str, files_count, "file(s)"
    );
    let style_str = r#"<style>html{background-color:#fff;-moz-osx-font-smoothing:grayscale;-webkit-font-smoothing:antialiased;min-width:20rem;text-rendering:optimizeLegibility;-webkit-text-size-adjust:100%;-moz-text-size-adjust:100%;text-size-adjust:100%}body{padding:1rem;font-family:Consolas,'Liberation Mono',Menlo,monospace;font-size:.875rem;max-width:70rem;margin:0 auto;color:#4a4a4a;font-weight:400;line-height:1.5}h1{margin:0;padding:0;font-size:1.375rem;line-height:1.25;margin-bottom:0.5rem;}table{width:100%;border-spacing: 0;}table th,table td{padding:.2rem .5rem;white-space:nowrap;vertical-align:top}table th a,table td a{display:inline-block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:95%;vertical-align:top}table tr:hover td{background-color:#f5f5f5}footer{padding-top:0.5rem}table tr th{text-align:left;}</style>"#;
    let footer_str = r#"<footer>Powered by <a target="_blank" href="https://github.com/joseluisq/static-web-server">static-web-server</a> | MIT &amp; Apache 2.0</footer>"#;
    let page_str = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Index of {}</title>{}</head><body><h1>Index of {}</h1>{}<hr><table>{}{}</table><hr>{}</body></html>", current_path, style_str, current_path, summary_str, table_header, entries_str, footer_str
    );

    let mut resp = Response::new(Body::empty());
    resp.headers_mut()
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
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
fn file_reply<'a>(
    headers: &'a HeaderMap<HeaderValue>,
    res: (ArcPath, &'a Metadata, bool),
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send + 'a {
    let (path, meta, auto_index) = res;
    let conditionals = get_conditional_headers(headers);
    TkFile::open(path.clone()).then(move |res| match res {
        Ok(file) => Either::Left(file_conditional(file, path, meta, auto_index, conditionals)),
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
    let path_decoded = match percent_decode_str(tail.trim_start_matches('/')).decode_utf8() {
        Ok(p) => p,
        Err(err) => {
            tracing::debug!("dir: failed to decode route={:?}: {:?}", tail, err);
            return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
    };

    let path_decoded = Path::new(&*path_decoded);
    let mut full_path = base.as_ref().to_path_buf();
    tracing::trace!("dir? base={:?}, route={:?}", full_path, path_decoded);

    for component in path_decoded.components() {
        match component {
            Component::Normal(comp) => {
                // Protect against paths like `/foo/c:/bar/baz`
                // https://github.com/seanmonstar/warp/issues/937
                if Path::new(&comp)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
                {
                    full_path.push(comp)
                } else {
                    tracing::debug!("dir: skipping segment with invalid prefix");
                }
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                tracing::debug!(
                    "dir: skipping segment containing invalid prefix, dots or backslashes"
                );
            }
        }
    }
    Ok(full_path)
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

async fn file_conditional(
    file: TkFile,
    path: ArcPath,
    meta: &Metadata,
    auto_index: bool,
    conditionals: Conditionals,
) -> Result<Response<Body>, StatusCode> {
    if auto_index {
        match file.metadata().await {
            Ok(meta) => Ok(response_body(file, &meta, path, conditionals)),
            Err(err) => {
                tracing::debug!("file metadata error: {}", err);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Ok(response_body(file, meta, path, conditionals))
    }
}

fn response_body(
    file: TkFile,
    meta: &Metadata,
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
                    let buf_size = optimal_buf_size(meta);
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

                    let mime = mime_guess::from_path(path).first_or_octet_stream();

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

    let res = range
        .iter()
        .map(|(start, end)| {
            let (start, end) = match (start, end) {
                (Bound::Unbounded, Bound::Unbounded) => (0, max_len),
                (Bound::Included(a), Bound::Included(b)) => {
                    // For the special case where b == the file size
                    (a, if b == max_len { b } else { b + 1 })
                }
                (Bound::Included(a), Bound::Unbounded) => (a, max_len),
                (Bound::Unbounded, Bound::Included(b)) => {
                    if b > max_len {
                        return Err(BadRange);
                    }
                    (max_len - b, max_len)
                }
                _ => unreachable!(),
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
    res
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
    use std::path::PathBuf;

    fn root_dir() -> PathBuf {
        PathBuf::from("docker/public/")
    }

    #[test]
    fn test_sanitize_path() {
        const BASE_DIR: &str = "docker/public";

        assert_eq!(
            sanitize_path(BASE_DIR, "/index.html").unwrap(),
            root_dir().join("index.html")
        );

        // bad paths
        assert_eq!(
            sanitize_path(BASE_DIR, "/../foo.html").unwrap(),
            root_dir().join("foo.html"),
        );

        #[cfg(unix)]
        let expected_path = root_dir().join("C:\\/foo.html");
        #[cfg(windows)]
        let expected_path = PathBuf::from("docker/public/\\foo.html");
        assert_eq!(
            sanitize_path(BASE_DIR, "/C:\\/foo.html").unwrap(),
            expected_path
        );
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
