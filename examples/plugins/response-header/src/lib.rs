// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.

//! Example native plugin: appends a custom header to every response.
//!
//! The header is read from the plugin configuration as `Name: Value`:
//!
//! ```toml
//! [[advanced.plugins]]
//! path = "target/release/libsws_plugin_response_header.so"
//! config = "X-Powered-By: Static Web Server"
//! ```

use sws_plugin_api::{Plugin, PluginRequest, PluginResponse, declare_plugin};

/// Adds a fixed `name: value` header to each response.
struct ResponseHeader {
    name: String,
    value: String,
}

impl ResponseHeader {
    fn new(config: &str) -> Result<Self, String> {
        let (name, value) = config
            .split_once(':')
            .ok_or_else(|| format!("expected `Name: Value`, got {config:?}"))?;
        let name = name.trim().to_owned();
        let value = value.trim().to_owned();
        if name.is_empty() {
            return Err("header name must not be empty".to_owned());
        }
        Ok(Self { name, value })
    }
}

impl Plugin for ResponseHeader {
    fn on_response(&self, _req: &PluginRequest<'_>, resp: &mut PluginResponse<'_>) {
        resp.set_header(&self.name, self.value.as_bytes());
    }
}

declare_plugin!("response-header", ResponseHeader, ResponseHeader::new);
