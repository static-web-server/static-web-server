// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! HTTP-related extension traits.

use hyper::{Method, Response, header::HeaderValue};

/// A fixed list of HTTP methods supported by SWS.
pub const HTTP_SUPPORTED_METHODS: &[Method; 3] = &[Method::OPTIONS, Method::HEAD, Method::GET];

/// SWS HTTP Method extensions trait.
pub trait MethodExt {
    /// If method is allowed.
    fn is_allowed(&self) -> bool;
    /// If method is `GET`.
    #[allow(unused)]
    fn is_get(&self) -> bool;
    /// If method is `HEAD`.
    fn is_head(&self) -> bool;
    /// If method is `OPTIONS`.
    fn is_options(&self) -> bool;
}

impl MethodExt for Method {
    /// Checks if the HTTP method is allowed (supported) by SWS.
    #[inline(always)]
    fn is_allowed(&self) -> bool {
        for method in HTTP_SUPPORTED_METHODS {
            if method == self {
                return true;
            }
        }
        false
    }

    /// Checks if the HTTP method is `GET`.
    #[inline(always)]
    fn is_get(&self) -> bool {
        self == Method::GET
    }

    /// Checks if the HTTP method is `HEAD`.
    #[inline(always)]
    fn is_head(&self) -> bool {
        self == Method::HEAD
    }

    /// Checks if the HTTP method is `OPTIONS`.
    #[inline(always)]
    fn is_options(&self) -> bool {
        self == Method::OPTIONS
    }
}

/// Pre-computed static Vary header value for accept-encoding.
static VARY_ACCEPT_ENCODING: HeaderValue = HeaderValue::from_static("accept-encoding");

/// Append `accept-encoding` to the response's `Vary` header, creating it if absent.
/// Skips the update if `accept-encoding` is already listed.
pub(crate) fn append_vary_accept_encoding<B>(resp: &mut Response<B>) {
    let accept_enc = hyper::header::ACCEPT_ENCODING.as_str();
    match resp.headers().get(hyper::header::VARY) {
        None => {
            resp.headers_mut()
                .insert(hyper::header::VARY, VARY_ACCEPT_ENCODING.clone());
        }
        Some(existing) => {
            let s = existing.to_str().unwrap_or_default();
            if s.contains(accept_enc) {
                return;
            }
            // Append to existing value
            let mut new_val = String::with_capacity(s.len() + 2 + accept_enc.len());
            new_val.push_str(s);
            if !s.is_empty() {
                new_val.push_str(", ");
            }
            new_val.push_str(accept_enc);
            if let Ok(val) = HeaderValue::from_str(&new_val) {
                resp.headers_mut().insert(hyper::header::VARY, val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hyper::{Method, Response, StatusCode};

    use super::{MethodExt, append_vary_accept_encoding};

    #[test]
    fn method_get_is_allowed() {
        assert!(Method::GET.is_allowed());
        assert!(Method::GET.is_get());
        assert!(!Method::GET.is_head());
        assert!(!Method::GET.is_options());
    }

    #[test]
    fn method_head_is_allowed() {
        assert!(Method::HEAD.is_allowed());
        assert!(Method::HEAD.is_head());
        assert!(!Method::HEAD.is_get());
        assert!(!Method::HEAD.is_options());
    }

    #[test]
    fn method_options_is_allowed() {
        assert!(Method::OPTIONS.is_allowed());
        assert!(Method::OPTIONS.is_options());
        assert!(!Method::OPTIONS.is_get());
        assert!(!Method::OPTIONS.is_head());
    }

    #[test]
    fn method_post_is_not_allowed() {
        assert!(!Method::POST.is_allowed());
        assert!(!Method::POST.is_get());
        assert!(!Method::POST.is_head());
        assert!(!Method::POST.is_options());
    }

    #[test]
    fn method_put_delete_patch_are_not_allowed() {
        for method in [Method::PUT, Method::DELETE, Method::PATCH] {
            assert!(!method.is_allowed(), "{method} should not be allowed");
        }
    }

    #[test]
    fn vary_added_when_absent() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;
        append_vary_accept_encoding(&mut resp);
        let vary = resp.headers().get(hyper::header::VARY).unwrap();
        assert_eq!(vary.to_str().unwrap(), "accept-encoding");
    }

    #[test]
    fn vary_not_duplicated_when_already_present() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;
        resp.headers_mut()
            .insert(hyper::header::VARY, "accept-encoding".parse().unwrap());
        append_vary_accept_encoding(&mut resp);
        let vary = resp.headers().get(hyper::header::VARY).unwrap();
        assert_eq!(vary.to_str().unwrap(), "accept-encoding");
    }

    #[test]
    fn vary_appended_to_existing_value() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;
        resp.headers_mut()
            .insert(hyper::header::VARY, "origin".parse().unwrap());
        append_vary_accept_encoding(&mut resp);
        let vary = resp.headers().get(hyper::header::VARY).unwrap();
        assert_eq!(vary.to_str().unwrap(), "origin, accept-encoding");
    }

    #[test]
    fn vary_not_duplicated_when_mixed_with_others() {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = StatusCode::OK;
        resp.headers_mut().insert(
            hyper::header::VARY,
            "origin, accept-encoding".parse().unwrap(),
        );
        append_vary_accept_encoding(&mut resp);
        let vary = resp.headers().get(hyper::header::VARY).unwrap();
        assert_eq!(vary.to_str().unwrap(), "origin, accept-encoding");
    }
}
