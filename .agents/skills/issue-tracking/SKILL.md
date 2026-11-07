---
name: issue-tracking
description: Triage, reproduce, debug, and fix issues in the Static Web Server (SWS) project — bug reports, regressions, root-cause analysis, minimal fixes with regression tests, v2 backports, and security reports. Use when investigating a bug report or failing behavior, writing a fix, or reviewing a bug-fix PR.
---

# Issue Triage and Debugging

**When to load**: a bug report or regression arrives, behavior differs from the docs, a fix is being written, or a bug-fix PR is under review.

## Triage

1. **Security first**: path traversal, file disclosure, auth bypass, header injection, cache poisoning, resource exhaustion, or a crash from request input is a vulnerability. Follow `SECURITY.md` and `security-advisory/SKILL.md`: no public issue or descriptive commit before disclosure
2. **Version and branch**: v3 is `master`; v2 is the LTS line (`2.x`). v2 fixes are separate PRs titled `(v2 backport)`
3. **Gather**: `static-web-server -V`, OS/arch, install method, full command line, env vars, `sws.toml`, request (`curl -v`), logs at `-g trace`. Bug reports use `.github/ISSUE_TEMPLATE/bug_report.yml`; ask for missing fields rather than guessing
4. **Check defaults**: port `8080`, root `.`, index `index.html`, `--compression-static` on, listing off, security headers on with TLS builds. A TOML `[general]` value overrides the CLI flag, which surprises users

## Reproduce

Reproduce before theorizing; reduce to the smallest root and config that still fails.

```bash
cargo run --features all -- -d tests/fixtures/public -p 8080 -g trace
cargo run --features all -- -w ./repro.toml -g trace --log-format pretty
```

### HTTP probes

```bash
curl -sv http://localhost:8080/path -o /dev/null                          # status + headers
curl -sI -H 'Accept-Encoding: br, gzip' http://localhost:8080/app.js       # pre-compressed variant
curl -s -H 'Range: bytes=0-99' -D - http://localhost:8080/file -o /dev/null
curl -sI -H 'If-None-Match: W/"..."' http://localhost:8080/index.html     # expect 304
curl -sv -X OPTIONS http://localhost:8080/secret.txt                       # with --basic-auth: expect 401
curl -sv -X OPTIONS -H 'Origin: https://example.com' \
     -H 'Access-Control-Request-Method: GET' http://localhost:8080/        # CORS preflight: 204, no auth
curl -sI -H 'Accept: text/markdown' http://localhost:8080/article          # markdown negotiation
curl -si http://localhost:8080/readyz                                      # --health: 200, or 503
curl --path-as-is -sI 'http://localhost:8080/../../etc/passwd'            # traversal must 404
openssl s_client -connect localhost:8080 -servername localhost
```

Use `--path-as-is`; plain `curl` normalizes `..` client-side and hides traversal bugs. Test both HTTP/1 and HTTP/2+TLS when the report involves headers or bodies.

### Logs

- `-g` / `SERVER_LOG_LEVEL`: `error` (default), `warn`, `info`, `debug`, `trace`. `--log-format pretty` for reading; `--log-file` to also write to a file
- `info` logs effective settings at startup (each `init()`); compare with what the user believes they configured
- `trace` shows path resolution, variant selection, compression decisions
- One access log line per request with the status, written when the body finishes; `--access-log-format '%{remote_addr} "%{method} %{uri}" %{status} %{bytes}'` customizes it. Health probes log at `debug` only

### Symptom → Cause

| Symptom | Check |
|---------|-------|
| 404 for a file that exists | Hidden component without `--include-hidden`; symlinked path; wrong root (default `.`); `--index-files` |
| 403 | A path component is a symlink and `--follow-symlinks` is off |
| 401 on `OPTIONS` | Expected with `--basic-auth`; only CORS preflights skip auth |
| `/health` 404 | Replaced by `/livez` and `/readyz` in v3 (`--health`) |
| Setting ignored | Same key in `sws.toml` `[general]`; feature not compiled in; `./config.toml` picked up with a warning |
| Wrong `Content-Type` | Extension unknown to `mime_guess`; use an `[[advanced.headers]]` rule |
| Not compressed | Body under 200 bytes, MIME not compressible, `OPTIONS`, 206/304/204, or a pre-compressed variant already applied |
| Fallback page not served | Request looks like an asset (`Sec-Fetch-Dest` not document/frame, or non-HTML extension without `Accept: text/html`); fixed in #787 to keep 404 for assets |
| HSTS missing | Sent only with `--tls`; add it at the proxy or with a header rule |
| Proxy IP not logged | `--trusted-proxies` empty (trusts nobody) or doesn't list the proxy; IPv4 entries match `::ffff:` forms |
| Header missing | Custom headers override; CORS headers only on a matched origin; `scope = "files"` rules skip early responses (redirects, 401, 405, health) |
| Rewrite `$1` empty | Capture numbering follows pattern order; `{*}` stays within one segment, `{**}` spans segments |
| Connection closed | `--idle-timeout` (30 s default) or `--max-header-size` (431) |

## Fix

1. **Failing test first**: handler test in `tests/<feature>.rs` or unit test beside the code (`testing/SKILL.md`). Confirm it fails for the reported reason
2. **Root cause**: the exact line or condition, not the symptom. Explain it in the commit body
3. **Minimal fix**: only what the root cause requires. No unrelated refactors
4. **Siblings**: `rg` for the same pattern elsewhere (another pipeline step, `directory_listing` vs `static_files`, the `2.x` branch)
5. **Full suite** for both feature sets (`rust-conventions/SKILL.md`). Any new failure is yours to explain
6. **Record it**: note the docs-repo follow-up in the PR if documented behavior changes. `CHANGELOG.md` is maintainer-only

### Commit (`docs/COMMITS.md`)

Lines ≤ 100 chars, imperative, lowercase subject, no trailing period, no AI trailers:

```
fix(control_headers): apply file type cache-control only to 2xx and 304

Error responses for asset URLs inherited the one-year max-age of the
requested extension, so browsers cached a missing asset as a 404.
Use `no-cache` for any status other than 2xx and 304.

Fixes #757
```

## Regression Prevention

- The reproduction test stays as a regression test
- Parsers that failed on unexpected input get a proptest; commit any `proptest-regressions/` file
- Hot-path fixes get a bench

## Checklist

- [ ] Security implications assessed first
- [ ] Reproduced, with a failing test
- [ ] Root cause identified and explained in the commit body
- [ ] Minimal fix; sibling code paths checked
- [ ] Tests pass for `--features all` and `--no-default-features`
- [ ] v2 backport noted if affected
