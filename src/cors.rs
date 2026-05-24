// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! CORS module to handle incoming requests.
//!

// Part of the file is borrowed from https://github.com/seanmonstar/warp/blob/master/src/filters/cors.rs

use headers::{
    AccessControlAllowHeaders, AccessControlAllowMethods, AccessControlExposeHeaders, HeaderMap,
    HeaderMapExt, HeaderName, HeaderValue, Origin,
};
use http::header;
use hyper::{Request, Response, StatusCode};
use std::collections::HashSet;

use crate::body::Body;
use crate::{Error, error_page, handler::RequestHandlerOpts};

/// It defines CORS instance.
#[derive(Clone, Debug)]
pub struct Cors {
    allowed_headers: HashSet<HeaderName>,
    exposed_headers: HashSet<HeaderName>,
    max_age: Option<u64>,
    allowed_methods: HashSet<http::Method>,
    origins: Option<HashSet<HeaderValue>>,
}

/// It builds a new CORS instance.
pub fn new(
    origins_str: &str,
    allow_headers_str: &str,
    expose_headers_str: &str,
) -> Option<Configured> {
    let cors = Cors::new();
    let cors = if origins_str.is_empty() {
        None
    } else {
        let [allow_headers_vec, expose_headers_vec] =
            [allow_headers_str, expose_headers_str].map(|s| {
                if s.is_empty() {
                    vec!["origin", "content-type"]
                } else {
                    s.split(',').map(|s| s.trim()).collect::<Vec<_>>()
                }
            });
        // SECURITY/ROBUSTNESS: Reject malformed admin-supplied tokens with
        // a structured `tracing::error!` instead of letting the builder
        // panic at startup. This keeps SWS aligned with the rest of the
        // codebase's error model and avoids an attacker-controlled abort
        // surface if origin/header lists ever get fed from a less-trusted
        // configuration source.
        let allow_headers_vec = validate_header_names("cors.allow_headers", &allow_headers_vec);
        let expose_headers_vec = validate_header_names("cors.expose_headers", &expose_headers_vec);
        let [allow_headers_str, expose_headers_str] =
            [&allow_headers_vec, &expose_headers_vec].map(|v| v.join(","));

        let cors_res = if origins_str == "*" {
            match cors
                .allow_any_origin()
                .allow_headers(allow_headers_vec)
                .and_then(|cors| cors.expose_headers(expose_headers_vec))
                .and_then(|cors| cors.allow_methods(["GET", "HEAD", "OPTIONS"]))
            {
                Ok(cors) => Some(cors),
                Err(err) => {
                    tracing::error!("cors: failed to build configuration: {err:?}");
                    None
                }
            }
        } else {
            let hosts = origins_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| validate_origin_str("cors.allow_origins", s))
                .collect::<Vec<_>>();
            if hosts.is_empty() {
                tracing::error!(
                    "cors: no valid origins found in `{origins_str}`; CORS will be disabled"
                );
                None
            } else {
                match cors
                    .allow_origins(hosts)
                    .and_then(|cors| cors.allow_headers(allow_headers_vec))
                    .and_then(|cors| cors.expose_headers(expose_headers_vec))
                    .and_then(|cors| cors.allow_methods(["GET", "HEAD", "OPTIONS"]))
                {
                    Ok(cors) => Some(cors),
                    Err(err) => {
                        tracing::error!("cors: failed to build configuration: {err:?}");
                        None
                    }
                }
            }
        };

        if cors_res.is_some() {
            tracing::info!(
                "cors enabled=true, allow_methods=[GET,HEAD,OPTIONS], allow_origins={}, allow_headers=[{}], expose_headers=[{}]",
                origins_str,
                allow_headers_str,
                expose_headers_str,
            );
        }
        cors_res
    };

    Cors::build(cors)
}

/// Filter out entries that would cause `Cors::allow_headers` /
/// `expose_headers` to panic, logging each rejected value. Returns the
/// surviving entries.
fn validate_header_names<'a>(field: &str, names: &[&'a str]) -> Vec<&'a str> {
    names
        .iter()
        .copied()
        .filter(|h| {
            if HeaderName::try_from(*h).is_ok() {
                true
            } else {
                tracing::error!("{field}: ignoring invalid HTTP header name `{h}`");
                false
            }
        })
        .collect()
}

