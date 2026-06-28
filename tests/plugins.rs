#![cfg(feature = "plugins-native")]
#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

//! End-to-end tests for native plugins.
//!
//! Each test builds one of the example plugins as a real shared library and runs
//! it through the full request pipeline.

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use std::process::Command;

    use hyper::{Method, Request};

    use static_web_server::plugins::{PluginSpec, Plugins};
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    /// Returns `Some(reason)` when the environment cannot reliably build and
    /// `dlopen` a plugin shared library — e.g. inside the `cross` Docker
    /// sandbox used for cross-compile CI targets, where re-invoking `cargo`
    /// from a running test does not produce an artifact compatible with the
    /// test process.
    ///
    /// Tests use this to opt out gracefully instead of failing on hosts where
    /// the e2e path is structurally unsupported.
    fn skip_reason() -> Option<&'static str> {
        // musl libc does not implement `dlopen`, so dynamic loading is
        // structurally impossible on any *-musl target regardless of whether
        // the build is native or cross-compiled.
        if cfg!(target_env = "musl") {
            return Some("musl libc does not support dlopen; native plugin e2e is skipped");
        }
        // `cross` sets these env vars inside its build container. The test
        // binary may be cross-compiled (e.g. i686-musl) while the container's
        // host toolchain produces artifacts for a different triple, so the
        // dlopen path is not viable.
        if std::env::var_os("CROSS_SYSROOT").is_some()
            || std::env::var_os("CROSS_RUNNER").is_some()
            || std::env::var_os("CROSS_CMD").is_some()
        {
            return Some("running under `cross`; native plugin e2e is skipped");
        }
        // Explicit opt-out for CI runs that can't load the plugin (e.g.
        // cross-compile targets that still execute natively via the kernel
        // but lack a matching toolchain to produce a same-ABI cdylib).
        if std::env::var_os("SWS_SKIP_PLUGIN_E2E").is_some() {
            return Some("SWS_SKIP_PLUGIN_E2E is set");
        }
        None
    }

    /// Builds an example plugin and returns the path to its shared library.
    ///
    /// The target triple is propagated from the outer cargo invocation via the
    /// `CARGO_BUILD_TARGET` environment variable (cargo sets this when `--target`
    /// is passed), so the plugin is compiled for the same target the test binary
    /// runs under and can be `dlopen`ed by it.
    fn build_example_plugin(dir: &str, lib_name: &str) -> PathBuf {
        let manifest = format!("examples/plugins/{dir}/Cargo.toml");

        // Propagate the target triple so the plugin is built for the same
        // architecture as the test binary (matters on cross-compile CI targets
        // like i686-unknown-linux-musl).
        let target = std::env::var("CARGO_BUILD_TARGET").ok();
        let mut cmd = Command::new(env!("CARGO"));
        cmd.args(["build", "--quiet", "--manifest-path", &manifest]);
        if let Some(ref t) = target {
            cmd.args(["--target", t]);
        }
        let status = cmd.status().expect("failed to spawn cargo");
        assert!(status.success(), "failed to build example plugin '{dir}'");

        let (prefix, ext) = if cfg!(target_os = "windows") {
            ("", "dll")
        } else if cfg!(target_os = "macos") {
            ("lib", "dylib")
        } else {
            ("lib", "so")
        };
        let file = format!("{prefix}{lib_name}.{ext}");

        // The artifact may land in `target/<triple>/debug/` (when `--target` is
        // passed) or `target/debug/` (host build). Probe both, in that order.
        let candidates: Vec<PathBuf> = {
            let mut v = Vec::with_capacity(2);
            if let Some(t) = &target {
                v.push(PathBuf::from(format!(
                    "examples/plugins/{dir}/target/{t}/debug/{file}"
                )));
            }
            v.push(PathBuf::from(format!(
                "examples/plugins/{dir}/target/debug/{file}"
            )));
            v
        };

        for path in &candidates {
            if path.exists() {
                return path.clone();
            }
        }
        panic!(
            "plugin artifact not found; tried: {}",
            candidates
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    fn get(uri: &str) -> Request<()> {
        let mut req = Request::new(());
        *req.method_mut() = Method::GET;
        *req.uri_mut() = uri.parse().unwrap();
        req
    }

    #[tokio::test]
    async fn response_header_plugin_amends_response() {
        if let Some(reason) = skip_reason() {
            eprintln!("skipping response_header_plugin_amends_response: {reason}");
            return;
        }
        let path = build_example_plugin("response-header", "sws_plugin_response_header");
        let plugins = Plugins::load(&[PluginSpec {
            path,
            config: "X-Test-Plugin: yes".to_owned(),
        }])
        .expect("load response-header plugin");

        let opts = fixture_settings("toml/handler.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        req_handler_opts.plugins = Some(plugins);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = get("http://localhost/index.html");
        let res = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("request handled");

        assert_eq!(res.status(), 200);
        assert_eq!(
            res.headers().get("x-test-plugin").map(|v| v.as_bytes()),
            Some(b"yes".as_slice())
        );
    }

    #[tokio::test]
    async fn token_auth_plugin_short_circuits_unauthorized() {
        if let Some(reason) = skip_reason() {
            eprintln!("skipping token_auth_plugin_short_circuits_unauthorized: {reason}");
            return;
        }
        let path = build_example_plugin("token-auth", "sws_plugin_token_auth");
        let plugins = Plugins::load(&[PluginSpec {
            path,
            config: "s3cret".to_owned(),
        }])
        .expect("load token-auth plugin");

        let opts = fixture_settings("toml/handler.toml");
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        req_handler_opts.plugins = Some(plugins);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        // No credentials -> 401 from the plugin, file never served.
        let mut req = get("http://localhost/index.html");
        let res = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("request handled");
        assert_eq!(res.status(), 401);
        assert_eq!(
            res.headers().get("www-authenticate").map(|v| v.as_bytes()),
            Some(b"Bearer".as_slice())
        );

        // Valid bearer token -> request proceeds to the static file.
        let mut req = get("http://localhost/index.html");
        req.headers_mut()
            .insert("authorization", "Bearer s3cret".parse().unwrap());
        let res = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("request handled");
        assert_eq!(res.status(), 200);
    }
}
