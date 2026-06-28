// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Stable FFI contract and authoring SDK for Static Web Server (SWS) native plugins.
//!
//! This crate defines the binary boundary shared by the SWS host and every
//! native plugin. It has two layers:
//!
//! - **ABI layer** ([`abi`] module: [`PluginDecl`], [`HostVTable`],
//!   [`CallContext`], [`FfiSlice`]): a small, `#[repr(C)]`, versioned C ABI.
//!   Only primitive integers and borrowed byte spans cross the boundary; the
//!   host's `hyper` request/response types never do. This layer is what makes
//!   plugins compiled separately ABI-compatible with the host.
//! - **Authoring layer** ([`plugin`] module: [`Plugin`], [`PluginRequest`],
//!   [`PluginResponse`], and the [`declare_plugin!`] macro): safe, ergonomic Rust
//!   wrappers a plugin author uses. Authors implement [`Plugin`] and register it
//!   with [`declare_plugin!`]; the macro generates the `#[repr(C)]` glue and the
//!   exported entry symbol.
//!
//! # ABI stability
//!
//! The host validates [`ABI_VERSION`] at load time and refuses to load a plugin
//! built against a different version (fail-closed). The ABI is designed to grow
//! by appending fields/functions and bumping [`ABI_VERSION`]; existing layouts
//! are never reordered.
//!
//! # Safety model
//!
//! Native plugins run in the host process with full privileges. They are a
//! *trusted* extension mechanism and must only be loaded from sources the
//! operator controls. The macro-generated glue isolates every panic with
//! [`std::panic::catch_unwind`] so a faulty plugin cannot unwind across the FFI
//! boundary (which would be undefined behavior).

#![deny(rust_2018_idioms)]
#![deny(missing_docs)]

mod abi;
mod macros;
mod plugin;

#[cfg(test)]
mod tests;

// `declare_plugin!` is exported at the crate root by `#[macro_export]` in
// `macro_.rs`; no re-export is needed. The private `macro_` module exists only
// to keep the macro definition out of the root file.

// Re-export the full public surface at the crate root so existing
// `use sws_plugin_api::{...}` imports keep working unchanged.
pub use crate::abi::{
    ABI_VERSION, ACTION_CONTINUE, ACTION_ERROR, ACTION_SHORT_CIRCUIT, CallContext, DestroyFn,
    ENTRY_SYMBOL, FfiSlice, HookFn, HostVTable, InitFn, PluginDecl, RequestHandle, ResponseHandle,
};
pub use crate::plugin::{Action, Plugin, PluginRequest, PluginResponse};
