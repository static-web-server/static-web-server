//! Redirection module to handle config redirect URLs with pattern matching support.
//!

use hyper::StatusCode;

use crate::settings::Redirects;

/// It returns a redirect's destination path and status code if the current request uri
/// matches against the provided redirect's array.
pub fn get_redirection<'a>(
    uri_path: &'a str,
    redirects_opts_vec: &'a Option<Vec<Redirects>>,
) -> Option<(&'a str, &'a StatusCode)> {
    if let Some(redirects_vec) = redirects_opts_vec {
        for redirect_entry in redirects_vec.iter() {
            // Match source glob pattern against the request uri path
            if redirect_entry.source.is_match(uri_path) {
                return Some((redirect_entry.destination.as_str(), &redirect_entry.kind));
            }
        }
    }

    None
}
