// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Parser of the W3C [`traceparent`](https://www.w3.org/TR/trace-context/#traceparent-header)
//! request header value.

/// Length of a version `00` `traceparent` value.
const TRACEPARENT_LEN: usize = 55;

/// Fields of a valid `traceparent` header value.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct TraceParent<'a> {
    pub(super) trace_id: &'a str,
    pub(super) parent_id: &'a str,
    pub(super) trace_flags: &'a str,
}

/// Parses a `traceparent` header value according to the W3C Trace Context
/// specification. Returns `None` for any invalid value.
///
/// A version higher than `00` is parsed as `00` when the value has at least
/// the `00` length and the extra fields start with a dash. The extra fields
/// are ignored, and so are the flags other than `sampled`, which this version
/// does not define.
pub(super) fn parse_traceparent(value: &str) -> Option<TraceParent<'_>> {
    let bytes = value.as_bytes();
    // The dashes are checked before any slicing, so every slice below starts
    // and ends at a char boundary.
    if bytes.len() < TRACEPARENT_LEN || bytes[2] != b'-' || bytes[35] != b'-' || bytes[52] != b'-' {
        return None;
    }

    let version = &bytes[..2];
    if !is_lower_hex(version) || version == b"ff" {
        return None;
    }
    let valid_len = if version == b"00" {
        bytes.len() == TRACEPARENT_LEN
    } else {
        bytes.len() == TRACEPARENT_LEN || bytes[TRACEPARENT_LEN] == b'-'
    };
    if !valid_len {
        return None;
    }

    let trace_id = &value[3..35];
    let parent_id = &value[36..52];
    let trace_flags = &value[53..55];
    if !is_non_zero_lower_hex(trace_id.as_bytes())
        || !is_non_zero_lower_hex(parent_id.as_bytes())
        || !is_lower_hex(trace_flags.as_bytes())
    {
        return None;
    }
    // The other flags of a higher version are unknown fields
    let trace_flags = match version {
        b"00" => trace_flags,
        _ if is_sampled(trace_flags) => "01",
        _ => "00",
    };

    Some(TraceParent {
        trace_id,
        parent_id,
        trace_flags,
    })
}

/// Whether the `sampled` flag, the least significant bit of `trace_flags`, is set.
#[inline]
fn is_sampled(trace_flags: &str) -> bool {
    u8::from_str_radix(trace_flags, 16).is_ok_and(|flags| flags & 1 == 1)
}

