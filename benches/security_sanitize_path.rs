// Adversarial benches for `sanitize_path`: long traversal patterns,
// percent-encoded escapes, Windows-style separators, and embedded NUL
// bytes. These complement the simple sanity bench in
// `benches/static_files.rs` and serve as a regression detector for
// path-handling performance.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use static_web_server::static_files;

fn pathological_inputs() -> Vec<(&'static str, String)> {
    vec![
        ("dot_dot_x1024", "../".repeat(1024)),
        ("encoded_dots_x1024", "%2e%2e%2f".repeat(1024)),
        ("mixed_slashes_x512", "..\\..\\../".repeat(512)),
        ("nul_bytes_x256", "\0a/".repeat(256)),
        ("deep_segments_x512", "a/".repeat(512)),
        ("drive_prefix_x256", "C:\\foo/".repeat(256)),
    ]
}

fn sanitize_path_adversarial(c: &mut Criterion) {
    let base = std::path::Path::new("root/");
    let mut g = c.benchmark_group("sanitize_path_adversarial");
    for (label, tail) in pathological_inputs() {
        g.bench_with_input(BenchmarkId::from_parameter(label), &tail, |b, t| {
            b.iter(|| static_files::sanitize_path(base, t))
        });
    }
    g.finish();
}

criterion_group!(security_sanitize_path, sanitize_path_adversarial);
criterion_main!(security_sanitize_path);
