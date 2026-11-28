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
use crate::{Context, Result, error::bail};

/// Bytes [`rule_spelling`] percent-encodes in the text of a `source` glob between its
/// escapes and classes: the ones a rule path only holds encoded, except `/` and the glob
/// syntax `?`, `{` and `}`.
const SOURCE_ENCODE: &AsciiSet = &RULE_PATH_ENCODE
    .remove(b'/')
    .remove(b'?')
    .remove(b'{')
    .remove(b'}');

/// Compiles a `source` glob into an equivalent regex.
///
/// `*` and `?` never cross a `/`, and each `{a,b}` alternation or `*` inside a group
/// becomes a capture group numbered from `$1`, in source order.
///
/// The regex matches [`crate::fs::path::rule_path`] spellings. On macOS and Windows,
/// whose default file systems ignore letter case, it ignores ASCII case too, as paths in
/// any case reach the same file.
pub(crate) fn to_regex(glob: &str) -> Result<Regex> {
    let invalid = || format!("can not compile glob pattern for source: {glob}");
    let spelling = checked_spelling(glob, "url rule").with_context(invalid)?;
    let glob = GlobBuilder::new(&spelling)
        .literal_separator(true)
        .build()
        .with_context(invalid)?;

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
    let invalid = || format!("can not compile glob pattern for header source: {glob}");
    let spelling = checked_spelling(glob, "header").with_context(invalid)?;
    let matcher = GlobBuilder::new(&spelling)
        .case_insensitive(cfg!(any(target_os = "macos", windows)))
        .build()
        .with_context(invalid)?
        .compile_matcher();
    Ok(matcher)
}

/// Returns the rule path spelling of the `source` glob of a `kind` of rule, and warns
/// when it can match no request.
fn checked_spelling(glob: &str, kind: &str) -> Result<String> {
    let spelling = rule_spelling(glob)?;
    if !can_match(&spelling) {
        tracing::warn!(
            "{kind} source {glob:?} matches no request: request paths are matched with a \
             leading `/`, without empty, `.` or `..` segments, and with `/` unencoded"
        );
    }
    Ok(spelling)
}

/// Spells the text of a `source` glob the way rule paths are spelled: percent-escapes in
/// uppercase hex, and spaces, non-ASCII and other characters that need an escape encoded.
/// An escape of a character that rule paths hold as it is, such as `%61` or `%2B`, is
/// decoded (RFC 3986 §6.2.2.2), `*` and `,` as the literals `[*]` and `[,]`, since they
/// are glob syntax. A `%` that starts no escape becomes `%25`.
///
/// A character that rule paths hold percent-encoded is spelled that way too when the
/// glob quotes it, with a backslash (`\[`) or as the only member of a class (`[[]`).
/// Fails for a class that holds such a character in any other way, and for an unclosed
/// class.
fn rule_spelling(glob: &str) -> Result<String> {
    let mut spelling = String::with_capacity(glob.len());
    let mut rest = glob;
    while let Some(i) = rest.find(['%', '\\', '[']) {
        spelling.extend(utf8_percent_encode(&rest[..i], SOURCE_ENCODE));
        rest = &rest[i..];
        let len = if rest.starts_with('%') {
            push_percent_escape(&mut spelling, rest)
        } else if rest.starts_with('[') {
            push_class(&mut spelling, rest)?
        } else {
            push_backslash_escape(&mut spelling, rest)
        };
        rest = &rest[len..];
    }
    spelling.extend(utf8_percent_encode(rest, SOURCE_ENCODE));
    Ok(spelling)
}

/// Appends the spelling of the `%` that `rest` starts with, a percent-escape or a stray
/// `%`, and returns the length of the text it stands for.
fn push_percent_escape(spelling: &mut String, rest: &str) -> usize {
    let escape = rest
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
    escape.map_or(1, str::len)
}

/// Appends the spelling of the `\` that `rest` starts with and of the character it
/// escapes, and returns the length of the text they stand for.
///
/// Where `\` separates path segments it escapes nothing: `globset` reads it as `/`.
fn push_backslash_escape(spelling: &mut String, rest: &str) -> usize {
    let escaped = rest
        .chars()
        .nth(1)
        .filter(|_| !std::path::is_separator('\\'));
    match escaped {
        Some(c) if is_encoded_in_rule_paths(c) => push_encoded(spelling, c),
        Some(c) => spelling.extend(['\\', c]),
        None => spelling.push('\\'),
    }
    1 + escaped.map_or(0, char::len_utf8)
}

