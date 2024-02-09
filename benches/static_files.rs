use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use static_web_server::static_files;

#[derive(Debug)]
struct Inputs<'a> {
    base_path: &'a std::path::Path,
    uri_path: &'a str,
}
impl std::fmt::Display for Inputs<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}

fn sanitize_path_benchmark(c: &mut Criterion) {
    let base_path = std::path::Path::new("root/../");
    let uri_path: &str = "../assets/../../.../image.jpg";
    let inputs = Inputs {
        base_path,
        uri_path,
    };
    c.bench_with_input(BenchmarkId::new("path_inputs", &inputs), &inputs, |b, s| {
        b.iter(|| static_files::sanitize_path(s.base_path, s.uri_path))
    });
}

criterion_group!(static_files_bench, sanitize_path_benchmark);
criterion_main!(static_files_bench);
