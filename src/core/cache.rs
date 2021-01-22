const CACHE_EXT_ONE_HOUR: [&str; 4] = ["atom", "json", "rss", "xml"];
const CACHE_EXT_ONE_YEAR: [&str; 30] = [
    "bmp", "bz2", "css", "map", "doc", "gif", "gz", "htc", "ico", "jpg", "mp3", "mp4", "ogg",
    "ogv", "pdf", "png", "rar", "tar", "tgz", "wav", "weba", "webm", "webp", "woff", "zip", "jpeg",
    "js", "mjs", "rtf", "woff2",
];

/// It applies the corresponding Cache-Control headers based on a set of file types.
pub fn control_headers(res: warp::fs::File) -> warp::reply::WithHeader<warp::fs::File> {
    // Default max-age value in seconds (one day)
    let mut max_age = 60 * 60 * 24_u64;

    if let Some(ext) = res.path().extension() {
        if let Some(ext) = ext.to_str() {
            if CACHE_EXT_ONE_HOUR.iter().any(|x| *x == ext) {
                max_age = 60 * 60;
            } else if CACHE_EXT_ONE_YEAR.iter().any(|x| *x == ext) {
                max_age = 60 * 60 * 24 * 365;
            }
        }
    }

    // HTML file types and others
    warp::reply::with_header(
        res,
        "cache-control",
        [
            "public, max-age=".to_string(),
            duration(max_age).to_string(),
        ]
        .concat(),
    )
}

/// It caps a duration value at ~136 years.
fn duration(n: u64) -> u32 {
    std::cmp::min(n, u32::MAX as u64) as u32
}

/// Warp filter in order to check for an `Accept-Encoding` header value.
pub fn has_accept_encoding(
    val: &'static str,
) -> impl warp::Filter<Extract = (), Error = warp::Rejection> + Copy {
    warp::header::contains("accept-encoding", val)
}
