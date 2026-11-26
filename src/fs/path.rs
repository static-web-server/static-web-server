// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module that provides file path-related facilities.

use anyhow::Context;
use hyper::StatusCode;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_decode_str, percent_encode};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::{Component, Path, PathBuf};
#[cfg(target_os = "macos")]
use unicode_normalization::char::{canonical_combining_class, decompose_canonical};

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
    /// has no entry of that name (ignoring case, and on macOS Unicode
    /// normalization) in its parent directory.
    ///
    /// NTFS and FAT volumes give each entry an 8.3 short name, such as `GIT~1`
    /// for `.git`, and open the entry by either name. The directory lists the
    /// long name only, so a short-name alias has no entry of its own.
    /// Canonicalization can't detect it: Linux keeps the alias spelling.
    ///
    /// Only components containing `~` cost a directory read. Returns an error
    /// when that directory can't be listed. Matching ignores case, so on a
    /// case-sensitive volume that keeps short names, a real entry spelled like
    /// an alias in another case hides that alias. A name matched ignoring
    /// normalization must also open the file listed for the entry.
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

/// Checks if `dir` has an entry named `name` once both are folded by
/// `fold_name`, and `name` opens that entry (`opens_entry`). `name` is folded
/// once; only a non-ASCII entry on macOS allocates to be compared.
fn has_dir_entry(dir: &Path, name: &OsStr) -> Result<bool> {
    let folded = name.to_str().map(fold_name);
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("unable to read directory '{}'", dir.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("unable to read entry of '{}'", dir.display()))?;
        let entry_name = entry.file_name();
        if entry_name == name {
            return Ok(true);
        }
        if folds_to(&entry_name, folded.as_deref()) && opens_entry(dir, name, &entry) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Folds `name` for comparison: lowercase. Unicode lowercasing approximates
/// the case folding of NTFS and FAT.
///
/// Normalization is kept: Linux vfat tells the forms apart, so a real entry
/// in another form than a non-ASCII alias such as `RÉSUMÉ~1` would hide it.
#[cfg(not(target_os = "macos"))]
fn fold_name(name: &str) -> Vec<char> {
    name.chars().flat_map(char::to_lowercase).collect()
}

/// Folds `name` for comparison: lowercase, in canonical decomposition (NFD).
///
/// APFS, HFS+ and the macOS FAT and exFAT drivers open a name in either
/// normalization form but list it as stored, so a request for a precomposed
/// `é` opens an entry listed as `e` and U+0301. A lowercase mapping is
/// decomposed again, since it may be precomposed.
///
/// Ignoring normalization alone could hide an alias, so `opens_entry` checks
/// a name matched this way.
#[cfg(target_os = "macos")]
fn fold_name(name: &str) -> Vec<char> {
    let mut folded = Vec::with_capacity(name.len());
    for c in name.chars() {
        decompose_canonical(c, |c| {
            for c in c.to_lowercase() {
                decompose_canonical(c, |c| push_in_canonical_order(&mut folded, c));
            }
        });
    }
    folded
}

/// Appends `c`, moving a combining mark before the preceding marks of a
/// higher combining class, the order NFD requires.
#[cfg(target_os = "macos")]
fn push_in_canonical_order(folded: &mut Vec<char>, c: char) {
    let class = canonical_combining_class(c);
    let mut at = folded.len();
    while class != 0 && at > 0 && canonical_combining_class(folded[at - 1]) > class {
        at -= 1;
    }
    folded.insert(at, c);
}

/// Checks if `name`, which folds to the name of `entry`, opens that entry. It
/// does: folding ignores case only, as NTFS and FAT do.
#[cfg(not(target_os = "macos"))]
fn opens_entry(_dir: &Path, _name: &OsStr, _entry: &DirEntry) -> bool {
    true
}

/// Checks if `name`, which folds to the name of `entry`, opens that entry.
///
/// A name in another case is taken as the entry's, as on other platforms. A
/// name in another normalization form must open the inode listed for the
/// entry. The macOS NTFS driver opens 8.3 short names, and so may an SMB or
/// NFS server, so the short name `RÉSUMÉ~1` of a hidden file would otherwise
/// pass for an entry listed as `re\u{301}sume\u{301}~1`.
///
/// The inode opened by the entry name can't be compared instead: the NTFS
/// driver composes a name before looking it up, so that entry name opens
/// the hidden file too. The macOS FAT and exFAT drivers list an empty file
/// under another inode than it opens with, so one isn't matched.
#[cfg(target_os = "macos")]
fn opens_entry(dir: &Path, name: &OsStr, entry: &DirEntry) -> bool {
    use std::os::unix::fs::{DirEntryExt, MetadataExt};

    is_same_ignoring_case(name, &entry.file_name())
        || std::fs::symlink_metadata(dir.join(name)).is_ok_and(|meta| meta.ino() == entry.ino())
}

/// Checks if both names lowercase to the same name.
#[cfg(target_os = "macos")]
fn is_same_ignoring_case(name: &OsStr, other: &OsStr) -> bool {
    match (name.to_str(), other.to_str()) {
        (Some(name), Some(other)) => name
            .chars()
            .flat_map(char::to_lowercase)
            .eq(other.chars().flat_map(char::to_lowercase)),
        _ => false,
    }
}

/// Checks if `name` folds to `folded`, without allocating.
#[cfg(not(target_os = "macos"))]
fn folds_to(name: &OsStr, folded: Option<&[char]>) -> bool {
    match (name.to_str(), folded) {
        (Some(name), Some(folded)) => name
            .chars()
            .flat_map(char::to_lowercase)
            .eq(folded.iter().copied()),
        _ => false,
    }
}

/// Checks if `name` folds to `folded`. An ASCII name is compared without
/// allocating: folding only lowercases it.
#[cfg(target_os = "macos")]
fn folds_to(name: &OsStr, folded: Option<&[char]>) -> bool {
    match (name.to_str(), folded) {
        (Some(name), Some(folded)) if name.is_ascii() => name
            .chars()
            .map(|c| c.to_ascii_lowercase())
            .eq(folded.iter().copied()),
        (Some(name), Some(folded)) => fold_name(name) == folded,
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
#[doc(hidden)]
pub fn sanitize_path(base: &Path, tail: &str) -> Result<PathBuf, StatusCode> {
    let path_decoded = decode_tail_path(tail);
    let mut full_path = base.to_path_buf();
    tracing::trace!("dir: base={:?}, route={:?}", full_path, path_decoded);
    full_path.extend(served_components(&path_decoded));
    Ok(full_path)
}

/// Returns the segments of a decoded request path that [`sanitize_path`] keeps, in order.
///
/// Empty, `.` and `..` segments are dropped (`..` does not remove the segment before it),
/// and so are segments with a drive or root prefix, such as `c:` in `/foo/c:/bar`.
fn served_components(path_decoded: &Path) -> impl Iterator<Item = &OsStr> {
    path_decoded
        .components()
        .filter_map(|component| match component {
            Component::Normal(comp) => {
                // Protect against paths like `/foo/c:/bar/baz`
                // https://github.com/seanmonstar/warp/issues/937
                let is_plain = Path::new(comp)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)));
                if !is_plain {
                    tracing::debug!("dir: skipping segment with invalid prefix");
                }
                is_plain.then_some(comp)
            }
            Component::CurDir => None,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                tracing::debug!(
                    "dir: skipping segment containing invalid prefix, dots or backslashes"
                );
                None
            }
        })
}

