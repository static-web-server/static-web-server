# URL Redirects 

**SWS** provides the ability to redirect request URLs with Glob pattern-matching support.

URI redirects are particularly useful with pattern matching ([globs](https://en.wikipedia.org/wiki/Glob_(programming))). Use them for example to prevent broken links if you've moved a page or to shorten URLs.

## Structure

The URL redirect rules should be defined mainly as an [Array of Tables](https://toml.io/en/v1.0.0#array-of-tables).

Each table entry should have the following key/value pairs:

- `host`: optional key containing a string hostname to be matched against the incoming host URI.
- `source`: key containing a string _glob pattern_.
- `destination`: local file path or a full URL with optional replacements (placeholders).
- `kind`: optional number containing the HTTP response code (redirection).

!!! info "Note"
    The incoming request(s) will reach the `destination` only if the request(s) URI matches the `source` pattern.

### Host

Optional `host` redirect entry to be matched against the incoming host URI. If a `host` redirect setting is specified then SWS will attempt to match the value against the incoming URI host (request), applying the required redirect entry or ignoring it otherwise.

!!! tip "www to non-www redirects"
    The host entry allows for instance to perform www to non-www redirects or vice versa (see example below).

### Source

The source is a [Glob pattern](https://en.wikipedia.org/wiki/Glob_(programming)) that should match against the URI that is requesting a resource file.

The glob pattern functionality is powered by the [globset](https://docs.rs/globset/latest/globset/) crate which supports Standard Unix-style glob syntax.

!!! tip "Glob pattern syntax"
    For more details about the Glob pattern syntax check out https://docs.rs/globset/latest/globset/#syntax

### Destination

The value can be either a local file path that maps to an existing file on the system or an external URL.
It could look like `/some/directory/file.html`. It is worth noting that the `/` at the beginning indicates the server's root directory.

#### Replacements

Additionally, a `destination` supports replacements for every Glob pattern group that matches against the `source`.
The replacement order starts from `0` to `n` and is defined with a dollar sign followed by an index (Glob pattern group occurrence).

!!! tip "Group your Glob patterns"
    When using replacements, also group your Glob pattern by surrounding them with curly braces so every group should map to its corresponding replacement.<br>
    For example: `source = "**/{*}.{jpg,jpeg,svg}"`

### Kind

It is a number that indicates the HTTP response code (redirect).
The values can be:

- `301` for "Moved Permanently"
- `302` for "Found" (Temporary Redirect)

## Examples

```toml
[advanced]

### URL Redirects

# a. Simple route redirect example (existing file)
[[advanced.redirects]]
source = "**/*.{jpg,jpeg}"
destination = "/images/generic1.png"
kind = 301

# b. Simple route redirect example (external URL)
[[advanced.redirects]]
source = "/index.html"
destination = "https://static-web-server.net"
kind = 302

# c. Simple route redirect example with destination replacements
[[advanced.redirects]]
## Note that we're using curly braces to group the `*` wildcard.
## See https://docs.rs/globset/latest/globset/#syntax
source = "**/{*}.{jpg,jpeg,svg}"
## For example, the destination will result in `http://localhost/assets/abcdef.jpeg`
destination = "http://localhost/assets/$1.$2"
kind = 301

# d. Simple route redirect using the `host` option
# to perform www to non-www redirection.
[[advanced.redirects]]
host = "www.domain.com"
source = "/{*}"
destination = "https://domain.com/$1"
kind = 301
```

If you request something like:

```sh
curl -I http://localhost:4433/abcdef.jpeg
```

Then the server logs should look something like this:

```log
2023-07-11T21:11:22.217358Z  INFO static_web_server::handler: incoming request: method=HEAD uri=/abcdef.jpeg
2023-07-11T21:11:22.217974Z DEBUG static_web_server::handler: url redirects glob pattern: ["$0", "$1", "$2"]
2023-07-11T21:11:22.217992Z DEBUG static_web_server::handler: url redirects regex equivalent: (?-u:\b)(?:/?|.*/)(.*)\.(jpeg|jpg)$
2023-07-11T21:11:22.218002Z DEBUG static_web_server::handler: url redirects glob pattern captures: ["abcdef.jpeg", "abcdef", "jpeg"]
2023-07-11T21:11:22.218076Z DEBUG static_web_server::handler: url redirects glob pattern destination: "http://localhost/assets/$1.$2"
2023-07-11T21:11:22.218712Z DEBUG static_web_server::handler: url redirects glob pattern destination replaced: "http://localhost/assets/abcdef.jpeg"
2023-07-11T21:11:22.218739Z TRACE static_web_server::handler: uri matches redirects glob pattern, redirecting with status '301 Moved Permanently'
...
```
