#![no_main]

use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use static_web_server::static_files;

#[derive(Debug, Arbitrary)]
struct RequestPath {
    base: Vec<u8>,
    uri: Vec<u8>,
}

fuzz_target!(|input: RequestPath| {
    let uri = unsafe { std::str::from_utf8_unchecked(&input.uri[..]) };
    let base = unsafe { std::str::from_utf8_unchecked(&input.base[..]) };
    let _ = static_files::sanitize_path(std::path::Path::new(base), uri);
});
