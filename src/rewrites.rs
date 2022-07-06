use crate::settings::Rewrites;

/// It returns a rewrite's destination path if the current request uri
/// matches againt the provided rewrites array.
pub fn rewrite_uri_path<'a>(
    uri_path: &'a str,
    rewrites_opts_vec: &'a Option<Vec<Rewrites>>,
) -> Option<&'a str> {
    if let Some(rewrites_vec) = rewrites_opts_vec {
        for rewrites_entry in rewrites_vec.iter() {
            // Match source glob pattern against request uri path
            if rewrites_entry.source.is_match(uri_path) {
                return Some(rewrites_entry.destination.as_str());
            }
        }
    }

    None
}
