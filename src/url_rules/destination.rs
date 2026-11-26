// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Building a rule's destination: `$N` placeholder substitution and query string
//! forwarding.

use aho_corasick::{AhoCorasick, MatchKind};
use regex_lite::Regex;

use crate::{Context, Error, Result};

/// Builds the automaton that finds the placeholders `$0` to `$N` of `regex`, where `N`
/// is its number of capture groups. The longest placeholder wins, so `$10` is not read
/// as `$1` followed by `0`.
pub fn build_placeholder_replacer(regex: &Regex) -> Result<AhoCorasick> {
    let patterns: Vec<String> = (0..regex.captures_len()).map(|i| format!("${i}")).collect();
    AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&patterns)
        .with_context(|| "can not build the placeholder replacer")
}

/// Checks that a destination template only holds characters allowed in a URI
/// (RFC 3986 §2), so every request gets a valid `Location` or request target.
pub(crate) fn validate(dest: &str) -> Result<()> {
    if dest.is_empty() {
        bail!("destination must not be empty");
    }
    let is_uri_char = |c: char| c.is_ascii_alphanumeric() || "-._~:/?#[]@!$&'()*+,;=%".contains(c);
    if let Some(c) = dest.chars().find(|c| !is_uri_char(*c)) {
        bail!("destination {dest:?} contains {c:?}, which a URI can only hold percent-encoded");
    }
    Ok(())
}

/// Replaces the `$N` placeholders of `dest_uri` with the capture groups of `regex` in
/// `orig_uri`. `$0` is the whole match; an unmatched optional group becomes empty.
///
/// A destination stays on its host: unless `dest_uri` itself starts with `//`, the `/`
/// and `\` that captures put at its start are collapsed into one `/`.
///
/// Fails if `regex` does not match `orig_uri`.
#[doc(hidden)]
pub fn replace_placeholders(
    orig_uri: &str,
    regex: &Regex,
    dest_uri: &str,
    ac: &AhoCorasick,
) -> Result<String, Error> {
    let Some(regex_caps) = regex.captures(orig_uri) else {
        bail!("regex didn't match, extracting captures failed");
    };

    let caps: Vec<&str> = (0..regex_caps.len())
        .map(|i| regex_caps.get(i).map_or("", |s| s.as_str()))
        .collect();

    tracing::debug!("url rule regex equivalent: {regex}");
    tracing::debug!("url rule glob pattern captures: {caps:?}");
    tracing::debug!("url rule glob pattern destination: {dest_uri:?}");

    let dest = ac
        .try_replace_all(dest_uri, &caps)
        .with_context(|| "failed replacing captures")?;
    let dest = keep_on_host(dest_uri, dest);
    tracing::debug!("url rule glob pattern destination replaced: {dest:?}");
    Ok(dest)
}

/// Collapses the `/` and `\` that `dest` starts with into one `/`, unless the `template`
/// it was built from starts with `//` itself.
///
/// RFC 3986 §4.2: `//host` is a network-path reference, and browsers read `\` as `/` in
/// http(s) URLs (WHATWG URL Standard), so a capture that adds a second one would make a
/// `Location` name another host.
fn keep_on_host(template: &str, dest: String) -> String {
    if template.starts_with("//") {
        return dest;
    }
    let path = dest.trim_start_matches(['/', '\\']);
    if dest.len() - path.len() < 2 {
        return dest;
    }
    ["/", path].concat()
}

/// Appends the request `query` to `dest`, after any query `dest` already has and before
/// its fragment, the way Apache's `QSA` flag and nginx's `rewrite` do.
///
/// An empty request query (`/path?`) is kept as such.
#[must_use]
pub(crate) fn append_query(dest: String, query: Option<&str>) -> String {
    let Some(query) = query else {
        return dest;
    };
    let (target, fragment) = dest.split_at(dest.find('#').unwrap_or(dest.len()));
    let separator = match target.find('?') {
        None => "?",
        Some(_) if query.is_empty() || target.ends_with(['?', '&']) => "",
        Some(_) => "&",
    };
    [target, separator, query, fragment].concat()
}

#[cfg(test)]
mod tests {
    use super::{append_query, build_placeholder_replacer, replace_placeholders, validate};
    use proptest::prelude::*;
    use regex_lite::Regex;

