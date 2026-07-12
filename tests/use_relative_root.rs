#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

//! Integration tests for the `--use-relative-root` feature.
//!
//! The flag makes SWS skip startup canonicalization of the root
//! directory (and virtual host roots) so a symlinked root can be
//! swapped at runtime without restarting the server.
//!
//! Coverage:
//! 1. Settings parsing from TOML.
//! 2. CLI flag parsing.
//! 3. Startup behavior: `root_dir` remains non-canonical when
//!    `use_relative_root` is enabled and is canonicalized otherwise.
//! 4. Files are served through a symlinked root while the flag is on.
//! 5. Runtime symlink swap is picked up on the next request when the
//!    flag is on, and (crucially) is NOT picked up when it is off.
//! 6. Path traversal defense still holds against a non-canonical base.

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;
    use hyper::{Request, StatusCode};
    use static_web_server::Settings;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };
    use std::fs;
    use std::net::SocketAddr;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Minimal RAII temp-directory helper. We deliberately avoid pulling
    /// `tempfile` into the dev-dependency tree since the crate already
    /// keeps that list very small.
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(tag: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let path = std::env::temp_dir().join(format!(
                "sws-test-{}-{}-{}-{}",
                tag,
                std::process::id(),
                nanos,
                seq,
            ));
            fs::create_dir_all(&path).expect("failed to create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// Create a directory symlink / junction pointing at `target`.
    /// Symlink swap is the whole point of `use_relative_root`, so the
    /// tests that rely on it need a portable way to link directories.
    #[cfg(unix)]
    fn symlink_dir(target: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn symlink_dir(target: &Path, link: &Path) -> std::io::Result<()> {
        // Directory junctions do not require admin privileges or Developer Mode,
        // making them a portable way to create directory links on Windows.
        let output = std::process::Command::new("cmd")
            .args([
                "/c",
                "mklink",
                "/J",
                &link.to_string_lossy(),
                &target.to_string_lossy(),
            ])
            .output()?;
        if !output.status.success() {
            return Err(std::io::Error::other(String::from_utf8_lossy(
                &output.stderr,
            )));
        }
        Ok(())
    }

    #[cfg(unix)]
    fn remove_symlink_dir(link: &Path) -> std::io::Result<()> {
        fs::remove_file(link)
    }

    #[cfg(windows)]
    fn remove_symlink_dir(link: &Path) -> std::io::Result<()> {
        fs::remove_dir(link)
    }

    fn write_index(dir: &Path, body: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("index.html"), body).unwrap();
    }

    async fn get(
        handler: &static_web_server::handler::RequestHandler,
        uri: &str,
    ) -> (StatusCode, bytes::Bytes) {
        let mut req = Request::new(());
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = uri.parse().unwrap();
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let res = handler
            .handle(&mut req, remote_addr)
            .await
            .expect("handler returned an error");
        let status = res.status();
        let body = res
            .into_body()
            .collect()
            .await
            .expect("failed to read body")
            .to_bytes();
        (status, body)
    }

    // TOML parsing

    #[test]
    fn toml_enables_use_relative_root() {
        let s = fixture_settings("toml/use_relative_root_enabled.toml");
        assert!(
            s.general.use_relative_root,
            "expected `use_relative_root = true` from the TOML fixture"
        );
    }

    #[test]
    fn toml_disables_use_relative_root() {
        let s = fixture_settings("toml/use_relative_root_disabled.toml");
        assert!(
            !s.general.use_relative_root,
            "expected `use_relative_root = false` from the TOML fixture"
        );
    }

    // CLI flag parsing

    #[test]
    fn cli_flag_defaults_to_false() {
        let s = Settings::get_unparsed(false, &["static-web-server", "--root", "docker/public"])
            .unwrap();
        assert!(!s.general.use_relative_root);
    }

    #[test]
    fn cli_flag_bare_enables_it() {
        // `--use-relative-root` without a value should turn it on
        // (default_missing_value = "true").
        let s = Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                "docker/public",
                "--use-relative-root",
            ],
        )
        .unwrap();
        assert!(s.general.use_relative_root);
    }

    #[test]
    fn cli_flag_explicit_value_enables_it() {
        let s = Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                "docker/public",
                "--use-relative-root",
                "true",
            ],
        )
        .unwrap();
        assert!(s.general.use_relative_root);
    }

    // Startup wiring: `root_dir` stays non-canonical when the flag is
    // enabled and is canonicalized when it is disabled.

    #[test]
    fn root_dir_is_not_canonicalized_when_flag_is_on() {
        let tmp = TempDir::new("root-noncan");
        let real = tmp.path().join("real");
        let link = tmp.path().join("link");
        write_index(&real, "hello");
        symlink_dir(&real, &link).unwrap();

        let mut s = Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                link.to_str().unwrap(),
                "--use-relative-root",
                "true",
            ],
        )
        .unwrap();
        // The path we set is what should live on the handler.
        s.general.root = link.clone();
        let opts = fixture_req_handler_opts(s.general, s.advanced);

        assert_eq!(
            opts.root_dir, link,
            "root_dir should preserve the symlink path when \
             use_relative_root is enabled"
        );
        assert!(opts.use_relative_root, "flag must round-trip into opts");
    }

    #[test]
    fn root_dir_is_canonicalized_when_flag_is_off() {
        let tmp = TempDir::new("root-can");
        let real = tmp.path().join("real");
        let link = tmp.path().join("link");
        write_index(&real, "hello");
        symlink_dir(&real, &link).unwrap();

        let mut s = Settings::get_unparsed(
            false,
            &["static-web-server", "--root", link.to_str().unwrap()],
        )
        .unwrap();
        s.general.root = link.clone();
        let opts = fixture_req_handler_opts(s.general, s.advanced);

        let expected = real.canonicalize().unwrap();
        assert_eq!(
            opts.root_dir, expected,
            "root_dir should resolve through the symlink at startup \
             when use_relative_root is disabled"
        );
        assert!(!opts.use_relative_root);
    }

    // End-to-end serving through a symlinked root.

    #[tokio::test]
    async fn serves_files_through_symlinked_root_when_flag_is_on() {
        let tmp = TempDir::new("serve-symlink");
        let real = tmp.path().join("site");
        let link = tmp.path().join("current");
        write_index(&real, "SYMLINK-ROOT-OK");
        symlink_dir(&real, &link).unwrap();

        let mut s = Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                link.to_str().unwrap(),
                "--use-relative-root",
                "true",
            ],
        )
        .unwrap();
        s.general.root = link.clone();
        let opts = fixture_req_handler_opts(s.general, s.advanced);
        assert_eq!(opts.root_dir, link);

        let handler = fixture_req_handler(opts);
        let (status, body) = get(&handler, "http://localhost/index.html").await;
        assert_eq!(status, 200);
        assert_eq!(&body[..], b"SYMLINK-ROOT-OK");
    }

    // Runtime symlink swap.
    // - With `use_relative_root=true` the next request reflects the
    //   new target.
    // - With the default behavior the swap is not observed because
    //   the root was canonicalized at startup and pinned to the
    //   original inode.

    #[tokio::test]
    async fn runtime_symlink_swap_is_observed_when_flag_is_on() {
        let tmp = TempDir::new("swap-on");
        let blue = tmp.path().join("blue");
        let green = tmp.path().join("green");
        let link = tmp.path().join("current");
        write_index(&blue, "BLUE");
        write_index(&green, "GREEN");
        symlink_dir(&blue, &link).unwrap();

        let mut s = Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                link.to_str().unwrap(),
                "--use-relative-root",
                "true",
            ],
        )
        .unwrap();
        s.general.root = link.clone();
        let handler = fixture_req_handler(fixture_req_handler_opts(s.general, s.advanced));

        let (status, body) = get(&handler, "http://localhost/index.html").await;
        assert_eq!(status, 200);
        assert_eq!(&body[..], b"BLUE");

        // Atomic swap: replace the symlink so it points at `green`.
        remove_symlink_dir(&link).unwrap();
        symlink_dir(&green, &link).unwrap();

        let (status, body) = get(&handler, "http://localhost/index.html").await;
        assert_eq!(status, 200);
        assert_eq!(
            &body[..],
            b"GREEN",
            "expected the swapped-in target to be served"
        );
    }

    #[tokio::test]
    async fn runtime_symlink_swap_is_ignored_when_flag_is_off() {
        let tmp = TempDir::new("swap-off");
        let blue = tmp.path().join("blue");
        let green = tmp.path().join("green");
        let link = tmp.path().join("current");
        write_index(&blue, "BLUE");
        write_index(&green, "GREEN");
        symlink_dir(&blue, &link).unwrap();

        let mut s = Settings::get_unparsed(
            false,
            &["static-web-server", "--root", link.to_str().unwrap()],
        )
        .unwrap();
        s.general.root = link.clone();
        let handler = fixture_req_handler(fixture_req_handler_opts(s.general, s.advanced));

        let (status, body) = get(&handler, "http://localhost/index.html").await;
        assert_eq!(status, 200);
        assert_eq!(&body[..], b"BLUE");

        remove_symlink_dir(&link).unwrap();
        symlink_dir(&green, &link).unwrap();

        // The root was canonicalized at startup, so the pre-swap target
        // (`blue`) is still served.
        let (status, body) = get(&handler, "http://localhost/index.html").await;
        assert_eq!(status, 200);
        assert_eq!(
            &body[..],
            b"BLUE",
            "canonicalized root must remain pinned to the original target"
        );
    }

    // Path traversal defense against a non-canonical base.

    #[tokio::test]
    async fn path_traversal_is_still_rejected_when_flag_is_on() {
        let tmp = TempDir::new("traversal");
        let outside = tmp.path().join("outside");
        let inside = tmp.path().join("inside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret.txt"), "SECRET").unwrap();
        write_index(&inside, "PUBLIC");

        let link = tmp.path().join("root");
        symlink_dir(&inside, &link).unwrap();

        let mut s = Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--root",
                link.to_str().unwrap(),
                "--use-relative-root",
                "true",
            ],
        )
        .unwrap();
        s.general.root = link.clone();
        let handler = fixture_req_handler(fixture_req_handler_opts(s.general, s.advanced));

        // The URL parser strips `..` segments so the request resolves
        // inside the root (404 for the non-existent file).
        // The outside `secret.txt` is never served.
        let (status, body) = get(&handler, "http://localhost/../outside/secret.txt").await;
        assert_ne!(status, 200, "must not serve files above the root");
        assert!(!body.windows(6).any(|w| w == b"SECRET"));
    }
}
