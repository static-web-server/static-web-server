# Default Charset for Text Responses

**`SWS`** can declare a default `charset` parameter on every `text/*` response that doesn't already carry one, mirroring Apache's `AddDefaultCharset` and nginx's `charset` directives.

Without this option, files like `robots.txt`, `llms.txt`, plain READMEs or any other `text/*` resource are served without a charset parameter, and browsers fall back to a locale-specific guess (typically Windows-1252) that mojibake-corrupts any non-ASCII byte.

The option is enabled by default with `utf-8` and can be controlled by the string `--text-charset` option or the equivalent [SERVER_TEXT_CHARSET](./../configuration/environment-variables.md#server_text_charset) env. Pass an empty value to disable it.

## Behaviour

- Applies to every response whose `Content-Type` starts with `text/`.
- Skipped when the response already carries a `charset` parameter (e.g. markdown content negotiation, error pages).
- Non-text responses (`application/json`, `application/javascript`, images, archives) are never touched.
- Custom headers configured under `[[advanced.headers]]` still win over this option, because they are applied after.

## Usage examples

Default (no flag needed, already on):

```sh
static-web-server --root ./public
# robots.txt is served as: Content-Type: text/plain; charset=utf-8
```

Set a different charset:

```sh
static-web-server --root ./public --text-charset iso-8859-1
```

Disable the feature entirely:

```sh
static-web-server --root ./public --text-charset ""
```

Equivalent configuration file:

```toml
[general]
root = "./public"
text-charset = "utf-8"
```

Equivalent environment variable:

```sh
export SERVER_TEXT_CHARSET=utf-8
static-web-server --root ./public
```
