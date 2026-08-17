// Bench for `MethodExt::is_allowed`, the very first check of the request
// pipeline. A single call is only a couple of instructions, which is below
// the measurement resolution, so every iteration walks a mixed batch of
// methods (accepted and rejected) to get a stable, comparable signal.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use hyper::Method;
use static_web_server::exts::http::MethodExt;

fn methods_batch() -> Vec<Method> {
    let mixed = [
        Method::GET,
        Method::HEAD,
        Method::OPTIONS,
        Method::POST,
        Method::PUT,
        Method::DELETE,
    ];
    mixed.iter().cycle().take(64).cloned().collect()
}

fn is_allowed_benchmark(c: &mut Criterion) {
    let methods = methods_batch();
    c.bench_function("method_is_allowed", |b| {
        b.iter(|| {
            let mut allowed = 0_usize;
            for method in black_box(&methods) {
                if method.is_allowed() {
                    allowed += 1;
                }
            }
            black_box(allowed)
        })
    });
}

criterion_group!(http_ext_bench, is_allowed_benchmark);
criterion_main!(http_ext_bench);
