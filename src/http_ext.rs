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

/// Append `accept-encoding` to the response's `Vary` header, creating it if absent.
/// Skips the update if `accept-encoding` is already listed.
pub(crate) fn append_vary_accept_encoding<B>(resp: &mut Response<B>) {
    let accept_enc = hyper::header::ACCEPT_ENCODING.as_str();
    let value = resp.headers().get(hyper::header::VARY).map_or_else(
        || HeaderValue::from_name(hyper::header::ACCEPT_ENCODING),
        |existing| {
            let mut s = existing.to_str().unwrap_or_default().to_owned();
            if !s.contains(accept_enc) {
                if !s.is_empty() {
                    s.push_str(", ");
                }
                s.push_str(accept_enc);
            }
            HeaderValue::from_str(&s).unwrap()
        },
    );
    resp.headers_mut().insert(hyper::header::VARY, value);
}
