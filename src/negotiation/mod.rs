// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Content negotiation between variants of a file.
//!
//! A variant is stored next to its original file, named after it with a
//! suffix appended: `photo.jpg.avif` is an AVIF variant of `photo.jpg`. Each
//! kind of negotiation ranks the suffixes the client accepts from a request
//! header, and [`try_variant_uri`] picks the first of them found on disk. The
//! original is what every other client gets, so a variant is only served when
//! its original exists.

pub(crate) mod image;

use std::path::{Path, PathBuf};

use crate::fs::meta::try_metadata;
use crate::fs::path::{PathExt, sanitize_path};

/// Returns the URI path of the first variant of the file at `uri_path`
/// found on disk, trying `suffixes` in order.
///
/// The URI path returned is `uri_path` with `.` and the suffix appended. The
/// static-files handler sanitizes it and applies the containment, symlink and
/// hidden-file checks again before serving it. `None` when no variant is
/// found, the original is not a file, the path is hidden, or a suffix is not
/// made of ASCII letters, digits and `-` only, which keeps a suffix taken from
/// a request header from adding a path component.
pub(crate) fn try_variant_uri<'a>(
    base_path: &Path,
    uri_path: &str,
    include_hidden: bool,
    suffixes: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    // SECURITY: percent-decode and strip traversal components before
    // touching the filesystem.
    let file_path = sanitize_path(base_path, uri_path).ok()?;
    let relative = file_path.strip_prefix(base_path).ok()?;
    // The root itself has no variant: its siblings are outside `base_path`
    if relative.as_os_str().is_empty() {
        return None;
    }
    if !include_hidden && relative.is_hidden() {
        tracing::debug!("negotiation: skipping hidden path {:?}", relative);
        return None;
    }

    let (suffix, variant_path) = suffixes
        .into_iter()
        .filter(|suffix| is_valid_suffix(suffix))
        .find_map(|suffix| try_variant(&file_path, suffix).map(|path| (suffix, path)))?;
    if !is_file(&file_path) {
        tracing::debug!(
            "negotiation: skipping variant of missing original {:?}",
            relative
        );
        return None;
    }

    // 8.3 short-name aliases such as `GIT~1` are checked on the variant found,
    // like markdown negotiation does. A failed check counts as an alias.
    let relative_variant = variant_path.strip_prefix(base_path).ok()?;
    if !include_hidden
        && relative_variant
            .contains_short_name_alias(base_path)
            .unwrap_or(true)
    {
        tracing::debug!(
            "negotiation: skipping short-name alias {:?}",
            relative_variant
        );
        return None;
    }
    tracing::debug!("negotiation: found variant {:?}", variant_path);

    Some([uri_path, ".", suffix].concat())
}

