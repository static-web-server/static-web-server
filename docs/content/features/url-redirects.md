# URL Redirects 

**SWS** provides the ability to redirect request URLs with pattern matching support.

URI redirects are particularly useful with pattern matching ([globs](https://en.wikipedia.org/wiki/Glob_(programming))). Use them for example to prevent broken links if you've moved a page or to shorten URLs.

## Structure

The URL redirect rules should be defined mainly as an [Array of Tables](https://toml.io/en/v1.0.0#array-of-tables).

Each table entry should have the following key/value pairs:

- One `source` key containing a string _glob pattern_.
- One `destination` string containing the local file path or a full URL.
- One `kind` number containing the HTTP response code.

!!! info "Note"
    The incoming request(s) will reach the `destination` only if the request(s) URI matches the `source` pattern.

### Source

The source is a [Glob pattern](https://en.wikipedia.org/wiki/Glob_(programming)) that should match against the URI that is requesting a resource file.

### Destination

A local file path must exist. It can be a local path `/some/directory/file.html` or a full URL. It is worth noting that the `/` at the beginning indicates the server's root directory.

### Kind

It indicates the HTTP response code.
The values can be:

- `301` for "Moved Permanently"
- `302` for "Found" (Temporary Redirect)

## Examples

```toml
[advanced]

### URL Redirects

[[advanced.redirects]]
source = "**/*.{jpg,jpeg}"
destination = "/images/generic1.png"
kind = 301

[[advanced.redirects]]
source = "/index.html"
destination = "https://sws.joseluisq.net"
kind = 302
```
