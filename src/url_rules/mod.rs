// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! URL rules shared by `[[advanced.redirects]]` and `[[advanced.rewrites]]`: an optional
//! `host`, a `source` glob matched against the request path, and a `destination` whose
//! `$N` placeholders take the glob's capture groups.
//!
//! Rules see the request path in one spelling, [`rule_path`]: the segments that are
//! served, percent-encoded. Every path that reaches a file matches the rules for it.
//!
//! # Security: ReDoS / pattern complexity
//!
//! Source patterns are admin-supplied at startup and evaluated by [`regex_lite`], which
//! does not backtrack, so catastrophic-backtracking ReDoS does not apply. Per-request work
//! is still proportional to `pattern_size * uri_len`, so rule paths longer than
//! [`MAX_URI_LEN_FOR_REGEX`] bytes match no rule. Treat rules as trusted configuration.

mod destination;
mod glob;
mod host;

use aho_corasick::AhoCorasick;
use hyper::{Request, Response, StatusCode, Uri};
use regex_lite::Regex;

use crate::body::Body;
use crate::fs::path::rule_path;
use crate::{Error, Result, error_page, handler_opts::RequestHandlerOpts};

pub use destination::{build_placeholder_replacer, replace_placeholders};
pub(crate) use host::RequestHost;
pub use host::RuleHost;

/// Maximum rule path length, in bytes, matched against rule sources.
///
/// 8 KiB matches the common HTTP server URI cap, is far above any realistic rule
/// source, and bounds per-request regex work to a small constant.
pub const MAX_URI_LEN_FOR_REGEX: usize = 8 * 1024;

/// A compiled `[[advanced.redirects]]` or `[[advanced.rewrites]]` rule.
pub struct UrlRule {
    /// Host the request must be addressed to, if any.
    pub host: Option<RuleHost>,
    /// Regex compiled from the `source` glob.
    pub source: Regex,
    /// Destination template with `$N` placeholders.
    pub destination: String,
    /// Automaton that finds the placeholders of `destination`.
    pub replacer: AhoCorasick,
}

impl UrlRule {
    /// Compiles a rule from its `host`, `source` glob, and `destination` template.
    ///
    /// Fails on an invalid host or glob, or a destination with characters a URI can't
    /// hold unencoded.
    pub fn new(host: Option<&str>, source: &str, destination: &str) -> Result<Self> {
        let host = host.map(RuleHost::parse).transpose()?;
        destination::validate(destination)?;
        let source = glob::to_regex(source)?;
        let replacer = build_placeholder_replacer(&source)?;
        Ok(Self {
            host,
            source,
            destination: destination.to_owned(),
            replacer,
        })
    }

    /// Returns whether the rule applies to a request for `uri_path` addressed to `host`.
    #[must_use]
    fn is_match(&self, host: Option<&RequestHost<'_>>, uri_path: &str) -> bool {
        let host_matches = match (&self.host, host) {
            (None, _) => true,
            (Some(rule), Some(host)) => rule.matches(host),
            (Some(_), None) => false,
        };
        host_matches && self.source.is_match(uri_path)
    }

    /// Resolves the destination for `uri`: placeholders replaced by the captures of its
    /// [`rule_path`], then its query appended.
    pub fn resolve(&self, uri: &Uri) -> Result<String> {
        self.resolve_path(&rule_path(uri.path()), uri.query())
    }

    /// Resolves the destination for a request with the rule path `uri_path` and `query`.
    fn resolve_path(&self, uri_path: &str, query: Option<&str>) -> Result<String> {
        let dest = replace_placeholders(uri_path, &self.source, &self.destination, &self.replacer)?;
        Ok(destination::append_query(dest, query))
    }
}

/// Returns the first rule of `rules` that applies to `req`, with its destination for it.
pub(crate) fn find<'a, R: AsRef<UrlRule>, T>(
    rules: &'a [R],
    req: &Request<T>,
) -> Option<(&'a R, Result<String>)> {
    let host = RequestHost::from_request(req);
    let uri_path = rule_path(req.uri().path());
    let rule = find_for(rules, host.as_ref(), &uri_path)?;
    let dest = rule.as_ref().resolve_path(&uri_path, req.uri().query());
    Some((rule, dest))
}

/// Returns the first rule of `rules` that applies to a request addressed to `host` with
/// the rule path `uri_path`.
pub(crate) fn find_for<'a, R: AsRef<UrlRule>>(
    rules: &'a [R],
    host: Option<&RequestHost<'_>>,
    uri_path: &str,
) -> Option<&'a R> {
    if uri_path.len() > MAX_URI_LEN_FOR_REGEX {
        tracing::debug!(
            "url rules: skipping match, uri path length {} exceeds cap {}",
            uri_path.len(),
            MAX_URI_LEN_FOR_REGEX
        );
        return None;
    }
    rules
        .iter()
        .find(|rule| rule.as_ref().is_match(host, uri_path))
}

/// Logs `err` and returns the 500 error page response.
#[cold]
pub(crate) fn error_response<T>(
    err: Error,
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    tracing::error!("{err:?}");
    Some(error_page::error_response(
        req.uri(),
        req.method(),
        &StatusCode::INTERNAL_SERVER_ERROR,
        &opts.page404,
        &opts.page50x,
    ))
}

