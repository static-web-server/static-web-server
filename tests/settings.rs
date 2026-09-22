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
        assert_eq!(general.open, Some(true));
        assert_eq!(general.path.as_deref(), Some("/docs"));

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
