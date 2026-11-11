// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module that provides file path-related facilities.

use anyhow::Context;
use hyper::StatusCode;
use percent_encoding::percent_decode_str;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use crate::Result;

/// `Path` extensions trait.
pub(crate) trait PathExt {
    /// If file path is hidden.
    fn is_hidden(&self) -> bool;
    /// If the path is a symlink or contains intermediate symlink components.
    fn contains_symlink(&self, base: &Path) -> Result<bool>;
    /// If a path component may be an 8.3 short-name alias of another entry.
    fn contains_short_name_alias(&self, base: &Path) -> Result<bool>;
}

impl PathExt for Path {
    /// Checks if the current path is hidden (dot file).
    fn is_hidden(&self) -> bool {
        self.components()
            .filter_map(|cmp| match cmp {
                Component::Normal(s) => s.to_str(),
                _ => None,
            })
            .any(|s| s.starts_with('.'))
    }

    /// Checks if the path is a symlink or contains intermediate symlink components.
    /// This could be an expensive operation as it requires filesystem access for each path component.
    fn contains_symlink(&self, base: &Path) -> Result<bool> {
        let mut current = base.to_path_buf();
        current.reserve(self.as_os_str().len());
        for component in self.components() {
            match component {
                Component::Normal(c) => {
                    current.push(c);
                    let meta = std::fs::symlink_metadata(&current).with_context(|| {
                        format!("unable to get metadata for path '{}'", current.display())
                    })?;
                    if meta.file_type().is_symlink() {
                        return Ok(true);
                    }
                }
                Component::CurDir => {}
                Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                    tracing::debug!(
                        "dir: skipping segment containing invalid prefix, dots or backslashes"
                    );
                }
            }
        }
        Ok(false)
    }

    /// Checks if a component of the path, relative to `base`, contains `~` but
    /// has no entry of that name (ignoring case) in its parent directory.
    ///
    /// NTFS and FAT volumes give each entry an 8.3 short name, such as `GIT~1`
    /// for `.git`, and open the entry by either name. The directory lists the
    /// long name only, so a short-name alias has no entry of its own.
    /// Canonicalization can't detect it: Linux keeps the alias spelling.
    ///
    /// Only components containing `~` cost a directory read. Returns an error
    /// when that directory can't be listed. Matching ignores case, so on a
    /// case-sensitive volume that keeps short names, a real entry spelled like
    /// an alias in another case hides that alias.
    fn contains_short_name_alias(&self, base: &Path) -> Result<bool> {
        let mut dir = base.to_path_buf();
        dir.reserve(self.as_os_str().len());
        for component in self.components() {
            let Component::Normal(name) = component else {
                continue;
            };
            if name.as_encoded_bytes().contains(&b'~') && !has_dir_entry(&dir, name)? {
                return Ok(true);
            }
            dir.push(name);
        }
        Ok(false)
    }
}

/// Checks if `dir` has an entry named `name`, ignoring case. `name` is
/// lowercased once, so comparing an entry doesn't allocate.
fn has_dir_entry(dir: &Path, name: &OsStr) -> Result<bool> {
    let lowercase = name.to_str().map(|name| {
        name.chars()
            .flat_map(char::to_lowercase)
            .collect::<String>()
    });
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("unable to read directory '{}'", dir.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("unable to read entry of '{}'", dir.display()))?;
        let entry_name = entry.file_name();
        if entry_name == name || is_lowercase_of(&entry_name, lowercase.as_deref()) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Checks if `name` lowercases to `lowercase`. Unicode lowercasing
/// approximates the case folding of NTFS and FAT.
fn is_lowercase_of(name: &OsStr, lowercase: Option<&str>) -> bool {
    match (name.to_str(), lowercase) {
        (Some(name), Some(lowercase)) => name
            .chars()
            .flat_map(char::to_lowercase)
            .eq(lowercase.chars()),
        _ => false,
    }
}

#[cfg(unix)]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    OsStr::from_bytes(bytes).into()
}

