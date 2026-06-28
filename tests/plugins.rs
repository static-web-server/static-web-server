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

    /// Builds an example plugin and returns the path to its shared library.
    fn build_example_plugin(dir: &str, lib_name: &str) -> PathBuf {
        let manifest = format!("examples/plugins/{dir}/Cargo.toml");
        let status = Command::new(env!("CARGO"))
            .args(["build", "--quiet", "--manifest-path", &manifest])
            .status()
            .expect("failed to spawn cargo");
        assert!(status.success(), "failed to build example plugin '{dir}'");

        let (prefix, ext) = if cfg!(target_os = "windows") {
            ("", "dll")
        } else if cfg!(target_os = "macos") {
            ("lib", "dylib")
        } else {
            ("lib", "so")
        };
        let path = PathBuf::from(format!(
            "examples/plugins/{dir}/target/debug/{prefix}{lib_name}.{ext}"
        ));
        assert!(
            path.exists(),
            "plugin artifact not found at {}",
            path.display()
        );
        path
    }

    fn get(uri: &str) -> Request<()> {
        let mut req = Request::new(());
        *req.method_mut() = Method::GET;
        *req.uri_mut() = uri.parse().unwrap();
        req
    }

    #[tokio::test]
    async fn response_header_plugin_amends_response() {
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
