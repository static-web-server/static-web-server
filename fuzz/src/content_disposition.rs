#![no_main]
#![forbid(unsafe_code)]

//! Fuzz the `Content-Disposition` filename helpers added in the
//! directory-listing-download security hardening. We assert the security
//! invariants directly from the fuzzer:
//!
//! - the quoted-string variant never emits `"` or `\` (would break header
//!   framing) nor any control byte (would risk header smuggling);
//! - the RFC-5987 variant produces a pure attr-char / `%HH` string and
//!   therefore is always valid 7-bit ASCII.

use libfuzzer_sys::fuzz_target;
use static_web_server::directory_listing_download::{
    rfc5987_encode_filename, sanitize_filename_for_quoted_string,
};

fuzz_target!(|data: &[u8]| {
    let Ok(name) = std::str::from_utf8(data) else {
        return;
    };

    let quoted = sanitize_filename_for_quoted_string(name);
    assert!(!quoted.is_empty(), "sanitizer must never return empty");
    for ch in quoted.chars() {
        assert_ne!(ch, '"', "quote leaked into quoted-string filename");
        assert_ne!(ch, '\\', "backslash leaked into quoted-string filename");
        assert!(
            (ch as u32) >= 0x20 && ch != '\x7f',
            "control byte leaked into quoted-string filename: {:?}",
            ch
        );
        assert!(
            ch.is_ascii(),
            "non-ASCII byte leaked into quoted-string filename"
        );
    }

    let ext = rfc5987_encode_filename(name);
    assert!(ext.is_ascii(), "rfc5987 output must be pure ASCII");
    for b in ext.as_bytes() {
        let ok = b.is_ascii_alphanumeric()
            || matches!(
                *b,
                b'!' | b'#'
                    | b'$'
                    | b'&'
                    | b'+'
                    | b'-'
                    | b'.'
                    | b'^'
                    | b'_'
                    | b'`'
                    | b'|'
                    | b'~'
                    | b'%'
            )
            || b.is_ascii_hexdigit();
        assert!(ok, "unexpected byte {:#x} in rfc5987 output", b);
    }
});
