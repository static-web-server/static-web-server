// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Loading, owning, and running native plugins.

use std::ffi::c_void;
use std::path::Path;

use http::HeaderMap;
use sws_plugin_api::{
    ABI_VERSION, ACTION_CONTINUE, ACTION_SHORT_CIRCUIT, CallContext, ENTRY_SYMBOL, FfiSlice,
    HookFn, PluginDecl,
};

use crate::error::LoadError;
use crate::host::{HOST_VTABLE, ReqCtx, RespCtx};
use crate::types::{PluginSpec, PreOutcome, RequestView, ResponseEdit, StagedResponse};

/// Wrapper that makes the opaque plugin instance pointer shareable across
/// threads.
///
/// The pointed-to instance is a `Box<dyn Plugin>`, which is `Send + Sync`, and
/// the host only ever invokes the plugin through `&self` hooks. The instance is
/// destroyed exactly once, at shutdown.
struct PluginData(*mut c_void);

// SAFETY: see the type-level comment — the underlying instance is `Send + Sync`
// and is only accessed through shared (`&self`) hooks.
unsafe impl Send for PluginData {}
// SAFETY: see above.
unsafe impl Sync for PluginData {}

pub(super) struct LoadedPlugin {
    name: String,
    data: PluginData,
    on_request: Option<HookFn>,
    on_response: Option<HookFn>,
    destroy: Option<sws_plugin_api::DestroyFn>,
    // Kept last so it is dropped (unloaded) only after `destroy` has run in
    // `Drop::drop`. `None` in in-process tests that supply a static descriptor.
    _library: Option<libloading::Library>,
}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        if let Some(destroy) = self.destroy
            && !self.data.0.is_null()
        {
            destroy(self.data.0);
            self.data.0 = std::ptr::null_mut();
        }
    }
}

impl LoadedPlugin {
    /// Validates a descriptor, constructs the instance, and records the hooks.
    ///
    /// # Safety
    ///
    /// `decl` must be a valid descriptor whose function pointers remain callable
    /// for as long as `library` (when `Some`) is loaded.
    pub(super) unsafe fn from_decl(
        path: &Path,
        decl: &PluginDecl,
        config: &str,
        library: Option<libloading::Library>,
    ) -> Result<Self, LoadError> {
        if decl.abi_version != ABI_VERSION {
            return Err(LoadError::AbiMismatch {
                path: path.to_path_buf(),
                expected: ABI_VERSION,
                found: decl.abi_version,
            });
        }

        // SAFETY: `name` points at the plugin's `'static` read-only data.
        let name = unsafe { decl.name.as_str() }
            .unwrap_or("<unnamed>")
            .to_owned();

        let data = match decl.init {
            Some(init) => {
                let ptr = init(FfiSlice::from_str(config));
                if ptr.is_null() {
                    return Err(LoadError::InitFailed(name));
                }
                ptr
            }
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            name,
            data: PluginData(data),
            on_request: decl.on_request,
            on_response: decl.on_response,
            destroy: decl.destroy,
            _library: library,
        })
    }
}

/// Loads and runs a set of native plugins.
///
/// The manager owns every loaded library and instance and is safe to share
/// across threads (`Send + Sync`).
pub struct PluginManager {
    pub(super) plugins: Vec<LoadedPlugin>,
}

impl PluginManager {
    /// Loads every plugin in `specs`, in order.
    ///
    /// Fails closed: if any plugin cannot be loaded, the whole call returns an
    /// error and no plugin is kept.
    pub fn load_all(specs: &[PluginSpec]) -> Result<Self, LoadError> {
        let mut plugins = Vec::with_capacity(specs.len());
        for spec in specs {
            plugins.push(load_one(spec)?);
        }
        Ok(Self { plugins })
    }

