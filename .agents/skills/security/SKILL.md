---
name: security
description: Review or implement security measures for the Static Web Server (SWS) project — path traversal prevention, TLS, security headers, CORS, and input validation
---

# Security Best Practices

Load this skill when implementing authentication, hardening the file-serving pipeline, configuring TLS, reviewing CORS policies, or auditing path handling.

**When to load**: touching `src/static_files/security.rs`, `src/security_headers.rs`, `src/cors.rs`, `src/basic_auth.rs`, `src/tls.rs`, or any code that handles user-supplied paths, headers, or credentials.

## General Principles

- **Least privilege**: Run SWS on a non-privileged port (8787 by default). Use systemd socket activation or a reverse proxy for port 80/443. Never run as root
- **Defense in depth**: Path traversal is prevented at multiple layers (see below). No single layer is sufficient
- **Fail closed**: If a security check errors, deny access. Traversal and hidden-file violations return 404 (not 403) to avoid leaking information about the filesystem layout. Symlink policy violations return 403
- **Don't roll your own crypto**: Use `tokio-rustls` (backed by `ring` or `aws-lc-rs` for FIPS) for TLS. Never implement ciphers or hashing

## Path Traversal Prevention

SWS's multi-layer defense against directory traversal:

### Layer 1: Path Sanitization

`sanitize_path()` in `src/fs/path.rs` processes each path component:
- Strips `..` (ParentDir), root prefixes, and Windows drive prefixes
- Normalizes `//` and `./` (CurDir)
- Percent-decodes the URI path before processing

### Layer 2: Containment Check

`resolve_and_contain()` and `enforce_containment()` in `src/static_files/security.rs`:
- Canonicalizes the resolved file path (resolves all symlinks to real paths)
- Verifies the canonical path starts with the canonical base directory
- Returns `StatusCode::NOT_FOUND` (404) if the path escapes the base — fail closed, no info leak

### Layer 3: Symlink Component Check

When `--follow-symlinks` is disabled (default), `enforce_symlink_policy()` in `src/static_files/security.rs` walks every path component and checks for symlinks using `symlink_metadata()`. Returns `StatusCode::FORBIDDEN` (403) if any component is a symlink.

### Layer 4: Hidden File Blocking

When `--include-hidden` is disabled (default), any path component starting with `.` is rejected with `StatusCode::NOT_FOUND` (404). This is a pure string check (zero syscalls) and runs before the more expensive symlink walk.

## TLS & HTTPS

- **Enable TLS in production**: Use `--tls --tls-cert cert.pem --tls-key key.pem`
- **TLS 1.2+ only**: Configured via `tokio-rustls`. Default cipher suites are secure
- **HTTP/2 requires TLS**: `--http2` depends on `--tls` being enabled
- **HTTPS redirect**: Use `--https-redirect` to redirect HTTP→HTTPS. Configure `--https-redirect-host` and `--https-redirect-from-port`
- **Security headers auto-enable with TLS**: When `--tls` is active, security headers default to `true`

## HTTP Security Headers

SWS sends these headers when `--security-headers` is enabled (default with TLS):

| Header | Value | Purpose |
|--------|-------|---------|
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains; preload` | Enforce HTTPS for 2 years |
| `X-Frame-Options` | `DENY` | Prevent clickjacking |
| `X-Content-Type-Options` | `nosniff` | Prevent MIME-type sniffing |
| `Content-Security-Policy` | `frame-ancestors 'self'` | Restrict embedding |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Control referrer information |

HSTS is only sent when TLS is active. Other headers are safe on plain HTTP.

## CORS

- **Restrictive by default**: CORS is disabled unless `--cors-allow-origins` is set
- **Avoid wildcard with credentials**: `Access-Control-Allow-Origin: *` is supported but incompatible with credentials
- **Explicit origin list preferred**: `--cors-allow-origins="https://example.com,https://app.example.com"`
- **Limit allowed methods**: SWS only allows GET, HEAD, OPTIONS. Other methods return 405
- **Custom allowed/exposed headers**: `--cors-allow-headers` and `--cors-expose-headers`

## Basic Authentication

- **Use `--basic-auth`**: Format is BCrypt-hashed password. Generate with `htpasswd -B` or SWS's built-in tooling
- **Credentials in every request**: HTTP Basic Auth sends credentials base64-encoded (not encrypted). Always use with TLS
- **No brute-force protection built in**: Put SWS behind a reverse proxy (nginx, Caddy) for rate limiting if needed

## Input Validation

- **HTTP method allowlist**: Only GET, HEAD, OPTIONS are permitted. Other methods → 405
- **Max URI length**: Hyper's default limits apply. Extremely long URIs are rejected by the HTTP parser
- **Request body is ignored**: SWS is a static file server. Request bodies are not read or processed
- **File path validation**: All user-supplied paths go through sanitization and canonicalization before filesystem access

## Dependency Security

- **Audit dependencies regularly**: Run `cargo audit` to check for known vulnerabilities
- **Minimal dependency footprint**: SWS has a carefully curated dependency tree. New dependencies must justify their inclusion
- **Pin critical deps**: `tokio`, `hyper`, `tokio-rustls`, `rustls` are the security-critical core

## Secrets Management

- **No secrets in source code**: TLS private keys, basic auth credentials, and config secrets live in files or environment variables
- **TLS private key file permissions**: Set `chmod 600` on private key files
- **`.env` files are gitignored**: Never commit `.env` files or configs with embedded secrets

## Checklist

- [ ] Are all user-supplied paths sanitized and contained?
- [ ] Is TLS enabled for production deployments?
- [ ] Are security headers enabled?
- [ ] Is CORS restricted to specific origins (not wildcard with credentials)?
- [ ] Are symlinks disabled if the served directory contains user-writable areas?
- [ ] Are hidden files ignored to prevent accidental exposure?
- [ ] Are dependencies audited (`cargo audit`)?
