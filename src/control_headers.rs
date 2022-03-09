// An arbitrary `Cache-Control` headers functionality for incoming requests based on a set of file types.
// Note: Since it's an ad-hoc feature it could be subject to change in the future.
// See https://github.com/joseluisq/static-web-server/issues/30

use headers::{CacheControl, HeaderMapExt};
use hyper::{Body, Response};

const CACHE_EXT_ONE_HOUR: [&str; 4] = ["atom", "json", "rss", "xml"];
const CACHE_EXT_ONE_YEAR: [&str; 32] = [
    "avif", "bmp", "bz2", "css", "doc", "gif", "gz", "htc", "ico", "jpeg", "jpg", "js", "jxl",
    "map", "mjs", "mp3", "mp4", "ogg", "ogv", "pdf", "png", "rar", "rtf", "tar", "tgz", "wav",
    "weba", "webm", "webp", "woff", "woff2", "zip",
];

/// It appends a `Cache-Control` header to a response if that one is part of a set of file types.
pub fn append_headers(uri: &str, resp: &mut Response<Body>) {
    // Default max-age value in seconds (one day)
    let mut max_age = 60 * 60 * 24_u64;

    if CACHE_EXT_ONE_HOUR
        .iter()
        .any(|x| uri.ends_with(&[".", *x].concat()))
    {
        max_age = 60 * 60;
    } else if CACHE_EXT_ONE_YEAR
        .iter()
        .any(|x| uri.ends_with(&[".", *x].concat()))
    {
        max_age = 60 * 60 * 24 * 365;
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