    fn replace(orig: &str, source: &str, dest: &str) -> String {
        let re = Regex::new(source).unwrap();
        let ac = build_placeholder_replacer(&re).unwrap();
        replace_placeholders(orig, &re, dest, &ac).unwrap()
    }

    #[test]
    fn placeholders_replaced_by_captures() {
        assert_eq!(
            replace("/a/b", r"^/(.*)/(.*)$", "/x/$2/$1"),
            "/x/b/a",
            "placeholders may appear in any order"
        );
        assert_eq!(replace("/a/b", r"^/(.*)/(.*)$", "/$1/$1"), "/a/a");
        assert_eq!(replace("/a/b", r"^/(.*)/(.*)$", "$0"), "/a/b");
    }

    #[test]
    fn placeholders_keep_destination_on_host() {
        for (orig, dest, expected) in [
            ("/blog/post", "/$1", "/post"),
            ("/blog//evil.example/x", "/$1", "/evil.example/x"),
            ("/blog/\\evil.example", "/$1", "/evil.example"),
            ("/blog/\\/\\evil.example", "/$1", "/evil.example"),
            ("/blog//evil.example", "$1", "/evil.example"),
            ("/blog/\\\\evil.example", "$1", "/evil.example"),
            ("/blog/", "/$1", "/"),
            ("/blog//x", "/new/$1", "/new//x"),
            ("/blog//x", "//cdn.example/$1", "//cdn.example//x"),
            ("/blog//x", "https://x.example/$1", "https://x.example//x"),
        ] {
            assert_eq!(
                replace(orig, r"^/blog/(.*)$", dest),
                expected,
                "{orig} -> {dest}"
            );
        }
    }

    #[test]
    fn placeholders_unmatched_group_is_empty() {
        assert_eq!(replace("/b", r"^/(a/)?(b)$", "/[$1][$2]"), "/[][b]");
    }

    #[test]
    fn placeholders_two_digits_not_read_as_one() {
        let source = format!("^/{}$", ["(.)"; 11].join("/"));
        let orig = "/a/b/c/d/e/f/g/h/i/j/k";
        assert_eq!(replace(orig, &source, "/$10/$11/$1"), "/j/k/a");
    }

    #[test]
    fn destination_rejects_characters_outside_uri() {
        for dest in [
            "", "/a b", "/a\nb", "/café", "/a<b>", "/a\"b", "/a{b}", "/a|b", "/a\\b", "/a^b",
            "/a`b",
        ] {
            assert!(validate(dest).is_err(), "{dest:?} must be rejected");
        }
        for dest in [
            "/",
            "/a/$1?b=$2&c=%20#top",
            "https://example.org:8080/[x]@!$&'()*+,;=~",
        ] {
            assert!(validate(dest).is_ok(), "{dest:?} must be accepted");
        }
    }

    #[test]
    fn placeholders_beyond_captures_are_left_alone() {
        assert_eq!(replace("/a", r"^/(a)$", "/$1/$2"), "/a/$2");
    }

    #[test]
    fn placeholders_without_match_is_an_error() {
        let re = Regex::new(r"^/a$").unwrap();
        let ac = build_placeholder_replacer(&re).unwrap();
        assert!(replace_placeholders("/b", &re, "/c", &ac).is_err());
    }

    #[test]
    fn query_none_leaves_destination_unchanged() {
        assert_eq!(append_query("/new".into(), None), "/new");
        assert_eq!(append_query("/new?a=1#top".into(), None), "/new?a=1#top");
    }

    #[test]
    fn query_appended_to_destination_without_query() {
        assert_eq!(append_query("/new".into(), Some("q=1")), "/new?q=1");
    }

    #[test]
    fn query_merged_with_destination_query() {
        assert_eq!(append_query("/new?a=1".into(), Some("q=1")), "/new?a=1&q=1");
        assert_eq!(append_query("/new?".into(), Some("q=1")), "/new?q=1");
        assert_eq!(
            append_query("/new?a=1&".into(), Some("q=1")),
            "/new?a=1&q=1"
        );
    }

    #[test]
    fn query_inserted_before_fragment() {
        assert_eq!(append_query("/new#top".into(), Some("q=1")), "/new?q=1#top");
        assert_eq!(
            append_query("/new?a=1#top".into(), Some("q=1")),
            "/new?a=1&q=1#top"
        );
    }

