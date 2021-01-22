use std::collections::HashSet;
use warp::filters::cors::Builder;

/// Warp filter which provides an optional CORS if its supported.
pub fn get_opt_cors_filter(origins: &str) -> (Option<Builder>, String) {
    let mut cors_allowed_hosts = String::new();
    let cors_filter = if origins.is_empty() {
        None
    } else if origins == "*" {
        cors_allowed_hosts = origins.into();
        Some(
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["GET", "HEAD", "OPTIONS"]),
        )
    } else {
        cors_allowed_hosts = origins.into();
        let hosts = cors_allowed_hosts
            .split(',')
            .map(|s| s.trim().as_ref())
            .collect::<HashSet<_>>();

        if hosts.is_empty() {
            cors_allowed_hosts = hosts.into_iter().collect::<Vec<&str>>().join(", ");
            None
        } else {
            Some(
                warp::cors()
                    .allow_origins(hosts)
                    .allow_methods(vec!["GET", "HEAD", "OPTIONS"]),
            )
        }
    };

    (cors_filter, cors_allowed_hosts)
}