#[cfg(windows)]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    // This should really be OsStr::from_encoded_bytes_unchecked() but it’s
    // unsafe. With this fallback non-Unicode file names will result in 404.
    String::from_utf8_lossy(bytes).into_owned().into()
}

fn decode_tail_path(tail: &str) -> PathBuf {
    let bytes = percent_decode_str(tail.trim_start_matches('/')).collect::<Vec<_>>();
    path_from_bytes(&bytes)
}

/// Sanitizes a base/tail path and then it returns a unified one.
pub(crate) fn sanitize_path(base: &Path, tail: &str) -> Result<PathBuf, StatusCode> {
    let path_decoded = decode_tail_path(tail);
    let mut full_path = base.to_path_buf();
    tracing::trace!("dir: base={:?}, route={:?}", full_path, path_decoded);

    for component in path_decoded.components() {
        match component {
            Component::Normal(comp) => {
                // Protect against paths like `/foo/c:/bar/baz`
                // https://github.com/seanmonstar/warp/issues/937
                if Path::new(&comp)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
                {
                    full_path.push(comp)
                } else {
                    tracing::debug!("dir: skipping segment with invalid prefix");
                }
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                tracing::debug!(
                    "dir: skipping segment containing invalid prefix, dots or backslashes"
                );
            }
        }
    }
    Ok(full_path)
}

/// Returns `true` if `path` resolves inside `base_path` after canonicalization.
///
/// Fail-closed: I/O or canonicalize errors yield `false`, so dangling
/// symlinks are rejected as well.
/// Mirrors `static_files::enforce_containment` without the request cache.
///
/// Note: this is a check-then-use test — a symlink retargeted between this
/// check and a subsequent open can still resolve outside `base_path`, the
/// same known limitation as `enforce_containment`.
#[doc(hidden)]
#[must_use]
pub fn is_path_within_base(path: &Path, base_path: &Path) -> bool {
    let Ok(resolved) = path.canonicalize() else {
        return false;
    };
    if resolved.starts_with(base_path) {
        return true;
    }
    let Ok(base_resolved) = base_path.canonicalize() else {
        return false;
    };
    resolved.starts_with(base_resolved)
}

#[cfg(test)]
mod tests {
    use super::{PathExt, is_path_within_base, sanitize_path};
    use std::path::{Path, PathBuf};

    fn root_dir() -> PathBuf {
        PathBuf::from("docker/public/")
    }

    #[test]
    fn test_sanitize_path() {
        let base_dir = &PathBuf::from("docker/public");

        assert_eq!(
            sanitize_path(base_dir, "/index.html").unwrap(),
            root_dir().join("index.html")
        );

        // bad paths
        assert_eq!(
            sanitize_path(base_dir, "/../foo.html").unwrap(),
            root_dir().join("foo.html"),
        );
        assert_eq!(
            sanitize_path(base_dir, "/../W�foo.html").unwrap(),
            root_dir().join("W�foo.html"),
        );
        assert_eq!(
            sanitize_path(base_dir, "/%EF%BF%BD/../bar.html").unwrap(),
            root_dir().join("�/bar.html"),
        );
        assert_eq!(
            sanitize_path(base_dir, "àí/é%20/öüñ").unwrap(),
            root_dir().join("àí/é /öüñ"),
        );

        #[cfg(unix)]
        let expected_path = root_dir().join("C:\\/foo.html");
        #[cfg(windows)]
        let expected_path = PathBuf::from("docker/public/\\foo.html");
        assert_eq!(
            sanitize_path(base_dir, "/C:\\/foo.html").unwrap(),
            expected_path
        );
    }

    #[test]
    fn is_path_within_base_rejects_outside_symlink() {
        // Given: fixtures/public/readme.md points outside the public root
        let base = PathBuf::from("tests/fixtures/public");
        let outside = base.join("readme.md");

        // When/Then: resolved target is not within the public root
        assert!(
            !is_path_within_base(&outside, &base),
            "outside-root symlink must not be treated as within base"
        );
    }

