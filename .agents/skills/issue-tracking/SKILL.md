---
name: issue-tracking
description: Triage, reproduce, debug, and fix issues in the Static Web Server (SWS) project — bug reports, regressions, root-cause analysis, minimal fixes with regression tests, v2 backports, and security reports. Use when investigating a bug report or failing behavior, writing a fix, or reviewing a bug-fix PR.
---

# Issue Triage and Debugging

**When to load**: a bug report or regression arrives, behavior differs from the docs, a fix is being written, or a bug-fix PR is under review.

## Triage

1. **Security first**: if the report involves path traversal, file disclosure, auth bypass, header injection, or a crash from request input, treat it as a vulnerability. Follow `SECURITY.md`: private channels and a GitHub Security Advisory, no public issue or descriptive commit until disclosure. See `security/SKILL.md`
2. **Version and branch**: v3 is `master` (development); v2 is the LTS line. Note which versions are affected; v2 fixes are backported separately and titled `(v2 backport)`
3. **Gather**: SWS version (`static-web-server -V`), OS/arch, install method (binary, Docker image, cargo), full command line, env vars, `sws.toml`, request (`curl -v`), and logs at `-g trace`
4. **Check defaults**: many reports come from changed defaults. v3 defaults: port `8080`, root `.`, index `index.html`, `--compression-static` on, directory listing off. A TOML `[general]` value overrides the CLI flag, which surprises users

Bug reports use `.github/ISSUE_TEMPLATE/bug_report.yml`; ask for the missing fields rather than guessing.

## Reproduce

Reproduce before theorizing. Reduce to the smallest root and config that still fails.

```bash
# Debug build against a fixture root with full tracing
cargo run --features all -- -d tests/fixtures/public -p 8080 -g trace

# Or with a config file
cargo run --features all -- -w ./repro.toml -g trace
```

### HTTP probes

```bash
curl -sv http://localhost:8080/path -o /dev/null                       # status + headers
curl -sI -H 'Accept-Encoding: br, gzip' http://localhost:8080/app.js    # compression variant
curl -s -H 'Range: bytes=0-99' -D - http://localhost:8080/file -o /dev/null
curl -sI -H 'If-None-Match: W/"..."' http://localhost:8080/index.html  # expect 304
curl -sv -X OPTIONS -H 'Origin: https://example.com' \
     -H 'Access-Control-Request-Method: GET' http://localhost:8080/   # CORS preflight
curl -sI -H 'Accept: text/markdown' http://localhost:8080/article     # markdown negotiation
curl --path-as-is -sI 'http://localhost:8080/../../etc/passwd'         # traversal must 404
openssl s_client -connect localhost:8080 -servername localhost         # TLS
```

Use `--path-as-is`; plain `curl` normalizes `..` client-side and hides traversal bugs.

### Logs

- `-g` / `SERVER_LOG_LEVEL`: `error` (default), `warn`, `info`, `debug`, `trace`
- `info` logs effective settings at startup (each feature's `init()`); compare them with what the user believes they configured
- `trace` shows path resolution, pre-compressed variant selection, and compression decisions
- `--log-format pretty` gives human-readable output (default is single-line `json`); `--log-remote-address` adds client addresses

### Common Symptom → Cause

| Symptom | Check |
|---------|-------|
| 404 for a file that exists | Hidden component (`.well-known`) without `--include-hidden`; symlinked path; wrong root (default is `.`); `--index-files` value |
| 403 | A path component is a symlink and `--follow-symlinks` is off |
| Setting ignored | Same key set in `sws.toml` `[general]` (overrides CLI/env); feature not compiled in; `./config.toml` picked up with a deprecation warning |
| Wrong `Content-Type` | Extension unknown to `mime_guess`; fix with an `[[advanced.headers]]` rule |
| Not compressed | Body under 200 bytes, MIME not compressible, `HEAD` request, or a pre-compressed variant already applied |
| Asset cached for a year after a 404 | Fixed in v3.0.0-beta.2: file-type `Cache-Control` applies only to 2xx/304 |
| Header missing | Step order: custom headers override, CORS headers only when an origin matched, security headers only with `--security-headers` (auto with `--tls`) |
| Rewrite `$1` empty | Capture numbering follows pattern order; `{*}` stays within one segment, `{**}` spans segments |

## Fix

1. **Write the failing test first**: handler test in `tests/<feature>.rs` or unit test beside the code (`testing/SKILL.md`). Confirm it fails for the reported reason
2. **Find the root cause**: the exact line or condition, not the symptom. Explain it in the commit body
3. **Minimal fix**: change only what the root cause requires. No unrelated refactors
4. **Search for siblings**: `rg` for the same pattern elsewhere (another pipeline step, the v2 branch, `directory_listing` vs `static_files`)
5. **Run the full suite** for both feature sets (`rust-backend/SKILL.md`). Any new failure is yours to explain
6. **Record it**: CHANGELOG entry under **Bug Fixes**, and a docs-repo follow-up if documented behavior changes

### Commit

Per `docs/COMMITS.md` (lines ≤ 100 chars, imperative, lowercase subject, no trailing period):

```
fix(control_headers): apply file type cache-control only to 2xx and 304

Error responses for asset URLs inherited the one-year max-age of the
requested extension, so browsers cached a missing asset as a 404.
Use `no-cache` for any status other than 2xx and 304.

Fixes #757
```

Scope is the module touched (`server`, `tls`, `compression`, `fs`, `handler`, `static_files`, `cors`, `rewrites`, `directory_listing`, ...).

## Regression Prevention

- The reproduction test stays as a regression test
- Parsers that failed on unexpected input get a proptest; commit any `proptest-regressions/` file it produces
- Hot-path fixes get a bench so the fix doesn't regress performance unnoticed

## Checklist

- [ ] Security implications assessed first
- [ ] Reproduced, with a failing test
- [ ] Root cause identified and explained in the commit body
- [ ] Minimal fix; sibling code paths checked
- [ ] Tests pass for `--features all` and `--no-default-features`
- [ ] CHANGELOG entry; v2 backport noted if affected