/// Bytes [`rule_path`] percent-encodes: all but RFC 3986 unreserved characters,
/// sub-delims, `:` and `@`.
pub(crate) const RULE_PATH_ENCODE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b'!')
    .remove(b'$')
    .remove(b'&')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')')
    .remove(b'*')
    .remove(b'+')
    .remove(b',')
    .remove(b';')
    .remove(b'=')
    .remove(b':')
    .remove(b'@');

/// [`RULE_PATH_ENCODE`] without `/`, to check a whole path at once.
const RULE_PATH_ENCODE_KEEP_SLASH: &AsciiSet = &RULE_PATH_ENCODE.remove(b'/');

/// Returns the one spelling of a request path that redirect and rewrite rules are matched
/// against: the segments [`sanitize_path`] serves, percent-encoded with uppercase hex.
///
/// Every request path that reaches the same file has the same rule path, so a rule can't
/// be skipped by `//`, `./`, `%2F` or needless percent-encoding (RFC 3986 §6.2.2). A path
/// ending in `/`, `/.` or `/..` keeps a trailing `/` (RFC 3986 §5.2.4).
#[must_use]
pub(crate) fn rule_path(uri_path: &str) -> Cow<'_, str> {
    if is_rule_path(uri_path) {
        return Cow::Borrowed(uri_path);
    }
    let path_decoded = decode_tail_path(uri_path);
    let mut path = String::with_capacity(uri_path.len());
    for comp in served_components(&path_decoded) {
        path.push('/');
        path.extend(percent_encode(comp.as_encoded_bytes(), RULE_PATH_ENCODE));
    }
    let names_dir = path_decoded.as_os_str().is_empty()
        || [b"/".as_slice(), b"/.", b"/.."]
            .iter()
            .any(|end| path_decoded.as_os_str().as_encoded_bytes().ends_with(end))
        || matches!(path_decoded.as_os_str().as_encoded_bytes(), b"." | b"..")
        || (cfg!(windows) && path_decoded.as_os_str().as_encoded_bytes().ends_with(b"\\"));
    if names_dir || path.is_empty() {
        path.push('/');
    }
    Cow::Owned(path)
}

