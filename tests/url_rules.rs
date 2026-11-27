#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

//! Integration tests for URL redirects and rewrites through the request pipeline: host
//! matching, query string forwarding, precedence, and their interaction.

#[cfg(test)]
pub mod tests {
    use http_body_util::BodyExt;
    use hyper::{
        Method, Request, Response, StatusCode, Uri,
        header::{CONTENT_LENGTH, HOST, LOCATION},
    };
    use std::net::SocketAddr;

    use static_web_server::body::Body;
    use static_web_server::handler::RequestHandlerOpts;
    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    const FIXTURE: &str = "toml/url_rules.toml";
    const CSS: &str = "font-family: sans-serif";
    const JS: &str = "Static Web Server running!";

    /// The response, and the request URI and `Host` as the pipeline left them.
    struct Sent {
        res: Response<Body>,
        uri: Uri,
        host: Option<String>,
    }

    async fn send_with(
        method: Method,
        uri: &str,
        host: Option<&str>,
        configure: impl FnOnce(&mut RequestHandlerOpts),
    ) -> Sent {
        let opts = fixture_settings(FIXTURE);
        let mut req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        configure(&mut req_handler_opts);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(host) = host {
            builder = builder.header(HOST, host);
        }
        let mut req = builder.body(()).unwrap();
        let res = req_handler
            .handle(&mut req, remote_addr)
            .await
            .expect("handler must succeed");
        let host = req
            .headers()
            .get(HOST)
            .map(|h| h.to_str().unwrap().to_owned());
        Sent {
            res,
            uri: req.uri().clone(),
            host,
        }
    }

    async fn send(method: Method, uri: &str, host: Option<&str>) -> Sent {
        send_with(method, uri, host, |_| {}).await
    }

    async fn get(uri: &str, host: Option<&str>) -> Sent {
        send(Method::GET, uri, host).await
    }

    async fn assert_redirect(uri: &str, host: Option<&str>, status: StatusCode, location: &str) {
        let sent = get(uri, host).await;
        assert_eq!(sent.res.status(), status, "{uri} with host {host:?}");
        assert_eq!(
            sent.res.headers()[LOCATION],
            location,
            "{uri} with host {host:?}"
        );
    }

    async fn assert_not_found(uri: &str, host: Option<&str>) {
        let sent = get(uri, host).await;
        assert_eq!(
            sent.res.status(),
            StatusCode::NOT_FOUND,
            "{uri} with host {host:?} must match no rule"
        );
        assert!(sent.res.headers().get(LOCATION).is_none());
    }

