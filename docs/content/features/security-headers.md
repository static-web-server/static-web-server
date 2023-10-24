# Security Headers

**`SWS`** provides several [security headers](https://web.dev/security-headers/) support.

When the [HTTP/2](../features/http2-tls.md) feature is activated *security headers* are enabled automatically.

This feature is disabled by default on HTTP/1 and can be controlled by the boolean `--security-headers` option or the equivalent [SERVER_SECURITY_HEADERS](./../configuration/environment-variables.md#server_security_headers) env.

!!! tip "Customize HTTP headers"
    If you want to customize HTTP headers on demand then have a look at the [Custom HTTP Headers](custom-http-headers.md) section.

## Headers included

The following headers are included by default.

- `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload" (2 years max-age)`
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `Content-Security-Policy: frame-ancestors`
