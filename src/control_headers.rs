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
    Request, Response, StatusCode,
    header::{CACHE_CONTROL, HeaderValue},
};

use crate::body::Body;
use crate::{Error, handler::RequestHandlerOpts};

// Pre-computed static Cache-Control header values
static CACHE_CONTROL_DEFAULT: HeaderValue = HeaderValue::from_static("no-cache");
static CACHE_CONTROL_ONE_HOUR: HeaderValue = HeaderValue::from_static("max-age=3600");
static CACHE_CONTROL_ONE_YEAR: HeaderValue = HeaderValue::from_static("max-age=31536000");

// `Cache-Control` list of extensions (arrays must be alphabetically sorted)
const CACHE_EXT_ONE_HOUR: [&str; 2] = ["atom", "rss"];
const CACHE_EXT_ONE_YEAR: [&str; 32] = [
    "avif", "bmp", "bz2", "css", "doc", "gif", "gz", "htc", "ico", "jpeg", "jpg", "js", "jxl",
    "map", "mjs", "mp3", "mp4", "ogg", "ogv", "pdf", "png", "rar", "rtf", "tar", "tgz", "wav",
    "weba", "webm", "webp", "woff", "woff2", "zip",
];

pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.cache_control_headers = enabled;
    tracing::info!(enabled, "cache control headers");
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
        &CACHE_CONTROL_DEFAULT
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
        // Zero-allocation stack buffer optimization for lowercase conversion
        let mut buf = [0u8; 16];
        if extension.len() <= buf.len() {
            let ext_bytes = &mut buf[..extension.len()];
            ext_bytes.copy_from_slice(extension.as_bytes());
            ext_bytes.make_ascii_lowercase();

            if let Ok(ext_lower) = std::str::from_utf8(ext_bytes) {
                if CACHE_EXT_ONE_HOUR.binary_search(&ext_lower).is_ok() {
                    return &CACHE_CONTROL_ONE_HOUR;
                } else if CACHE_EXT_ONE_YEAR.binary_search(&ext_lower).is_ok() {
                    return &CACHE_CONTROL_ONE_YEAR;
                }
            }
        } else {
            // Fallback allocations for abnormally long extensions
            let ext_lower = extension.to_ascii_lowercase();
            if CACHE_EXT_ONE_HOUR
                .binary_search(&ext_lower.as_str())
                .is_ok()
            {
                return &CACHE_CONTROL_ONE_HOUR;
            } else if CACHE_EXT_ONE_YEAR
                .binary_search(&ext_lower.as_str())
                .is_ok()
            {
                return &CACHE_CONTROL_ONE_YEAR;
            }
        }
    }
    &CACHE_CONTROL_DEFAULT
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Response, StatusCode};

    #[test]
    fn test_arrays_are_sorted() {
        assert!(
            CACHE_EXT_ONE_HOUR.windows(2).all(|w| w[0] < w[1]),
            "CACHE_EXT_ONE_HOUR is not sorted!"
        );
        assert!(
            CACHE_EXT_ONE_YEAR.windows(2).all(|w| w[0] < w[1]),
            "CACHE_EXT_ONE_YEAR is not sorted!"
        );
    }

    #[test]
    fn headers_case_insensitivity() {
        let mut resp = Response::new(crate::body::empty());
        append_headers("/assets/script.JS", &mut resp);
        let cache_control = resp.headers().get(CACHE_CONTROL).unwrap();
        assert_eq!(cache_control.to_str().unwrap(), "max-age=31536000");

        append_headers("/assets/IMAGE.PNG", &mut resp);
        let cache_control = resp.headers().get(CACHE_CONTROL).unwrap();
        assert_eq!(cache_control.to_str().unwrap(), "max-age=31536000");
    }

    #[test]
    fn headers_one_hour() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;

        for ext in CACHE_EXT_ONE_HOUR.iter() {
            append_headers(&["/some.", ext].concat(), &mut resp);
            let cache_control = resp.headers().get(CACHE_CONTROL).unwrap();
            assert_eq!(cache_control.to_str().unwrap(), "max-age=3600");
        }
    }

    #[test]
    fn headers_default_fallback() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;

        append_headers("/", &mut resp);
        assert_eq!(
            resp.headers().get(CACHE_CONTROL).unwrap().to_str().unwrap(),
            "no-cache"
        );

        append_headers("/index.html", &mut resp);
        assert_eq!(
            resp.headers().get(CACHE_CONTROL).unwrap().to_str().unwrap(),
            "no-cache"
        );

        append_headers("/config.json", &mut resp);
        assert_eq!(
            resp.headers().get(CACHE_CONTROL).unwrap().to_str().unwrap(),
            "no-cache"
        );

        append_headers("/api/data", &mut resp);
        assert_eq!(
            resp.headers().get(CACHE_CONTROL).unwrap().to_str().unwrap(),
            "no-cache"
        );
    }

    #[test]
    fn headers_non_success_status_default_fallback() {
        let statuses = [
            StatusCode::PERMANENT_REDIRECT,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::PRECONDITION_FAILED,
            StatusCode::RANGE_NOT_SATISFIABLE,
            StatusCode::INTERNAL_SERVER_ERROR,
        ];
        for status in statuses {
            for uri in ["/missing.css", "/feed.rss", "/missing.html"] {
                let mut resp = Response::new(crate::body::empty());
                *resp.status_mut() = status;
                append_headers(uri, &mut resp);
                assert_eq!(
                    resp.headers().get(CACHE_CONTROL).unwrap().to_str().unwrap(),
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
            let mut resp = Response::new(crate::body::empty());
            *resp.status_mut() = status;
            append_headers("/assets/main.css", &mut resp);
            assert_eq!(
                resp.headers().get(CACHE_CONTROL).unwrap().to_str().unwrap(),
                "max-age=31536000",
                "status {status}"
            );
        }
    }

    #[test]
    fn headers_one_year() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;

        for ext in CACHE_EXT_ONE_YEAR.iter() {
            append_headers(&["/some.", ext].concat(), &mut resp);
            let cache_control = resp.headers().get(CACHE_CONTROL).unwrap();
            assert_eq!(cache_control.to_str().unwrap(), "max-age=31536000");
        }
    }
}
