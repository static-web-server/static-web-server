// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Safe, ergonomic Rust wrappers a plugin author uses to implement a plugin.
//!
//! Authors implement [`Plugin`] and register it with
//! [`crate::declare_plugin!`]; the macro generates the `#[repr(C)]` glue and the
//! exported entry symbol.

use crate::abi::{HostVTable, RequestHandle, ResponseHandle};

/// The decision a [`Plugin::on_request`] hook returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Continue the request pipeline.
    Continue,
    /// Stop the pipeline and return the response the plugin staged.
    ShortCircuit,
}

/// A safe, borrowed view over the host's request for the current call.
pub struct PluginRequest<'a> {
    host: &'a HostVTable,
    handle: RequestHandle,
}

impl<'a> PluginRequest<'a> {
    /// Builds a request view from raw ABI parts.
    ///
    /// This is an implementation detail used by [`crate::declare_plugin!`]; you
    /// should not need to call it directly.
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn from_raw(host: &'a HostVTable, handle: RequestHandle) -> Self {
        Self { host, handle }
    }

    /// The request method, e.g. `GET`.
    #[must_use]
    pub fn method(&self) -> &str {
        // SAFETY: the host keeps the returned span valid for this call.
        unsafe { (self.host.req_method)(self.handle).as_str() }.unwrap_or("")
    }

    /// The decoded request path, e.g. `/index.html`.
    #[must_use]
    pub fn path(&self) -> &str {
        // SAFETY: the host keeps the returned span valid for this call.
        unsafe { (self.host.req_path)(self.handle).as_str() }.unwrap_or("")
    }

    /// The raw query string, if present.
    #[must_use]
    pub fn query(&self) -> Option<&str> {
        // SAFETY: the host keeps the returned span valid for this call.
        unsafe { (self.host.req_query)(self.handle).as_str() }
    }

    /// The first value of the named request header, if present.
    ///
    /// Header names are case-insensitive.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&[u8]> {
        // SAFETY: the host keeps the returned span valid for this call.
        unsafe {
            (self.host.req_header)(self.handle, crate::abi::FfiSlice::from_str(name)).as_bytes()
        }
    }
}

/// A safe, borrowed view over the host's response for the current call.
///
/// For `on_request` this wraps a freshly staged response; returning
/// [`Action::ShortCircuit`] tells the host to send it. For `on_response` it wraps
/// the real response about to be sent.
pub struct PluginResponse<'a> {
    host: &'a HostVTable,
    handle: ResponseHandle,
}

impl<'a> PluginResponse<'a> {
    /// Builds a response view from raw ABI parts.
    ///
    /// This is an implementation detail used by [`crate::declare_plugin!`]; you
    /// should not need to call it directly.
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn from_raw(host: &'a HostVTable, handle: ResponseHandle) -> Self {
        Self { host, handle }
    }

    /// The current response status code.
    #[must_use]
    pub fn status(&self) -> u16 {
        (self.host.resp_status)(self.handle)
    }

    /// Sets the response status code.
    pub fn set_status(&mut self, status: u16) {
        (self.host.resp_set_status)(self.handle, status);
    }

    /// The first value of the named response header, if present.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&[u8]> {
        // SAFETY: the host keeps the returned span valid for this call.
        unsafe {
            (self.host.resp_header)(self.handle, crate::abi::FfiSlice::from_str(name)).as_bytes()
        }
    }

    /// Inserts or replaces a response header.
    ///
    /// Returns `false` when the name or value is not a valid HTTP header.
    pub fn set_header(&mut self, name: &str, value: &[u8]) -> bool {
        (self.host.resp_set_header)(
            self.handle,
            crate::abi::FfiSlice::from_str(name),
            crate::abi::FfiSlice::from_bytes(value),
        )
    }

    /// Replaces the response body with the given bytes.
    ///
    /// The host copies the bytes, so `body` need not outlive the call.
    pub fn set_body(&mut self, body: &[u8]) {
        (self.host.resp_set_body)(self.handle, crate::abi::FfiSlice::from_bytes(body));
    }
}

/// The trait a native plugin implements.
///
/// Both hooks are optional (they default to no-ops). Implementations must be
/// `Send + Sync`: the host shares a single instance across worker threads and may
/// invoke the hooks concurrently.
///
/// Hooks are synchronous. Keep them fast and non-blocking — a slow hook stalls
/// the async worker thread handling that request.
pub trait Plugin: Send + Sync + 'static {
    /// Pre-processing hook, run before the file is served.
    ///
    /// Return [`Action::ShortCircuit`] after staging a response on `resp` to stop
    /// the pipeline and send it; return [`Action::Continue`] to proceed.
    fn on_request(&self, _req: &PluginRequest<'_>, _resp: &mut PluginResponse<'_>) -> Action {
        Action::Continue
    }

    /// Post-processing hook, run after a response is produced.
    ///
    /// Inspect or mutate `resp` (status, headers, body) before it is sent.
    fn on_response(&self, _req: &PluginRequest<'_>, _resp: &mut PluginResponse<'_>) {}
}
