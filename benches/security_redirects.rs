// Adversarial benches for `redirects::replace_placeholders` to catch
// regressions in regex/aho-corasick interaction. Uses regex_lite (no
// backtracking), so the curve should stay linear; this bench fails
// loudly if a future change reintroduces super-linear behaviour.

use aho_corasick::AhoCorasick;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use regex_lite::Regex;
use static_web_server::redirects::replace_placeholders;

fn replace_placeholders_pathological(c: &mut Criterion) {
    let re = Regex::new(r"^/(.*)/(.*)/(.*)/(.*)/(.*)$").unwrap();
    let ac = AhoCorasick::new(["$0", "$1", "$2", "$3", "$4", "$5"]).unwrap();
    let dest = "https://example.org/$0/$1/$2/$3/$4/$5";
    let cases = [
        ("len_64", "/a".repeat(32)),
        ("len_512", "/a".repeat(256)),
        ("len_4096", "/a".repeat(2048)),
    ];
    let mut g = c.benchmark_group("replace_placeholders");
    for (label, uri) in &cases {
        g.bench_with_input(BenchmarkId::from_parameter(label), uri, |b, u| {
            b.iter(|| replace_placeholders(u, &re, dest, &ac))
        });
    }
    g.finish();
}

criterion_group!(security_redirects, replace_placeholders_pathological);
criterion_main!(security_redirects);
