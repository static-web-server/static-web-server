# Custom HTTP Headers

**SWS** allows customizing the server [HTTP Response headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers) on demand.

## Structure

The Server HTTP response headers should be defined mainly as an [Array of Tables](https://toml.io/en/v1.0.0#array-of-tables).

Each table entry should have two key/value pairs:

- One `source` key containing a string _glob pattern_.
- One `headers` key containing a [set or hash table](https://toml.io/en/v1.0.0#table) describing plain HTTP headers to apply.

A particular set of HTTP headers can only be applied when a `source` matches against the request URI.

!!! info "Custom HTTP headers take precedence over existing ones"
    Whatever custom HTTP header could **replace** an existing one if it was previously defined (e.g. server default headers) and matches its `source`.

    The header's order is important because determines its precedence.

    **Example:** If the feature `--cache-control-headers=true` is enabled but also a custom `cache-control` header was defined then the custom header will have priority.

### Source

The source is a [Glob pattern](https://en.wikipedia.org/wiki/Glob_(programming)) that should match against the URI that is requesting a resource file.

### Headers

A set of valid plain [HTTP headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers) to be applied.

## Examples

Below are some examples of how to customize server HTTP headers in three variants.

### One-line version

```toml
[advanced]

[[advanced.headers]]
source = "**/*.{js,css}"
headers = { Access-Control-Allow-Origin = "*" }
```

### Multiline version

```toml
[advanced]

[[advanced.headers]]
source = "*.html"
[advanced.headers.headers]
Cache-Control = "public, max-age=36000"
Content-Security-Policy = "frame-ancestors 'self'"
Strict-Transport-Security = "max-age=63072000; includeSubDomains; preload"
```

### Multiline version with explicit header key (dotted)

```toml
[advanced]

[[advanced.headers]]
source = "**/*.{jpg,jpeg,png,ico,gif}"
headers.Strict-Transport-Security = "max-age=63072000; includeSubDomains; preload"
```
