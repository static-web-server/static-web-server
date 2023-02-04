// An arbitrary `Cache-Control` headers functionality for incoming requests based on a set of file types.

use headers::{CacheControl, HeaderMapExt};
use hyper::{Body, Response};

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

/// It appends a `Cache-Control` header to a response if that one is part of a set of file types.
pub fn append_headers(uri: &str, resp: &mut Response<Body>) {
    // Default max-age value in seconds (one day)
    let mut max_age = MAX_AGE_ONE_DAY;

    if CACHE_EXT_ONE_HOUR
        .iter()
        .any(|x| uri.ends_with(&[".", *x].concat()))
    {
        max_age = MAX_AGE_ONE_HOUR;
    } else if CACHE_EXT_ONE_YEAR
        .iter()
        .any(|x| uri.ends_with(&[".", *x].concat()))
    {
        max_age = MAX_AGE_ONE_YEAR;
    }

    let cache_control = CacheControl::new()
        .with_public()
        .with_max_age(duration_from_secs(max_age));
    resp.headers_mut().typed_insert(cache_control);
}

/// It caps a duration value at ~136 years.
fn duration_from_secs(secs: u64) -> std::time::Duration {
    std::time::Duration::from_secs(std::cmp::min(secs, u32::MAX as u64))
}

#[cfg(test)]
mod tests {
    use hyper::{Body, Response, StatusCode};

    use super::{
        append_headers, CACHE_EXT_ONE_HOUR, CACHE_EXT_ONE_YEAR, MAX_AGE_ONE_DAY, MAX_AGE_ONE_HOUR,
        MAX_AGE_ONE_YEAR,
    };

    #[tokio::test]
    async fn headers_one_hour() {
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

    #[tokio::test]
    async fn headers_one_day_default() {
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

    #[tokio::test]
    async fn headers_one_year() {
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
}
