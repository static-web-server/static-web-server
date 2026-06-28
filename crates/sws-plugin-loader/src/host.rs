// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Host-side FFI: the request/response context structs, the host vtable
//! functions, and the shared [`HOST_VTABLE`] handed to every plugin call.

use http::{HeaderMap, HeaderName, HeaderValue};
use sws_plugin_api::{ABI_VERSION, FfiSlice, HostVTable, RequestHandle, ResponseHandle};

/// Host-owned request context that an opaque [`RequestHandle`] points to.
pub(super) struct ReqCtx<'a> {
    /// Request method, e.g. `GET`.
    pub method: &'a str,
    /// Decoded request path.
    pub path: &'a str,
    /// Raw query string, if any.
    pub query: Option<&'a str>,
    /// Request headers.
    pub headers: &'a HeaderMap,
}

/// Host-owned response context that an opaque [`ResponseHandle`] points to.
pub(super) struct RespCtx<'a> {
    /// Status code.
    pub status: u16,
    /// Response headers, edited in place.
    pub headers: &'a mut HeaderMap,
    /// Replacement body, set by a plugin via `set_body`.
    pub body: Option<Vec<u8>>,
}

extern "C" fn req_method(h: RequestHandle) -> FfiSlice {
    // SAFETY: `h` points at a live `ReqCtx` for the duration of this call.
    let ctx = unsafe { &*(h as *const ReqCtx<'_>) };
    FfiSlice::from_str(ctx.method)
}

extern "C" fn req_path(h: RequestHandle) -> FfiSlice {
    // SAFETY: `h` points at a live `ReqCtx` for the duration of this call.
    let ctx = unsafe { &*(h as *const ReqCtx<'_>) };
    FfiSlice::from_str(ctx.path)
}

extern "C" fn req_query(h: RequestHandle) -> FfiSlice {
    // SAFETY: `h` points at a live `ReqCtx` for the duration of this call.
    let ctx = unsafe { &*(h as *const ReqCtx<'_>) };
    match ctx.query {
        Some(q) => FfiSlice::from_str(q),
        None => FfiSlice::NULL,
    }
}

extern "C" fn req_header(h: RequestHandle, name: FfiSlice) -> FfiSlice {
    // SAFETY: `h` points at a live `ReqCtx` for the duration of this call.
    let ctx = unsafe { &*(h as *const ReqCtx<'_>) };
    // SAFETY: `name` is valid for the duration of this call.
    let Some(name) = (unsafe { name.as_str() }) else {
        return FfiSlice::NULL;
    };
    match ctx.headers.get(name) {
        Some(v) => FfiSlice::from_bytes(v.as_bytes()),
        None => FfiSlice::NULL,
    }
}

extern "C" fn resp_status(h: ResponseHandle) -> u16 {
    // SAFETY: `h` points at a live `RespCtx` for the duration of this call.
    let ctx = unsafe { &*(h as *const RespCtx<'_>) };
    ctx.status
}

extern "C" fn resp_set_status(h: ResponseHandle, status: u16) {
    // SAFETY: `h` points at a live `RespCtx` and no other reference to it is
    // active while this call runs (plugins call host functions sequentially).
    let ctx = unsafe { &mut *(h as *mut RespCtx<'_>) };
    ctx.status = status;
}

extern "C" fn resp_header(h: ResponseHandle, name: FfiSlice) -> FfiSlice {
    // SAFETY: `h` points at a live `RespCtx` for the duration of this call.
    let ctx = unsafe { &*(h as *const RespCtx<'_>) };
    // SAFETY: `name` is valid for the duration of this call.
    let Some(name) = (unsafe { name.as_str() }) else {
        return FfiSlice::NULL;
    };
    match ctx.headers.get(name) {
        Some(v) => FfiSlice::from_bytes(v.as_bytes()),
        None => FfiSlice::NULL,
    }
}

extern "C" fn resp_set_header(h: ResponseHandle, name: FfiSlice, value: FfiSlice) -> bool {
    // SAFETY: `h` points at a live `RespCtx` and no other reference to it is
    // active while this call runs (plugins call host functions sequentially).
    let ctx = unsafe { &mut *(h as *mut RespCtx<'_>) };
    // SAFETY: `name`/`value` are valid for the duration of this call.
    let (Some(name), Some(value)) = (unsafe { name.as_bytes() }, unsafe { value.as_bytes() })
    else {
        return false;
    };
    let (Ok(name), Ok(value)) = (HeaderName::from_bytes(name), HeaderValue::from_bytes(value))
    else {
        return false;
    };
    ctx.headers.insert(name, value);
    true
}

extern "C" fn resp_set_body(h: ResponseHandle, body: FfiSlice) {
    // SAFETY: `h` points at a live `RespCtx` and no other reference to it is
    // active while this call runs (plugins call host functions sequentially).
    let ctx = unsafe { &mut *(h as *mut RespCtx<'_>) };
    // SAFETY: `body` is valid for the duration of this call.
    if let Some(bytes) = unsafe { body.as_bytes() } {
        ctx.body = Some(bytes.to_vec());
    }
}

/// The single shared host function table handed to every plugin call.
pub(super) static HOST_VTABLE: HostVTable = HostVTable {
    abi_version: ABI_VERSION,
    req_method,
    req_path,
    req_query,
    req_header,
    resp_status,
    resp_set_status,
    resp_header,
    resp_set_header,
    resp_set_body,
};
