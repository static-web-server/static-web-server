// Timing-stability bench for `basic_auth::check_request`. Compares two
// scenarios that historically had divergent timing:
//
// - `wrong_user`: provided username does not match -> early `unauthorized`
// - `right_user_wrong_password`: username matches, but bcrypt verify fails
//
// After the constant-time hardening, both paths should perform a bcrypt
// verify against either the real hash or a fixed sentinel hash, so the
// two timings should be within the same order of magnitude.

use criterion::{Criterion, criterion_group, criterion_main};
use headers::{Authorization, HeaderMapExt};
use http::HeaderMap;
use static_web_server::basic_auth::check_request;

/// Pre-computed bcrypt(`hunter2`, cost=4). Cost 4 to keep the bench
/// finite; the relative comparison between branches is what matters.
const BCRYPT_HASH: &str = "$2b$04$ABEjpXyqI8VrXkPYWUSlgejb7gP9G1QMd3KWG2pPuq0XSEU.SE9wK";

fn make_headers(user: &str, pass: &str) -> HeaderMap {
    let mut h = HeaderMap::new();
    h.typed_insert(Authorization::basic(user, pass));
    h
}

fn basic_auth_timing(c: &mut Criterion) {
    let credentials = format!("alice:{BCRYPT_HASH}");
    let userid = "alice";
    // We pass the bcrypt hash as the "password" arg per `check_request`
    // signature \u2014 see `basic_auth.rs`.
    let stored_hash = BCRYPT_HASH;

    let wrong_user_headers = make_headers("eve", "hunter2");
    let right_user_wrong_pw_headers = make_headers("alice", "not-hunter2");

    c.bench_function("basic_auth/wrong_user", |b| {
        b.iter(|| {
            let _ = check_request(&wrong_user_headers, userid, stored_hash);
        })
    });
    c.bench_function("basic_auth/right_user_wrong_password", |b| {
        b.iter(|| {
            let _ = check_request(&right_user_wrong_pw_headers, userid, stored_hash);
        })
    });

    drop(credentials);
}

criterion_group!(security_basic_auth, basic_auth_timing);
criterion_main!(security_basic_auth);
