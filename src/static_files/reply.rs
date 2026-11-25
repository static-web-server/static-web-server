// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! HTTP response construction for the static-files handler.
//!
//! Each builder produces a `Response<Body>`; the orchestrator in
//! [`super`](super) pairs the response with the file path it relates to.

use headers::{AcceptRanges, HeaderMapExt, HeaderValue};
use hyper::{Response, StatusCode, header::CONTENT_ENCODING};
use std::fs::{File, Metadata};
use std::io;
use std::path::{Path, PathBuf};

use crate::body::Body;
use crate::conditional_headers::ConditionalHeaders;
use crate::exts::headers::ContentCoding;
use crate::exts::http::HTTP_SUPPORTED_METHODS;
#[cfg(feature = "mem-cache")]
use crate::mem_cache::cache::Representation;
use crate::response::response_body;

use super::opts::HandleOpts;

/// Produces a `308 Permanent Redirect` to the redirect path suffixed with `/`
/// when the resolved path is a directory and the redirect path doesn't end
/// with `/`. Returns `Ok(None)` otherwise.
///
/// The redirect path is the client's when a rewrite applied and keeps the
/// same directory with `/` appended (see
/// [`HandleOpts::redirect_path_and_query`]).
pub(super) fn trailing_slash_redirect(
    is_dir: bool,
    opts: &HandleOpts<'_>,
) -> Result<Option<Response<Body>>, StatusCode> {
    if !(is_dir && opts.redirect_trailing_slash) {
        return Ok(None);
    }
    let (path, query) = opts.redirect_path_and_query();
    if path.ends_with('/') {
        return Ok(None);
    }

    let loc = HeaderValue::from_str(&redirect_location(path, query)).map_err(|err| {
        tracing::error!("invalid header value from current uri: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut resp = Response::new(crate::body::empty());
    resp.headers_mut().insert(hyper::header::LOCATION, loc);
    *resp.status_mut() = StatusCode::PERMANENT_REDIRECT;
    tracing::trace!("uri doesn't end with a slash so redirecting permanently");

    Ok(Some(resp))
}

/// Appends `/` to `path`, followed by the `query` if any.
///
/// Leading slashes collapse into one and backslashes are percent-encoded,
/// since browsers read a `Location` starting with `//` or `/\` as a URL on
/// another host, and any `\` as `/`.
fn redirect_location(path: &str, query: Option<&str>) -> String {
    let rest = path.trim_start_matches('/');
    let root = if rest.len() < path.len() { "/" } else { "" };
    let rest = rest.replace('\\', "%5C");
    match query {
        Some(query) => [root, &rest, "/?", query].concat(),
        None => [root, &rest, "/"].concat(),
    }
}

/// Builds the `204 No Content` response used to answer `OPTIONS` requests.
pub(super) fn options_reply() -> Response<Body> {
    let mut resp = Response::new(crate::body::empty());
    *resp.status_mut() = StatusCode::NO_CONTENT;
    resp.headers_mut()
        .typed_insert(headers::Allow::from_iter(HTTP_SUPPORTED_METHODS.clone()));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());
    resp
}

/// Serves the resolved file, transparently picking the pre-compressed
/// variant on disk when one was located by [`super::resolve`].
///
/// `pre_opened` is an optional, already-open `File` handle for `file_path`
/// that was opened by the resolver to avoid a redundant `open(2)` syscall
/// on the hot path. It is ignored when a precompressed variant is being
/// served (the precomp file is opened on demand).
pub(super) fn file_or_precompressed(
    opts: &HandleOpts<'_>,
    file_path: &Path,
    metadata: &Metadata,
    precompressed_variant: Option<(PathBuf, ContentCoding)>,
    pre_opened: Option<File>,
) -> Result<Response<Body>, StatusCode> {
    if let Some((precomp_path, precomp_encoding)) = precompressed_variant {
        // Pre-opened handle (if any) refers to the original file we are
        // about to replace with the precompressed variant; just drop it.
        drop(pre_opened);
        return precompressed_reply(opts, file_path, metadata, precomp_path, precomp_encoding);
    }

    file_reply(opts, file_path, metadata, None, pre_opened)
}

/// Serves a pre-compressed variant with its `Content-Encoding`. `metadata`
/// is the variant's, so `Content-Length` is the variant's length. Responses
/// without content such as `304`, `412` and `416` are left as they are.
fn precompressed_reply(
    opts: &HandleOpts<'_>,
    file_path: &Path,
    metadata: &Metadata,
    precomp_path: PathBuf,
    precomp_encoding: ContentCoding,
) -> Result<Response<Body>, StatusCode> {
    let mut resp = file_reply(
        opts,
        file_path,
        metadata,
        Some((precomp_path, precomp_encoding)),
        None,
    )?;

    if !resp.status().is_success() {
        return Ok(resp);
    }

    let encoding = HeaderValue::from_str(precomp_encoding.as_str()).map_err(|err| {
        tracing::error!(
            "unable to parse header value from content encoding: {:?}",
            err
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    resp.headers_mut().insert(CONTENT_ENCODING, encoding);

    Ok(resp)
}

/// Opens `file_path` (or the `precompressed` variant when present), wires up
/// conditional headers, and produces a streaming response body.
///
/// When `pre_opened` is `Some(file)` and no precompressed variant is being
/// served, the existing handle is reused instead of re-opening the file.
fn file_reply(
    opts: &HandleOpts<'_>,
    path: &Path,
    meta: &Metadata,
    precompressed: Option<(PathBuf, ContentCoding)>,
    pre_opened: Option<File>,
) -> Result<Response<Body>, StatusCode> {
    let conditionals = ConditionalHeaders::new(opts.headers);
    #[cfg(feature = "mem-cache")]
    let representation = Representation {
        content_encoding: precompressed
            .as_ref()
            .map_or(ContentCoding::IDENTITY, |(_, encoding)| *encoding),
        compression_static: opts.compression_static,
    };
    let path_precompressed = precompressed.map(|(precomp_path, _)| precomp_path);

    // Reuse the pre-opened handle when serving the original file. For
    // precompressed variants the open target differs, so the helper opens
    // the precomp file itself (and the caller dropped `pre_opened`).
    let file_result = match (path_precompressed.as_deref(), pre_opened) {
        (None, Some(file)) => Ok(file),
        (Some(precomp_path), _) => File::open(precomp_path),
        (None, None) => File::open(path),
    };
    let open_path: &Path = path_precompressed.as_deref().unwrap_or(path);

    match file_result {
        Ok(file) => {
            #[cfg(feature = "mem-cache")]
            {
                let _ = open_path;
                response_body(
                    file,
                    path,
                    meta,
                    conditionals,
                    opts.etag,
                    opts.memory_cache,
                    representation,
                )
            }

            #[cfg(not(feature = "mem-cache"))]
            {
                let _ = open_path;
                response_body(file, path, meta, conditionals, opts.etag)
            }
        }
        Err(err) => Err(open_error_to_status(err, path)),
    }
}

/// Maps a `File::open` failure to an HTTP status code with the
/// appropriate log level.
fn open_error_to_status(err: io::Error, path: &Path) -> StatusCode {
    match err.kind() {
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
    }
}

#[cfg(test)]
mod tests {
    use super::redirect_location;

    #[test]
    fn redirect_location_appends_slash_and_query() {
        assert_eq!(redirect_location("/dir", None), "/dir/");
        assert_eq!(redirect_location("dir", None), "dir/");
        assert_eq!(redirect_location("/dir", Some("a=1")), "/dir/?a=1");
        assert_eq!(redirect_location("/dir", Some("")), "/dir/?");
    }

    #[test]
    fn redirect_location_stays_on_host() {
        assert_eq!(redirect_location("//host", None), "/host/");
        assert_eq!(redirect_location("///host", None), "/host/");
        assert_eq!(redirect_location("/\\host", None), "/%5Chost/");
        assert_eq!(redirect_location("/\\/host", None), "/%5C/host/");
        assert_eq!(redirect_location("/\\", None), "/%5C/");
        assert_eq!(redirect_location("/a\\b", None), "/a%5Cb/");
    }
}
