# Trailing Slash Redirect

**`SWS`** provides automatic trailing slash redirect support for directory requests.

This feature is enabled by default and can be controlled by the boolean `--redirect-trailing-slash` option or the equivalent [SERVER_REDIRECT_TRAILING_SLASH](./../configuration/environment-variables.md#redirect_trailing_slash) env.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --redirect-trailing-slash true
```
