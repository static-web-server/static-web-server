/// Warp filter in order to check for an `Accept-Encoding` header value.
pub fn has_accept_encoding(
    val: &'static str,
) -> impl warp::Filter<Extract = (), Error = warp::Rejection> + Copy {
    warp::header::contains("accept-encoding", val)
}
