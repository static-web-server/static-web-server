// Benches for the per-request header work that runs on every response:
// security headers injection and `accept-encoding` negotiation. These are
// small, always-on code paths, so a regression here shows up on every
// single request served.

use criterion::{Criterion, criterion_group, criterion_main};
use http::{HeaderMap, HeaderValue, header::ACCEPT_ENCODING};
use hyper::{Response, StatusCode};
use static_web_server::{compression, security_headers};

fn security_headers_benchmark(c: &mut Criterion) {
    c.bench_function("security_headers/append_headers", |b| {
        b.iter(|| {
            let mut resp = Response::new(static_web_server::body::empty());
            *resp.status_mut() = StatusCode::OK;
            security_headers::append_headers(&mut resp);
            resp
        })
    });
}

fn accept_encoding_headers(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_str(value).unwrap());
    headers
}

fn compression_negotiation_benchmark(c: &mut Criterion) {
    // Header values as sent by current browsers plus a quality-weighted one.
    let cases = [
        ("browser", "gzip, deflate, br, zstd"),
        ("weighted", "br;q=1.0, gzip;q=0.8, *;q=0.1"),
        ("unsupported", "identity"),
    ];

    let mut g = c.benchmark_group("compression");
    for (label, value) in cases {
        let headers = accept_encoding_headers(value);
        g.bench_function(format!("get_preferred_encoding/{label}"), |b| {
            b.iter(|| compression::get_preferred_encoding(&headers))
        });
        g.bench_function(format!("get_encodings/{label}"), |b| {
            b.iter(|| compression::get_encodings(&headers))
        });
    }
    g.finish();
}

criterion_group!(
    response_headers_bench,
    security_headers_benchmark,
    compression_negotiation_benchmark
);
criterion_main!(response_headers_bench);
