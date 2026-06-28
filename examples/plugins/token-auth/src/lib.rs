// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.

//! Example native plugin: rejects requests lacking a valid bearer token.
//!
//! The expected token is read from the plugin configuration. Requests must carry
//! an `Authorization: Bearer <token>` header; otherwise they receive `401`.
//!
//! ```toml
//! [[advanced.plugins]]
//! path = "target/release/libsws_plugin_token_auth.so"
//! config = "s3cret-token"
//! ```

use sws_plugin_api::{Action, Plugin, PluginRequest, PluginResponse, declare_plugin};

/// Validates a static bearer token on every request.
struct TokenAuth {
    expected: String,
}

impl TokenAuth {
    fn new(config: &str) -> Result<Self, String> {
        let expected = config.trim();
        if expected.is_empty() {
            return Err("a non-empty token must be configured".to_owned());
        }
        Ok(Self {
            expected: expected.to_owned(),
        })
    }

    fn is_authorized(&self, req: &PluginRequest<'_>) -> bool {
        let Some(value) = req.header("authorization") else {
            return false;
        };
        let Ok(value) = std::str::from_utf8(value) else {
            return false;
        };
        value
            .strip_prefix("Bearer ")
            .is_some_and(|token| token == self.expected)
    }
}

impl Plugin for TokenAuth {
    fn on_request(&self, req: &PluginRequest<'_>, resp: &mut PluginResponse<'_>) -> Action {
        if self.is_authorized(req) {
            return Action::Continue;
        }
        resp.set_status(401);
        resp.set_header("www-authenticate", b"Bearer");
        resp.set_header("content-type", b"text/plain; charset=utf-8");
        resp.set_body(b"401 Unauthorized");
        Action::ShortCircuit
    }
}

declare_plugin!("token-auth", TokenAuth, TokenAuth::new);
