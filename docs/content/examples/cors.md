# CORS

**`SWS`** provides optional [Cross-Origin Resource Sharing (CORS)](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) support.

A list of allowed origin hosts (URLs) should be specified and separated by comas.
Or an asterisk (*) can be used in order to allow any host.

This feature is disabled by default and can be controlled by the string `-c, --cors-allow-origins` option or the equivalent [SERVER_CORS_ALLOW_ORIGINS](./../configuration/environment-variables.md#server_cors_allow_origins) env.

Below an example of how to enable CORS.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --cors-allow-origins "https://domain.com"

    # Or use an asterisk to allow any host
    # --cors-allow-origins "*"
```
