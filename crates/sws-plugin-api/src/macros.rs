// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The [`declare_plugin!`] macro that registers a [`crate::Plugin`] implementation
//! and generates the exported entry symbol.

/// Registers a [`crate::Plugin`] implementation and generates the exported entry
/// symbol.
///
/// `name` is a `'static` string identifying the plugin in logs. `ty` is the
/// implementing type. `ctor` is a path to a function
/// `fn(&str) -> Result<ty, impl Display>` that builds the plugin from its
/// configuration string.
///
/// # Example
///
/// ```ignore
/// use sws_plugin_api::{Action, Plugin, PluginRequest, PluginResponse, declare_plugin};
///
/// struct Hello;
///
/// impl Hello {
///     fn new(_config: &str) -> Result<Self, std::convert::Infallible> {
///         Ok(Self)
///     }
/// }
///
/// impl Plugin for Hello {
///     fn on_response(&self, _req: &PluginRequest<'_>, resp: &mut PluginResponse<'_>) {
///         resp.set_header("x-hello", b"world");
///     }
/// }
///
/// declare_plugin!("hello", Hello, Hello::new);
/// ```
#[macro_export]
macro_rules! declare_plugin {
    ($name:expr, $ty:ty, $ctor:path) => {
        const _: () = {
            extern "C" fn __sws_plugin_init(config: $crate::FfiSlice) -> *mut ::core::ffi::c_void {
                let result = ::std::panic::catch_unwind(|| {
                    // SAFETY: the host keeps `config` valid for this init call.
                    let cfg = unsafe { config.as_str() }.unwrap_or("");
                    let ctor: fn(&str) -> _ = $ctor;
                    ctor(cfg)
                });
                match result {
                    Ok(Ok(plugin)) => {
                        let plugin: $ty = plugin;
                        let boxed: ::std::boxed::Box<dyn $crate::Plugin> =
                            ::std::boxed::Box::new(plugin);
                        // Double-box so the opaque data pointer is thin.
                        ::std::boxed::Box::into_raw(::std::boxed::Box::new(boxed)).cast()
                    }
                    Ok(Err(err)) => {
                        ::std::eprintln!("[sws-plugin] '{}' init failed: {}", $name, err);
                        ::core::ptr::null_mut()
                    }
                    Err(_) => {
                        ::std::eprintln!("[sws-plugin] '{}' init panicked", $name);
                        ::core::ptr::null_mut()
                    }
                }
            }

            fn __sws_plugin_dispatch(
                data: *mut ::core::ffi::c_void,
                ctx: *const $crate::CallContext,
                is_request: bool,
            ) -> u8 {
                let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    // SAFETY: `data` is the pointer returned by `__sws_plugin_init`
                    // and `ctx`/its handles are valid for the duration of this call.
                    let plugin: &::std::boxed::Box<dyn $crate::Plugin> =
                        unsafe { &*(data as *const ::std::boxed::Box<dyn $crate::Plugin>) };
                    let ctx = unsafe { &*ctx };
                    let host = unsafe { &*ctx.host };
                    let req = $crate::PluginRequest::from_raw(host, ctx.request);
                    let mut resp = $crate::PluginResponse::from_raw(host, ctx.response);
                    if is_request {
                        plugin.on_request(&req, &mut resp)
                    } else {
                        plugin.on_response(&req, &mut resp);
                        $crate::Action::Continue
                    }
                }));
                match result {
                    Ok($crate::Action::Continue) => $crate::ACTION_CONTINUE,
                    Ok($crate::Action::ShortCircuit) => $crate::ACTION_SHORT_CIRCUIT,
                    Err(_) => $crate::ACTION_ERROR,
                }
            }

            extern "C" fn __sws_plugin_on_request(
                data: *mut ::core::ffi::c_void,
                ctx: *const $crate::CallContext,
            ) -> u8 {
                __sws_plugin_dispatch(data, ctx, true)
            }

            extern "C" fn __sws_plugin_on_response(
                data: *mut ::core::ffi::c_void,
                ctx: *const $crate::CallContext,
            ) -> u8 {
                __sws_plugin_dispatch(data, ctx, false)
            }

            extern "C" fn __sws_plugin_destroy(data: *mut ::core::ffi::c_void) {
                if data.is_null() {
                    return;
                }
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    // SAFETY: `data` was produced by `Box::into_raw` in init and is
                    // destroyed exactly once at shutdown.
                    let boxed: ::std::boxed::Box<::std::boxed::Box<dyn $crate::Plugin>> = unsafe {
                        ::std::boxed::Box::from_raw(
                            data as *mut ::std::boxed::Box<dyn $crate::Plugin>,
                        )
                    };
                    drop(boxed);
                }));
            }

            static __SWS_PLUGIN_DECL: $crate::PluginDecl = $crate::PluginDecl {
                abi_version: $crate::ABI_VERSION,
                name: $crate::FfiSlice::from_str($name),
                init: ::core::option::Option::Some(__sws_plugin_init),
                on_request: ::core::option::Option::Some(__sws_plugin_on_request),
                on_response: ::core::option::Option::Some(__sws_plugin_on_response),
                destroy: ::core::option::Option::Some(__sws_plugin_destroy),
            };

            #[unsafe(no_mangle)]
            pub extern "C" fn sws_plugin_declare() -> *const $crate::PluginDecl {
                &__SWS_PLUGIN_DECL
            }
        };
    };
}