    /// Returns `true` when no plugins are loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Number of loaded plugins.
    #[must_use]
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Runs the pre-processing chain in declared order.
    ///
    /// Returns as soon as a plugin short-circuits or fails.
    pub fn pre_process(&self, req: &RequestView<'_>) -> PreOutcome {
        let req_ctx = ReqCtx {
            method: req.method,
            path: req.path,
            query: req.query,
            headers: req.headers,
        };
        for plugin in &self.plugins {
            let Some(on_request) = plugin.on_request else {
                continue;
            };

            let mut staging = HeaderMap::new();
            let mut resp_ctx = RespCtx {
                status: 200,
                headers: &mut staging,
                body: None,
            };
            let action = invoke(on_request, plugin.data.0, &req_ctx, &mut resp_ctx);

            match action {
                ACTION_CONTINUE => continue,
                ACTION_SHORT_CIRCUIT => {
                    let status = resp_ctx.status;
                    let body = resp_ctx.body.take().unwrap_or_default();
                    drop(resp_ctx);
                    return PreOutcome::ShortCircuit(StagedResponse {
                        status,
                        headers: staging,
                        body,
                    });
                }
                _ => return PreOutcome::Error(plugin.name.clone()),
            }
        }
        PreOutcome::Continue
    }

    /// Runs the post-processing chain in declared order.
    pub fn post_process(&self, req: &RequestView<'_>, edit: &mut ResponseEdit<'_>) {
        let req_ctx = ReqCtx {
            method: req.method,
            path: req.path,
            query: req.query,
            headers: req.headers,
        };
        for plugin in &self.plugins {
            let Some(on_response) = plugin.on_response else {
                continue;
            };

            let mut resp_ctx = RespCtx {
                status: edit.status,
                headers: edit.headers,
                body: edit.body.take(),
            };
            let _ = invoke(on_response, plugin.data.0, &req_ctx, &mut resp_ctx);
            edit.status = resp_ctx.status;
            edit.body = resp_ctx.body.take();
        }
    }
}

/// Builds the per-call [`CallContext`] and invokes a plugin hook.
fn invoke(hook: HookFn, data: *mut c_void, req: &ReqCtx<'_>, resp: &mut RespCtx<'_>) -> u8 {
    let ctx = CallContext {
        host: &HOST_VTABLE,
        request: (req as *const ReqCtx<'_>).cast::<c_void>(),
        response: (resp as *mut RespCtx<'_>).cast::<c_void>(),
    };
    hook(data, &ctx)
}

/// Loads a single plugin from its shared library.
fn load_one(spec: &PluginSpec) -> Result<LoadedPlugin, LoadError> {
    // SAFETY: loading native code runs the library's initializers. This is an
    // explicit, operator-controlled, trusted operation (see crate docs).
    let library = unsafe { libloading::Library::new(&spec.path) }
        .map_err(|e| LoadError::Open(spec.path.clone(), e))?;

    // SAFETY: the entry symbol, when present, is the declared
    // `extern "C" fn() -> *const PluginDecl`. The symbol borrows `library`, so we
    // scope it and keep only the returned raw pointer (which points at the
    // library's still-loaded static descriptor).
    let decl_ptr = {
        // SAFETY: the entry symbol, when present, is the declared
        // `extern "C" fn() -> *const PluginDecl`.
        let declare: libloading::Symbol<'_, extern "C" fn() -> *const PluginDecl> =
            unsafe { library.get(ENTRY_SYMBOL) }
                .map_err(|e| LoadError::Symbol(spec.path.clone(), e))?;
        declare()
    };
    if decl_ptr.is_null() {
        return Err(LoadError::NullDecl(spec.path.clone()));
    }
    // SAFETY: non-null and points at the plugin's `'static` descriptor.
    let decl = unsafe { &*decl_ptr };

    // SAFETY: `decl`'s function pointers remain valid while `library` is loaded.
    let plugin = unsafe { LoadedPlugin::from_decl(&spec.path, decl, &spec.config, Some(library))? };
    tracing::info!(plugin = %plugin.name, path = %spec.path.display(), "loaded native plugin");
    Ok(plugin)
}