#[cfg(test)]
mod tests {
    use super::{MAX_URI_LEN_FOR_REGEX, UrlRule, find};
    use hyper::Request;

    impl AsRef<UrlRule> for UrlRule {
        fn as_ref(&self) -> &UrlRule {
            self
        }
    }

    fn rule(host: Option<&str>, source: &str) -> UrlRule {
        UrlRule::new(host, source, "/dest").unwrap()
    }

    fn request(uri: &str, host: Option<&str>) -> Request<()> {
        let mut builder = Request::builder().uri(uri);
        if let Some(host) = host {
            builder = builder.header("host", host);
        }
        builder.body(()).unwrap()
    }

    fn find_index(rules: &[UrlRule], uri: &str, host: Option<&str>) -> Option<usize> {
        let req = request(uri, host);
        let (found, _) = find(rules, &req)?;
        rules.iter().position(|r| std::ptr::eq(r, found))
    }

    #[test]
    fn find_first_matching_rule_wins() {
        let rules = [rule(None, "/a"), rule(None, "/{*}"), rule(None, "/a")];
        assert_eq!(find_index(&rules, "/a", None), Some(0));
        assert_eq!(find_index(&rules, "/b", None), Some(1));
        assert_eq!(find_index(&rules, "/a/b", None), None);
    }

    #[test]
    fn find_skips_rules_for_other_hosts() {
        let rules = [
            rule(Some("a.example"), "/{*}"),
            rule(Some("b.example"), "/{*}"),
            rule(None, "/{*}"),
        ];
        assert_eq!(find_index(&rules, "/x", Some("a.example")), Some(0));
        assert_eq!(find_index(&rules, "/x", Some("B.Example:8080")), Some(1));
        assert_eq!(find_index(&rules, "/x", Some("c.example")), Some(2));
        assert_eq!(find_index(&rules, "/x", None), Some(2));
    }

    #[test]
    fn find_skips_paths_longer_than_the_cap() {
        let rules = [rule(None, "/{**}")];
        let at_cap = ["/", &"a".repeat(MAX_URI_LEN_FOR_REGEX - 1)].concat();
        assert_eq!(find_index(&rules, &at_cap, None), Some(0));
        assert_eq!(find_index(&rules, &[&at_cap, "a"].concat(), None), None);
    }

    #[test]
    fn find_matches_every_spelling_of_a_path() {
        let rules = [rule(None, "/a/b"), rule(None, "/c d/**")];
        for uri in [
            "/a/b", "//a/b", "/a/./b", "/a/../b", "/a%2Fb", "/%61/%62", "/a/b?q",
        ] {
            assert_eq!(find_index(&rules, uri, None), Some(0), "{uri}");
        }
        for uri in ["/c%20d/x", "/./c%20d//x"] {
            assert_eq!(find_index(&rules, uri, None), Some(1), "{uri}");
        }
        for uri in ["/a/b/", "/a/b/.", "/a/b%2F", "/a/b%2Fc", "/a/bb"] {
            assert_eq!(find_index(&rules, uri, None), None, "{uri}");
        }
    }

    #[test]
    fn find_caps_the_served_spelling_of_a_path() {
        let rules = [rule(None, "/a/{**}")];
        let padded = ["/a/", &"./".repeat(MAX_URI_LEN_FOR_REGEX), "b"].concat();
        assert_eq!(find_index(&rules, &padded, None), Some(0));
    }

    #[test]
    fn find_host_rule_needs_source_match_too() {
        let rules = [rule(Some("a.example"), "/only")];
        assert_eq!(find_index(&rules, "/other", Some("a.example")), None);
    }

    #[test]
    fn destination_replaces_placeholders_and_appends_query() {
        let rule = UrlRule::new(None, "/{*}/{*}", "/new/$2/$1?a=1#top").unwrap();
        let uri = "/x/y?q=2".parse().unwrap();
        assert_eq!(rule.resolve(&uri).unwrap(), "/new/y/x?a=1&q=2#top");
    }

    #[test]
    fn destination_takes_captures_in_the_served_spelling() {
        let rule = UrlRule::new(None, "/old/{**}", "/new/$1").unwrap();
        let uri = "//old/a/./b%2Fc%20d?q=%2F".parse().unwrap();
        assert_eq!(rule.resolve(&uri).unwrap(), "/new/a/b/c%20d?q=%2F");
    }

    #[test]
    fn destination_reads_two_digit_placeholders() {
        let rule = UrlRule::new(None, &["/{*}"; 11].concat(), "/$11/$10/$1").unwrap();
        let uri = "/a/b/c/d/e/f/g/h/i/j/k".parse().unwrap();
        assert_eq!(rule.resolve(&uri).unwrap(), "/k/j/a");
    }

    #[test]
    fn new_rejects_invalid_host_and_source() {
        assert!(UrlRule::new(Some("http://a.example"), "/a", "/b").is_err());
        assert!(UrlRule::new(None, "/{a", "/b").is_err());
        assert!(UrlRule::new(None, "/a", "/b c").is_err());
    }

    /// `MAX_URI_LEN_FOR_REGEX` is a security/perf invariant; keep it as a tripwire.
    #[test]
    fn max_uri_len_for_regex_is_at_least_8kib() {
        const { assert!(MAX_URI_LEN_FOR_REGEX >= 8 * 1024) };
    }
}
