use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use hyper::Method;
use static_web_server::http_ext::MethodExt;

fn is_allowed_benchmark(c: &mut Criterion) {
    let method = Method::default();
    c.bench_with_input(
        BenchmarkId::new("method_input", &method),
        &method,
        |b, _| b.iter(|| method.is_allowed()),
    );
}

criterion_group!(http_ext_bench, is_allowed_benchmark);
criterion_main!(http_ext_bench);