/// Verifies that an origin string looks like `scheme://authority` so
/// `IntoOrigin::into_origin` will not panic on it.
#[doc(hidden)]
pub fn validate_origin_str(field: &str, origin: &str) -> bool {
    let mut parts = origin.splitn(2, "://");
    let scheme = parts.next();
    let rest = parts.next();
    match (scheme, rest) {
        (Some(s), Some(r)) if !s.is_empty() && !r.is_empty() => {
            if Origin::try_from_parts(s, r, None).is_ok() {
                true
            } else {
                tracing::error!("{field}: ignoring invalid origin `{origin}`");
                false
            }
        }
        _ => {
            tracing::error!(
                "{field}: ignoring origin `{origin}` (expected `scheme://host[:port]`)"
            );
            false
        }
    }
}

impl Cors {
    /// Creates a new Cors instance.
    pub fn new() -> Self {
        Self {
            origins: None,
            allowed_headers: HashSet::new(),
            exposed_headers: HashSet::new(),
            allowed_methods: HashSet::new(),
            max_age: None,
        }
    }

    /// Adds multiple methods to the existing list of allowed request methods.
    pub fn allow_methods<I>(mut self, methods: I) -> Result<Self, Error>
    where
        I: IntoIterator,
        http::Method: TryFrom<I::Item>,
    {
        for method in methods {
            let method =
                http::Method::try_from(method).map_err(|_| Error::msg("cors: illegal method"))?;
            self.allowed_methods.insert(method);
        }
        Ok(self)
    }

    /// Sets that *any* `Origin` header is allowed.
    ///
    /// # Warning
    ///
    /// This can allow websites you didn't intend to access this resource,
    /// it is usually better to set an explicit list.
    pub fn allow_any_origin(mut self) -> Self {
        self.origins = None;
        self
    }

    /// Add multiple origins to the existing list of allowed `Origin`s.
    pub fn allow_origins<I>(mut self, origins: I) -> Result<Self, Error>
    where
        I: IntoIterator,
        I::Item: IntoOrigin,
    {
        let allowed = self.origins.get_or_insert_with(HashSet::new);
        for origin in origins {
            let origin = origin.into_origin()?;
            let value = HeaderValue::from_str(&origin.to_string())
                .map_err(|err| Error::msg(format!("cors: invalid origin header value: {err}")))?;
            allowed.insert(value);
        }
        Ok(self)
    }

    /// Adds multiple headers to the list of allowed request headers.
    ///
    /// **Note**: These should match the values the browser sends via `Access-Control-Request-Headers`, e.g.`content-type`.
    ///
    pub fn allow_headers<I>(mut self, headers: I) -> Result<Self, Error>
    where
        I: IntoIterator,
        HeaderName: TryFrom<I::Item>,
    {
        for header in headers {
            let header = HeaderName::try_from(header)
                .map_err(|_| Error::msg("cors: illegal allow header"))?;
            self.allowed_headers.insert(header);
        }
        Ok(self)
    }

    /// Adds multiple headers to the list of exposed request headers.
    ///
    /// **Note**: These should match the values the browser sends via `Access-Control-Request-Headers`, e.g.`content-type`.
    ///
    pub fn expose_headers<I>(mut self, headers: I) -> Result<Self, Error>
    where
        I: IntoIterator,
        HeaderName: TryFrom<I::Item>,
    {
        for header in headers {
            let header = HeaderName::try_from(header)
                .map_err(|_| Error::msg("cors: illegal expose header"))?;
            self.exposed_headers.insert(header);
        }
        Ok(self)
    }

    /// Builds the `Cors` wrapper from the configured settings.
    pub fn build(cors: Option<Cors>) -> Option<Configured> {
        cors.as_ref()?;
        let cors = cors?;

        let allowed_headers = cors.allowed_headers.iter().cloned().collect();
        let exposed_headers = cors.exposed_headers.iter().cloned().collect();
        let methods_header = cors.allowed_methods.iter().cloned().collect();

        Some(Configured {
            cors,
            allowed_headers,
            exposed_headers,
            methods_header,
        })
    }
}

