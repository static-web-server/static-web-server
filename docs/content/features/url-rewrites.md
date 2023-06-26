# URL Rewrites 

**SWS** provides the ability to rewrite request URLs with pattern-matching support.

URI rewrites are particularly useful with pattern matching ([globs](https://en.wikipedia.org/wiki/Glob_(programming))), as the server can accept any URL that matches the pattern and let the client-side code decide what to display.

## Structure

The URL rewrite rules should be defined mainly as an [Array of Tables](https://toml.io/en/v1.0.0#array-of-tables).

Each table entry should have two key/value pairs:

- One `source` key containing a string _glob pattern_.
- One `destination` string containing the local file path.
- Optional `redirect` number containing the HTTP response code.

!!! info "Note"
    The incoming request(s) will reach the `destination` only if the request(s) URI matches the `source` pattern.

### Source

The source is a [Glob pattern](https://en.wikipedia.org/wiki/Glob_(programming)) that should match against the URI that is requesting a resource file.

### Destination

A local file path must exist. It has to look something like `/some/directory/file.html`. It is worth noting that the `/` at the beginning indicates the server's root directory.

### Redirect

An optional number that indicates the HTTP response code (redirect).
The values can be:

- `301` for "Moved Permanently"
- `302` for "Found" (Temporary Redirect)

## Examples

```toml
[advanced]

### URL Rewrites

[[advanced.rewrites]]
source = "**/*.{png,ico,gif}"
destination = "/assets/generic1.png"

[[advanced.rewrites]]
source = "**/*.{jpg,jpeg}"
destination = "/images/generic2.png"
redirect = 302
```
