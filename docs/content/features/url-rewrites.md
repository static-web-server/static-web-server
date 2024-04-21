# URL Rewrites

**SWS** provides the ability to rewrite request URLs (routes) with Glob pattern-matching support.

URI rewrites are particularly useful with pattern matching ([globs](https://en.wikipedia.org/wiki/Glob_(programming))), as the server can accept any URL that matches the pattern and let the client-side code decide what to display.

## Structure

URL rewrite rules should be defined mainly as an [Array of Tables](https://toml.io/en/v1.0.0#array-of-tables).

Each table entry should have two key/value pairs:

- `source`: key containing a string _glob pattern_.
- `destination`: file path with optional replacements (placeholders).
- `redirect`: optional number containing the HTTP response code (redirection).

!!! info "Note"
    The incoming request(s) will reach the `destination` only if the request(s) URI matches the `source` pattern.

### Source

It's a [Glob pattern](https://en.wikipedia.org/wiki/Glob_(programming)) that should match against the URI that is requesting a resource file.

The glob pattern functionality is powered by the [globset](https://docs.rs/globset/latest/globset/) crate which supports Standard Unix-style glob syntax.

!!! tip "Glob pattern syntax"
    For more details about the Glob pattern syntax check out https://docs.rs/globset/latest/globset/#syntax

### Destination

The value should be a relative or absolute URL. A relative URL could look like `/some/directory/file.html`. An absolute URL can be `https://external.example.com/` for example.

#### Replacements

Additionally, a `destination` supports replacements for every Glob pattern group that matches against the `source`.

Replacements order start from `0` to `n` and are defined with a dollar sign followed by an index (Glob pattern group occurrence).

!!! tip "Group your Glob patterns"
    When using replacements, also group your Glob pattern by surrounding them with curly braces so every group should map to its corresponding replacement.<br>
    For example: `source = "**/{*}.{png,gif}"`

#### Destination processing

How destination is processed depends on whether the `redirect` key (see below) is present. If it is present, SWS will perform an *external* redirect. It will send a redirect response to the client, and the browser will usually proceed to the destination. In case of a relative URL, it will be another page on the same server. An absolute URL can result in navigation to another server.

Without a `redirect` key, SWS will perform an *internal* redirect. It will attempt to retrieve the file denoted by the destination and send it to the client. While it is possible to specify an absolute URL here as well, it will always be processed by the same SWS instance. It will result by the request being mapped to a different [virtual host](virtual-hosting.md) however if a matching virtual host is present.

#### Different roots within the same virtual host

Normally, different root directories are only possible with different virtual hosts. Rewrites however allow exposing another root in a subdirectory for example. For that, you add an internal virtual host that isn't normally visible from outside, e.g. `internal.local`. You then rewrite the requests to the subdirectory to the internal virtual host. For example:

```toml
[general]
root = "/usr/srv/www"

[advanced]

[[advanced.rewrites]]
source = "/test/{**}"
destination = "http://internal.local/test/$1"

[[advanced.virtual-hosts]]
host = "internal.local"
root = "/usr/srv/alternative-root"
```

A request to `/index.html` will be mapped to `/usr/srv/www/index.html`, yet `/test/hi.txt` will be mapped to the file `/usr/srv/alternative-root/test/hi.txt`.

This approach has two caveats:

1. When SWS produces redirects (e.g. redirecting `http://internal.local/test/subdir` to `http://internal.local/test/subdir/`), it isn't aware of rewrites. Unless the path part of the URL is identical before and after rewrite (like in the example above), this will result in broken redirects.
2. While the `internal.local` virtual host isn't normally accessed directly, this doesn't mean that it isn't possible for someone knowing (or guessing) its name. You should consider all files under the virtual host's root as public. Don't put any secrets in it even if these aren't accessible via rewrites.

### Redirect

An optional number that indicates the HTTP response code (redirect).
The values can be:

- `301` for "Moved Permanently"
- `302` for "Found" (Temporary Redirect)

## Examples

```toml
[advanced]

### URL Rewrites

# a. Simple route rewrite example
[[advanced.rewrites]]
source = "**/*.{png,ico,gif}"
destination = "/assets/generic1.png"

# b. Route rewrite example with redirection
[[advanced.rewrites]]
source = "**/*.{jpg,jpeg}"
destination = "/images/generic2.png"
## NOTE: `redirect` can be omitted too
redirect = 301

# c. Route rewrite example with destination replacements
[[advanced.rewrites]]
## Note that we're using curly braces to group the `*` wildcard.
## See https://docs.rs/globset/latest/globset/#syntax
source = "**/{*}.{png,gif}"
## For example, the destination will result in `/assets/abcdef.png`
destination = "/assets/$1.$2"
```

If you request something like:

```sh
curl -I http://localhost/abcdef.png
```

Then the server logs should look something like this:

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