impl Default for Cors {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
/// CORS configured.
pub struct Configured {
    cors: Cors,
    allowed_headers: AccessControlAllowHeaders,
    exposed_headers: AccessControlExposeHeaders,
    methods_header: AccessControlAllowMethods,
}

#[derive(Debug)]
/// Validated CORS request.
pub enum Validated {
    /// Validated as preflight.
    Preflight(HeaderValue),
    /// Validated as simple.
    Simple(HeaderValue),
    /// Validated as not cors.
    NotCors,
}

#[derive(Debug, Default)]
/// Forbidden errors.
pub enum Forbidden {
    /// Forbidden error origin.
    #[default]
    Origin,
    /// Forbidden error method.
    Method,
    /// Forbidden error header.
    Header,
}

impl Configured {
    /// Check for the incoming CORS request.
    pub fn check_request(
        &self,
        method: &http::Method,
        headers: &HeaderMap,
    ) -> Result<(HeaderMap, Validated), Forbidden> {
        match (headers.get(header::ORIGIN), method) {
            (Some(origin), &http::Method::OPTIONS) => {
                // OPTIONS requests are preflight CORS requests...

                if !self.is_origin_allowed(origin) {
                    return Err(Forbidden::Origin);
                }

                if let Some(req_method) = headers.get(header::ACCESS_CONTROL_REQUEST_METHOD) {
                    if !self.is_method_allowed(req_method) {
                        return Err(Forbidden::Method);
                    }
                } else {
                    tracing::warn!(
                        "cors: preflight request missing `access-control-request-method` header"
                    );
                    return Err(Forbidden::Method);
                }

                if let Some(req_headers) = headers.get(header::ACCESS_CONTROL_REQUEST_HEADERS) {
                    let headers = match req_headers.to_str() {
                        Ok(val) => val,
                        Err(err) => {
                            tracing::error!(
                                "cors: error parsing header `access-control-request-headers` value: {:?}",
                                err,
                            );
                            return Err(Forbidden::Header);
                        }
                    };

                    for header in headers.split(',') {
                        let h = header.trim();
                        if !self.is_header_allowed(h) {
                            tracing::error!(
                                "cors: header `{}` is not allowed because is missing in `cors_allow_headers` server option",
                                h
                            );
                            return Err(Forbidden::Header);
                        }
                    }
                }

                let mut headers = HeaderMap::new();
                self.append_preflight_headers(&mut headers);
                headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.into());

                Ok((headers, Validated::Preflight(origin.clone())))
            }
            (Some(origin), _) => {
                // Any other method, simply check for a valid origin...
                tracing::trace!("cors origin header: {:?}", origin);

                if self.is_origin_allowed(origin) {
                    let mut headers = HeaderMap::new();
                    self.append_preflight_headers(&mut headers);
                    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.into());

                    Ok((headers, Validated::Simple(origin.clone())))
                } else {
                    Err(Forbidden::Origin)
                }
            }
            _ => {
                // No `ORIGIN` header means this isn't CORS!
                Ok((HeaderMap::new(), Validated::NotCors))
            }
        }
    }

    fn is_method_allowed(&self, header: &HeaderValue) -> bool {
        http::Method::from_bytes(header.as_bytes())
            .map(|method| self.cors.allowed_methods.contains(&method))
            .unwrap_or(false)
    }

    fn is_header_allowed(&self, header: &str) -> bool {
        if header.is_empty() {
            return false;
        }
        HeaderName::from_bytes(header.as_bytes())
            .map(|header| self.cors.allowed_headers.contains(&header))
            .unwrap_or(false)
    }

    fn is_origin_allowed(&self, origin: &HeaderValue) -> bool {
        if origin.is_empty() {
            return false;
        }
        if let Some(ref allowed) = self.cors.origins {
            allowed.contains(origin)
        } else {
            true
        }
    }

    fn append_preflight_headers(&self, headers: &mut HeaderMap) {
        headers.typed_insert(self.allowed_headers.clone());
        headers.typed_insert(self.exposed_headers.clone());
        headers.typed_insert(self.methods_header.clone());

        if let Some(max_age) = self.cors.max_age {
            headers.insert(header::ACCESS_CONTROL_MAX_AGE, max_age.into());
        }
    }
}

/// Cast values into the origin header.
pub trait IntoOrigin {
    /// Cast actual value into an origin header.
    fn into_origin(self) -> Result<Origin, Error>;
}

