// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Native plugin support (host side).
//!
//! This module is the safe bridge between the request pipeline and
//! [`sws_plugin_loader`]. All unsafe FFI lives in that crate; everything here is
//! ordinary safe Rust gated behind the `plugins-native` feature.
//!
//! Native plugins are **trusted** code: they run in the server process with full
//! privileges. Only load plugins from sources you control.
//!
//! - Pre-processing plugins run after rewrites and may short-circuit the request.
//! - Post-processing plugins run last and may amend the final response.
//! - Plugins run synchronously and in their configured order. Keep them fast.

use hyper::header::{CONTENT_LENGTH, HeaderValue};
use hyper::{Request, Response, StatusCode};

pub use sws_plugin_loader::PluginSpec;
use sws_plugin_loader::{PluginManager, PreOutcome, RequestView, ResponseEdit, StagedResponse};

use crate::body::{self, Body};
use crate::{Context, Error, Result, error_page, handler::RequestHandlerOpts};

/// Loaded native plugins, ready to run for each request.
pub struct Plugins {
    manager: PluginManager,
}

impl Plugins {
    /// Loads every plugin described by `specs`, in order.
    ///
    /// Fails closed: if any plugin cannot be loaded, startup is aborted.
    pub fn load(specs: &[PluginSpec]) -> Result<Self> {
        let manager = PluginManager::load_all(specs)
            .with_context(|| "failed to load one or more native plugins")?;
        Ok(Self { manager })
    }

    /// Returns `true` when no plugins are loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.manager.is_empty()
    }
}

/// Builds a borrowed request view for the plugin hooks.
fn request_view<'a, T>(req: &'a Request<T>) -> RequestView<'a> {
    let uri = req.uri();
    RequestView {
        method: req.method().as_str(),
        path: uri.path(),
        query: uri.query(),
        headers: req.headers(),
    }
}

/// Turns a plugin-staged response into a real HTTP response.
fn build_response(staged: StagedResponse) -> Response<Body> {
    let StagedResponse {
        status,
        mut headers,
        body,
    } = staged;
    let len = body.len() as u64;
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    headers.insert(CONTENT_LENGTH, HeaderValue::from(len));
    let mut resp = Response::new(body::full(body));
    *resp.status_mut() = status;
    *resp.headers_mut() = headers;
    resp
}

/// Runs the pre-processing plugin chain.
///
/// Returns `Some` when a plugin handled the request (short-circuit or error),
/// `None` to continue the pipeline.
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    let plugins = opts.plugins.as_ref()?;
    if plugins.is_empty() {
        return None;
    }

    let view = request_view(req);
    match plugins.manager.pre_process(&view) {
        PreOutcome::Continue => None,
        PreOutcome::ShortCircuit(staged) => Some(Ok(build_response(staged))),
        PreOutcome::Error(name) => {
            tracing::error!(plugin = %name, "native plugin failed during pre-processing");
            Some(error_page::error_response(
                req.uri(),
                req.method(),
                &StatusCode::INTERNAL_SERVER_ERROR,
                &opts.page404,
                &opts.page50x,
            ))
        }
    }
}

/// Runs the post-processing plugin chain, amending the response in place.
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    let Some(plugins) = opts.plugins.as_ref() else {
        return Ok(resp);
    };
    if plugins.is_empty() {
        return Ok(resp);
    }

    let view = request_view(req);

    let mut new_status = resp.status().as_u16();
    let new_body;
    {
        let mut edit = ResponseEdit {
            status: new_status,
            headers: resp.headers_mut(),
            body: None,
        };
        plugins.manager.post_process(&view, &mut edit);
        new_status = edit.status;
        new_body = edit.body.take();
    }

    if let Ok(code) = StatusCode::from_u16(new_status) {
        *resp.status_mut() = code;
    }
    if let Some(bytes) = new_body {
        let len = bytes.len() as u64;
        *resp.body_mut() = body::full(bytes);
        resp.headers_mut()
            .insert(CONTENT_LENGTH, HeaderValue::from(len));
    }

    Ok(resp)
}
