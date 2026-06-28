// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The `#[repr(C)]`, versioned C ABI shared by the SWS host and every native
//! plugin.
//!
//! Only primitive integers and borrowed byte spans ([`FfiSlice`]) cross this
//! boundary; the host's `hyper` request/response types never do. This is what
//! makes plugins compiled separately ABI-compatible with the host.
//!
//! # ABI stability
//!
//! The host validates [`ABI_VERSION`] at load time and refuses to load a plugin
//! built against a different version (fail-closed). The ABI is designed to grow
//! by appending fields/functions and bumping [`ABI_VERSION`]; existing layouts
//! are never reordered.

use core::ffi::c_void;

/// Current plugin ABI version.
///
/// The host loads a plugin only when its [`PluginDecl::abi_version`] matches this
/// value exactly. Bump it whenever the layout of any `#[repr(C)]` type in this
/// crate changes.
pub const ABI_VERSION: u32 = 1;

/// Name of the symbol every plugin must export.
///
/// The host looks this symbol up after `dlopen`/`LoadLibrary`. It must resolve to
/// an `extern "C" fn() -> *const PluginDecl`. The [`crate::declare_plugin!`]
/// macro generates it for you.
pub const ENTRY_SYMBOL: &[u8] = b"sws_plugin_declare\0";

/// Return value of a plugin hook: continue the pipeline.
pub const ACTION_CONTINUE: u8 = 0;
/// Return value of a plugin hook: short-circuit with the staged response.
pub const ACTION_SHORT_CIRCUIT: u8 = 1;
/// Return value of a plugin hook: the plugin failed; the host emits a 500.
pub const ACTION_ERROR: u8 = 2;

/// Opaque handle to a host-owned request, valid only for the duration of a call.
pub type RequestHandle = *const c_void;
/// Opaque handle to a host-owned response, valid only for the duration of a call.
pub type ResponseHandle = *mut c_void;

/// A borrowed, FFI-safe span of bytes (often UTF-8 text).
///
/// A null `ptr` represents "absent" (e.g. a missing header). A non-null `ptr`
/// with `len == 0` represents an empty-but-present value. The pointee is only
/// guaranteed valid for the duration of the call that produced it; never store
/// it beyond that.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiSlice {
    /// Pointer to the first byte, or null when absent.
    pub ptr: *const u8,
    /// Length in bytes.
    pub len: usize,
}

impl FfiSlice {
    /// The "absent" value (null pointer, zero length).
    pub const NULL: Self = Self {
        ptr: core::ptr::null(),
        len: 0,
    };

    /// Borrows the bytes of `b` as an [`FfiSlice`].
    #[inline]
    #[must_use]
    pub const fn from_bytes(b: &[u8]) -> Self {
        Self {
            ptr: b.as_ptr(),
            len: b.len(),
        }
    }