impl IntoOrigin for &str {
    fn into_origin(self) -> Result<Origin, Error> {
        let (scheme, rest) = self
            .split_once("://")
            .ok_or_else(|| Error::msg("cors::into_origin: expected `scheme://host[:port]`"))?;
        Origin::try_from_parts(scheme, rest, None)
            .map_err(|_| Error::msg("cors::into_origin: invalid Origin"))
    }
}

/// Initializes CORS settings
pub(crate) fn init(
    cors_allow_origins: &str,
    cors_allow_headers: &str,
    cors_expose_headers: &str,
    handler_opts: &mut RequestHandlerOpts,
) {
    handler_opts.cors = new(
        cors_allow_origins.trim(),
        cors_allow_headers.trim(),
        cors_expose_headers.trim(),
    );
}

/// Cached CORS headers stored in request extensions to avoid
/// re-validating the request in `post_process`.
#[derive(Clone)]
pub(crate) struct CorsHeaders(pub(crate) HeaderMap);

/// Rejects requests with wrong CORS headers
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &mut Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    let cors = opts.cors.as_ref()?;
    match cors.check_request(req.method(), req.headers()) {
        Ok((headers, state)) => {
            tracing::debug!("cors state: {:?}", state);
            // Stash validated headers for post_process to reuse
            if !headers.is_empty() {
                req.extensions_mut().insert(CorsHeaders(headers));
            }
            None
        }
        Err(err) => {
            tracing::error!("cors error kind: {:?}", err);
            Some(error_page::error_response(
                req.uri(),
                req.method(),
                &StatusCode::FORBIDDEN,
                &opts.page404,
                &opts.page50x,
            ))
        }
    }
}

