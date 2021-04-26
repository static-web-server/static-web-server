use warp::{reject, Filter, Rejection};

use crate::rejection::InvalidHeader;

/// Warp filter in order to check for an `Accept-Encoding` header value.
pub fn has_accept_encoding(
    occurrence: &'static str,
) -> impl Filter<Extract = (), Error = Rejection> + Copy {
    warp::header::<String>("accept-encoding")
        .and_then(move |s: String| async move {
            if s.contains(occurrence) {
                Ok(())
            } else {
                Err(reject::custom(InvalidHeader {
                    name: "accept-encoding",
                }))
            }
        })
        .untuple_one()
}