/// Appends the spelling of the class that `rest` starts with and returns the length of
/// the text it stands for.
///
/// A class matches one character of a rule path, where a percent-encoded character
/// takes three or more. Such a character is spelled encoded when it is the only member
/// (`[ ]` as `%20`). Among other members, in a range or negated it fails: the class
/// would match the characters of the encoding (`%`, `2` and `0`) and never a space.
/// So does a `%` among other members: a class reads no escape, `[%20]` holds no space.
fn push_class(spelling: &mut String, rest: &str) -> Result<usize> {
    let Some(class) = Class::parse(rest) else {
        bail!("unclosed character class; missing ']'");
    };
    let encoded = class.ranges.iter().find_map(|&(start, end)| {
        let mut members = [start, end].into_iter().chain(start..=end);
        members.find(|&c| is_encoded_in_rule_paths(c))
    });
    match encoded {
        None => spelling.push_str(class.text),
        Some(c) if !class.negated && class.ranges == [(c, c)] => push_encoded(spelling, c),
        Some('%') => bail!(
            "class {} holds '%', and an escape is not read inside a class: write the escape \
             (%25 for '%' itself) outside the class",
            class.text
        ),
        Some(c) => {
            let mut encoding = String::new();
            push_encoded(&mut encoding, c);
            bail!(
                "class {} matches a single character, but a request path spells {c:?} as \
                 {encoding}; use an alternation instead, such as {{a,{encoding}}}, which \
                 counts as a capture group in a redirect or rewrite",
                class.text
            );
        }
    }
    Ok(class.text.len())
}

/// A glob class such as `[!a-c]`, read the way `globset` reads it.
struct Class<'a> {
    /// The text of the class, brackets included.
    text: &'a str,
    /// Whether the class matches the characters it doesn't hold.
    negated: bool,
    /// The members: ranges of characters, a single one as a range of itself.
    ranges: Vec<(char, char)>,
}

impl<'a> Class<'a> {
    /// Parses the class that `rest` starts with, `None` when nothing closes it.
    ///
    /// A leading `!` or `^` negates the class, a `]` right after is a member, and so is
    /// a `-` that is first or last. Nothing escapes a character.
    fn parse(rest: &'a str) -> Option<Self> {
        let mut chars = rest.char_indices().skip(1).peekable();
        let negated = chars.next_if(|&(_, c)| c == '!' || c == '^').is_some();
        let mut ranges: Vec<(char, char)> = Vec::new();
        let mut in_range = false;
        for (i, c) in chars {
            let range = ranges.last_mut();
            match (c, range) {
                (']', Some(_)) => {
                    ranges.extend(in_range.then_some(('-', '-')));
                    let text = &rest[..=i];
                    return Some(Self {
                        text,
                        negated,
                        ranges,
                    });
                }
                ('-', Some(_)) if !in_range => in_range = true,
                (_, Some(range)) if in_range => {
                    range.1 = c;
                    in_range = false;
                }
                _ => ranges.push((c, c)),
            }
        }
        None
    }
}

/// Appends the character `c` the way rule paths hold it, percent-encoded.
fn push_encoded(spelling: &mut String, c: char) {
    spelling.extend(utf8_percent_encode(
        c.encode_utf8(&mut [0; 4]),
        RULE_PATH_ENCODE,
    ));
}

