// Static File handler
// -> Most of the file is borrowed from https://github.com/seanmonstar/warp/blob/master/src/filters/fs.rs

use bytes::{Bytes, BytesMut};
use futures_util::future::Either;
use futures_util::{future, ready, stream, FutureExt, Stream, StreamExt};
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt, HeaderValue,
    IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range,
};
use http::header::CONTENT_LENGTH;
use hyper::{header::CONTENT_ENCODING, Body, Method, Response, StatusCode};
use percent_encoding::percent_decode_str;
use std::fs::Metadata;
use std::future::Future;
use std::io;
use std::ops::Bound;
use std::path::{Component, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::{cmp, path::Path};
use tokio::fs::File as TkFile;
use tokio::io::AsyncSeekExt;
use tokio_util::io::poll_read_buf;

use crate::{compression_static, directory_listing, Result};

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
    pub compression_static: bool,
}

/// Entry point to handle incoming requests which map to specific files
/// on file system and return a file response.
pub async fn handle<'a>(opts: &HandleOpts<'a>) -> Result<(Response<Body>, bool), StatusCode> {
    let method = opts.method;
    let uri_path = opts.uri_path;

    // Check for disallowed HTTP methods and reject request accordently
    if !(method == Method::GET || method == Method::HEAD || method == Method::OPTIONS) {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    let base = Arc::<PathBuf>::new(opts.base_path.into());
    let file_path = sanitize_path(base.as_ref(), uri_path)?;

    let (file_path, meta, is_dir, precompressed_variant) =
        composed_file_metadata(file_path, opts.headers, opts.compression_static).await?;

    // `is_precompressed` relates to `opts.compression_static` value
    let is_precompressed = precompressed_variant.is_some();

    // NOTE: `is_dir` means an "auto index" for the current directory request

    // Check for a trailing slash on the current directory path
    // and redirect if that path doesn't end with the slash char
    if opts.redirect_trailing_slash && is_dir && !uri_path.ends_with('/') {
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
        return Ok((resp, is_precompressed));
    }

    // Respond with the permitted communication options
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
        return Ok((resp, is_precompressed));
    }

    // Directory listing
    // Check if "directory listing" feature is enabled,
    // if current path is a valid directory and
    // if it does not contain an `index.html` file (if a proper auto index is generated)
    if opts.dir_listing && is_dir && !file_path.as_ref().exists() {
        let resp = directory_listing::auto_index(
            method,
            uri_path,
            opts.uri_query,
            file_path.as_ref(),
            opts.dir_listing_order,
        )
        .await?;

        return Ok((resp, is_precompressed));
    }

    // Check for a pre-compressed file variant if present under the `opts.compression_static` context
    if let Some(meta_precompressed) = precompressed_variant {
        let (path_precomp, meta_precomp, precomp_ext) = meta_precompressed;
        let mut resp = file_reply(
            opts.headers,
            (file_path, &meta),
            Some((path_precomp, &meta_precomp)),
        )
        .await?;

        // Prepare corresponding headers to let know how to decode the payload
        resp.headers_mut().remove(CONTENT_LENGTH);
        resp.headers_mut()
            .insert(CONTENT_ENCODING, precomp_ext.parse().unwrap());

        return Ok((resp, is_precompressed));
    }

    let resp = file_reply(opts.headers, (file_path, &meta), None).await?;

    Ok((resp, is_precompressed))
}

/// Returns the final composed metadata information (tuple) containing
/// the Arc `PathBuf` reference wrapper for the current `file_path` with its file metadata
/// as well as its optional pre-compressed variant.
async fn composed_file_metadata(
    mut file_path: PathBuf,
    headers: &HeaderMap<HeaderValue>,
    compression_static: bool,
) -> Result<(ArcPath, Metadata, bool, Option<(ArcPath, Metadata, &str)>), StatusCode> {
    tracing::debug!("getting metadata for file {}", file_path.display());

    let (mut meta, is_dir) = file_metadata(file_path.as_ref()).await?;
    let mut precompressed = None;

    if is_dir {
        // Append a HTML index page by default if it's a directory path (`autoindex`)
        tracing::debug!("dir: appending an index.html to the directory path");
        file_path.push("index.html");

        // If file exists then overwrite the `meta`.
        // Also noting that it's still a directory request.
        if let Ok(meta_res) = file_metadata(file_path.as_ref()).await {
            (meta, _) = meta_res
        }
    }

    // Pre-compressed variant check for the given file path
    if compression_static {
        precompressed = compression_static::precompressed_variant(file_path.clone(), headers).await;
    }

    Ok((ArcPath(Arc::new(file_path)), meta, is_dir, precompressed))
}

/// Try to find the file system metadata for the given file path.
pub async fn file_metadata(file_path: &Path) -> Result<(Metadata, bool), StatusCode> {
    match tokio::fs::metadata(file_path).await {
        Ok(meta) => {
            let is_dir = meta.is_dir();
            tracing::trace!("file found: {:?}", file_path);
            Ok((meta, is_dir))
        }
        Err(err) => {
            tracing::debug!("file not found: {:?} {:?}", file_path, err);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Reply with the corresponding file content.
fn file_reply<'a>(
    headers: &'a HeaderMap<HeaderValue>,
    meta_composed: (ArcPath, &'a Metadata),
    meta_precompressed: Option<(ArcPath, &'a Metadata)>,
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send + 'a {
    let conditionals = get_conditional_headers(headers);

    let mut path_comp = None;
    let mut meta_comp = None;
    if let Some((path_precomp, meta_precomp)) = meta_precompressed {
        path_comp = Some(path_precomp);
        meta_comp = Some(meta_precomp);
    };

    let (path, meta) = meta_composed;
    let path_comp = path_comp.unwrap_or_else(|| path.clone());

    TkFile::open(path_comp).then(move |res| match res {
        Ok(file) => Either::Left(file_conditional(
            file,
            path,
            meta_comp.unwrap_or(meta),
            conditionals,
        )),
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!(
                        "file can't be opened or not found: {:?}",
                        path.as_ref().display()
                    );
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

/// Sanitizes a base/tail paths and then it returns an unified one.
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
    tracing::trace!("dir: base={:?}, route={:?}", full_path, path_decoded);

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
    conditionals: Conditionals,
) -> Result<Response<Body>, StatusCode> {
    Ok(response_body(file, meta, path, conditionals))
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
