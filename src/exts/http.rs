//! HTTP-related extension traits.

use hyper::Method;

/// A fixed list of HTTP methods supported by SWS.
pub const HTTP_SUPPORTED_METHODS: &[Method; 3] = &[Method::OPTIONS, Method::HEAD, Method::GET];

/// SWS HTTP Method extensions trait.
pub trait MethodExt {
    fn is_allowed(&self) -> bool;
    fn is_get(&self) -> bool;
    fn is_head(&self) -> bool;
    fn is_options(&self) -> bool;
}

impl MethodExt for Method {
    /// Checks if the HTTP method is allowed (supported) by SWS.
    fn is_allowed(&self) -> bool {
        HTTP_SUPPORTED_METHODS.iter().any(|h| self == h)
    }

    /// Checks if the HTTP method is `GET`.
    fn is_get(&self) -> bool {
        self == Method::GET
    }

    /// Checks if the HTTP method is `HEAD`.
    fn is_head(&self) -> bool {
        self == Method::HEAD
    }

    /// Checks if the HTTP method is `OPTIONS`.
    fn is_options(&self) -> bool {
        self == Method::OPTIONS
    }
}
