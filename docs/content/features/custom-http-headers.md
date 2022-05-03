# Custom HTTP Headers

**`SWS`** allows to customize the server [HTTP Response headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers) on demand.

## Structure

The Server HTTP response headers should be defined mainly as [Array of Tables](https://toml.io/en/v1.0.0#array-of-tables).

Each table entry should have two key/value pairs:

- One `source` key containing an string *glob pattern*.
- One `headers` key containing a [set or hash table](https://toml.io/en/v1.0.0#table) describing plain HTTP headers to apply.

A particular set of HTTP headers can only be applied when a `source` matches against the request uri.

!!! info "Custom HTTP headers take precedence over existing ones"
    Whatever custom HTTP header could **replace** an existing one if it was previously defined (E.g server default headers) and matches its `source`.

    The headers order is important since it determine its precedence.

    **Example:** if the feature `--cache-control-headers=true` is enabled but also a custom `cache-control` header was defined then the custom header will have priority.

### Source

Source is a [Glob pattern](https://en.wikipedia.org/wiki/Glob_(programming)) that should match against the uri that is requesting a resource file.

### Headers

A set of valid plain [HTTP headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers) to be applied.

## Examples

Below some examples of how to customize server HTTP headers in three variants.

### Oneline version

```toml
[advanced]

[[advanced.headers]]
source = "**/*.{js,css}"
headers = { Access-Control-Allow-Origin = "*", X-XSS-PROTECTION = "1; mode=block" }
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
