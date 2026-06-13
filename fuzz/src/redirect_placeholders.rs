#![no_main]
#![forbid(unsafe_code)]

//! Fuzz the redirect placeholder replacement engine. Inputs are split
//! into (orig_uri, destination_template) using a fixed source pattern
//! that has 5 capture groups. The fuzzer asserts that
//! `replace_placeholders` never panics and that the output, if any,
//! contains no leftover `$N` placeholders for indices we substituted.

use aho_corasick::AhoCorasick;
use libfuzzer_sys::fuzz_target;
use regex_lite::Regex;
use static_web_server::redirects::replace_placeholders;
use std::sync::OnceLock;

fn shared() -> &'static (Regex, AhoCorasick) {
    static S: OnceLock<(Regex, AhoCorasick)> = OnceLock::new();
    S.get_or_init(|| {
        // A regex with 5 capture groups, mirroring realistic redirect rules.
        let re = Regex::new(r"^/(.*)/(.*)/(.*)/(.*)/(.*)$").unwrap();
        let ac = AhoCorasick::new(["$0", "$1", "$2", "$3", "$4", "$5"]).unwrap();
        (re, ac)
    })
}

fuzz_target!(|data: &[u8]| {
    // Bound input length to keep regex_lite work proportional and avoid
    // wedging the fuzzer on absurdly long inputs (CPU-bounded only).
    if data.len() > 4096 {
        return;
    }
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    // Split half/half into orig URI and destination template. Walk to the
    // next char boundary so multi-byte sequences don't panic `split_at`.
    let mut mid = text.len() / 2;
    while mid < text.len() && !text.is_char_boundary(mid) {
        mid += 1;
    }
    let (orig, dest) = text.split_at(mid);
    let (re, ac) = shared();
    let _ = replace_placeholders(orig, re, dest, ac);
});
