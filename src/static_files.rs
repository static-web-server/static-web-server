// Static File handler
// -> Most of the file is borrowed from https://github.com/seanmonstar/warp/blob/master/src/filters/fs.rs

use bytes::{Bytes, BytesMut};
use futures::future::Either;
use futures::{future, ready, stream, FutureExt, Stream, StreamExt, TryFutureExt};
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt, HeaderValue,
    IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range,
};
use hyper::{Body, Response, StatusCode};
use percent_encoding::percent_decode_str;
use std::fs::Metadata;
use std::future::Future;
use std::io;
use std::ops::Bound;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::{cmp, path::Path};
use tokio::fs::File as TkFile;
use tokio::io::AsyncSeekExt;
use tokio_util::io::poll_read_buf;

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
    base: &Path,
    headers: &HeaderMap<HeaderValue>,
    uri_path: &str,
) -> Result<Response<Body>, StatusCode> {
    let base = Arc::new(base.into());
    let res = path_from_tail(base, uri_path).await?;
    file_reply(headers, res).await
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

/// Reply with a file content.
fn file_reply(
    headers: &HeaderMap<HeaderValue>,
    res: (ArcPath, Metadata, bool),
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send {
    // TODO: directory listing

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