    #[test]
    fn query_empty_is_kept() {
        assert_eq!(append_query("/new".into(), Some("")), "/new?");
        assert_eq!(append_query("/new?a=1".into(), Some("")), "/new?a=1");
    }

    #[test]
    fn query_kept_verbatim() {
        assert_eq!(
            append_query("/new".into(), Some("a=%20b&a=c&d")),
            "/new?a=%20b&a=c&d"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128, ..ProptestConfig::default()
        })]

        /// `replace_placeholders` must never panic on arbitrary input pairs, whether or
        /// not the regex matches.
        #[test]
        fn prop_replace_placeholders_never_panics(
            orig in "\\PC{0,512}",
            dest in "\\PC{0,512}",
        ) {
            let re = Regex::new(r"^/(.*)/(.*)/(.*)/(.*)/(.*)$").unwrap();
            let ac = build_placeholder_replacer(&re).unwrap();
            let _ = replace_placeholders(&orig, &re, &dest, &ac);
        }

        /// When the regex does not match, `replace_placeholders` must return an error
        /// instead of falling through silently.
        #[test]
        fn prop_replace_placeholders_no_match_returns_err(
            tail in "[a-zA-Z0-9_.-]{0,64}",
            dest in "\\PC{0,64}",
        ) {
            let orig = format!("no-leading-slash-{tail}");
            let re = Regex::new(r"^/(.*)/(.*)/(.*)/(.*)/(.*)$").unwrap();
            let ac = build_placeholder_replacer(&re).unwrap();
            prop_assert!(replace_placeholders(&orig, &re, &dest, &ac).is_err());
        }

        /// On a match, every `$N` with `N` below the number of captures is substituted.
        #[test]
        fn prop_replace_placeholders_substitutes_all_indices(
            a in "[a-zA-Z0-9]{1,16}",
            b in "[a-zA-Z0-9]{1,16}",
            c in "[a-zA-Z0-9]{1,16}",
            d in "[a-zA-Z0-9]{1,16}",
            e in "[a-zA-Z0-9]{1,16}",
        ) {
            let orig = format!("/{a}/{b}/{c}/{d}/{e}");
            let re = Regex::new(r"^/(.*)/(.*)/(.*)/(.*)/(.*)$").unwrap();
            let ac = build_placeholder_replacer(&re).unwrap();
            let out = replace_placeholders(&orig, &re, "/p$0|$1|$2|$3|$4|$5", &ac).unwrap();
            prop_assert_eq!(out, format!("/p{orig}|{a}|{b}|{c}|{d}|{e}"));
        }

        /// Whatever the request path holds, a destination that does not start with `//`
        /// never becomes a reference to another host.
        #[test]
        fn prop_replace_placeholders_stays_on_host(
            tail in "[/\\\\a-z.]{0,32}",
            dest in prop_oneof![Just("/$1"), Just("$1"), Just("/$1/$1"), Just("$0")],
        ) {
            let re = Regex::new(r"^/blog/(.*)$").unwrap();
            let ac = build_placeholder_replacer(&re).unwrap();
            let out = replace_placeholders(&format!("/blog/{tail}"), &re, dest, &ac).unwrap();
            let slashes = out.len() - out.trim_start_matches(['/', '\\']).len();
            prop_assert!(slashes < 2, "{out}");
        }

        /// The request query always ends up intact in the destination, before any
        /// fragment, and the destination's own query is kept.
        #[test]
        fn prop_append_query_keeps_both_queries(
            path in "/[a-z/]{0,16}",
            dest_query in proptest::option::of("[a-z=&]{0,16}"),
            fragment in proptest::option::of("[a-z]{0,8}"),
            query in "[a-z0-9=&%]{0,32}",
        ) {
            let mut dest = path.clone();
            if let Some(q) = &dest_query {
                dest.push('?');
                dest.push_str(q);
            }
            if let Some(f) = &fragment {
                dest.push('#');
                dest.push_str(f);
            }
            let out = append_query(dest, Some(&query));
            let (target, out_fragment) = out.split_once('#').map_or((out.as_str(), None), |(t, f)| (t, Some(f)));
            prop_assert_eq!(out_fragment, fragment.as_deref());
            prop_assert!(target.starts_with(&path));
            prop_assert!(target.ends_with(&query));
            if let Some(q) = &dest_query {
                prop_assert!(target.contains(q.as_str()));
            }
        }
    }
}
