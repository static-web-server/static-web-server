// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Native plugin loader for Static Web Server (SWS).
//!
//! This crate is the *only* place in the project that performs unsafe FFI: it
//! `dlopen`s a plugin shared library, validates its ABI version, and bridges the
//! host's request/response data to the plugin through the
//! [`sws_plugin_api`] C ABI. The SWS server crate stays `#![forbid(unsafe_code)]`
//! and talks exclusively to the safe API exposed here.
//!
//! # Safety invariants
//!
//! 1. **ABI gate.** A plugin is loaded only when its declared ABI version equals
//!    [`sws_plugin_api::ABI_VERSION`]. A mismatch fails closed.
//! 2. **Library lifetime.** A loaded library is kept alive (`_library`) for as
//!    long as any of its function pointers may be called. The instance data is
//!    destroyed before the library is unloaded.
//! 3. **Handle scope.** The opaque request/response handles passed to a plugin
//!    point at stack-local context structs that live for exactly the duration of
//!    one hook call; they never escape it.
//! 4. **No unwinding across FFI.** Plugin glue (in `sws-plugin-api`) catches
//!    panics; the host glue here never panics across the boundary.
//! 5. **Trusted code.** Native plugins run with full process privileges. Loading
//!    one is an explicit, operator-controlled decision.

#![forbid(clippy::undocumented_unsafe_blocks)]
#![deny(rust_2018_idioms)]
#![deny(missing_docs)]

mod error;
mod host;
mod manager;
mod types;

#[cfg(test)]
mod tests;

pub use crate::error::LoadError;
pub use crate::manager::PluginManager;
pub use crate::types::{PluginSpec, PreOutcome, RequestView, ResponseEdit, StagedResponse};