/// Whether a suffix is made of ASCII letters, digits and `-` only.
#[inline]
fn is_valid_suffix(suffix: &str) -> bool {
    !suffix.is_empty()
        && suffix
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// Returns the path of the variant of `file_path` with the given suffix, if
/// that variant is a file on disk.
fn try_variant(file_path: &Path, suffix: &str) -> Option<PathBuf> {
    let mut variant_name = file_path.file_name()?.to_os_string();
    variant_name.push(".");
    variant_name.push(suffix);
    let variant_path = file_path.with_file_name(variant_name);
    is_file(&variant_path).then_some(variant_path)
}

/// Whether the path is a file on disk.
#[inline]
fn is_file(path: &Path) -> bool {
    matches!(try_metadata(path), Ok((_, false)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A web root with `photo.jpg` and both of its variants, `logo.png` and
    /// its WebP variant, an orphan `orphan.gif.webp`, and an `index.html` with
    /// a French variant.
    fn variant_root() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        for name in [
            "photo.jpg",
            "photo.jpg.avif",
            "photo.jpg.webp",
            "logo.png",
            "logo.png.webp",
            "orphan.gif.webp",
            "index.html",
            "index.html.fr",
        ] {
            std::fs::write(base.join(name), name).unwrap();
        }
        (tmp, base)
    }

    fn variant_uri(base: &Path, uri_path: &str, suffixes: &[&str]) -> Option<String> {
        try_variant_uri(base, uri_path, false, suffixes.iter().copied())
    }

    #[test]
    fn variant_uri_takes_first_suffix_on_disk() {
        let (_tmp, base) = variant_root();
        let result = variant_uri(&base, "/photo.jpg", &["avif", "webp"]);
        assert_eq!(result.as_deref(), Some("/photo.jpg.avif"));

        let result = variant_uri(&base, "/photo.jpg", &["webp", "avif"]);
        assert_eq!(result.as_deref(), Some("/photo.jpg.webp"));

        let result = variant_uri(&base, "/logo.png", &["avif", "webp"]);
        assert_eq!(result.as_deref(), Some("/logo.png.webp"));
    }

    #[test]
    fn variant_uri_with_any_suffix_kind() {
        let (_tmp, base) = variant_root();
        let result = variant_uri(&base, "/index.html", &["de", "fr", "en"]);
        assert_eq!(result.as_deref(), Some("/index.html.fr"));
    }

    #[test]
    fn variant_uri_none_without_variant_or_original() {
        let (_tmp, base) = variant_root();
        for uri_path in ["/missing.jpg", "/orphan.gif", "/photo.jpg.webp", "/"] {
            let result = variant_uri(&base, uri_path, &["avif", "webp"]);
            assert_eq!(result, None, "{uri_path}");
        }
        assert_eq!(variant_uri(&base, "/photo.jpg", &[]), None);
    }

    #[test]
    fn variant_uri_skips_variant_directory() {
        let (_tmp, base) = variant_root();
        std::fs::write(base.join("dir.png"), "png").unwrap();
        std::fs::create_dir(base.join("dir.png.avif")).unwrap();
        assert_eq!(variant_uri(&base, "/dir.png", &["avif"]), None);
    }

    #[test]
    fn variant_uri_rejects_unsafe_suffixes() {
        let (_tmp, base) = variant_root();
        std::fs::create_dir(base.join("photo.jpg.d")).unwrap();
        std::fs::write(base.join("photo.jpg.d/x"), "x").unwrap();
        for suffix in ["", "d/x", "d\\x", "..", "d%2fx", "a b", "fr;q"] {
            assert_eq!(
                variant_uri(&base, "/photo.jpg", &[suffix]),
                None,
                "{suffix}"
            );
        }
        assert_eq!(
            variant_uri(&base, "/photo.jpg", &["../x", "webp"]).as_deref(),
            Some("/photo.jpg.webp")
        );
    }

    #[test]
    fn variant_uri_rejects_hidden_path() {
        let (_tmp, base) = variant_root();
        std::fs::create_dir(base.join(".private")).unwrap();
        std::fs::write(base.join(".private/photo.jpg"), "jpg").unwrap();
        std::fs::write(base.join(".private/photo.jpg.webp"), "webp").unwrap();
        let uri_path = "/.private/photo.jpg";

        assert_eq!(variant_uri(&base, uri_path, &["webp"]), None);
        assert_eq!(
            try_variant_uri(&base, uri_path, true, ["webp"]).as_deref(),
            Some("/.private/photo.jpg.webp")
        );
    }

    #[test]
    fn variant_uri_resolves_to_variant() {
        let (_tmp, base) = variant_root();
        std::fs::create_dir(base.join("a b")).unwrap();
        std::fs::write(base.join("a b/50%.png"), "png").unwrap();
        std::fs::write(base.join("a b/50%.png.webp"), "webp").unwrap();

        for uri_path in [
            "/a%20b/50%25.png",
            "/../a%20b/50%25.png",
            "/%2e%2e/a%20b/./50%25.png",
            "//a%20b//50%25.png",
        ] {
            let result = variant_uri(&base, uri_path, &["webp"]).expect(uri_path);
            assert_eq!(result, [uri_path, ".webp"].concat());
            let resolved = sanitize_path(&base, &result).unwrap();
            assert_eq!(resolved, base.join("a b/50%.png.webp"), "{uri_path}");
        }
    }

    #[test]
    fn variant_uri_traversal_stays_in_base() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let base = root.join("public");
        std::fs::create_dir(&base).unwrap();
        std::fs::write(root.join("outside.jpg"), "jpg").unwrap();
        std::fs::write(root.join("outside.jpg.webp"), "webp").unwrap();

        for uri_path in [
            "/../outside.jpg",
            "/%2e%2e/outside.jpg",
            "/..%2foutside.jpg",
        ] {
            assert_eq!(variant_uri(&base, uri_path, &["webp"]), None, "{uri_path}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn variant_uri_symlinked_variant_defers_to_static_handler() {
        // The symlink policy is enforced by the static-files handler on the
        // variant URI, like it is for any other file
        let (_tmp, base) = variant_root();
        std::fs::write(base.join("target.webp"), "webp").unwrap();
        std::fs::write(base.join("link.png"), "png").unwrap();
        std::os::unix::fs::symlink(base.join("target.webp"), base.join("link.png.webp")).unwrap();
        let result = variant_uri(&base, "/link.png", &["webp"]);
        assert_eq!(result.as_deref(), Some("/link.png.webp"));
    }
}
