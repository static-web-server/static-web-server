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
        let config_path = Path::new("tests/toml/config.toml");
        let settings = Settings::read(config_path).unwrap();
        let root = settings.general.unwrap().root.unwrap();
        assert_eq!(root, PathBuf::from("docker/public"));

        let virtual_hosts = settings.advanced.unwrap().virtual_hosts.unwrap();
        let expected_roots = [PathBuf::from("docker"), PathBuf::from("docker/abc")];
        for vhost in virtual_hosts {
            if let Some(other_root) = &vhost.root {
                assert!(expected_roots.contains(other_root));
            } else {
                panic!("Could not determine value of advanced.virtual-hosts.root")
            }
        }
    }
}