#[inline]
fn is_lower_hex(s: &[u8]) -> bool {
    s.iter().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

#[inline]
fn is_non_zero_lower_hex(s: &[u8]) -> bool {
    is_lower_hex(s) && s.iter().any(|&b| b != b'0')
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const VALID: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    #[test]
    fn traceparent_valid() {
        assert_eq!(
            parse_traceparent(VALID),
            Some(TraceParent {
                trace_id: "4bf92f3577b34da6a3ce929d0e0e4736",
                parent_id: "00f067aa0ba902b7",
                trace_flags: "01",
            })
        );
        let not_sampled = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00";
        assert_eq!(parse_traceparent(not_sampled).unwrap().trace_flags, "00");
    }

    #[test]
    fn traceparent_invalid_values_are_rejected() {
        let invalid = [
            // empty and truncated
            "",
            "00",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0",
            // version `00` with extra data
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-what",
            // forbidden or malformed version
            "ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "0g-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "0A-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            // all-zero trace-id and parent-id
            "00-00000000000000000000000000000000-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",
            // uppercase hex
            "00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00F067AA0BA902B7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0A",
            // non-hex characters
            "00-4bf92f3577b34da6a3ce929d0e0e473x-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902bz-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0z",
            // wrong field lengths with the same total length
            "00-4bf92f3577b34da6a3ce929d0e0e47366-0f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b701-",
            // wrong delimiters
            "00_4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736_00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7_01",
        ];
        for value in invalid {
            assert_eq!(parse_traceparent(value), None, "{value:?} must be rejected");
        }
    }

    #[test]
    fn traceparent_non_ascii_is_rejected() {
        // A two-byte char in place of two ASCII bytes keeps the length valid
        let invalid = [
            // The char spans the version and the first dash
            "0\u{e9}4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "\u{e9}-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e47\u{e9}-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-\u{e9}",
        ];
        for value in invalid {
            assert_eq!(parse_traceparent(value), None, "{value:?} must be rejected");
        }
    }

    #[test]
    fn traceparent_future_version() {
        let expected = Some(TraceParent {
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736",
            parent_id: "00f067aa0ba902b7",
            trace_flags: "01",
        });
        // Same layout as `00`
        let same = "cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        assert_eq!(parse_traceparent(same), expected);
        // Extra fields after a dash are ignored
        let extra = "cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-what-the-future";
        assert_eq!(parse_traceparent(extra), expected);
        // Extra data without a dash after the flags is invalid
        let no_dash = "cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01what";
        assert_eq!(parse_traceparent(no_dash), None);
    }

    /// A higher version keeps only the `sampled` flag, version `00` keeps the flags as received.
    #[test]
    fn traceparent_future_version_keeps_the_sampled_flag_only() {
        for (flags, expected) in [("ff", "01"), ("03", "01"), ("fe", "00"), ("0a", "00")] {
            let future = format!("cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-{flags}");
            assert_eq!(parse_traceparent(&future).unwrap().trace_flags, expected);
            // Version `00` flags are logged as received
            let current = format!("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-{flags}");
            assert_eq!(parse_traceparent(&current).unwrap().trace_flags, flags);
        }
    }

    /// A value close to the `traceparent` grammar: the forbidden version and all-zero ids are
    /// likely, and one char may be replaced, mostly next to a field boundary.
    fn edited_traceparent() -> impl Strategy<Value = String> {
        let version = prop_oneof![2 => "00", 1 => "ff", 2 => "[0-9a-f]{2}"];
        let trace_id = prop_oneof![1 => "0{32}", 6 => "[0-9a-f]{32}"];
        let parent_id = prop_oneof![1 => "0{16}", 6 => "[0-9a-f]{16}"];
        let extra = prop_oneof![3 => "", 1 => "-[a-z-]{0,4}", 1 => "[a-z]{1,4}"];
        let boundary = proptest::sample::select(vec![0, 1, 2, 3, 34, 35, 36, 51, 52, 53, 54, 55]);
        let position = prop_oneof![3 => boundary, 1 => 0usize..60];
        let replacement = proptest::sample::select(vec!['0', 'f', 'g', 'F', '-', ' ', '\u{e9}']);
        let edit = proptest::option::of((position, replacement));

        (version, trace_id, parent_id, "[0-9a-f]{2}", extra, edit).prop_map(
            |(version, trace_id, parent_id, trace_flags, extra, edit)| {
                let value = format!("{version}-{trace_id}-{parent_id}-{trace_flags}{extra}");
                let mut value: Vec<char> = value.chars().collect();
                if let Some((i, c)) = edit
                    && let Some(old) = value.get_mut(i)
                {
                    *old = c;
                }
                value.into_iter().collect()
            },
        )
    }

    /// Checked without `is_lower_hex`, so the properties do not rely on the parser's own check.
    fn is_lower_hex_of_len(s: &str, len: usize) -> bool {
        s.len() == len && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 1024, ..ProptestConfig::default()
        })]

        /// Parsing never panics, and an accepted value has the version `00` layout: the logged
        /// fields are lowercase hex of a fixed length, the ids are not all zeros, and a higher
        /// version keeps the `sampled` flag only.
        #[test]
        fn prop_parse_traceparent_accepts_only_the_grammar(
            value in prop_oneof![1 => any::<String>(), 4 => edited_traceparent()]
        ) {
            let Some(tp) = parse_traceparent(&value) else { return Ok(()) };

            prop_assert!(is_lower_hex_of_len(tp.trace_id, 32), "{value:?}");
            prop_assert!(is_lower_hex_of_len(tp.parent_id, 16), "{value:?}");
            prop_assert!(is_lower_hex_of_len(tp.trace_flags, 2), "{value:?}");
            prop_assert!(tp.trace_id.bytes().any(|b| b != b'0'), "{value:?}");
            prop_assert!(tp.parent_id.bytes().any(|b| b != b'0'), "{value:?}");

            let (version, fields) = value.split_at(2);
            prop_assert!(is_lower_hex_of_len(version, 2) && version != "ff", "{value:?}");
            let ids = format!("-{}-{}-", tp.trace_id, tp.parent_id);
            let rest = fields.strip_prefix(&ids).unwrap_or_default();
            let (flags, extra) = rest.split_at_checked(2).unwrap_or_default();
            prop_assert!(is_lower_hex_of_len(flags, 2), "{value:?}");
            if version == "00" {
                prop_assert_eq!(flags, tp.trace_flags);
                prop_assert_eq!(extra, "");
            } else {
                // An odd last hex digit, checked without `is_sampled`
                let sampled = if flags.ends_with(['1', '3', '5', '7', '9', 'b', 'd', 'f']) {
                    "01"
                } else {
                    "00"
                };
                prop_assert_eq!(sampled, tp.trace_flags);
                prop_assert!(extra.is_empty() || extra.starts_with('-'), "{value:?}");
            }
        }

        /// Any value of the version `00` grammar is accepted.
        #[test]
        fn prop_parse_traceparent_accepts_version_00(
            trace_id in "[0-9a-f]{31}[1-9a-f]",
            parent_id in "[0-9a-f]{15}[1-9a-f]",
            trace_flags in "[0-9a-f]{2}",
        ) {
            let value = format!("00-{trace_id}-{parent_id}-{trace_flags}");
            let expected = TraceParent {
                trace_id: &trace_id,
                parent_id: &parent_id,
                trace_flags: &trace_flags,
            };
            prop_assert_eq!(parse_traceparent(&value), Some(expected));
        }
    }
}
