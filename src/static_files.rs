//! Static File handler
//!
//! Part of the file is borrowed and adapted at a convenience from
//! https://github.com/seanmonstar/warp/blob/master/src/filters/fs.rs

use bytes::{Bytes, BytesMut};
use futures_util::future::{Either, Future};
use futures_util::{future, Stream};
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt, HeaderValue,
    IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range,
};
use http::header::CONTENT_LENGTH;
use hyper::{header::CONTENT_ENCODING, Body, Method, Response, StatusCode};
use percent_encoding::percent_decode_str;
use std::fs::{File, Metadata};
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::ops::Bound;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::directory_listing::DirListFmt;
use crate::exts::http::{MethodExt, HTTP_SUPPORTED_METHODS};
use crate::exts::path::PathExt;
use crate::{compression_static, directory_listing, Result};

/// Defines all options needed by the static-files handler.
pub struct HandleOpts<'a> {
    pub method: &'a Method,
    pub headers: &'a HeaderMap<HeaderValue>,
    pub base_path: &'a PathBuf,
    pub uri_path: &'a str,
    pub uri_query: Option<&'a str>,
    pub dir_listing: bool,
    pub dir_listing_order: u8,
    pub dir_listing_format: &'a DirListFmt,
    pub redirect_trailing_slash: bool,
    pub compression_static: bool,
    pub ignore_hidden_files: bool,
}

/// Entry point to handle incoming requests which map to specific files
/// on file system and return a file response.
pub async fn handle<'a>(opts: &HandleOpts<'a>) -> Result<(Response<Body>, bool), StatusCode> {
    let method = opts.method;
    let uri_path = opts.uri_path;

    // Check if current HTTP method for incoming request is supported
    if !method.is_allowed() {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    let headers_opt = opts.headers;
    let compression_static_opt = opts.compression_static;

    let mut file_path = sanitize_path(opts.base_path, uri_path)?;

    let (file_path, meta, is_dir, precompressed_variant) =
        composed_file_metadata(&mut file_path, headers_opt, compression_static_opt).await?;

    // Check for a hidden file/directory (dotfile) and ignore it if feature enabled
    if opts.ignore_hidden_files && file_path.is_hidden() {
        return Err(StatusCode::NOT_FOUND);
    }

    // `is_precompressed` relates to `opts.compression_static` value
    let is_precompressed = precompressed_variant.is_some();

    if is_dir {
        // Check for a trailing slash on the current directory path
        // and redirect if that path doesn't end with the slash char
        if opts.redirect_trailing_slash && !uri_path.ends_with('/') {
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
        if method.is_options() {
            let mut resp = Response::new(Body::empty());
            *resp.status_mut() = StatusCode::NO_CONTENT;
            resp.headers_mut()
                .typed_insert(headers::Allow::from_iter(HTTP_SUPPORTED_METHODS.clone()));
            resp.headers_mut().typed_insert(AcceptRanges::bytes());

            return Ok((resp, is_precompressed));
        }

        // Directory listing
        // Check if "directory listing" feature is enabled,
        // if current path is a valid directory and
        // if it does not contain an `index.html` file (if a proper auto index is generated)
        if opts.dir_listing && !file_path.exists() {
            let resp = directory_listing::auto_index(
                method,
                uri_path,
                opts.uri_query,
                file_path.as_ref(),
                opts.dir_listing_order,
                opts.dir_listing_format,
                opts.ignore_hidden_files,
            )
            .await?;

            return Ok((resp, is_precompressed));
        }
    }

    // Check for a pre-compressed file variant if present under the `opts.compression_static` context
    if let Some(meta_precompressed) = precompressed_variant {
        let (path_precomp, precomp_ext) = meta_precompressed;

        let mut resp = file_reply(headers_opt, file_path, &meta, Some(path_precomp)).await?;

        // Prepare corresponding headers to let know how to decode the payload
        resp.headers_mut().remove(CONTENT_LENGTH);
        resp.headers_mut()
            .insert(CONTENT_ENCODING, precomp_ext.parse().unwrap());

        return Ok((resp, is_precompressed));
    }

    let resp = file_reply(headers_opt, file_path, &meta, None).await?;

    Ok((resp, is_precompressed))
}

/// Returns the result of trying to append `.html` to the file path.
/// * If the prefixed html path exists, it mutates the path to the prefixed one and returns the Metadata
/// * If the prefixed html path doesn't exist, it reverts the path to it's original value
fn prefix_file_html_metadata(file_path: &mut PathBuf) -> (&mut PathBuf, Option<Metadata>) {
    tracing::debug!("file: appending .html to the path");
    if let Some(filename) = file_path.file_name() {
        let owned_filename = filename.to_os_string();
        let mut owned_filename_with_html = owned_filename.clone();
        owned_filename_with_html.push(".html");
        file_path.set_file_name(owned_filename_with_html);
        if let Ok(meta_res) = file_metadata(file_path.as_ref()) {
            let (meta, _) = meta_res;
            return (file_path, Some(meta));
        } else {
            // We roll-back to the previous filename
            file_path.set_file_name(owned_filename);
        }
    }
    (file_path, None)
}

/// Returns the final composed metadata information (tuple) containing
/// the Arc `PathBuf` reference wrapper for the current `file_path` with its file metadata
/// as well as its optional pre-compressed variant.
async fn composed_file_metadata<'a>(
    mut file_path: &'a mut PathBuf,
    headers: &'a HeaderMap<HeaderValue>,
    compression_static: bool,
) -> Result<(&'a PathBuf, Metadata, bool, Option<(PathBuf, &'a str)>), StatusCode> {
    // First pre-compressed variant check for the given file path
    let mut tried_precompressed = false;
    if compression_static {
        tried_precompressed = true;
        if let Some((path, meta, ext)) =
            compression_static::precompressed_variant(file_path, headers).await
        {
            return Ok((file_path, meta, false, Some((path, ext))));
        }
    }

    tracing::trace!("getting metadata for file {}", file_path.display());

    match file_metadata(file_path.as_ref()) {
        Ok((mut meta, is_dir)) => {
            if is_dir {
                // Append a HTML index page by default if it's a directory path (`autoindex`)
                tracing::debug!("dir: appending an index.html to the directory path");
                file_path.push("index.html");

                // If file exists then overwrite the `meta`
                // Also noting that it's still a directory request
                if let Ok(meta_res) = file_metadata(file_path.as_ref()) {
                    (meta, _) = meta_res
                } else {
                    // We remove the appended index.html
                    file_path.pop();
                    let new_meta: Option<Metadata>;
                    (file_path, new_meta) = prefix_file_html_metadata(file_path);
                    if let Some(new_meta) = new_meta {
                        meta = new_meta;
                    } else {
                        // We append the index.html to preserve previous behavior
                        file_path.push("index.html");
                    }
                }
            }

            Ok((file_path, meta, is_dir, None))
        }
        Err(err) => {
            if err == StatusCode::NOT_FOUND {
                // If the file path doesn't exist, we try to find the path suffixed with `.html`.
                // For example: `/posts/article` will fallback to `/posts/article.html`
                let new_meta: Option<Metadata>;
                (file_path, new_meta) = prefix_file_html_metadata(file_path);
                if let Some(new_meta) = new_meta {
                    return Ok((file_path, new_meta, false, None));
                }
            }

            // Second pre-compressed variant check for the given file path
            if compression_static && !tried_precompressed {
                if let Some((path, meta, ext)) =
                    compression_static::precompressed_variant(file_path, headers).await
                {
                    return Ok((file_path, meta, false, Some((path, ext))));
                }
            }

            Err(err)
        }
    }
}

