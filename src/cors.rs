//! CORS module to handle incoming requests.
//!

// Part of the file is borrowed from https://github.com/seanmonstar/warp/blob/master/src/filters/cors.rs

use headers::{
    AccessControlAllowHeaders, AccessControlAllowMethods, AccessControlExposeHeaders, HeaderMapExt,
    HeaderName, HeaderValue, Origin,
};
use http::header;
use std::{collections::HashSet, convert::TryFrom};

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
        let [allow_headers_str, expose_headers_str] =
            [&allow_headers_vec, &expose_headers_vec].map(|v| v.join(","));

        let cors_res = if origins_str == "*" {
            Some(
                cors.allow_any_origin()
                    .allow_headers(allow_headers_vec)
                    .expose_headers(expose_headers_vec)
                    .allow_methods(vec!["GET", "HEAD", "OPTIONS"]),
            )
        } else {
            let hosts = origins_str.split(',').map(|s| s.trim()).collect::<Vec<_>>();
            if hosts.is_empty() {
                None
            } else {
                Some(
                    cors.allow_origins(hosts)
                        .allow_headers(allow_headers_vec)
                        .expose_headers(expose_headers_vec)
                        .allow_methods(vec!["GET", "HEAD", "OPTIONS"]),
                )
            }
        };

        if cors_res.is_some() {
            tracing::info!(
                    "enabled=true, allow_methods=[GET,HEAD,OPTIONS], allow_origins={}, allow_headers=[{}], expose_headers=[{}]",
                    origins_str,
                    allow_headers_str,
                    expose_headers_str,
                );
        }
        cors_res
    };

    Cors::build(cors)
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
    ///
    /// # Panics
    ///
    /// Panics if the provided argument is not a valid `http::Method`.
    pub fn allow_methods<I>(mut self, methods: I) -> Self
    where
        I: IntoIterator,
        http::Method: TryFrom<I::Item>,
    {
        let iter = methods.into_iter().map(|m| match TryFrom::try_from(m) {
            Ok(m) => m,
            Err(_) => panic!("cors: illegal method"),
        });
        self.allowed_methods.extend(iter);
        self
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
    ///
    /// # Panics
    ///
    /// Panics if the provided argument is not a valid `Origin`.
    pub fn allow_origins<I>(mut self, origins: I) -> Self
    where
        I: IntoIterator,
        I::Item: IntoOrigin,
    {
        let iter = origins
            .into_iter()
            .map(IntoOrigin::into_origin)
            .map(|origin| {
                origin
                    .to_string()
                    .parse()
                    .expect("cors: Origin is always a valid HeaderValue")
            });

        self.origins.get_or_insert_with(HashSet::new).extend(iter);
        self
    }

    /// Adds multiple headers to the list of allowed request headers.
    ///
    /// **Note**: These should match the values the browser sends via `Access-Control-Request-Headers`, e.g.`content-type`.
    ///
    /// # Panics
    ///
    /// Panics if any of the headers are not a valid `http::header::HeaderName`.
    pub fn allow_headers<I>(mut self, headers: I) -> Self
    where
        I: IntoIterator,
        HeaderName: TryFrom<I::Item>,
    {
        let iter = headers.into_iter().map(|h| match TryFrom::try_from(h) {
            Ok(h) => h,
            Err(_) => panic!("cors: illegal Header"),
        });
        self.allowed_headers.extend(iter);
        self
    }

    /// Adds multiple headers to the list of exposed request headers.
    ///
    /// **Note**: These should match the values the browser sends via `Access-Control-Request-Headers`, e.g.`content-type`.
    ///
    /// # Panics
    ///
    /// Panics if any of the headers are not a valid `http::header::HeaderName`.
    pub fn expose_headers<I>(mut self, headers: I) -> Self
    where
        I: IntoIterator,
        HeaderName: TryFrom<I::Item>,
    {
        let iter = headers.into_iter().map(|h| match TryFrom::try_from(h) {
            Ok(h) => h,
            Err(_) => panic!("cors: illegal Header"),
        });
        self.exposed_headers.extend(iter);
        self
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
/// CORS is configurated.
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

#[derive(Debug)]
/// Forbidden errors.
pub enum Forbidden {
    /// Forbidden error origin.
    Origin,
    /// Forbidden error method.
    Method,
    /// Forbidden error header.
    Header,
}

impl Default for Forbidden {
    fn default() -> Self {
        Self::Origin
    }
}

impl Configured {
    /// Check for the incoming CORS request.
    pub fn check_request(
        &self,
        method: &http::Method,
        headers: &http::HeaderMap,
    ) -> Result<(http::HeaderMap, Validated), Forbidden> {
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
                    tracing::trace!(
                        "cors: preflight request missing access-control-request-method header"
                    );
                    return Err(Forbidden::Method);
                }

                if let Some(req_headers) = headers.get(header::ACCESS_CONTROL_REQUEST_HEADERS) {
                    let headers = req_headers.to_str().map_err(|_| Forbidden::Header)?;
                    for header in headers.split(',') {
                        if !self.is_header_allowed(header.trim()) {
                            return Err(Forbidden::Header);
                        }
                    }
                }

                let mut headers = http::HeaderMap::new();
                self.append_preflight_headers(&mut headers);
                headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.into());

                Ok((headers, Validated::Preflight(origin.clone())))
            }
            (Some(origin), _) => {
                // Any other method, simply check for a valid origin...
                tracing::trace!("cors origin header: {:?}", origin);

                if self.is_origin_allowed(origin) {
                    let mut headers = http::HeaderMap::new();
                    self.append_preflight_headers(&mut headers);
                    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.into());

                    Ok((headers, Validated::Simple(origin.clone())))
                } else {
                    Err(Forbidden::Origin)
                }
            }
            (None, _) => {
                // No `ORIGIN` header means this isn't CORS!
                Ok((http::HeaderMap::new(), Validated::NotCors))
            }
        }
    }

    fn is_method_allowed(&self, header: &HeaderValue) -> bool {
        http::Method::from_bytes(header.as_bytes())
            .map(|method| self.cors.allowed_methods.contains(&method))
            .unwrap_or(false)
    }

    fn is_header_allowed(&self, header: &str) -> bool {
        HeaderName::from_bytes(header.as_bytes())
            .map(|header| self.cors.allowed_headers.contains(&header))
            .unwrap_or(false)
    }

    fn is_origin_allowed(&self, origin: &HeaderValue) -> bool {
        if let Some(ref allowed) = self.cors.origins {
            allowed.contains(origin)
        } else {
            true
        }
    }

    fn append_preflight_headers(&self, headers: &mut http::HeaderMap) {
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
    fn into_origin(self) -> Origin;
}

impl<'a> IntoOrigin for &'a str {
    fn into_origin(self) -> Origin {
        let mut parts = self.splitn(2, "://");
        let scheme = parts.next().expect("cors::into_origin: missing url scheme");
        let rest = parts.next().expect("cors::into_origin: missing url scheme");

        Origin::try_from_parts(scheme, rest, None).expect("cors::into_origin: invalid Origin")
    }
}