/// Adds CORS headers to response
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if opts.cors.is_some()
        && let Some(cors_headers) = req.extensions().get::<CorsHeaders>()
    {
        for (k, v) in cors_headers.0.iter() {
            resp.headers_mut().insert(k, v.to_owned());
        }
        resp.headers_mut().insert(
            http::header::VARY,
            HeaderValue::from_name(http::header::ORIGIN),
        );
        resp.headers_mut().remove(http::header::ALLOW);
    }
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::{Configured, Cors, post_process, pre_process};
    use crate::body::Body;
    use crate::{Error, handler::RequestHandlerOpts};
    use hyper::{Request, Response, StatusCode};

    fn make_request(method: &str, origin: &str) -> Request<Body> {
        let mut builder = Request::builder();
        if !origin.is_empty() {
            builder = builder.header("Origin", origin);
        }
        builder
            .method(method)
            .uri("/")
            .body(crate::body::empty())
            .unwrap()
    }

    fn make_response() -> Response<Body> {
        Response::builder().body(crate::body::empty()).unwrap()
    }

    fn make_cors_config() -> Option<Configured> {
        let cors = Cors::new()
            .allow_origins(vec!["https://example.com/"])
            .unwrap()
            .allow_headers(vec!["X-Allowed"])
            .unwrap()
            .allow_methods(vec!["GET", "HEAD"])
            .unwrap();
        Cors::build(Some(cors))
    }

    fn get_allowed_origin(resp: Response<Body>) -> Option<String> {
        resp.headers()
            .get("Access-Control-Allow-Origin")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
    }

    fn is_403(result: Option<Result<Response<Body>, Error>>) -> bool {
        if let Some(Ok(response)) = result {
            response.status() == StatusCode::FORBIDDEN
        } else {
            false
        }
    }

    #[test]
    fn test_cors_disabled() -> Result<(), Error> {
        let opts = RequestHandlerOpts {
            cors: None,
            ..Default::default()
        };
        let mut req = make_request("GET", "https://example.com/");

        assert!(pre_process(&opts, &mut req).is_none());

        let resp = post_process(&opts, &req, make_response())?;
        assert_eq!(get_allowed_origin(resp), None);

        Ok(())
    }

    #[test]
    fn test_non_cors_request() -> Result<(), Error> {
        let opts = RequestHandlerOpts {
            cors: make_cors_config(),
            ..Default::default()
        };
        let mut req = make_request("GET", "");

        assert!(pre_process(&opts, &mut req).is_none());

        let resp = post_process(&opts, &req, make_response())?;
        assert_eq!(get_allowed_origin(resp), None);

        Ok(())
    }

    #[test]
    fn test_forbidden_request() {
        let opts = RequestHandlerOpts {
            cors: make_cors_config(),
            ..Default::default()
        };

        assert!(is_403(pre_process(
            &opts,
            &mut make_request("GET", "https://example.info")
        )));
        assert!(is_403(pre_process(
            &opts,
            &mut make_request("OPTIONS", "https://example.com")
        )));

        let mut req = make_request("OPTIONS", "https://example.com");
        req.headers_mut()
            .insert("Access-Control-Request-Method", "POST".try_into().unwrap());
        assert!(is_403(pre_process(&opts, &mut req)));

        let mut req = make_request("OPTIONS", "https://example.com");
        req.headers_mut()
            .insert("Access-Control-Request-Method", "GET".try_into().unwrap());
        req.headers_mut().insert(
            "Access-Control-Request-Headers",
            "X-Forbidden".try_into().unwrap(),
        );
        assert!(is_403(pre_process(&opts, &mut req)));
    }

    #[test]
    fn test_allowed_request() -> Result<(), Error> {
        let opts = RequestHandlerOpts {
            cors: make_cors_config(),
            ..Default::default()
        };

        let mut req = make_request("GET", "https://example.com");
        assert!(pre_process(&opts, &mut req).is_none());

        let resp = post_process(&opts, &req, make_response())?;
        assert_eq!(get_allowed_origin(resp), Some("https://example.com".into()));

        let mut req = make_request("GET", "https://example.com");
        req.headers_mut()
            .insert("Access-Control-Request-Method", "GET".try_into().unwrap());
        req.headers_mut().insert(
            "Access-Control-Request-Headers",
            "X-Allowed".try_into().unwrap(),
        );
        assert!(pre_process(&opts, &mut req).is_none());

        let resp = post_process(&opts, &req, make_response())?;
        assert_eq!(get_allowed_origin(resp), Some("https://example.com".into()));

        Ok(())
    }

    // Property-based regression tests for the CORS configuration
    // validators. These functions exist precisely to keep panicking
    // builder methods (`Cors::allow_origins` / `allow_headers`) away
    // from arbitrary admin-supplied tokens, so the property to enforce
    // is "never panic, and when we return `true` the builder must
    // accept the value".
    use super::{validate_header_names, validate_origin_str};
    use headers::{HeaderName, Origin};
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256, ..ProptestConfig::default()
        })]

        /// `validate_origin_str` MUST be total: it must never panic for
        /// any UTF-8 input. Additionally, when it returns `true`, the
        /// downstream `Origin::try_from_parts(scheme, rest, None)` call
        /// performed by `IntoOrigin for &str` MUST succeed.
        #[test]
        fn prop_validate_origin_str_never_panics(origin in "\\PC{0,128}") {
            let ok = validate_origin_str("cors.allow_origins", &origin);
            if ok {
                let (scheme, rest) = origin.split_once("://").unwrap();
                prop_assert!(
                    Origin::try_from_parts(scheme, rest, None).is_ok(),
                    "validator accepted `{origin}` but `Origin::try_from_parts` rejects it"
                );
            }
        }

        /// `validate_origin_str` rejects any string that does not match
        /// the `scheme://rest` shape, regardless of payload.
        #[test]
        fn prop_validate_origin_str_rejects_missing_scheme(s in "[^/:\\s]{0,32}") {
            prop_assert!(
                !validate_origin_str("cors.allow_origins", &s),
                "validator accepted `{s}` which has no `scheme://` separator"
            );
        }

        /// `validate_header_names` MUST be total and may only retain
        /// entries that `HeaderName::try_from` actually accepts.
        #[test]
        fn prop_validate_header_names_retains_only_valid(
            names in proptest::collection::vec("\\PC{0,32}", 0..16),
        ) {
            let slice: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            let kept = validate_header_names("cors.allow_headers", &slice);
            for h in &kept {
                prop_assert!(
                    HeaderName::try_from(*h).is_ok(),
                    "validator kept invalid header name `{h}`"
                );
            }
            // Filtering must be order-preserving and idempotent.
            let kept2 = validate_header_names("cors.allow_headers", &kept);
            prop_assert_eq!(kept, kept2);
        }
    }
}
