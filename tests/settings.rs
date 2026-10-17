#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(test)]
mod tests {
    use static_web_server::settings::file::Settings;
    use std::path::{Path, PathBuf};

    #[cfg(feature = "directory-listing-download")]
    #[test]
    fn directory_download_tarzst_cli_and_toml() {
        use clap::Parser;
        use static_web_server::settings::cli::General;

        let cli = General::try_parse_from([
            "static-web-server",
            "--directory-listing",
            "--directory-listing-download=targz,tarzst",
        ])
        .unwrap();
        assert_eq!(cli.directory_listing_download.len(), 2);
        assert!(
            General::try_parse_from(["static-web-server", "--directory-listing-download=tarzst",])
                .is_err()
        );

        let config = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        std::fs::write(
            config.path(),
            "[general]\ndirectory-listing = true\ndirectory-listing-download = ['tarzst']\n",
        )
        .unwrap();
        let settings = static_web_server::settings::Settings::get_unparsed(
            false,
            &[
                "static-web-server",
                "--directory-listing",
                "--directory-listing-download=targz",
                "--config-file",
                config.path().to_str().unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(settings.general.directory_listing_download).unwrap(),
            serde_json::json!(["tarzst"])
        );
    }

    #[test]
    fn toml_file_parsing() {
        let config_path = Path::new("tests/toml/sws.toml");
        let settings = Settings::read(config_path).unwrap();

        let general = settings.general.as_ref().unwrap();
        let root = general.root.as_deref().unwrap();

        assert_eq!(root, Path::new("docker/public"));
        assert_eq!(general.open, Some(true));
        assert_eq!(general.open_path.as_deref(), Some("/docs"));
        assert_eq!(general.idle_timeout, Some(30));

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
}