/// Returns whether `uri_path` already is its own [`rule_path`]: it starts with `/`, holds no
/// empty, `.` or `..` segment, and every byte stays as is when encoded. On Windows, nor does
/// it hold a `\` or a `:`, which separate segments and name drives there.
fn is_rule_path(uri_path: &str) -> bool {
    let Some(rest) = uri_path.strip_prefix('/') else {
        return false;
    };
    let mut segments = rest.split('/').peekable();
    while let Some(segment) = segments.next() {
        let is_last = segments.peek().is_none();
        if (segment.is_empty() && !is_last) || segment == "." || segment == ".." {
            return false;
        }
    }
    let is_kept = matches!(
        Cow::from(percent_encode(
            uri_path.as_bytes(),
            RULE_PATH_ENCODE_KEEP_SLASH
        )),
        Cow::Borrowed(_)
    );
    is_kept && (cfg!(not(windows)) || !uri_path.contains(['\\', ':']))
}

/// Returns `true` if `path` resolves inside `base_path` after canonicalization.
///
/// Fail-closed: I/O or canonicalize errors yield `false`, so dangling
/// symlinks are rejected as well.
/// Mirrors `static_files::security::enforce_containment` without the request cache.
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
    use super::{PathExt, is_path_within_base, rule_path, sanitize_path};
    use std::borrow::Cow;
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
        // Given: fixtures/public/readme.md → ../../../README.md (outside base)
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

    /// Names with `é` stored decomposed and precomposed. HFS+ and the macOS
    /// FAT driver list both decomposed.
    fn normalization_fixture() -> tempfile::TempDir {
        let base = tempfile::tempdir().unwrap();
        let root = base.path();
        std::fs::create_dir(root.join("de\u{301}~x")).unwrap();
        std::fs::write(root.join("de\u{301}~x/file.txt"), "file").unwrap();
        std::fs::write(root.join("cafe\u{301}~.txt"), "decomposed").unwrap();
        std::fs::write(root.join("r\u{e9}sum\u{e9}~1.txt"), "precomposed").unwrap();
        base
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn contains_short_name_alias_ignores_unicode_normalization() {
        let base = normalization_fixture();
        if !base.path().join("caf\u{e9}~.txt").exists() {
            eprintln!("skipped: this volume tells normalization forms apart");
            return;
        }
        let in_another_case: &[&str] = if base.path().join("CAFE\u{301}~.TXT").exists() {
            &["CAF\u{c9}~.TXT", "RE\u{301}SUM\u{c9}~1.TXT"]
        } else {
            &[]
        };

        for path in [
            "caf\u{e9}~.txt",
            "cafe\u{301}~.txt",
            "d\u{e9}~x/file.txt",
            "r\u{e9}sum\u{e9}~1.txt",
            "re\u{301}sume\u{301}~1.txt",
        ]
        .iter()
        .chain(in_another_case)
        {
            assert!(
                !Path::new(path)
                    .contains_short_name_alias(base.path())
                    .unwrap(),
                "{path:?} is an entry name in another normalization form, not an alias"
            );
        }
    }

    #[cfg(target_os = "macos")]
    fn dir_entry(dir: &Path, name: &str) -> std::fs::DirEntry {
        let mut entries = std::fs::read_dir(dir).unwrap().map(Result::unwrap);
        entries.find(|entry| entry.file_name() == name).unwrap()
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn opens_entry_accepts_name_in_another_case_or_normalization_form() {
        let base = normalization_fixture();
        let entry = dir_entry(base.path(), "cafe\u{301}~.txt");
        if !base.path().join("caf\u{e9}~.txt").exists() {
            eprintln!("skipped: this volume tells normalization forms apart");
            return;
        }

        for name in ["CAFE\u{301}~.TXT", "caf\u{e9}~.txt"] {
            assert!(
                super::opens_entry(base.path(), name.as_ref(), &entry),
                "{name:?} opens the entry"
            );
        }
    }

    /// A short name opens another file than the entry it is spelled like:
    /// here `r\u{e9}sum\u{e9}~1.txt` stands in for the file it is an alias of.
    #[cfg(target_os = "macos")]
    #[test]
    fn opens_entry_rejects_name_opening_another_file() {
        let base = normalization_fixture();
        let entry = dir_entry(base.path(), "cafe\u{301}~.txt");

        for name in ["r\u{e9}sum\u{e9}~1.txt", "de\u{301}~x", "missing~1.txt"] {
            assert!(
                !super::opens_entry(base.path(), name.as_ref(), &entry),
                "{name:?} doesn't open the entry"
            );
        }
    }

    #[test]
    fn contains_short_name_alias_detects_alias_with_other_letters() {
        let base = normalization_fixture();

        for path in [
            "cafe~.txt",
            "caf\u{e8}~.txt",
            "cafe\u{300}~.txt",
            "CAF\u{c9}~1.TXT",
            "de\u{301}~x/FILE~1.TXT",
        ] {
            assert!(
                Path::new(path)
                    .contains_short_name_alias(base.path())
                    .unwrap(),
                "{path:?} must be reported as a short-name alias"
            );
        }
    }

    /// Linux vfat tells normalization forms apart and opens a non-ASCII short
    /// name such as `RÉSUMÉ~1`, so an entry in another form can't hide it.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn contains_short_name_alias_detects_alias_in_another_normalization_form() {
        let base = normalization_fixture();

        for path in [
            "caf\u{e9}~.txt",
            "CAF\u{c9}~.TXT",
            "re\u{301}sume\u{301}~1.txt",
            "RE\u{301}SUME\u{301}~1.TXT",
        ] {
            assert!(
                Path::new(path)
                    .contains_short_name_alias(base.path())
                    .unwrap(),
                "{path:?} must be reported as a short-name alias"
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

    // Property-based regression tests for `sanitize_path`
    //
    // These properties encode the security invariants of path
    // sanitisation: regardless of the request tail (random bytes,
    // percent-encoded escapes, dot segments, Windows drive prefixes,
    // mixed slashes, embedded NULs) the resulting path MUST stay
    // inside the configured `base` directory.
    //
    // We assert this structurally (no `RootDir`/`Prefix`/`ParentDir`
    // components beyond `base` itself) so the property holds without
    // touching the filesystem.
    use proptest::prelude::*;
    use std::path::Component;

    fn assert_under_base(base: &std::path::Path, full: &std::path::Path) {
        let extra = full
            .strip_prefix(base)
            .expect("sanitized path must extend base");
        for comp in extra.components() {
            match comp {
                Component::Normal(_) | Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    panic!("sanitize_path leaked traversal component {comp:?} in {full:?}");
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256, ..ProptestConfig::default()
        })]

        /// Arbitrary ASCII-ish tails (including `../`, `\\`, `%2e`, NULs,
        /// drive letters) must never escape the base directory.
        #[test]
        fn prop_sanitize_path_never_escapes_base(tail in "\\PC{0,128}") {
            let base = PathBuf::from("docker/public");
            if let Ok(full) = sanitize_path(&base, &tail) {
                assert_under_base(&base, &full);
            }
        }

        /// `fold_name` must agree with the reference NFD implementation,
        /// including the order of stacked combining marks: the second
        /// alternative mixes marks of classes 10, 202, 220, 230 and 240.
        #[cfg(target_os = "macos")]
        #[test]
        fn prop_fold_name_matches_reference_nfd(
            name in prop_oneof![
                "\\PC{0,24}",
                "[aE\u{e9}\u{c5}\u{212b}\u{1e69}\u{1f88}\u{ac01}\u{5b0}\u{328}\u{323}\u{307}\u{301}\u{345}]{0,12}",
            ]
        ) {
            use unicode_normalization::UnicodeNormalization;

            let expected = name
                .nfd()
                .flat_map(char::to_lowercase)
                .nfd()
                .collect::<Vec<_>>();
            prop_assert_eq!(super::fold_name(&name), expected);
        }

        /// Targeted traversal patterns: many `../` segments interleaved
        /// with arbitrary content.
        #[test]
        fn prop_sanitize_path_resists_dot_dot_floods(
            tail in proptest::collection::vec(
                prop_oneof![
                    Just("../".to_string()),
                    Just("..\\".to_string()),
                    Just("%2e%2e/".to_string()),
                    Just("./".to_string()),
                    "[a-zA-Z0-9_.-]{1,8}".prop_map(String::from),
                ],
                0..32,
            )
        ) {
            let base = PathBuf::from("docker/public");
            let tail = tail.concat();
            if let Ok(full) = sanitize_path(&base, &tail) {
                assert_under_base(&base, &full);
            }
        }
    }

    #[test]
    fn rule_path_keeps_canonical_paths_borrowed() {
        for path in ["/", "/index.html", "/a/b/", "/ab:c@d!$&'()*+,;=-._~"] {
            if cfg!(windows) && path.contains(':') {
                continue;
            }
            assert!(matches!(rule_path(path), Cow::Borrowed(_)), "{path}");
            assert_eq!(rule_path(path), path);
        }
    }

    #[test]
    fn rule_path_normalizes_equivalent_spellings() {
        for (path, expected) in [
            ("/a/./b", "/a/b"),
            ("//a//b", "/a/b"),
            ("/a/../b", "/a/b"),
            ("/%61/b", "/a/b"),
            ("/a%2Fb", "/a/b"),
            ("/a/%2e/b", "/a/b"),
            ("/a%20b", "/a%20b"),
            ("/caf%C3%A9", "/caf%C3%A9"),
            ("/caf%c3%a9", "/caf%C3%A9"),
            ("/ab%3a%40", "/ab:@"),
            ("/a%25b", "/a%25b"),
            ("/a/", "/a/"),
            ("/a/.", "/a/"),
            ("/a/..", "/a/"),
            ("/a%2F", "/a/"),
            ("/.", "/"),
            ("//", "/"),
            ("", "/"),
        ] {
            assert_eq!(rule_path(path), expected, "{path}");
        }
    }

    proptest! {
        /// Rule paths take untrusted input: they must never panic, be their own rule path,
        /// and name the file `sanitize_path` serves for the original path.
        #[test]
        fn prop_rule_path_is_canonical_and_serves_same_file(
            path in "/[a-zA-Z0-9%./_~!$&'()*+,;=:@-]{0,48}",
        ) {
            let canonical = rule_path(&path).into_owned();
            prop_assert!(canonical.starts_with('/'));
            prop_assert_eq!(rule_path(&canonical), canonical.as_str());
            let base = Path::new("/base");
            prop_assert_eq!(sanitize_path(base, &canonical), sanitize_path(base, &path));
        }
    }
}
