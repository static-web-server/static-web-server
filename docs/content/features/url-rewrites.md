# URL Rewrites 

**SWS** provides the ability to rewrite request URLs with pattern-matching support.

URI rewrites are particularly useful with pattern matching ([globs](https://en.wikipedia.org/wiki/Glob_(programming))), as the server can accept any URL that matches the pattern and let the client-side code decide what to display.

## Structure

URL rewrite rules should be defined mainly as an [Array of Tables](https://toml.io/en/v1.0.0#array-of-tables).

Each table entry should have two key/value pairs:

- `source`: a key containing a string _glob pattern_.
- `destination` a file path with optional replacements (placeholders).
- `redirect` an optional number containing the HTTP response code (redirection).

!!! info "Note"
    The incoming request(s) will reach the `destination` only if the request(s) URI matches the `source` pattern.

### Source

It's a [Glob pattern](https://en.wikipedia.org/wiki/Glob_(programming)) that should match against the URI that is requesting a resource file.

The glob pattern functionality is powered by the [globset](https://docs.rs/globset/latest/globset/) crate which supports Standard Unix-style glob syntax.

!!! tip "Glob pattern syntax"
    For more details about the Glob pattern syntax check out https://docs.rs/globset/latest/globset/#syntax

### Destination

The value can be either a local file path that maps to an existing file on the system or an external URL.
It could look like `/some/directory/file.html`. It is worth noting that the `/` at the beginning indicates the server's root directory.

#### Replacements

Additionally, a `destination` supports replacements for every Glob pattern group that matches against the `source`.

Replacements order start from `0` to `n` and are defined with a dollar sign followed by an index (Glob pattern group occurrence).

!!! tip "Group your Glob patterns"
    When using replacements, also group your Glob pattern by surrounding them with curly braces so every group should map to its corresponding replacement.<br>
    For example: `source = "**/{*}.{png,gif}"`

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

# a. Route rewrite example with redirection
[[advanced.rewrites]]
source = "**/*.{jpg,jpeg}"
destination = "/images/generic2.png"
## NOTE: `redirect` can be omitted too
redirect = 301

# b. Route rewrite example with destination replacements
[[advanced.rewrites]]
## Note that we're using curly braces to group the `*` wildcard.
## See https://docs.rs/globset/latest/globset/#syntax
source = "**/{*}.{png,gif}"
## For exmaple, the destination will result in `/assets/abcdef.png`
destination = "/assets/$1.$2"
```

If you request something like:

```sh
curl -I http://localhost/abcdef.png
```

Then the Server logs should look like this:

```log
2023-07-08T20:31:36.606035Z  INFO static_web_server::handler: incoming request: method=HEAD uri=/abcdef.png
2023-07-08T20:31:36.608439Z DEBUG static_web_server::handler: url rewrites glob patterns: ["$0", "$1", "$2"]
2023-07-08T20:31:36.608491Z DEBUG static_web_server::handler: url rewrites regex equivalent: (?-u:\b)(?:/?|.*/)(.*)\.(gif|png)$
2023-07-08T20:31:36.608525Z DEBUG static_web_server::handler: url rewrites glob pattern captures: ["abcdef.png", "abcdef", "png"]
2023-07-08T20:31:36.608561Z DEBUG static_web_server::handler: url rewrites glob pattern destination: "/assets/$1.$2"
2023-07-08T20:31:36.609655Z DEBUG static_web_server::handler: url rewrites glob patterns destination replaced: "/assets/abcdef.png"
2023-07-08T20:31:36.609735Z TRACE static_web_server::static_files: dir: base="public", route="assets/abcdef.png"
...
```
