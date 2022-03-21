# Security Headers

**`SWS`** provides several [security headers](https://web.dev/security-headers/) support.

When the [HTTP/2](../features/http2-tls.md) feature is activated *security headers* are enabled automatically.

This feature is disabled by default on HTTP/1 and can be controlled by the boolean `--security-headers` option or the equivalent [SERVER_SECURITY_HEADERS](./../configuration/environment-variables.md#server_security_headers) env.

## Headers included

The following headers are only included.

- `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload" (2 years max-age)`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `X-Content-Type-Options: nosniff`
- `Content-Security-Policy: frame-ancestors`
