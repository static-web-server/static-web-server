# CORS

**`SWS`** provides optional [Cross-Origin Resource Sharing (CORS)](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) support.

A list of allowed origin hosts (URLs) should be specified and separated by commas.
Or an asterisk (*) can be used to allow any host.

This feature is disabled by default and can be controlled by the string `-c, --cors-allow-origins` option or the equivalent [SERVER_CORS_ALLOW_ORIGINS](../configuration/environment-variables.md#server_cors_allow_origins) env.

Below is an example of how to enable CORS.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --cors-allow-origins "https://domain.com"

    # Or use an asterisk to allow any host
    # --cors-allow-origins "*"
```

## Allowed headers

The server also supports a list of [CORS allowed headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers) separated by commas.

This feature depends on `--cors-allow-origins` to be used along with this feature. It can be controlled by the string `-j, --cors-allow-headers` option or the equivalent [SERVER_CORS_ALLOW_HEADERS](../configuration/environment-variables.md#server_cors_allow_headers) env.

!!! info "Tips"
    - The default allowed headers value is `origin, content-type, authorization`.
    - The server also supports [preflight requests](https://developer.mozilla.org/en-US/docs/Glossary/Preflight_request) via the `OPTIONS` method. See [Preflighted requests in CORS](./http-methods.md#preflighted-requests-in-cors).

Below is an example of how to CORS.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --cors-allow-origins "https://domain.com"
    --cors-allow-headers "origin, content-type, x-requested-with"
```

## Exposed headers

The server also supports a list of [CORS-exposed headers to scripts](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers) separated by commas.

This feature depends on `--cors-allow-origins` to be used along with this feature. It can be controlled by the string `--cors-expose-headers` option or the equivalent [SERVER_CORS_EXPOSE_HEADERS](../configuration/environment-variables.md#server_cors_expose_headers) env.

!!! info "Tips"
    - The default exposed header's is `origin, content-type`.
    - The server also supports [preflight requests](https://developer.mozilla.org/en-US/docs/Glossary/Preflight_request) via the `OPTIONS` method. See [Preflighted requests in CORS](./http-methods.md#preflighted-requests-in-cors).

Below is an example of how to CORS.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --cors-allow-origins "https://domain.com"
    --cors-expose-headers "origin, content-type, x-requested-with"
```
