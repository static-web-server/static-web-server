// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.

//! Example native plugin: logs the method and path of every request.
//!
//! Build it as a shared library and point the server at the resulting file:
//!
//! ```toml
//! [[advanced.plugins]]
//! path = "target/release/libsws_plugin_request_logger.so"
//! ```

use std::convert::Infallible;

use sws_plugin_api::{Action, Plugin, PluginRequest, PluginResponse, declare_plugin};

/// A stateless request logger.
struct RequestLogger;

impl RequestLogger {
    fn new(_config: &str) -> Result<Self, Infallible> {
        Ok(Self)
    }
}

impl Plugin for RequestLogger {
    fn on_request(&self, req: &PluginRequest<'_>, _resp: &mut PluginResponse<'_>) -> Action {
        match req.query() {
            Some(query) => println!("[request-logger] {} {}?{}", req.method(), req.path(), query),
            None => println!("[request-logger] {} {}", req.method(), req.path()),
        }
        Action::Continue
    }
}

declare_plugin!("request-logger", RequestLogger, RequestLogger::new);
