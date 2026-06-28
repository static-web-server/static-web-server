// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

#![cfg(test)]

use crate::abi::{ABI_VERSION, FfiSlice, HostVTable, RequestHandle, ResponseHandle};
use crate::plugin::PluginRequest;

// A fake host backed by a fixed request, exercising the read-only wrappers
// exactly as the real host would.
struct FakeReq {
    method: &'static str,
    path: &'static str,
    query: Option<&'static str>,
    header_name: &'static str,
    header_value: &'static [u8],
}

extern "C" fn req_method(h: RequestHandle) -> FfiSlice {
    let r = unsafe { &*(h as *const FakeReq) };
    FfiSlice::from_str(r.method)
}
extern "C" fn req_path(h: RequestHandle) -> FfiSlice {
    let r = unsafe { &*(h as *const FakeReq) };
    FfiSlice::from_str(r.path)
}
extern "C" fn req_query(h: RequestHandle) -> FfiSlice {
    let r = unsafe { &*(h as *const FakeReq) };
    match r.query {
        Some(q) => FfiSlice::from_str(q),
        None => FfiSlice::NULL,
    }
}
extern "C" fn req_header(h: RequestHandle, name: FfiSlice) -> FfiSlice {
    let r = unsafe { &*(h as *const FakeReq) };
    let name = unsafe { name.as_str() }.unwrap_or("");
    if name.eq_ignore_ascii_case(r.header_name) {
        FfiSlice::from_bytes(r.header_value)
    } else {
        FfiSlice::NULL
    }
}
extern "C" fn resp_status(_: ResponseHandle) -> u16 {
    200
}
extern "C" fn resp_set_status(_: ResponseHandle, _: u16) {}
extern "C" fn resp_header(_: ResponseHandle, _: FfiSlice) -> FfiSlice {
    FfiSlice::NULL
}
extern "C" fn resp_set_header(_: ResponseHandle, _: FfiSlice, _: FfiSlice) -> bool {
    true
}
extern "C" fn resp_set_body(_: ResponseHandle, _: FfiSlice) {}

fn fake_host() -> HostVTable {
    HostVTable {
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
    }
}

#[test]
fn ffi_slice_roundtrip_and_absence() {
    let s = FfiSlice::from_str("hello");
    assert!(!s.is_null());
    assert_eq!(unsafe { s.as_str() }, Some("hello"));
    assert!(FfiSlice::NULL.is_null());
    assert_eq!(unsafe { FfiSlice::NULL.as_bytes() }, None);
}

#[test]
fn request_wrapper_reads_fields() {
    let host = fake_host();
    let req = FakeReq {
        method: "GET",
        path: "/index.html",
        query: Some("a=1"),
        header_name: "x-token",
        header_value: b"secret",
    };
    let view = PluginRequest::from_raw(&host, (&req as *const FakeReq).cast::<core::ffi::c_void>());
    assert_eq!(view.method(), "GET");
    assert_eq!(view.path(), "/index.html");
    assert_eq!(view.query(), Some("a=1"));
    assert_eq!(view.header("X-Token"), Some(b"secret".as_slice()));
    assert_eq!(view.header("missing"), None);
}

#[test]
fn request_wrapper_absent_query() {
    let host = fake_host();
    let req = FakeReq {
        method: "POST",
        path: "/",
        query: None,
        header_name: "",
        header_value: b"",
    };
    let view = PluginRequest::from_raw(&host, (&req as *const FakeReq).cast::<core::ffi::c_void>());
    assert_eq!(view.query(), None);
}
