// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! HTTP-related extension traits.

use hyper::Method;

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
