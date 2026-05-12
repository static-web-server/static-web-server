# Default UTF-8 Charset for Text-Based Responses

**SWS** automatically appends a `charset=utf-8` parameter to the `Content-Type` header for a predefined list of text-based MIME types.

## Included MIME Types

The `charset=utf-8` parameter will be added to responses with the following MIME types:

- `application/atom+xml`
- `application/rss+xml`
- `text/calendar`
- `text/csv`
- `text/html`
- `text/markdown`
- `text/plain`
- `text/xml`

For example, a `robots.txt` file will be served with the header: `Content-Type: text/plain; charset=utf-8`.

## Rationale

Without this default behavior, files like `robots.txt`, `llms.txt`, plain READMEs and other text-based resources are served without an explicit character encoding. When the charset is missing, browsers and HTTP clients fall back to outdated, locale-specific guesses (such as Windows-1252 or US-ASCII). This default guessing often results in mojibake—corrupting international symbols, accents, and non-ASCII characters.
Forcing UTF-8 ensures reliable, cross-platform text rendering.

The option is enabled by default and can be controlled by boolean `--text-charset` option or the equivalent [SERVER_TEXT_CHARSET](./../configuration/environment-variables.md#server_text_charset) env.

!!! info "Custom HTTP Headers Take Precedence"

    Users can turn off the default UTF-8 charset if wanted or use [Custom HTTP Headers](./custom-http-headers.md) with glob patterns to selectively apply charset to specific files.

## Usage examples

Default (no flag needed, already on):

```sh
static-web-server --root ./public
# robots.txt is served as: Content-Type: text/plain; charset=utf-8
```

Disable the feature entirely:

```sh
static-web-server --root ./public --text-charset=false
```

Equivalent configuration file:

```toml
[general]
root = "./public"
text-charset = false
```

Equivalent environment variable:

```sh
export SERVER_TEXT_CHARSET=false
static-web-server --root ./public
```
