use hyper::{Body, Response};

use crate::settings::Header;

/** Append custom HTTP headers to current response. */
pub fn append_headers(
    uri: &str,
    headers_opts_vec: &Option<Vec<Header>>,
    resp: &mut Response<Body>,
) {
    if let Some(multiple_headers) = headers_opts_vec {
        for headers_entry in multiple_headers.iter() {
            // Match header glob pattern against request uri
            if headers_entry.source.is_match(uri) {
                // Add/update headers if uri matches
                for (name, value) in &headers_entry.headers {
                    resp.headers_mut().insert(name, value.to_owned());
                }
            }
        }
    }
}
