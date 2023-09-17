use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use hyper::{Body, Response, StatusCode};
use static_web_server::control_headers;

fn append_headers_benchmark(c: &mut Criterion) {
    let mut resp = Response::new(Body::empty());
    *resp.status_mut() = StatusCode::OK;
    let uri_path: &str = "assets/image.jpg";

    c.bench_with_input(
        BenchmarkId::new("uri_path_input", uri_path),
        &uri_path,
        |b, &s| b.iter(|| control_headers::append_headers(s, &mut resp)),
    );
}

criterion_group!(control_headers_bench, append_headers_benchmark);
criterion_main!(control_headers_bench);
