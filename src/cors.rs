// CORS handler for incoming requests.
// -> Part of the file is borrowed from https://github.com/seanmonstar/warp/blob/master/src/filters/cors.rs

use headers::{HeaderName, HeaderValue, Origin};
use http::header;
use std::{collections::HashSet, convert::TryFrom, sync::Arc};

/// It defines CORS instance.
#[derive(Clone, Debug)]
pub struct Cors {
    allowed_headers: HashSet<HeaderName>,
    max_age: Option<u64>,
    allowed_methods: HashSet<http::Method>,
    origins: Option<HashSet<HeaderValue>>,
}

/// It builds a new CORS instance.
pub fn new(origins_str: String) -> Option<Arc<Configured>> {
    let cors = Cors::new();
    let cors = if origins_str.is_empty() {
        None
    } else if origins_str == "*" {
        Some(cors.allow_any_origin().allow_methods(vec!["GET", "HEAD"]))
    } else {
        let hosts = origins_str.split(',').map(|s| s.trim()).collect::<Vec<_>>();
        if hosts.is_empty() {
            None
        } else {
            Some(cors.allow_origins(hosts).allow_methods(vec!["GET", "HEAD"]))
        }
    };
    if cors.is_some() {
        tracing::info!(
            "enabled=true, allow_methods=[GET, HEAD], allow_origins={}",
            origins_str
        );
    }
    Cors::build(cors)
}

impl Cors {
    /// Creates a new Cors instance.
    pub fn new() -> Self {
        Self {
            origins: None,
            allowed_headers: HashSet::new(),
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

    /// Sets the `Access-Control-Max-Age` header.
    /// TODO: we could enable this in the future.
    pub fn max_age(mut self, seconds: impl Seconds) -> Self {
        self.max_age = Some(seconds.seconds());
        self
    }

    /// Builds the `Cors` wrapper from the configured settings.
    pub fn build(cors: Option<Cors>) -> Option<Arc<Configured>> {
        cors.as_ref()?;
        let cors = cors?;
        Some(Arc::new(Configured { cors }))
    }
}

impl Default for Cors {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct Configured {
    cors: Cors,
}

#[derive(Debug)]
pub enum Validated {
    Preflight(HeaderValue),
    Simple(HeaderValue),
    NotCors,
}

#[derive(Debug)]
pub enum Forbidden {
    Origin,
    Method,
    Header,
}

impl Default for Forbidden {
    fn default() -> Self {
        Self::Origin
    }
}

impl Configured {
    pub fn check_request(
        &self,
        method: &http::Method,
        headers: &http::HeaderMap,
    ) -> Result<Validated, Forbidden> {
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

                Ok(Validated::Preflight(origin.clone()))
            }
            (Some(origin), _) => {
                // Any other method, simply check for a valid origin...
                tracing::trace!("cors origin header: {:?}", origin);

                if self.is_origin_allowed(origin) {
                    Ok(Validated::Simple(origin.clone()))
                } else {
                    Err(Forbidden::Origin)
                }
            }
            (None, _) => {
                // No `ORIGIN` header means this isn't CORS!
                Ok(Validated::NotCors)
            }
        }
    }

    pub fn is_method_allowed(&self, header: &HeaderValue) -> bool {
        http::Method::from_bytes(header.as_bytes())
            .map(|method| self.cors.allowed_methods.contains(&method))
            .unwrap_or(false)
    }

    pub fn is_header_allowed(&self, header: &str) -> bool {
        HeaderName::from_bytes(header.as_bytes())
            .map(|header| self.cors.allowed_headers.contains(&header))
            .unwrap_or(false)
    }

    pub fn is_origin_allowed(&self, origin: &HeaderValue) -> bool {
        if let Some(ref allowed) = self.cors.origins {
            allowed.contains(origin)
        } else {
            true
        }
    }
}

pub trait Seconds {
    fn seconds(self) -> u64;
}

impl Seconds for u32 {
    fn seconds(self) -> u64 {
        self.into()
    }
}

impl Seconds for ::std::time::Duration {
    fn seconds(self) -> u64 {
        self.as_secs()
    }
}

pub trait IntoOrigin {
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
