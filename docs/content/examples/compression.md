# Compression

**`SWS`** provides [`Gzip`](https://datatracker.ietf.org/doc/html/rfc1952), [`Deflate`](https://datatracker.ietf.org/doc/html/rfc1951#section-Abstract) and [`Brotli`](https://www.ietf.org/rfc/rfc7932.txt) compression of HTTP responses.

The compression functionality is determined by the [`Accept-Encoding`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header and is only applied to text-based web file types only.

## MIME types compressed

Only this list of common text-based MIME type files will be compressed either with `Gzip`, `Deflate` or `Brotli` via the `Accept-Encoding` header value.

```txt
text/html
text/css
text/javascript
text/xml
text/plain
text/csv
text/calendar
text/markdown
text/x-yaml
text/x-toml
text/x-component
application/rtf
application/xhtml+xml
application/javascript
application/x-javascript
application/json
application/xml
application/rss+xml
application/atom+xml
application/vnd.ms-fontobject
font/truetype
font/opentype
image/svg+xml
```

This feature is enabled by default and can be controlled by the boolean `-x, --compression` option or the equivalent [SERVER_COMPRESSION](./../configuration/environment-variables.md#server_compression) env.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --compression true
```
