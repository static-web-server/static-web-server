// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Translation of a rule's `source` glob into the regex whose capture groups feed the
//! `$N` placeholders of its `destination`.

use globset::{GlobBuilder, GlobMatcher};
use percent_encoding::{AsciiSet, percent_encode, utf8_percent_encode};
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
    let spelling = checked_spelling(glob, "url rule");
    let glob = GlobBuilder::new(&spelling)
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

/// Compiles the `source` glob of a custom headers rule into a matcher of
/// [`crate::fs::path::rule_path`] spellings.
///
/// `*` crosses `/` here, as it always did for header sources. The source takes the
/// spelling and the letter case handling of [`to_regex`].
pub(crate) fn to_matcher(glob: &str) -> Result<GlobMatcher> {
    let matcher = GlobBuilder::new(&checked_spelling(glob, "header"))
        .case_insensitive(cfg!(any(target_os = "macos", windows)))
        .build()
        .with_context(|| format!("can not compile glob pattern for header source: {glob}"))?
        .compile_matcher();
    Ok(matcher)
}

/// Returns the rule path spelling of the `source` glob of a `kind` of rule, and warns
/// when it can match no request.
fn checked_spelling(glob: &str, kind: &str) -> String {
    let spelling = rule_spelling(glob);
    if !can_match(&spelling) {
        tracing::warn!(
            "{kind} source {glob:?} matches no request: request paths are matched with a \
             leading `/`, without empty, `.` or `..` segments, and with `/` unencoded"
        );
    }
    spelling
}

/// Spells the text of a `source` glob the way rule paths are spelled: percent-escapes in
/// uppercase hex, and spaces, non-ASCII and other characters that need an escape encoded.
/// An escape of a character that rule paths hold as it is, such as `%61` or `%2B`, is
/// decoded (RFC 3986 §6.2.2.2), `*` and `,` as the literals `[*]` and `[,]`, since they
/// are glob syntax. A `%` that starts no escape becomes `%25`.
fn rule_spelling(glob: &str) -> String {
    let mut spelling = String::with_capacity(glob.len());
    let mut rest = glob;
    while let Some(i) = rest.find('%') {
        spelling.extend(utf8_percent_encode(&rest[..i], SOURCE_ENCODE));
        let escape = rest[i..]
            .get(..3)
            .filter(|escape| escape.bytes().skip(1).all(|b| b.is_ascii_hexdigit()));
        let byte = escape.and_then(|escape| u8::from_str_radix(&escape[1..], 16).ok());
        match (escape, byte) {
            (_, Some(b @ (b'*' | b','))) => spelling.extend(['[', char::from(b), ']']),
            (_, Some(b)) if is_kept_in_rule_paths(b) => {
                spelling.push(char::from(b));
            }
            (Some(escape), _) => spelling.push_str(&escape.to_ascii_uppercase()),
            (None, _) => spelling.push_str("%25"),
        }
        rest = &rest[i + escape.map_or(1, str::len)..];
    }
    spelling.extend(utf8_percent_encode(rest, SOURCE_ENCODE));
    spelling
}

/// Returns whether rule paths hold the byte `b` as it is, not percent-encoded.
fn is_kept_in_rule_paths(b: u8) -> bool {
    percent_encode(&[b], RULE_PATH_ENCODE)
        .next()
        .is_some_and(|chunk| chunk.len() == 1)
}

/// Returns whether a source glob in rule path spelling can match a rule path: it starts
/// with `/` or with a wildcard, group or class, and holds no empty, `.` or `..` segment,
/// nor an escape of `/`. On Windows, nor of `\`, which separates segments there.
fn can_match(spelling: &str) -> bool {
    spelling.starts_with(['/', '*', '?', '{', '['])
        && !spelling.contains("//")
        && !spelling
            .split('/')
            .any(|segment| segment == "." || segment == "..")
        && !spelling.contains("%2F")
        && !(cfg!(windows) && spelling.contains("%5C"))
}

#[cfg(test)]
mod tests {
    use super::{can_match, rule_spelling, to_matcher, to_regex};

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
    fn glob_source_unreserved_escapes_are_decoded() {
        for (glob, spelling) in [
            ("/%61dmin/**", "/admin/**"),
            ("/%2d%2E%5f%7e%41%7A%30%39", "/-._~Az09"),
            ("/a%2Bb%3a%40%21%24%26%27%28%29%3B%3D", "/a+b:@!$&'();="),
            ("/%2f%5c%25%20%5B%7b%3F", "/%2F%5C%25%20%5B%7B%3F"),
            ("/a%2Ab%2cc", "/a[*]b[,]c"),
        ] {
            assert_eq!(rule_spelling(glob), spelling, "{glob}");
        }
        assert!(
            to_matcher("/C%2B%2B/a%2Ab%2Cc")
                .unwrap()
                .is_match("/C++/a*b,c")
        );
        assert!(!to_matcher("/a%2Ab").unwrap().is_match("/axb"));
        assert_eq!(captures("/{x%2Cy,z}", "/x,y"), Some(vec!["x,y".into()]));
        assert_eq!(
            captures("/%61dmin/{**}", "/admin/x/y"),
            Some(vec!["x/y".into()])
        );
    }

    #[test]
    fn glob_source_that_can_never_match_is_detected() {
        for source in [
            "admin/**",
            "%2Fadmin",
            "/a//b",
            "//a",
            "/a/./b",
            "/a/..",
            "/a/%2e%2E/b",
            "/a%2fb",
            "/a/%2F",
        ] {
            assert!(!can_match(&rule_spelling(source)), "{source}");
        }
        for source in [
            "/",
            "/admin/**",
            "/a/b/",
            "**/{*}.js",
            "*.js",
            "{/a,/b}",
            "[/]x",
            "/a.b/..c/.d",
            "/%61dmin",
            "/a%20b",
        ] {
            assert!(can_match(&rule_spelling(source)), "{source}");
        }
        assert_eq!(can_match(&rule_spelling("/a%5Cb")), cfg!(not(windows)));
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
    fn glob_matcher_takes_rule_path_spelling() {
        let matcher = to_matcher("/café/*.js").unwrap();
        assert!(matcher.is_match("/caf%C3%A9/app.js"));
        assert!(matcher.is_match("/caf%C3%A9/a/b.js"), "`*` crosses `/`");
        assert!(!matcher.is_match("/café/app.js"));
        assert!(to_matcher("/%61dmin/**").unwrap().is_match("/admin/x"));
    }

    #[test]
    fn glob_matcher_letter_case_follows_the_platform() {
        let ignores_case = cfg!(any(target_os = "macos", windows));
        assert_eq!(
            to_matcher("/Old/*").unwrap().is_match("/old/x"),
            ignores_case
        );
        assert!(to_matcher("/old/*").unwrap().is_match("/old/x"));
    }

    #[test]
    fn glob_matcher_invalid_pattern_is_an_error() {
        let err = to_matcher("/{unclosed").unwrap_err();
        assert!(err.to_string().contains("/{unclosed"), "{err}");
    }

    #[test]
    fn glob_invalid_pattern_is_an_error() {
        let err = to_regex("/{unclosed").unwrap_err();
        assert!(err.to_string().contains("/{unclosed"), "{err}");
    }
}