    async fn body(res: Response<Body>) -> String {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    // Redirects: host matching

    #[tokio::test]
    async fn redirects_host_matches_case_insensitively() {
        for host in ["example.com", "EXAMPLE.COM", "Example.Com"] {
            assert_redirect(
                "/host/any-port",
                Some(host),
                StatusCode::MOVED_PERMANENTLY,
                "/any-port",
            )
            .await;
        }
    }

    #[tokio::test]
    async fn redirects_host_without_port_matches_any_port() {
        for host in [
            "example.com:80",
            "example.com:443",
            "example.com:8080",
            "example.com:",
        ] {
            assert_redirect(
                "/host/any-port",
                Some(host),
                StatusCode::MOVED_PERMANENTLY,
                "/any-port",
            )
            .await;
        }
    }

    #[tokio::test]
    async fn redirects_host_with_port_requires_that_port() {
        for host in ["example.com:8080", "EXAMPLE.com:8080"] {
            assert_redirect(
                "/host/port",
                Some(host),
                StatusCode::MOVED_PERMANENTLY,
                "/port",
            )
            .await;
        }
        for host in ["example.com", "example.com:9090", "example.com:80"] {
            assert_not_found("/host/port", Some(host)).await;
        }
    }

    #[tokio::test]
    async fn redirects_host_ipv6() {
        for host in ["[::1]", "[::1]:8080"] {
            assert_redirect(
                "/host/ipv6",
                Some(host),
                StatusCode::MOVED_PERMANENTLY,
                "/ipv6",
            )
            .await;
        }
        assert_not_found("/host/ipv6", Some("[::2]")).await;
        assert_redirect(
            "http://[0:0:0:0:0:0:0:1]:8080/host/ipv6",
            None,
            StatusCode::MOVED_PERMANENTLY,
            "/ipv6",
        )
        .await;
    }

    #[tokio::test]
    async fn redirects_host_mismatch_falls_through() {
        for host in ["example.org", "sub.example.com", "example.com.evil"] {
            assert_not_found("/host/any-port", Some(host)).await;
        }
    }

    #[tokio::test]
    async fn redirects_host_missing_or_malformed_skips_host_rules() {
        assert_not_found("/host/any-port", None).await;
        for host in [
            "",
            "example.com:abc",
            "example.com:65536",
            "user@example.com",
        ] {
            assert_not_found("/host/any-port", Some(host)).await;
        }
    }

    #[tokio::test]
    async fn redirects_host_from_absolute_uri_takes_precedence() {
        // RFC 9112 §3.2.2: the authority of an absolute-form target replaces `Host`
        assert_redirect(
            "http://example.com/host/any-port",
            Some("other.org"),
            StatusCode::MOVED_PERMANENTLY,
            "/any-port",
        )
        .await;
        assert_not_found("http://other.org/host/any-port", Some("example.com")).await;
        // User info in the target is ignored, as virtual hosts ignore it
        assert_redirect(
            "http://user@example.com/host/any-port",
            None,
            StatusCode::MOVED_PERMANENTLY,
            "/any-port",
        )
        .await;
    }

    #[tokio::test]
    async fn redirects_host_rule_guards_virtual_host_despite_user_info() {
        // RFC 9110 §4.2.4: user info must not let a request reach the virtual host while
        // skipping the rules bound to it
        for uri in [
            "http://other.example/test.html",
            "http://x@other.example/test.html",
            "http://@other.example/test.html",
        ] {
            assert_redirect(uri, None, StatusCode::FOUND, "/login").await;
        }
    }

    #[tokio::test]
    async fn redirects_host_rule_guards_virtual_host_on_any_port() {
        for host in ["other.example:8080", "other.example:080", "other.example:"] {
            assert_redirect("/test.html", Some(host), StatusCode::FOUND, "/login").await;
        }
        // A malformed port names no host, so neither the rule nor the virtual host applies
        for host in [
            "other.example:+80",
            "other.example:8a",
            "other.example:65536",
        ] {
            assert_not_found("/test.html", Some(host)).await;
        }
    }

    #[tokio::test]
    async fn redirects_host_rule_guards_virtual_host_in_any_case() {
        for host in ["OTHER.EXAMPLE", "Other.Example:8080"] {
            assert_redirect("/test.html", Some(host), StatusCode::FOUND, "/login").await;
        }
    }

    #[tokio::test]
    async fn redirects_host_from_uri_without_host_header() {
        // HTTP/2 requests carry `:authority` in the URI and may omit `Host`
        assert_redirect(
            "https://example.com:8080/host/port",
            None,
            StatusCode::MOVED_PERMANENTLY,
            "/port",
        )
        .await;
    }

    #[tokio::test]
    async fn redirects_first_match_wins() {
        assert_redirect("/order", Some("a.example"), StatusCode::FOUND, "/order-a").await;
        assert_redirect("/order", Some("b.example"), StatusCode::FOUND, "/order-any").await;
        assert_redirect("/order", None, StatusCode::FOUND, "/order-any").await;
    }

    // Redirects: destination

    #[tokio::test]
    async fn redirects_query_appended() {
        assert_redirect("/query/plain", None, StatusCode::FOUND, "/new").await;
        assert_redirect(
            "/query/plain?q=1&r=%20s&q=2",
            None,
            StatusCode::FOUND,
            "/new?q=1&r=%20s&q=2",
        )
        .await;
    }

    #[tokio::test]
    async fn redirects_query_merged_with_destination_query() {
        assert_redirect("/query/merge", None, StatusCode::FOUND, "/new?a=1").await;
        assert_redirect("/query/merge?q=1", None, StatusCode::FOUND, "/new?a=1&q=1").await;
    }

    #[tokio::test]
    async fn redirects_query_inserted_before_fragment() {
        assert_redirect("/query/fragment", None, StatusCode::FOUND, "/new?a=1#top").await;
        assert_redirect(
            "/query/fragment?q=1",
            None,
            StatusCode::FOUND,
            "/new?a=1&q=1#top",
        )
        .await;
    }

    #[tokio::test]
    async fn redirects_query_empty_kept() {
        assert_redirect("/query/plain?", None, StatusCode::FOUND, "/new?").await;
    }

    #[tokio::test]
    async fn redirects_query_appended_to_absolute_destination() {
        assert_redirect(
            "/query/absolute/page?q=1",
            None,
            StatusCode::MOVED_PERMANENTLY,
            "https://example.org/page?q=1",
        )
        .await;
    }

    #[tokio::test]
    async fn redirects_capture_in_query_is_one_value() {
        for (uri, location) in [
            ("/query/value/a", "/index.html?page=a#a"),
            (
                "/query/value/a&admin=1",
                "/index.html?page=a%26admin%3D1#a&admin=1",
            ),
            ("/query/value/a%26b", "/index.html?page=a%26b#a&b"),
            ("/query/value/a+b;c", "/index.html?page=a%2Bb%3Bc#a+b;c"),
            (
                "/query/value/a%20b%2Bc",
                "/index.html?page=a%20b%2Bc#a%20b+c",
            ),
            ("/query/value/a=b?x=1&y", "/index.html?page=a%3Db&x=1&y#a=b"),
        ] {
            assert_redirect(uri, None, StatusCode::FOUND, location).await;
        }
    }

    #[tokio::test]
    async fn redirects_capture_keeps_percent_encoding() {
        assert_redirect(
            "/encoded/caf%C3%A9%20menu",
            None,
            StatusCode::MOVED_PERMANENTLY,
            "/decoded/caf%C3%A9%20menu",
        )
        .await;
        assert_redirect(
            "/encoded/a%0d%0aX:1",
            None,
            StatusCode::MOVED_PERMANENTLY,
            "/decoded/a%0D%0AX:1",
        )
        .await;
    }

    #[tokio::test]
    async fn redirects_capture_cannot_name_another_host() {
        // RFC 3986 §4.2: `//host` is a network-path reference, and browsers read `\` as `/`
        for (uri, location) in [
            ("/lead/post", "/post"),
            ("/lead//evil.example/x", "/evil.example/x"),
            ("/lead///evil.example/x?q=1", "/evil.example/x?q=1"),
            ("/lead/%2Fevil.example", "/evil.example"),
            ("/lead/%5Cevil.example", "/%5Cevil.example"),
            ("/lead/\\evil.example", "/%5Cevil.example"),
            ("/lead/\\/evil.example", "/%5C/evil.example"),
            ("/lead-glob/evil.example", "/evil.example"),
            ("/lead-glob//evil.example/x", "/evil.example/x"),
            ("/lead/a//b", "/a/b"),
            (
                "/lead-absolute//evil.example",
                "https://example.org/evil.example",
            ),
        ] {
            // On Windows `\\` separates segments, so it never reaches a capture
            let location = if cfg!(windows) {
                location.replace("%5C/", "").replace("%5C", "")
            } else {
                location.to_owned()
            };
            assert_redirect(uri, None, StatusCode::MOVED_PERMANENTLY, &location).await;
        }
    }

    // Redirects: path spellings

    /// Spellings of `/assets/<file>` that the file layer resolves to the same file,
    /// the last one longer than the 8 KiB cap on matched paths.
    fn spellings(file: &str) -> Vec<String> {
        let first = &file[..1];
        let rest = &file[1..];
        let escaped = format!("%{:02X}{rest}", first.as_bytes()[0]);
        vec![
            format!("/assets/{file}"),
            format!("/assets/./{file}"),
            format!("//assets/{file}"),
            format!("/assets//{file}"),
            format!("/assets/{escaped}"),
            format!("/assets/{}", escaped.to_lowercase()),
            format!("/assets%2F{file}"),
            format!("/assets/%2e/{file}"),
            format!("/./assets/%2E%2F/{file}"),
            format!("/assets/{}{file}", "./".repeat(5 * 1024)),
        ]
    }

    #[tokio::test]
    async fn redirects_match_every_spelling_of_a_path() {
        for uri in spellings("index.html") {
            assert_redirect(&uri, None, StatusCode::FOUND, "/login").await;
        }
    }

    #[cfg(any(target_os = "macos", windows))]
    #[tokio::test]
    async fn redirects_match_any_letter_case_where_files_do() {
        for uri in ["/ASSETS/index.html", "/Assets/Index.HTML"] {
            assert_redirect(uri, None, StatusCode::FOUND, "/login").await;
        }
    }

    #[tokio::test]
    async fn url_rules_on_a_file_are_not_skipped_by_a_directory_spelling() {
        // Rules tell `/file` and `/file/` apart, and the file layer serves no file for
        // a path that names a directory
        let spellings = ["/", "//", "/.", "/..", "%2F", "%2f", "/%2e", "%2F."];
        for redirect_trailing_slash in [true, false] {
            let configure = |opts: &mut RequestHandlerOpts| {
                opts.redirect_trailing_slash = redirect_trailing_slash;
            };
            for method in [Method::GET, Method::HEAD] {
                for file in ["/assets/index.html", "/assets/tiny.js"] {
                    for spelling in spellings {
                        let uri = [file, spelling].concat();
                        let sent = send_with(method.clone(), &uri, None, configure).await;
                        assert_eq!(
                            sent.res.status(),
                            StatusCode::NOT_FOUND,
                            "{method} {uri} with redirect-trailing-slash {redirect_trailing_slash}"
                        );
                        assert!(sent.res.headers().get(LOCATION).is_none(), "{uri}");
                        assert_eq!(sent.uri, uri.as_str(), "{uri} must not be rewritten");
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn redirects_capture_takes_the_served_spelling() {
        for (uri, location) in [
            ("/spelling/a/b", "/spelled/a/b"),
            ("/spelling/a/./b//c", "/spelled/a/b/c"),
            ("/spelling/a%2Fb", "/spelled/a/b"),
            ("/spelling/%61%20b", "/spelled/a%20b"),
            ("/spelling/caf%c3%a9", "/spelled/caf%C3%A9"),
            ("/spelling/a\"b", "/spelled/a%22b"),
            ("/spelling/a/", "/spelled/a/"),
            ("/spelling/a/.", "/spelled/a/"),
            ("/./spelling/a?q=%2f", "/spelled/a?q=%2f"),
        ] {
            assert_redirect(uri, None, StatusCode::MOVED_PERMANENTLY, location).await;
        }
    }

    #[tokio::test]
    async fn redirects_source_matches_in_any_spelling() {
        for uri in ["/caf%C3%A9", "/caf%c3%a9", "/café"] {
            assert_redirect(uri, None, StatusCode::MOVED_PERMANENTLY, "/cafe").await;
        }
        for uri in ["/men%C3%BA%20del%20d%C3%ADa/x", "/menú%20del%20día/x"] {
            assert_redirect(uri, None, StatusCode::MOVED_PERMANENTLY, "/menu/x").await;
        }
    }

    /// Removes the rules of the fixture that `remove` names.
    fn without(remove: &'static str) -> impl FnOnce(&mut RequestHandlerOpts) {
        move |opts| {
            if let Some(advanced) = opts.advanced_opts.as_mut() {
                if remove.contains("redirects") {
                    advanced.redirects = None;
                }
                if remove.contains("rewrites") {
                    advanced.rewrites = None;
                }
            }
        }
    }

    #[tokio::test]
    async fn redirects_paths_longer_than_the_cap_are_too_long() {
        let at_cap = ["/encoded/", &"a".repeat(8 * 1024 - "/encoded/".len())].concat();
        let location = at_cap.replacen("/encoded/", "/decoded/", 1);
        assert_redirect(&at_cap, None, StatusCode::MOVED_PERMANENTLY, &location).await;

        for method in [Method::GET, Method::HEAD, Method::OPTIONS] {
            let sent = send(method.clone(), &[&at_cap, "a"].concat(), None).await;
            assert_eq!(sent.res.status(), StatusCode::URI_TOO_LONG, "{method}");
            assert!(sent.res.headers().get(LOCATION).is_none(), "{method}");
        }

        // The cap is on the path, the query is appended whatever its length
        let query = "q".repeat(1024);
        assert_redirect(
            &[&at_cap, "?", &query].concat(),
            None,
            StatusCode::MOVED_PERMANENTLY,
            &[&location, "?", &query].concat(),
        )
        .await;
    }

    #[tokio::test]
    async fn rewrites_paths_longer_than_the_cap_are_too_long() {
        let at_cap = [
            "/rewrite/query/",
            &"a".repeat(8 * 1024 - "/rewrite/query/".len()),
        ]
        .concat();
        let sent = send_with(Method::GET, &at_cap, None, without("redirects")).await;
        let rewritten = at_cap.replacen("/rewrite/query/", "/assets/", 1) + "?v=1";
        assert_eq!(sent.uri, rewritten.as_str());

        let too_long = [&at_cap, "a"].concat();
        let sent = send_with(Method::GET, &too_long, None, without("redirects")).await;
        assert_eq!(sent.res.status(), StatusCode::URI_TOO_LONG);
        assert_eq!(sent.uri, too_long.as_str(), "the uri must not be rewritten");
    }

    #[tokio::test]
    async fn url_rules_none_configured_serve_paths_longer_than_the_cap() {
        let too_long = ["/", &"a".repeat(8 * 1024)].concat();
        let sent = send_with(Method::GET, &too_long, None, without("redirects rewrites")).await;
        assert_eq!(sent.res.status(), StatusCode::NOT_FOUND);
    }

    // Redirects: methods

    #[tokio::test]
    async fn redirects_empty_body() {
        let sent = get("/query/plain", None).await;
        assert_eq!(sent.res.status(), StatusCode::FOUND);
        assert_eq!(body(sent.res).await, "");
    }

    #[tokio::test]
    async fn redirects_apply_to_every_allowed_method() {
        for method in [Method::GET, Method::HEAD, Method::OPTIONS] {
            let sent = send(method.clone(), "/query/plain?q=1", None).await;
            assert_eq!(sent.res.status(), StatusCode::FOUND, "{method}");
            assert_eq!(sent.res.headers()[LOCATION], "/new?q=1", "{method}");
        }
    }

    #[tokio::test]
    async fn redirects_not_reached_by_disallowed_methods() {
        for method in [Method::POST, Method::PUT, Method::DELETE] {
            let sent = send(method.clone(), "/query/plain", None).await;
            assert_eq!(
                sent.res.status(),
                StatusCode::METHOD_NOT_ALLOWED,
                "{method}"
            );
            assert!(sent.res.headers().get(LOCATION).is_none(), "{method}");
        }
    }

    // Rewrites

    #[tokio::test]
    async fn rewrites_host_matches_case_insensitively_on_any_port() {
        for host in ["example.com", "EXAMPLE.COM", "example.com:9999"] {
            let sent = get("/rewrite/host", Some(host)).await;
            assert_eq!(sent.res.status(), StatusCode::OK, "host {host}");
            assert_eq!(sent.uri, "/assets/main.css");
            assert!(body(sent.res).await.contains(CSS));
        }
    }

    #[tokio::test]
    async fn rewrites_host_mismatch_leaves_uri() {
        for host in [Some("example.org"), Some("example.com:abc"), None] {
            let sent = get("/rewrite/host", host).await;
            assert_eq!(sent.res.status(), StatusCode::NOT_FOUND, "host {host:?}");
            assert_eq!(sent.uri, "/rewrite/host", "host {host:?}");
        }
    }

    #[tokio::test]
    async fn rewrites_host_with_port_requires_that_port() {
        let sent = get("/rewrite/port", Some("example.com:8080")).await;
        assert_eq!(sent.res.status(), StatusCode::OK);

        for host in ["example.com", "example.com:9090"] {
            let sent = get("/rewrite/port", Some(host)).await;
            assert_eq!(sent.res.status(), StatusCode::NOT_FOUND, "host {host}");
        }
    }

    #[tokio::test]
    async fn rewrites_query_forwarded() {
        let sent = get("/rewrite/query/main.css", None).await;
        assert_eq!(sent.res.status(), StatusCode::OK);
        assert_eq!(sent.uri, "/assets/main.css?v=1");

        let sent = get("/rewrite/query/main.css?q=1&r=%20s", None).await;
        assert_eq!(sent.res.status(), StatusCode::OK);
        assert_eq!(sent.uri, "/assets/main.css?v=1&q=1&r=%20s");
        assert!(body(sent.res).await.contains(CSS));
    }

    #[tokio::test]
    async fn rewrites_capture_in_query_is_one_value() {
        let sent = get("/rewrite/value/a&admin=1+b?q=1&r", None).await;
        assert_eq!(sent.res.status(), StatusCode::OK);
        assert_eq!(sent.uri, "/assets/main.css?page=a%26admin%3D1%2Bb&q=1&r");
        assert!(body(sent.res).await.contains(CSS));
    }

    #[tokio::test]
    async fn rewrites_capture_in_path_is_kept() {
        let sent = get("/rewrite/query/a&b=c+d;e", None).await;
        assert_eq!(sent.uri, "/assets/a&b=c+d;e?v=1");
    }

    #[tokio::test]
    async fn rewrites_query_empty_kept() {
        let sent = get("/rewrite/host?", Some("example.com")).await;
        assert_eq!(sent.res.status(), StatusCode::OK);
        assert_eq!(sent.uri.to_string(), "/assets/main.css?");

        let sent = get("/rewrite/query/main.css?", None).await;
        assert_eq!(sent.res.status(), StatusCode::OK);
        assert_eq!(sent.uri.to_string(), "/assets/main.css?v=1");
    }

    #[tokio::test]
    async fn rewrites_match_every_spelling_of_a_path() {
        for uri in spellings("tiny.js") {
            let sent = get(&uri, None).await;
            assert_eq!(sent.res.status(), StatusCode::OK, "{uri}");
            assert_eq!(sent.uri, "/assets/main.css", "{uri}");
            assert!(body(sent.res).await.contains(CSS), "{uri}");
        }
    }

    #[tokio::test]
    async fn rewrites_are_not_chained() {
        // `/both` is the source of a redirect and of a rewrite, and neither applies again
        let sent = get("/chain", None).await;
        assert_eq!(sent.res.status(), StatusCode::NOT_FOUND);
        assert!(sent.res.headers().get(LOCATION).is_none());
        assert_eq!(sent.uri, "/both");
    }

    #[tokio::test]
    async fn rewrites_host_rule_serves_its_virtual_host() {
        // The redirect of that host for `/test.html` does not apply to a rewritten request
        let sent = get("/vhost-page", Some("other.example")).await;
        assert_eq!(sent.res.status(), StatusCode::OK);
        assert_eq!(sent.uri, "/test.html");
        assert_eq!(sent.host.as_deref(), Some("other.example"));
        assert!(body(sent.res).await.contains("Test Page"));

        assert_not_found("/vhost-page", Some("localhost")).await;
    }

    #[tokio::test]
    async fn rewrites_to_virtual_host() {
        let sent = get("/vhost/test.html?q=1", Some("localhost")).await;
        assert_eq!(sent.res.status(), StatusCode::OK);
        assert_eq!(sent.uri, "http://other.example/test.html?q=1");
        assert_eq!(sent.host.as_deref(), Some("other.example"));
        assert!(body(sent.res).await.contains("Test Page"));
    }

    #[tokio::test]
    async fn rewrites_host_rule_serves_its_virtual_host_in_any_case() {
        for host in ["OTHER.EXAMPLE", "Other.Example:8080"] {
            let sent = get("/vhost-page", Some(host)).await;
            assert_eq!(sent.res.status(), StatusCode::OK, "{host}");
            assert_eq!(sent.uri, "/test.html", "{host}");
            assert!(body(sent.res).await.contains("Test Page"), "{host}");
        }
    }

    #[tokio::test]
    async fn rewrites_to_virtual_host_in_another_case() {
        let sent = get("/vhost-mixed-case/test.html", Some("localhost")).await;
        assert_eq!(sent.res.status(), StatusCode::OK);
        assert_eq!(sent.uri, "http://Other.Example/test.html");
        assert!(body(sent.res).await.contains("Test Page"));
    }

    // Virtual hosts

    #[tokio::test]
    async fn virtual_hosts_match_host_in_any_case() {
        for host in ["other.example", "OTHER.EXAMPLE", "Other.Example:8080"] {
            let sent = get("/article.html", Some(host)).await;
            assert_eq!(sent.res.status(), StatusCode::OK, "{host}");
            assert!(body(sent.res).await.contains("Article HTML"), "{host}");
        }
        assert_not_found("/article.html", Some("localhost")).await;
    }

    #[tokio::test]
    async fn virtual_hosts_match_ipv6_in_any_notation() {
        for (uri, host) in [
            ("/article.html", Some("[::1]")),
            ("/article.html", Some("[0:0:0:0:0:0:0:1]:8080")),
            ("http://[0::1]/article.html", None),
        ] {
            let sent = get(uri, host).await;
            assert_eq!(
                sent.res.status(),
                StatusCode::OK,
                "{uri} with host {host:?}"
            );
            assert!(body(sent.res).await.contains("Article HTML"), "{uri}");
        }
        assert_not_found("/article.html", Some("[::2]")).await;
    }

    #[tokio::test]
    async fn rewrites_apply_to_head() {
        // hyper drops the body of a `HEAD` response on the wire, the headers must match `GET`
        let head = send(Method::HEAD, "/rewrite/host", Some("example.com")).await;
        let get = get("/rewrite/host", Some("example.com")).await;
        assert_eq!(head.res.status(), StatusCode::OK);
        assert_eq!(head.uri, "/assets/main.css");
        assert_eq!(
            head.res.headers()[CONTENT_LENGTH],
            get.res.headers()[CONTENT_LENGTH]
        );
    }

    #[tokio::test]
    async fn rewrites_apply_to_options_but_not_to_disallowed_methods() {
        let sent = send(Method::OPTIONS, "/rewrite/host", Some("example.com")).await;
        assert_eq!(sent.res.status(), StatusCode::NO_CONTENT);
        assert_eq!(sent.uri, "/assets/main.css");

        let sent = send(Method::POST, "/rewrite/host", Some("example.com")).await;
        assert_eq!(sent.res.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(sent.uri, "/rewrite/host", "the uri must not be rewritten");
    }

    // Redirects and rewrites together

    #[tokio::test]
    async fn url_rules_redirects_run_before_rewrites() {
        assert_redirect("/both", None, StatusCode::FOUND, "/from-redirect").await;
    }

    #[tokio::test]
    async fn url_rules_same_source_split_by_host() {
        assert_redirect(
            "/split?q=1",
            Some("redirect.example"),
            StatusCode::MOVED_PERMANENTLY,
            "/split-redirected?q=1",
        )
        .await;

        let sent = get("/split", Some("Rewrite.Example:8080")).await;
        assert_eq!(sent.res.status(), StatusCode::OK);
        assert_eq!(sent.uri, "/assets/main.js");
        assert!(body(sent.res).await.contains(JS));

        assert_not_found("/split", Some("unknown.example")).await;
    }

    #[cfg(feature = "basic-auth")]
    #[tokio::test]
    async fn url_rules_require_basic_auth_first() {
        let protect = |opts: &mut RequestHandlerOpts| {
            opts.basic_auth =
                "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q".into();
        };
        for (uri, host) in [
            ("/query/plain", None),
            ("/rewrite/host", Some("example.com")),
        ] {
            let sent = send_with(Method::GET, uri, host, protect).await;
            assert_eq!(sent.res.status(), StatusCode::UNAUTHORIZED, "{uri}");
            assert!(sent.res.headers().get(LOCATION).is_none(), "{uri}");
            assert_eq!(sent.uri, uri, "{uri} must not be rewritten");
        }
    }
}
