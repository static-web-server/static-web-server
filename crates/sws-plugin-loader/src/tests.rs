// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

#![cfg(test)]

use std::path::Path;

use http::{HeaderMap, HeaderValue};
use sws_plugin_api::{
    ABI_VERSION, Action, FfiSlice, Plugin, PluginDecl, PluginRequest, PluginResponse,
    declare_plugin,
};

use crate::error::LoadError;
use crate::manager::{LoadedPlugin, PluginManager};
use crate::types::{PreOutcome, RequestView, ResponseEdit, StagedResponse};

struct TestPlugin {
    token: String,
}

impl TestPlugin {
    fn new(config: &str) -> Result<Self, std::convert::Infallible> {
        Ok(Self {
            token: config.to_owned(),
        })
    }
}

impl Plugin for TestPlugin {
    fn on_request(&self, req: &PluginRequest<'_>, resp: &mut PluginResponse<'_>) -> Action {
        if req.path() == "/blocked" {
            resp.set_status(403);
            resp.set_header("content-type", b"text/plain");
            resp.set_body(b"forbidden");
            return Action::ShortCircuit;
        }
        // Token gate: require a matching `x-token` header.
        if !self.token.is_empty() && req.header("x-token") != Some(self.token.as_bytes()) {
            resp.set_status(401);
            return Action::ShortCircuit;
        }
        Action::Continue
    }

    fn on_response(&self, _req: &PluginRequest<'_>, resp: &mut PluginResponse<'_>) {
        resp.set_header("x-plugin", b"on");
    }
}

declare_plugin!("test-plugin", TestPlugin, TestPlugin::new);

// The macro exports `sws_plugin_declare` with C linkage; reference it the same
// way the real loader resolves it from a shared library.
unsafe extern "C" {
    fn sws_plugin_declare() -> *const PluginDecl;
}

// Builds a `LoadedPlugin` from the in-process descriptor (no real dlopen),
// exercising the full macro glue + host vtable end to end.
fn load_in_process(config: &str) -> LoadedPlugin {
    // SAFETY: the descriptor is a process-static produced by `declare_plugin!`.
    let decl = unsafe { &*sws_plugin_declare() };
    // SAFETY: the descriptor's functions live for the whole process.
    unsafe { LoadedPlugin::from_decl(Path::new("<test>"), decl, config, None) }
        .expect("load test plugin")
}

fn manager(config: &str) -> PluginManager {
    PluginManager {
        plugins: vec![load_in_process(config)],
    }
}

#[test]
fn pre_process_short_circuits_blocked_path() {
    let mgr = manager("");
    let headers = HeaderMap::new();
    let view = RequestView {
        method: "GET",
        path: "/blocked",
        query: None,
        headers: &headers,
    };
    match mgr.pre_process(&view) {
        PreOutcome::ShortCircuit(resp) => {
            assert_eq!(resp.status, 403);
            assert_eq!(resp.body, b"forbidden");
            assert_eq!(
                resp.headers.get("content-type").map(|v| v.as_bytes()),
                Some(b"text/plain".as_slice())
            );
        }
        other => panic!("expected short-circuit, got {other:?}"),
    }
}

#[test]
fn pre_process_continues_allowed_path() {
    let mgr = manager("");
    let headers = HeaderMap::new();
    let view = RequestView {
        method: "GET",
        path: "/ok",
        query: None,
        headers: &headers,
    };
    assert!(matches!(mgr.pre_process(&view), PreOutcome::Continue));
}

#[test]
fn pre_process_token_gate() {
    let mgr = manager("s3cret");
    let mut headers = HeaderMap::new();

    // Missing token -> 401 short-circuit.
    let view = RequestView {
        method: "GET",
        path: "/ok",
        query: None,
        headers: &headers,
    };
    assert!(matches!(
        mgr.pre_process(&view),
        PreOutcome::ShortCircuit(StagedResponse { status: 401, .. })
    ));

    // Correct token -> continue.
    headers.insert("x-token", HeaderValue::from_static("s3cret"));
    let view = RequestView {
        method: "GET",
        path: "/ok",
        query: None,
        headers: &headers,
    };
    assert!(matches!(mgr.pre_process(&view), PreOutcome::Continue));
}

#[test]
fn post_process_sets_header() {
    let mgr = manager("");
    let req_headers = HeaderMap::new();
    let mut resp_headers = HeaderMap::new();
    let view = RequestView {
        method: "GET",
        path: "/ok",
        query: None,
        headers: &req_headers,
    };
    let mut edit = ResponseEdit {
        status: 200,
        headers: &mut resp_headers,
        body: None,
    };
    mgr.post_process(&view, &mut edit);
    assert_eq!(
        resp_headers.get("x-plugin").map(|v| v.as_bytes()),
        Some(b"on".as_slice())
    );
}

#[test]
fn abi_mismatch_is_rejected() {
    let bad = PluginDecl {
        abi_version: ABI_VERSION + 1,
        name: FfiSlice::from_str("bad"),
        init: None,
        on_request: None,
        on_response: None,
        destroy: None,
    };
    // SAFETY: descriptor is valid; only the version field is wrong.
    let result = unsafe { LoadedPlugin::from_decl(Path::new("<bad>"), &bad, "", None) };
    assert!(matches!(result, Err(LoadError::AbiMismatch { .. })));
}
