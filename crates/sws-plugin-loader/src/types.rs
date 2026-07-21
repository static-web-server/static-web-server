// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Public data types exchanged between the SWS host and the plugin manager.

use http::HeaderMap;
use std::path::PathBuf;

/// Declarative description of a plugin to load.
#[derive(Debug, Clone)]
pub struct PluginSpec {
    /// Path to the plugin shared library (`.so`/`.dll`/`.dylib`).
    pub path: PathBuf,
    /// Opaque configuration string handed to the plugin's constructor.
    pub config: String,
}

/// A borrowed view of the request, passed to the plugin hooks.
pub struct RequestView<'a> {
    /// Request method, e.g. `GET`.
    pub method: &'a str,
    /// Decoded request path.
    pub path: &'a str,
    /// Raw query string, if any.
    pub query: Option<&'a str>,
    /// Request headers.
    pub headers: &'a HeaderMap,
}

/// A response staged by a plugin that short-circuited the request pipeline.
#[derive(Debug)]
pub struct StagedResponse {
    /// Status code.
    pub status: u16,
    /// Response headers.
    pub headers: HeaderMap,
    /// Response body bytes.
    pub body: Vec<u8>,
}

/// Outcome of running the pre-processing chain.
#[derive(Debug)]
pub enum PreOutcome {
    /// No plugin handled the request; continue the pipeline.
    Continue,
    /// A plugin produced a response; send it and stop.
    ShortCircuit(StagedResponse),
    /// A plugin failed; the host should emit a 500. Carries the plugin name.
    Error(String),
}

/// A mutable handle to the response, passed to the post-processing chain.
///
/// `headers` are edited in place. `status` and `body` are read back by the host
/// after the chain runs.
pub struct ResponseEdit<'a> {
    /// Status code (in/out).
    pub status: u16,
    /// Response headers, edited in place.
    pub headers: &'a mut HeaderMap,
    /// Replacement body, set by a plugin via `set_body` (out).
    pub body: Option<Vec<u8>>,
}
