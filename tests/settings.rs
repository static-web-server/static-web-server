#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use static_web_server::settings::file::Settings;
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

    #[test]
    fn image_negotiation_from_cli_and_toml() {
        let args = ["static-web-server", "--image-negotiation"];
        let settings = static_web_server::Settings::get_unparsed(false, &args).unwrap();
        assert!(settings.general.image_negotiation);

        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("sws.toml");
        std::fs::write(&config, "[general]\nimage-negotiation = false\n").unwrap();
        let args = [
            "static-web-server",
            "--config-file",
            config.to_str().unwrap(),
            "--image-negotiation",
        ];
        let settings = static_web_server::Settings::get_unparsed(false, &args).unwrap();
        assert!(!settings.general.image_negotiation);
    }
}
