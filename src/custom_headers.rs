use hyper::{Body, Response};

use crate::settings::Headers;

/// Append custom HTTP headers to current response.
pub fn append_headers(
    uri: &str,
    headers_opts_vec: &Option<Vec<Headers>>,
    resp: &mut Response<Body>,
) {
    if let Some(headers_vec) = headers_opts_vec {
        for headers_entry in headers_vec.iter() {
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
