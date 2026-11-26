// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Translation of a rule's `source` glob into the regex whose capture groups feed the
//! `$N` placeholders of its `destination`.

use globset::GlobBuilder;
use percent_encoding::{AsciiSet, utf8_percent_encode};
use regex_lite::Regex;

use crate::fs::path::RULE_PATH_ENCODE;
use crate::{Context, Result};

/// Bytes [`rule_spelling`] percent-encodes in a `source` glob: the ones a rule path only
/// holds encoded, except `/`, `%`, and glob syntax.
const SOURCE_ENCODE: &AsciiSet = &RULE_PATH_ENCODE
    .remove(b'/')
    .remove(b'%')
    .remove(b'?')
    .remove(b'[')
    .remove(b']')
    .remove(b'{')
    .remove(b'}')
    .remove(b'\\')
    .remove(b'^');

/// Compiles a `source` glob into an equivalent regex.
///
/// `*` and `?` never cross a `/`, and each `{a,b}` alternation or `*` inside a group
/// becomes a capture group numbered from `$1`, in source order.
///
/// The regex matches [`crate::fs::path::rule_path`] spellings. On macOS and Windows,
/// whose default file systems ignore letter case, it ignores ASCII case too, as paths in
/// any case reach the same file.
pub(crate) fn to_regex(glob: &str) -> Result<Regex> {
    let glob = GlobBuilder::new(&rule_spelling(glob))
        .literal_separator(true)
        .build()
        .with_context(|| format!("can not compile glob pattern for source: {glob}"))?;

    // Turn globset's non-capturing groups into capturing ones, so every alternation
    // and wildcard group can be referenced from the destination
    let pattern = glob
        .regex()
        .trim_start_matches("(?-u)")
        .replace("?:.*", ".*")
        .replace("?:", "")
        .replace(".*.*", ".*")
        .replace("([^/]*[^/]*)", "(.*)");
    let pattern = if cfg!(any(target_os = "macos", windows)) {
        format!("(?i){pattern}")
    } else {
        pattern
    };
    tracing::debug!("url rule glob pattern: {}", glob.glob());
    tracing::debug!("url rule regex equivalent: {pattern}");

    Regex::new(&pattern)
        .with_context(|| format!("can not compile regex pattern equivalent for source: {pattern}"))
}

/// Spells the text of a `source` glob the way rule paths are spelled: percent-escapes in
/// uppercase hex, and spaces, non-ASCII and other characters that need an escape encoded.
/// A `%` that starts no escape becomes `%25`.
fn rule_spelling(glob: &str) -> String {
    let mut spelling = String::with_capacity(glob.len());
    let mut rest = glob;
    while let Some(i) = rest.find('%') {
        spelling.extend(utf8_percent_encode(&rest[..i], SOURCE_ENCODE));
        let escape = rest[i..]
            .get(..3)
            .filter(|escape| escape.bytes().skip(1).all(|b| b.is_ascii_hexdigit()));
        match escape {
            Some(escape) => spelling.push_str(&escape.to_ascii_uppercase()),
            None => spelling.push_str("%25"),
        }
        rest = &rest[i + escape.map_or(1, str::len)..];
    }
    spelling.extend(utf8_percent_encode(rest, SOURCE_ENCODE));
    spelling
}

#[cfg(test)]
mod tests {
    use super::{rule_spelling, to_regex};

    fn captures(glob: &str, path: &str) -> Option<Vec<String>> {
        let re = to_regex(glob).unwrap();
        let caps = re.captures(path)?;
        Some(
            caps.iter()
                .skip(1)
                .map(|c| c.map_or(String::new(), |m| m.as_str().to_owned()))
                .collect(),
        )
    }

    #[test]
    fn glob_literal_matches_whole_path_only() {
        assert_eq!(captures("/old", "/old"), Some(vec![]));
        assert_eq!(captures("/old", "/old/x"), None);
        assert_eq!(captures("/old", "/older"), None);
    }

    #[test]
    fn glob_alternation_is_a_capture_group() {
        assert_eq!(
            captures("/{main,style}.css", "/style.css"),
            Some(vec!["style".into()])
        );
        assert_eq!(captures("/{main,style}.css", "/other.css"), None);
    }

    #[test]
    fn glob_star_does_not_cross_separator() {
        assert_eq!(
            captures("/assets/{*}.js", "/assets/app.js"),
            Some(vec!["app".into()])
        );
        assert_eq!(captures("/assets/{*}.js", "/assets/a/b.js"), None);
    }

    #[test]
    fn glob_recursive_group_crosses_separator() {
        assert_eq!(
            captures("/recursive/{**}", "/recursive/a/b/c.html"),
            Some(vec!["a/b/c.html".into()])
        );
    }

    #[test]
    fn glob_leading_globstar_captures_directories() {
        assert_eq!(
            captures("**/{*}.{jpg,jpeg}", "/a/b/photo.jpeg"),
            Some(vec!["/a/b/".into(), "photo".into(), "jpeg".into()])
        );
    }

    #[test]
    fn glob_source_takes_rule_path_spelling() {
        for (glob, spelling) in [
            ("/assets/{*}.{js,css}", "/assets/{*}.{js,css}"),
            ("/caf%c3%a9/x", "/caf%C3%A9/x"),
            ("/café/**", "/caf%C3%A9/**"),
            ("/my file|<1>", "/my%20file%7C%3C1%3E"),
            ("/a[!b]?\\*^", "/a[!b]?\\*^"),
            ("/100%/%zz/%a", "/100%25/%25zz/%25a"),
            ("/ünï/%e2%82%ac", "/%C3%BCn%C3%AF/%E2%82%AC"),
            ("/%é", "/%25%C3%A9"),
        ] {
            assert_eq!(rule_spelling(glob), spelling, "{glob}");
        }
    }

    #[test]
    fn glob_encoded_source_matches_rule_path() {
        assert_eq!(
            captures("/café/{*}", "/caf%C3%A9/x"),
            Some(vec!["x".into()])
        );
        assert_eq!(captures("/caf%c3%a9", "/caf%C3%A9"), Some(vec![]));
        assert_eq!(captures("/a b", "/a%20b"), Some(vec![]));
    }

    #[test]
    fn glob_letter_case_follows_the_platform() {
        let ignores_case = cfg!(any(target_os = "macos", windows));
        assert_eq!(captures("/Old", "/old").is_some(), ignores_case);
        assert_eq!(captures("/old/{*}", "/OLD/x").is_some(), ignores_case);
        assert_eq!(captures("/old", "/old"), Some(vec![]));
    }

    #[test]
    fn glob_invalid_pattern_is_an_error() {
        let err = to_regex("/{unclosed").unwrap_err();
        assert!(err.to_string().contains("/{unclosed"), "{err}");
    }
}