/// Returns whether rule paths hold the character `c` percent-encoded. `/` is not: it
/// separates their segments.
fn is_encoded_in_rule_paths(c: char) -> bool {
    c != '/' && !u8::try_from(c).is_ok_and(is_kept_in_rule_paths)
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
            ("/a[!b]?\\*^]", "/a[!b]?\\*%5E%5D"),
            ("/100%/%zz/%a", "/100%25/%25zz/%25a"),
            ("/ünï/%e2%82%ac", "/%C3%BCn%C3%AF/%E2%82%AC"),
            ("/%é", "/%25%C3%A9"),
        ] {
            assert_eq!(rule_spelling(glob).unwrap(), spelling, "{glob}");
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
            assert_eq!(rule_spelling(glob).unwrap(), spelling, "{glob}");
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
    fn glob_source_quoted_characters_take_rule_path_spelling() {
        for (glob, spelling) in [
            ("/a[[]b", "/a%5Bb"),
            ("/a[{]b[}]", "/a%7Bb%7D"),
            ("/a[ ]b", "/a%20b"),
            ("/caf[é]", "/caf%C3%A9"),
            ("/a[?]b[%]", "/a%3Fb%25"),
            ("/a[]]b", "/a%5Db"),
            ("/a[\\]]b", "/a%5C%5Db"),
            ("/a[.]b[*][,][-][/]", "/a[.]b[*][,][-][/]"),
            ("/[a-c][!a-c][^xyz][!-]", "/[a-c][!a-c][^xyz][!-]"),
            ("/[+-9][a-]", "/[+-9][a-]"),
            ("/{[[],x}", "/{%5B,x}"),
        ] {
            assert_eq!(rule_spelling(glob).unwrap(), spelling, "{glob}");
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn glob_source_backslash_escapes_take_rule_path_spelling() {
        for (glob, spelling) in [
            ("/a\\[b\\]", "/a%5Bb%5D"),
            ("/a\\{b\\}", "/a%7Bb%7D"),
            ("/a\\?b\\^\\\\", "/a%3Fb%5E%5C"),
            ("/a\\ b\\é", "/a%20b%C3%A9"),
            ("/100\\%41", "/100%2541"),
            ("/a\\*b\\,c\\/d", "/a\\*b\\,c\\/d"),
            ("/a\\", "/a\\"),
        ] {
            assert_eq!(rule_spelling(glob).unwrap(), spelling, "{glob}");
        }
        assert!(to_matcher("/a\\[b\\{c").unwrap().is_match("/a%5Bb%7Bc"));
        assert_eq!(captures("/a\\[{*}", "/a%5Bb"), Some(vec!["b".into()]));
        assert!(to_matcher("/a\\").is_err(), "nothing to escape");
    }

    #[cfg(windows)]
    #[test]
    fn glob_source_backslash_separates_segments() {
        assert_eq!(rule_spelling("/a\\b\\").unwrap(), "/a\\b\\");
        assert!(to_matcher("/a\\b").unwrap().is_match("/a/b"));
        assert!(to_matcher("/a\\[b").is_err(), "an unclosed class");
    }

    #[test]
    fn glob_source_quoted_characters_match_their_rule_path() {
        for (glob, path) in [
            ("/a[[]b", "/a%5Bb"),
            ("/a[ ]b", "/a%20b"),
            ("/caf[é]/*", "/caf%C3%A9/x"),
            ("/a[?]", "/a%3F"),
        ] {
            assert!(to_matcher(glob).unwrap().is_match(path), "{glob}");
            assert!(to_regex(glob).unwrap().is_match(path), "{glob}");
        }
        assert!(!to_matcher("/a[ ]b").unwrap().is_match("/a2b"));
        assert_eq!(captures("/a[ ]{*}", "/a%20b"), Some(vec!["b".into()]));
    }

    #[test]
    fn glob_source_class_of_encoded_characters_is_an_error() {
        for (glob, encoding) in [
            ("/[a b]x", "%20"),
            ("/caf[éa]", "%C3%A9"),
            ("/a[! ]b", "%20"),
            ("/a[^[]b", "%5B"),
            ("/a[ -~]b", "%20"),
            ("/a[A-z]b", "%5B"),
            ("/a[0-A]b", "%3C"),
            ("/a[à-ÿ]b", "%C3%A0"),
            ("/a[é-a]b", "%C3%A9"),
            ("/a[a- ]b", "%20"),
            ("/{a,[b ]}", "%20"),
        ] {
            for err in [to_matcher(glob).unwrap_err(), to_regex(glob).unwrap_err()] {
                let err = format!("{err:#}");
                assert!(err.contains(glob), "{err}");
                assert!(err.contains(&format!("as {encoding};")), "{err}");
            }
        }
    }

    #[test]
    fn glob_source_class_with_an_escape_is_an_error() {
        for glob in ["/a[%20]b", "/a[x%2Ay]b", "/a[!%41]", "/a[a%]"] {
            for err in [to_matcher(glob).unwrap_err(), to_regex(glob).unwrap_err()] {
                let err = format!("{err:#}");
                assert!(err.contains(glob), "{err}");
                assert!(
                    err.contains("an escape is not read inside a class"),
                    "{err}"
                );
                assert!(!err.contains("alternation"), "{err}");
            }
        }
        assert_eq!(rule_spelling("/a[%]b").unwrap(), "/a%25b");
    }

    #[test]
    fn glob_source_unclosed_class_is_an_error() {
        for glob in ["/a[bc", "/a[", "/a[]", "/a[!]", "/a[b%2Ac", "/{a,[b}"] {
            assert!(to_matcher(glob).is_err(), "{glob}");
            assert!(to_regex(glob).is_err(), "{glob}");
        }
        let err = format!("{:#}", to_matcher("/a[b%2Ac").unwrap_err());
        assert!(
            err.contains("/a[b%2Ac") && err.contains("unclosed"),
            "{err}"
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
            assert!(!can_match(&rule_spelling(source).unwrap()), "{source}");
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
            assert!(can_match(&rule_spelling(source).unwrap()), "{source}");
        }
        assert_eq!(
            can_match(&rule_spelling("/a%5Cb").unwrap()),
            cfg!(not(windows))
        );
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