    /// Borrows the bytes of `s` as an [`FfiSlice`].
    #[inline]
    #[must_use]
    pub const fn from_str(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    /// Returns `true` when this slice is the "absent" value.
    #[inline]
    #[must_use]
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Reconstructs a byte slice from this span.
    ///
    /// Returns `None` when the span is absent (null pointer).
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `ptr` is valid for `len` bytes and that the
    /// pointee outlives the returned borrow `'a`. The host upholds this for the
    /// duration of a single plugin call.
    #[inline]
    #[must_use]
    pub unsafe fn as_bytes<'a>(&self) -> Option<&'a [u8]> {
        if self.ptr.is_null() {
            None
        } else {
            // SAFETY: delegated to the caller's contract above.
            Some(unsafe { core::slice::from_raw_parts(self.ptr, self.len) })
        }
    }

    /// Reconstructs a UTF-8 string from this span.
    ///
    /// Returns `None` when the span is absent or not valid UTF-8.
    ///
    /// # Safety
    ///
    /// Same contract as [`FfiSlice::as_bytes`].
    #[inline]
    #[must_use]
    pub unsafe fn as_str<'a>(&self) -> Option<&'a str> {
        // SAFETY: delegated to the caller's contract on `as_bytes`.
        unsafe { self.as_bytes() }.and_then(|b| core::str::from_utf8(b).ok())
    }
}

/// Functions the host exposes to plugins for inspecting the request and
/// inspecting/mutating the response.
///
/// All functions operate on opaque [`RequestHandle`]/[`ResponseHandle`] values
/// supplied through a [`CallContext`]. Reads return [`FfiSlice`] views into
/// host memory valid only for the current call.
#[repr(C)]
pub struct HostVTable {
    /// ABI version of the host that produced this table (always [`ABI_VERSION`]).
    pub abi_version: u32,
    /// Returns the request method (e.g. `GET`).
    pub req_method: extern "C" fn(RequestHandle) -> FfiSlice,
    /// Returns the decoded request path (e.g. `/index.html`).
    pub req_path: extern "C" fn(RequestHandle) -> FfiSlice,
    /// Returns the raw request query string, or [`FfiSlice::NULL`] when absent.
    pub req_query: extern "C" fn(RequestHandle) -> FfiSlice,
    /// Returns the first value of the named request header, or [`FfiSlice::NULL`].
    pub req_header: extern "C" fn(RequestHandle, FfiSlice) -> FfiSlice,
    /// Returns the current response status code.
    pub resp_status: extern "C" fn(ResponseHandle) -> u16,
    /// Sets the response status code.
    pub resp_set_status: extern "C" fn(ResponseHandle, u16),
    /// Returns the first value of the named response header, or [`FfiSlice::NULL`].
    pub resp_header: extern "C" fn(ResponseHandle, FfiSlice) -> FfiSlice,
    /// Inserts/replaces a response header. Returns `false` on an invalid name/value.
    pub resp_set_header: extern "C" fn(ResponseHandle, FfiSlice, FfiSlice) -> bool,
    /// Replaces the response body with the given bytes (copied by the host).
    pub resp_set_body: extern "C" fn(ResponseHandle, FfiSlice),
}

/// Per-call context handed to a plugin hook.
#[repr(C)]
pub struct CallContext {
    /// The host function table.
    pub host: *const HostVTable,
    /// Opaque request handle.
    pub request: RequestHandle,
    /// Opaque response handle (the staged response for `on_request`).
    pub response: ResponseHandle,
}

/// Constructs a plugin instance from its configuration string. Returns an opaque
/// data pointer, or null on failure.
pub type InitFn = extern "C" fn(config: FfiSlice) -> *mut c_void;
/// A plugin hook. Receives the instance data and a [`CallContext`]; returns one
/// of the `ACTION_*` codes.
pub type HookFn = extern "C" fn(data: *mut c_void, ctx: *const CallContext) -> u8;
/// Destroys a plugin instance previously returned by an [`InitFn`].
pub type DestroyFn = extern "C" fn(data: *mut c_void);

/// The descriptor a plugin returns from its exported entry symbol.
///
/// All function pointers are optional so a plugin can implement only the hooks it
/// needs. The [`crate::declare_plugin!`] macro builds this for you.
#[repr(C)]
pub struct PluginDecl {
    /// ABI version this plugin was built against. Must equal [`ABI_VERSION`].
    pub abi_version: u32,
    /// Human-readable plugin name (points to `'static` read-only data).
    pub name: FfiSlice,
    /// Optional constructor invoked once at startup.
    pub init: Option<InitFn>,
    /// Optional pre-processing hook (may short-circuit the request).
    pub on_request: Option<HookFn>,
    /// Optional post-processing hook (may mutate the response).
    pub on_response: Option<HookFn>,
    /// Optional destructor invoked once at shutdown.
    pub destroy: Option<DestroyFn>,
}

// SAFETY: `PluginDecl` is only ever instantiated as an immutable `static` whose
// `name` points to `'static` read-only data and whose function pointers are
// thread-safe. Sharing it across threads is therefore sound.
unsafe impl Sync for PluginDecl {}