    #[test]
    fn is_path_within_base_accepts_inside_file() {
        // Given: fixtures/public/index.htm lives inside the public root
        let base = PathBuf::from("tests/fixtures/public");
        let inside = base.join("index.htm");

        // When/Then: path resolves within base
        assert!(
            is_path_within_base(&inside, &base),
            "in-root file must be treated as within base"
        );
    }

    #[test]
    fn test_contains_symlink_returns_false() {
        let base = PathBuf::from("tests/fixtures/public");
        let user_path = PathBuf::from("./index.htm");

        match user_path.contains_symlink(&base) {
            Ok(contains) => assert!(!contains),
            Err(err) => panic!("unexpected error when checking for symlinks: {err}"),
        }
    }

    #[test]
    fn test_contains_symlink_returns_true() {
        let base = PathBuf::from("tests/fixtures/public");
        let user_path = PathBuf::from("./readme.md");

        match user_path.contains_symlink(&base) {
            Ok(contains) => assert!(contains),
            Err(err) => panic!("unexpected error when checking for symlinks: {err}"),
        }
    }

    #[test]
    fn test_contains_symlink_returns_error() {
        let base = PathBuf::from("tests/fixtures/public");
        let user_path = PathBuf::from("./unknown_file.txt");

        match user_path.contains_symlink(&base) {
            Ok(_) => panic!("expected error when checking for symlinks on non-existent path"),
            Err(err) => assert!(
                err.to_string().contains("unable to get metadata for path"),
                "unexpected error message: {err}"
            ),
        }
    }

    fn short_name_fixture() -> tempfile::TempDir {
        let base = tempfile::tempdir().unwrap();
        let root = base.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::create_dir_all(root.join("a~b/.env")).unwrap();
        std::fs::write(root.join(".git/config"), "config").unwrap();
        std::fs::write(root.join("report~1.txt"), "report").unwrap();
        base
    }

    #[test]
    fn contains_short_name_alias_detects_alias_of_hidden_dir() {
        let base = short_name_fixture();

        for path in ["GIT~1/config", "git~1", "a~b/ENV~1"] {
            assert!(
                Path::new(path)
                    .contains_short_name_alias(base.path())
                    .unwrap(),
                "{path} must be reported as a short-name alias"
            );
        }
    }

    #[test]
    fn contains_short_name_alias_accepts_names_with_tilde() {
        let base = short_name_fixture();

        for path in ["report~1.txt", "REPORT~1.TXT", "a~b", "a~b/.env"] {
            assert!(
                !Path::new(path)
                    .contains_short_name_alias(base.path())
                    .unwrap(),
                "{path} is an entry name, not an alias"
            );
        }
    }

    #[test]
    fn contains_short_name_alias_skips_names_without_tilde() {
        let base = short_name_fixture();

        assert!(
            !Path::new("missing/dir/file.txt")
                .contains_short_name_alias(base.path())
                .unwrap()
        );
    }

    #[test]
    fn contains_short_name_alias_errors_on_missing_dir() {
        let base = short_name_fixture();

        assert!(
            Path::new("missing/x~1")
                .contains_short_name_alias(base.path())
                .is_err()
        );
    }

    #[cfg(unix)]
    #[test]
    fn contains_short_name_alias_skips_base_components() {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        let parent = tempfile::tempdir().unwrap();
        let base = parent.path().join("r~1");
        std::fs::create_dir(&base).unwrap();
        std::fs::set_permissions(parent.path(), Permissions::from_mode(0o311)).unwrap();

        let result = Path::new("file.txt").contains_short_name_alias(&base);
        std::fs::set_permissions(parent.path(), Permissions::from_mode(0o755)).unwrap();

        assert!(!result.unwrap());
    }
}
