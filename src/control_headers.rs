// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides an arbitrary `Cache-Control` headers functionality
//! for incoming requests based on a set of file types.
//!
//! For responses served by the static-file handler (including its error pages),
//! file type values apply only to `2xx` and `304`; any other status gets `no-cache`.
//!

use hyper::{
    Body, Request, Response, StatusCode,
    header::{CACHE_CONTROL, HeaderValue},
};

use crate::{Error, handler::RequestHandlerOpts};

// Pre-computed static Cache-Control header values
static CACHE_CONTROL_NO_CACHE: HeaderValue = HeaderValue::from_static("no-cache");
static CACHE_CONTROL_ONE_HOUR: HeaderValue = HeaderValue::from_static("max-age=3600");
static CACHE_CONTROL_ONE_DAY: HeaderValue = HeaderValue::from_static("max-age=86400");
static CACHE_CONTROL_ONE_YEAR: HeaderValue = HeaderValue::from_static("max-age=31536000");

// `Cache-Control` list of extensions
const CACHE_EXT_ONE_HOUR: [&str; 4] = ["atom", "json", "rss", "xml"];
const CACHE_EXT_ONE_YEAR: [&str; 32] = [
    "avif", "bmp", "bz2", "css", "doc", "gif", "gz", "htc", "ico", "jpeg", "jpg", "js", "jxl",
    "map", "mjs", "mp3", "mp4", "ogg", "ogv", "pdf", "png", "rar", "rtf", "tar", "tgz", "wav",
    "weba", "webm", "webp", "woff", "woff2", "zip",
];

pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.cache_control_headers = enabled;
    tracing::info!("cache control headers: enabled={enabled}");
}

/// Appends `Cache-Control` header to a response if necessary
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if opts.cache_control_headers {
        append_headers(req.uri().path(), &mut resp);
    }
    Ok(resp)
}

/// It appends a `Cache-Control` header to a response if that one is part of a set of file types.
pub fn append_headers(uri: &str, resp: &mut Response<Body>) {
    let status = resp.status();
    // File type values describe a successful representation: only `2xx` and `304` use them
    // (a `304` repeats the `200`'s `Cache-Control`, RFC 9110 §15.4.5), others get `no-cache`.
    let header_value = if status.is_success() || status == StatusCode::NOT_MODIFIED {
        get_cache_control_header(uri)
    } else {
        &CACHE_CONTROL_NO_CACHE
    };
    resp.headers_mut()
        .insert(CACHE_CONTROL, header_value.clone());
}

/// Gets the file extension for a URI.
///
/// This assumes the extension contains a single dot. e.g. for "/file.tar.gz" it returns "gz".
#[inline(always)]
fn get_file_extension(uri: &str) -> Option<&str> {
    uri.rsplit_once('.').map(|(_, rest)| rest)
}

/// Returns the pre-computed static Cache-Control header value for the given URI.
#[inline(always)]
fn get_cache_control_header(uri: &str) -> &'static HeaderValue {
    if let Some(extension) = get_file_extension(uri) {
        if CACHE_EXT_ONE_HOUR.binary_search(&extension).is_ok() {
            return &CACHE_CONTROL_ONE_HOUR;
        } else if CACHE_EXT_ONE_YEAR.binary_search(&extension).is_ok() {
            return &CACHE_CONTROL_ONE_YEAR;
        }
    }
    &CACHE_CONTROL_ONE_DAY
}

#[cfg(test)]
mod tests {
    use hyper::{Body, Response, StatusCode};

    use super::{CACHE_EXT_ONE_HOUR, CACHE_EXT_ONE_YEAR, append_headers, get_file_extension};

    #[test]
    fn headers_one_hour() {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::OK;

        for ext in CACHE_EXT_ONE_HOUR.iter() {
            append_headers(&["/some.", ext].concat(), &mut resp);

            let cache_control = resp.headers().get(http::header::CACHE_CONTROL).unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(cache_control.to_str().unwrap(), "max-age=3600");
        }
    }

    #[test]
    fn headers_one_day_default() {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::OK;

        append_headers("/", &mut resp);

        let cache_control = resp.headers().get(http::header::CACHE_CONTROL).unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(cache_control.to_str().unwrap(), "max-age=86400");
    }

    #[test]
    fn headers_non_success_status_no_cache() {
        let statuses = [
            StatusCode::PERMANENT_REDIRECT,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::PRECONDITION_FAILED,
            StatusCode::RANGE_NOT_SATISFIABLE,
            StatusCode::INTERNAL_SERVER_ERROR,
        ];
        for status in statuses {
            for uri in ["/missing.css", "/feed.rss", "/missing.html", "/"] {
                let mut resp = Response::new(Body::empty());
                *resp.status_mut() = status;
                append_headers(uri, &mut resp);
                assert_eq!(
                    resp.headers()[http::header::CACHE_CONTROL],
                    "no-cache",
                    "status {status} for {uri}"
                );
            }
        }
    }

    #[test]
    fn headers_success_and_not_modified_status() {
        let statuses = [
            StatusCode::OK,
            StatusCode::NO_CONTENT,
            StatusCode::PARTIAL_CONTENT,
            StatusCode::NOT_MODIFIED,
        ];
        for status in statuses {
            for (uri, expected) in [
                ("/assets/main.css", "max-age=31536000"),
                ("/feed.rss", "max-age=3600"),
                ("/data.json", "max-age=3600"),
                ("/data.xml", "max-age=3600"),
                ("/index.html", "max-age=86400"),
                ("/", "max-age=86400"),
            ] {
                let mut resp = Response::new(Body::empty());
                *resp.status_mut() = status;
                append_headers(uri, &mut resp);
                assert_eq!(
                    resp.headers()[http::header::CACHE_CONTROL],
                    expected,
                    "status {status} for {uri}"
                );
            }
        }
    }

    #[test]
    fn headers_one_year() {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::OK;

        for ext in CACHE_EXT_ONE_YEAR.iter() {
            append_headers(&["/some.", ext].concat(), &mut resp);

            let cache_control = resp.headers().get(http::header::CACHE_CONTROL).unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(cache_control.to_str().unwrap(), "max-age=31536000");
        }
    }

    #[test]
    fn find_uri_extension() {
        assert_eq!(get_file_extension("/potato.zip"), Some("zip"));
        assert_eq!(get_file_extension("/potato."), Some(""));
        assert_eq!(get_file_extension("/"), None);
    }
}