/// Try to find the file system metadata for the given file path.
pub fn file_metadata(file_path: &Path) -> Result<(Metadata, bool), StatusCode> {
    match std::fs::metadata(file_path) {
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

/// Reply with the corresponding file content taking into account
/// its precompressed variant if any.
/// The `path` param should contains always the original requested file path and
/// the `meta` param value should corresponds to it.
/// However, if `path_precompressed` contains some value then
/// the `meta` param  value will belong to the `path_precompressed` (precompressed file variant).
fn file_reply<'a>(
    headers: &'a HeaderMap<HeaderValue>,
    path: &'a PathBuf,
    meta: &'a Metadata,
    path_precompressed: Option<PathBuf>,
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send + 'a {
    let conditionals = get_conditional_headers(headers);

    let file_path = path_precompressed.as_ref().unwrap_or(path);

    match File::open(file_path) {
        Ok(file) => Either::Left(response_body(file, path, meta, conditionals)),
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!("file can't be opened or not found: {:?}", path.display());
                    StatusCode::NOT_FOUND
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::warn!("file permission denied: {:?}", path.display());
                    StatusCode::FORBIDDEN
                }
                _ => {
                    tracing::error!("file open error (path={:?}): {} ", path.display(), err);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            };
            Either::Right(future::err(status))
        }
    }
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

#[cfg(unix)]
const READ_BUF_SIZE: usize = 4_096;

#[cfg(not(unix))]
const READ_BUF_SIZE: usize = 8_192;

#[derive(Debug)]
pub struct FileStream<T> {
    reader: T,
}

impl<T: Read + Unpin> Stream for FileStream<T> {
    type Item = Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = BytesMut::zeroed(READ_BUF_SIZE);
        match Pin::into_inner(self).reader.read(&mut buf[..]) {
            Ok(n) => {
                if n == 0 {
                    Poll::Ready(None)
                } else {
                    buf.truncate(n);
                    Poll::Ready(Some(Ok(buf.freeze())))
                }
            }
            Err(err) => Poll::Ready(Some(Err(anyhow::Error::from(err)))),
        }
    }
}

async fn response_body(
    mut file: File,
    path: &PathBuf,
    meta: &Metadata,
    conditionals: Conditionals,
) -> Result<Response<Body>, StatusCode> {
    let mut len = meta.len();
    let modified = meta.modified().ok().map(LastModified::from);

    match conditionals.check(modified) {
        Cond::NoBody(resp) => Ok(resp),
        Cond::WithBody(range) => {
            bytes_range(range, len)
                .map(|(start, end)| {
                    match file.seek(SeekFrom::Start(start)) {
                        Ok(_) => (),
                        Err(err) => {
                            tracing::error!("seek file from start error: {:?}", err);
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                    };

                    let sub_len = end - start;
                    let reader = BufReader::new(file).take(sub_len);
                    let stream = FileStream { reader };

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

                    Ok(resp)
                })
                .unwrap_or_else(|BadRange| {
                    // bad byte range
                    let mut resp = Response::new(Body::empty());
                    *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                    resp.headers_mut()
                        .typed_insert(ContentRange::unsatisfied_bytes(len));
                    Ok(resp)
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

#[cfg(test)]
mod tests {
    use super::sanitize_path;
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
}
