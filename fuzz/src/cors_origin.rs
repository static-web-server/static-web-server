#![no_main]
#![forbid(unsafe_code)]

//! Fuzz the CORS origin validation that gates admin-supplied origin
//! strings before they reach `IntoOrigin::into_origin` (which is
//! documented to panic on malformed input).
//!
//! Invariant: `validate_origin_str` must NEVER panic, regardless of the
//! input, and a `true` result MUST imply that
//! `Origin::try_from_parts(scheme, rest, None)` succeeds (so the panic
//! branch in `IntoOrigin` is unreachable in production).

use headers::Origin;
use libfuzzer_sys::fuzz_target;
use static_web_server::cors::validate_origin_str;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    if validate_origin_str("fuzz", s) {
        let mut parts = s.splitn(2, "://");
        let scheme = parts.next().unwrap();
        let rest = parts
            .next()
            .expect("validator accepted input without `://`");
        Origin::try_from_parts(scheme, rest, None)
            .expect("validator accepted input that headers::Origin rejects");
    }
});
