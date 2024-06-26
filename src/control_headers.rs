// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides an arbitrary `Cache-Control` headers functionality
//! for incoming requests based on a set of file types.
//!

use hyper::{Body, Request, Response};

use crate::{handler::RequestHandlerOpts, Error};

// Cache-Control `max-age` variants
const MAX_AGE_ONE_HOUR: u64 = 60 * 60;
const MAX_AGE_ONE_DAY: u64 = 60 * 60 * 24;
const MAX_AGE_ONE_YEAR: u64 = 60 * 60 * 24 * 365;

// `Cache-Control` list of extensions
const CACHE_EXT_ONE_HOUR: [&str; 4] = ["atom", "json", "rss", "xml"];
const CACHE_EXT_ONE_YEAR: [&str; 32] = [
    "avif", "bmp", "bz2", "css", "doc", "gif", "gz", "htc", "ico", "jpeg", "jpg", "js", "jxl",
    "map", "mjs", "mp3", "mp4", "ogg", "ogv", "pdf", "png", "rar", "rtf", "tar", "tgz", "wav",
    "weba", "webm", "webp", "woff", "woff2", "zip",
];

pub(crate) fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.cache_control_headers = enabled;
    server_info!("cache control headers: enabled={enabled}");
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
    let max_age = get_max_age(uri);
    resp.headers_mut().insert(
        "cache-control",
        format!(
            "public, max-age={}",
            // It caps value in seconds at ~136 years
            std::cmp::min(max_age, u32::MAX as u64)
        )
        .parse()
        .unwrap(),
    );
}

/// Gets the file extension for a URI.
///
/// This assumes the extension contains a single dot. e.g. for "/file.tar.gz" it returns "gz".
#[inline(always)]
fn get_file_extension(uri: &str) -> Option<&str> {
    uri.rsplit_once('.').map(|(_, rest)| rest)
}

#[inline(always)]
fn get_max_age(uri: &str) -> u64 {
    // Default max-age value in seconds (one day)
    let mut max_age = MAX_AGE_ONE_DAY;

    if let Some(extension) = get_file_extension(uri) {
        if CACHE_EXT_ONE_HOUR.binary_search(&extension).is_ok() {
            max_age = MAX_AGE_ONE_HOUR;
        } else if CACHE_EXT_ONE_YEAR.binary_search(&extension).is_ok() {
            max_age = MAX_AGE_ONE_YEAR;
        }
    }
    max_age
}

#[cfg(test)]
mod tests {
    use hyper::{Body, Response, StatusCode};

    use super::{
        append_headers, get_file_extension, CACHE_EXT_ONE_HOUR, CACHE_EXT_ONE_YEAR,
        MAX_AGE_ONE_DAY, MAX_AGE_ONE_HOUR, MAX_AGE_ONE_YEAR,
    };

    #[test]
    fn headers_one_hour() {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::OK;

        for ext in CACHE_EXT_ONE_HOUR.iter() {
            append_headers(&["/some.", ext].concat(), &mut resp);

            let cache_control = resp.headers().get(http::header::CACHE_CONTROL).unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(
                cache_control.to_str().unwrap(),
                format!("public, max-age={MAX_AGE_ONE_HOUR}")
            );
        }
    }

    #[test]
    fn headers_one_day_default() {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::OK;

        append_headers("/", &mut resp);

        let cache_control = resp.headers().get(http::header::CACHE_CONTROL).unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            cache_control.to_str().unwrap(),
            format!("public, max-age={MAX_AGE_ONE_DAY}")
        );
    }

    #[test]
    fn headers_one_year() {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::OK;

        for ext in CACHE_EXT_ONE_YEAR.iter() {
            append_headers(&["/some.", ext].concat(), &mut resp);

            let cache_control = resp.headers().get(http::header::CACHE_CONTROL).unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            assert_eq!(
                cache_control.to_str().unwrap(),
                format!("public, max-age={MAX_AGE_ONE_YEAR}")
            );
        }
    }

    #[test]
    fn find_uri_extension() {
        assert_eq!(get_file_extension("/potato.zip"), Some("zip"));
        assert_eq!(get_file_extension("/potato."), Some(""));
        assert_eq!(get_file_extension("/"), None);
    }
}
