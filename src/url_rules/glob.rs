// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Translation of a rule's `source` glob into the regex whose capture groups feed the
//! `$N` placeholders of its `destination`.

use globset::GlobBuilder;
use regex_lite::Regex;

use crate::{Context, Result};

/// Compiles a `source` glob into an equivalent regex.
///
/// `*` and `?` never cross a `/`, and each `{a,b}` alternation or `*` inside a group
/// becomes a capture group numbered from `$1`, in source order.
pub(crate) fn to_regex(glob: &str) -> Result<Regex> {
    let glob = GlobBuilder::new(glob)
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
    tracing::debug!("url rule glob pattern: {}", glob.glob());
    tracing::debug!("url rule regex equivalent: {pattern}");

    Regex::new(&pattern)
        .with_context(|| format!("can not compile regex pattern equivalent for source: {pattern}"))
}

#[cfg(test)]
mod tests {
    use super::to_regex;

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
    fn glob_invalid_pattern_is_an_error() {
        let err = to_regex("/{unclosed").unwrap_err();
        assert!(err.to_string().contains("/{unclosed"), "{err}");
    }
}
