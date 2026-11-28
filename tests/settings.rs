#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use static_web_server::settings::file::Settings;
    use static_web_server::url_rules::RuleHost;
    use std::path::{Path, PathBuf};

    #[test]
    fn toml_file_parsing() {
        let config_path = Path::new("tests/toml/sws.toml");
        let settings = Settings::read(config_path).unwrap();

        let general = settings.general.as_ref().unwrap();
        let root = general.root.as_deref().unwrap();

        assert_eq!(root, Path::new("docker/public"));
        assert_eq!(general.ipv6_only, Some(false));
        assert_eq!(general.open, Some(true));
        assert_eq!(general.open_path.as_deref(), Some("/docs"));
        assert_eq!(general.idle_timeout, Some(30));
        assert_eq!(general.max_header_size, Some(16384));
        assert_eq!(general.max_connections, Some(0));

        let advanced = settings.advanced.as_ref().unwrap();
        let virtual_hosts = advanced.virtual_hosts.as_ref().unwrap();

        let expected_roots = [PathBuf::from("docker"), PathBuf::from("docker/abc")];

        for vhost in virtual_hosts {
            if let Some(other_root) = &vhost.root {
                assert!(expected_roots.contains(other_root));
            } else {
                panic!("Could not determine value of advanced.virtual-hosts.root");
            }
        }
    }

    #[test]
    fn toml_ipv6_only_overrides_cli() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("sws.toml");
        std::fs::write(&config, "[general]\nipv6-only = true\n").unwrap();

        let args = [
            "static-web-server",
            "--config-file",
            config.to_str().unwrap(),
            "--ipv6-only",
            "false",
        ];
        let settings = static_web_server::Settings::get_unparsed(false, &args).unwrap();
        assert!(settings.general.ipv6_only);
    }

    #[test]
    fn toml_ipv4_host_rejects_ipv6_only() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("sws.toml");
        std::fs::write(&config, "[general]\nhost = \"0.0.0.0\"\n").unwrap();

        let args = [
            "static-web-server",
            "--config-file",
            config.to_str().unwrap(),
            "--ipv6-only",
        ];
        let error = static_web_server::Settings::get_unparsed(false, &args)
            .err()
            .expect("ipv6-only with an ipv4 host from the config file should be rejected");
        assert_eq!(
            error.to_string(),
            "--ipv6-only cannot be used with the ipv4 host 0.0.0.0"
        );
    }

    #[test]
    fn toml_max_connections_overrides_cli() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("sws.toml");
        std::fs::write(&config, "[general]\nmax-connections = 64\n").unwrap();

        let args = [
            "static-web-server",
            "--config-file",
            config.to_str().unwrap(),
            "--max-connections",
            "8",
        ];
        let settings = static_web_server::Settings::get_unparsed(false, &args).unwrap();
        assert_eq!(settings.general.max_connections, 64);
    }

    fn settings_from_toml(toml: &str) -> static_web_server::Result<static_web_server::Settings> {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("sws.toml");
        std::fs::write(&config, toml).unwrap();
        let args = [
            "static-web-server",
            "--config-file",
            config.to_str().unwrap(),
        ];
        static_web_server::Settings::get_unparsed(false, &args)
    }

    fn settings_error(toml: &str) -> String {
        match settings_from_toml(toml) {
            Ok(_) => panic!("settings must be rejected:\n{toml}"),
            Err(err) => format!("{err:#}"),
        }
    }

    #[test]
    fn toml_rewrite_redirect_option_is_rejected() {
        for value in ["301", "302", "true", "\"301\""] {
            let err = settings_error(&format!(
                "[[advanced.rewrites]]\nsource = \"/old\"\ndestination = \"/new\"\nredirect = {value}\n"
            ));
            assert!(
                err.contains("the `redirect` option of rewrite source /old was removed"),
                "{err}"
            );
            assert!(err.contains("[[advanced.redirects]]"), "{err}");
        }
    }

    #[test]
    fn toml_rewrite_host_is_compiled() {
        let settings = settings_from_toml(
            "[[advanced.rewrites]]\nhost = \"Example.com:8080\"\nsource = \"/a\"\ndestination = \"/b\"\n\n\
             [[advanced.rewrites]]\nsource = \"/c\"\ndestination = \"/d\"\n",
        )
        .unwrap();
        let rewrites = settings.advanced.unwrap().rewrites.unwrap();
        assert_eq!(
            rewrites[0].rule.host,
            Some(RuleHost::parse("example.com:8080").unwrap())
        );
        assert_eq!(rewrites[1].rule.host, None);
    }

    #[test]
    fn toml_redirect_host_is_compiled() {
        let settings = settings_from_toml(
            "[[advanced.redirects]]\nhost = \"[::1]\"\nsource = \"/a\"\ndestination = \"/b\"\nkind = 302\n",
        )
        .unwrap();
        let redirects = settings.advanced.unwrap().redirects.unwrap();
        assert_eq!(
            redirects[0].rule.host,
            Some(RuleHost::parse("[::1]").unwrap())
        );
        assert_eq!(redirects[0].kind, 302);
    }

    #[test]
    fn toml_url_rule_invalid_host_is_rejected() {
        for host in [
            "",
            "http://example.com",
            "example.com/path",
            "user@example.com",
            "example.com:99999",
            "example.com:abc",
            "::1",
            "example.com:0",
            "[zz]",
            "[::1]junk",
            "exa[mple]",
            "a,b",
        ] {
            let err = settings_error(&format!(
                "[[advanced.rewrites]]\nhost = \"{host}\"\nsource = \"/a\"\ndestination = \"/b\"\n"
            ));
            assert!(err.contains("invalid host"), "{host:?}: {err}");

            let err = settings_error(&format!(
                "[[advanced.redirects]]\nhost = \"{host}\"\nsource = \"/a\"\ndestination = \"/b\"\nkind = 301\n"
            ));
            assert!(err.contains("invalid host"), "{host:?}: {err}");
        }
    }

    #[test]
    fn toml_rewrite_destination_fragment_is_rejected() {
        let err =
            settings_error("[[advanced.rewrites]]\nsource = \"/a\"\ndestination = \"/b#top\"\n");
        assert!(err.contains("must not contain a fragment"), "{err}");
    }

    #[test]
    fn toml_redirect_destination_fragment_is_allowed() {
        let settings = settings_from_toml(
            "[[advanced.redirects]]\nsource = \"/a\"\ndestination = \"/b#top\"\nkind = 301\n",
        )
        .unwrap();
        let redirects = settings.advanced.unwrap().redirects.unwrap();
        assert_eq!(redirects[0].rule.destination, "/b#top");
    }

    #[test]
    fn toml_url_rule_invalid_source_is_rejected() {
        let err = settings_error("[[advanced.rewrites]]\nsource = \"/{a\"\ndestination = \"/b\"\n");
        assert!(err.contains("invalid rewrite for source: /{a"), "{err}");

        let err = settings_error(
            "[[advanced.redirects]]\nsource = \"/{a\"\ndestination = \"/b\"\nkind = 301\n",
        );
        assert!(err.contains("invalid redirect for source: /{a"), "{err}");
    }

    #[test]
    fn toml_url_rule_source_class_of_encoded_characters_is_rejected() {
        for source in ["/[a b]x", "/caf[éa]", "/a[! ]b", "/a[A-z]b"] {
            let err = settings_error(&format!(
                "[[advanced.rewrites]]\nsource = \"{source}\"\ndestination = \"/b\"\n"
            ));
            assert!(
                err.contains(&format!("invalid rewrite for source: {source}")),
                "{err}"
            );
            assert!(err.contains("matches a single character"), "{err}");

            let err = settings_error(&format!(
                "[[advanced.redirects]]\nsource = \"{source}\"\ndestination = \"/b\"\nkind = 301\n"
            ));
            assert!(
                err.contains(&format!("invalid redirect for source: {source}")),
                "{err}"
            );
            assert!(err.contains("matches a single character"), "{err}");
        }
    }

    #[test]
    fn toml_url_rule_destination_outside_uri_is_rejected() {
        for dest in ["", "/a b", "/a\\nb", "/café", "/a<b>"] {
            let err = settings_error(&format!(
                "[[advanced.redirects]]\nsource = \"/a\"\ndestination = \"{dest}\"\nkind = 301\n"
            ));
            assert!(
                err.contains("invalid redirect for source: /a"),
                "{dest:?}: {err}"
            );

            let err = settings_error(&format!(
                "[[advanced.rewrites]]\nsource = \"/a\"\ndestination = \"{dest}\"\n"
            ));
            assert!(
                err.contains("invalid rewrite for source: /a"),
                "{dest:?}: {err}"
            );
        }
    }

    #[test]
    fn toml_rewrite_destination_must_be_path_or_absolute_uri() {
        for dest in ["example.net", "a/b", "http://[bad/", "http:/no-authority"] {
            let err = settings_error(&format!(
                "[[advanced.rewrites]]\nsource = \"/a\"\ndestination = \"{dest}\"\n"
            ));
            assert!(
                err.contains("must be a path starting with `/` or an absolute URI"),
                "{dest:?}: {err}"
            );
        }
        // A capture may be empty, so it can't stand for the leading `/`
        let err =
            settings_error("[[advanced.rewrites]]\nsource = \"/{*}\"\ndestination = \"$1.html\"\n");
        assert!(
            err.contains("placeholders are only allowed in its path"),
            "{err}"
        );

        for dest in [
            "/b",
            "/$1",
            "$0.html",
            "http://other.example/$1",
            "https://[::1]:8443/x?y=$1",
        ] {
            let toml =
                format!("[[advanced.rewrites]]\nsource = \"/{{*}}\"\ndestination = \"{dest}\"\n");
            assert!(
                settings_from_toml(&toml).is_ok(),
                "{dest:?} must be accepted"
            );
        }
    }

    #[test]
    fn toml_url_rule_placeholder_before_destination_path_is_rejected() {
        let tables = [("redirects", "kind = 301\n"), ("rewrites", "")];
        for dest in [
            "$1",
            "x$1/y",
            "https://$1.example.org/",
            "https://example.org$1",
            "//$1/x",
            "http://$1.internal/x",
            "https:$1",
        ] {
            for (table, extra) in tables {
                let err = settings_error(&format!(
                    "[[advanced.{table}]]\nsource = \"/{{**}}\"\ndestination = \"{dest}\"\n{extra}"
                ));
                assert!(
                    err.contains(&format!("destination {dest:?} of source \"/{{**}}\"")),
                    "{table} {dest:?}: {err}"
                );
                assert!(
                    err.contains("placeholders are only allowed in its path, query or fragment"),
                    "{table} {dest:?}: {err}"
                );
            }
        }
    }

    #[test]
    fn toml_url_rule_placeholder_in_destination_path_is_accepted() {
        for dest in [
            "/$1",
            "$0.html",
            "$0/$1",
            "https://example.org/$1",
            "https://example.org?q=$1",
            "/a?b=$1#$2",
            "//cdn.example/$1",
            "rel/$1",
            "?q=$1",
            "https://example.org",
            "/a$3",
        ] {
            let toml = format!(
                "[[advanced.redirects]]\nsource = \"/{{*}}/{{*}}\"\ndestination = \"{dest}\"\nkind = 301\n"
            );
            if let Err(err) = settings_from_toml(&toml) {
                panic!("{dest:?} must be accepted: {err:#}");
            }
        }
        for dest in ["/$1", "$0.html", "https://example.org/$1?q=$2"] {
            let toml = format!(
                "[[advanced.rewrites]]\nsource = \"/{{*}}/{{*}}\"\ndestination = \"{dest}\"\n"
            );
            if let Err(err) = settings_from_toml(&toml) {
                panic!("{dest:?} must be accepted: {err:#}");
            }
        }
    }

    #[test]
    fn toml_url_rule_host_wildcard_is_rejected() {
        let err = settings_error(
            "[[advanced.redirects]]\nhost = \"*.example.com\"\nsource = \"/a\"\ndestination = \"/b\"\nkind = 301\n",
        );
        assert!(err.contains("wildcards are not supported"), "{err}");
    }

    #[test]
    fn toml_virtual_host_invalid_host_is_rejected() {
        for host in [
            "",
            "*.example.com",
            "http://example.com",
            "example.com/path",
            "user@example.com",
            "example.com:abc",
            "::1",
            "[zz]",
        ] {
            let err = settings_error(&format!(
                "[[advanced.virtual-hosts]]\nhost = \"{host}\"\nroot = \"tests/fixtures/public\"\n"
            ));
            assert!(
                err.contains(&format!("invalid virtual host {host:?}")),
                "{host:?}: {err}"
            );
        }
    }

    #[test]
    fn toml_redirect_kind_must_be_301_or_302() {
        for kind in ["303", "307", "308", "200"] {
            let err = settings_error(&format!(
                "[[advanced.redirects]]\nsource = \"/a\"\ndestination = \"/b\"\nkind = {kind}\n"
            ));
            assert!(err.contains("deserialization"), "{kind}: {err}");
        }
    }
}
