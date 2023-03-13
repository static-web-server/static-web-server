# Command-Line Arguments

The server can be configured via the following command-line arguments.

!!! tip "Remember"
    - Command-line arguments are equivalent to their [environment variables](./environment-variables.md).
    - Command-line arguments take precedence over their equivalent environment variables.


```
$ static-web-server -h

static-web-server 2.15.0
Jose Quintana <https://joseluisq.net>
A cross-platform, blazing fast and asynchronous web server for static files-serving.

USAGE:
    static-web-server [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --basic-auth <basic-auth>
            It provides The "Basic" HTTP Authentication scheme using credentials as "user-id:password" pairs. Password
            must be encoded using the "BCrypt" password-hashing function [env: SERVER_BASIC_AUTH=]  [default: ]
    -e, --cache-control-headers <cache-control-headers>
            Enable cache control headers for incoming requests based on a set of file types. The file type list can be
            found on `src/control_headers.rs` file [env: SERVER_CACHE_CONTROL_HEADERS=]  [default: true]
    -x, --compression <compression>
            Gzip, Deflate or Brotli compression on demand determined by the Accept-Encoding header and applied to text-
            based web file types only [env: SERVER_COMPRESSION=]  [default: true]
        --compression-static <compression-static>
            Look up the pre-compressed file variant (`.gz` or `.br`) on disk of a requested file and serves it directly
            if available. The compression type is determined by the `Accept-Encoding` header [env:
            SERVER_COMPRESSION_STATIC=]  [default: false]
    -w, --config-file <config-file>
            Server TOML configuration file path [env: SERVER_CONFIG_FILE=]

    -j, --cors-allow-headers <cors-allow-headers>
            Specify an optional CORS list of allowed headers separated by commas. Default "origin, content-type". It
            requires `--cors-allow-origins` to be used along with [env: SERVER_CORS_ALLOW_HEADERS=]  [default: origin,
            content-type]
    -c, --cors-allow-origins <cors-allow-origins>
            Specify an optional CORS list of allowed origin hosts separated by commas. Host ports or protocols aren't
            being checked. Use an asterisk (*) to allow any host [env: SERVER_CORS_ALLOW_ORIGINS=]  [default: ]
        --cors-expose-headers <cors-expose-headers>
            Specify an optional CORS list of exposed headers separated by commas. Default "origin, content-type". It
            requires `--cors-expose-origins` to be used along with [env: SERVER_CORS_EXPOSE_HEADERS=]  [default: origin,
            content-type]
    -z, --directory-listing <directory-listing>
            Enable directory listing for all requests ending with the slash character (‘/’) [env:
            SERVER_DIRECTORY_LISTING=]  [default: false]
        --directory-listing-format <directory-listing-format>
            Specify a content format for directory listing entries. Formats supported: "html" or "json". Default "html"
            [env: SERVER_DIRECTORY_LISTING_FORMAT=]  [default: html]  [possible values: Html, Json]
        --directory-listing-order <directory-listing-order>
            Specify a default code number to order directory listing entries per `Name`, `Last modified` or `Size`
            attributes (columns). Code numbers supported: 0 (Name asc), 1 (Name desc), 2 (Last modified asc), 3 (Last
            modified desc), 4 (Size asc), 5 (Size desc). Default 6 (unordered) [env: SERVER_DIRECTORY_LISTING_ORDER=]
            [default: 6]
    -f, --fd <fd>
            Instead of binding to a TCP port, accept incoming connections to an already-bound TCP socket listener on the
            specified file descriptor number (usually zero). Requires that the parent process (e.g. inetd, launchd, or
            systemd) binds an address and port on behalf of static-web-server, before arranging for the resulting file
            descriptor to be inherited by static-web-server. Cannot be used in conjunction with the port and host
            arguments. The included systemd unit file utilises this feature to increase security by allowing the static-
            web-server to be sandboxed more completely [env: SERVER_LISTEN_FD=]
    -q, --grace-period <grace-period>
            Defines a grace period in seconds after a `SIGTERM` signal is caught which will delay the server before to
            shut it down gracefully. The maximum value is 255 seconds [env: SERVER_GRACE_PERIOD=]  [default: 0]
    -a, --host <host>
            Host address (E.g 127.0.0.1 or ::1) [env: SERVER_HOST=]  [default: ::]

    -t, --http2 <http2>
            Enable HTTP/2 with TLS support [env: SERVER_HTTP2_TLS=]  [default: false]

        --http2-tls-cert <http2-tls-cert>
            Specify the file path to read the certificate [env: SERVER_HTTP2_TLS_CERT=]

        --http2-tls-key <http2-tls-key>
            Specify the file path to read the private key [env: SERVER_HTTP2_TLS_KEY=]

        --ignore-hidden-files <ignore-hidden-files>
            Ignore hidden files/directories (dotfiles), preventing them to be served and being included in auto HTML
            index pages (directory listing) [env: SERVER_IGNORE_HIDDEN_FILES=]  [default: false]
    -g, --log-level <log-level>
            Specify a logging level in lower case. Values: error, warn, info, debug or trace [env: SERVER_LOG_LEVEL=]
            [default: error]
        --log-remote-address <log-remote-address>
            Log incoming requests information along with its remote address if available using the `info` log level
            [env: SERVER_LOG_REMOTE_ADDRESS=]  [default: false]
    -b, --max-blocking-threads <max-blocking-threads>
            Maximum number of blocking threads [env: SERVER_MAX_BLOCKING_THREADS=]  [default: 512]

        --page-fallback <page-fallback>
            HTML file path that is used for GET requests when the requested path doesn't exist. The fallback page is
            served with a 200 status code, useful when using client routers. If the path is not specified or simply
            doesn't exist then this feature will not be active [env: SERVER_FALLBACK_PAGE=]
        --page404 <page404>
            HTML file path for 404 errors. If the path is not specified or simply doesn't exist then the server will use
            a generic HTML error message [env: SERVER_ERROR_PAGE_404=]  [default: ./public/404.html]
        --page50x <page50x>
            HTML file path for 50x errors. If the path is not specified or simply doesn't exist then the server will use
            a generic HTML error message [env: SERVER_ERROR_PAGE_50X=]  [default: ./public/50x.html]
    -p, --port <port>                                            Host port [env: SERVER_PORT=]  [default: 80]
        --redirect-trailing-slash <redirect-trailing-slash>
            Check for a trailing slash in the requested directory URI and redirect permanently (308) to the same path
            with a trailing slash suffix if it is missing [env: SERVER_REDIRECT_TRAILING_SLASH=]  [default: true]
    -d, --root <root>
            Root directory path of static files [env: SERVER_ROOT=]  [default: ./public]

        --security-headers <security-headers>
            Enable security headers by default when HTTP/2 feature is activated. Headers included: "Strict-Transport-
            Security: max-age=63072000; includeSubDomains; preload" (2 years max-age), "X-Frame-
            Options: DENY", "X-XSS-Protection: 1; mode=block" and "Content-Security-Policy: frame-ancestors
            'self'" [env: SERVER_SECURITY_HEADERS=]  [default: false]
    -n, --threads-multiplier <threads-multiplier>
            Number of worker threads multiplier that'll be multiplied by the number of system CPUs using the formula:
            `worker threads = number of CPUs * n` where `n` is the value that changes here. When multiplier value is 0
            or 1 then one thread per core is used. Number of worker threads result should be a number between 1 and
            32,768 though it is advised to keep this value on the smaller side [env: SERVER_THREADS_MULTIPLIER=]
            [default: 1]
```

## Windows

The following options and commands are Windows platform specific.

```
 -s, --windows-service <windows-service>
            Run the web server as a Windows Service [env: SERVER_WINDOWS_SERVICE=]  [default: false]

SUBCOMMANDS:
    help         Prints this message or the help of the given subcommand(s)
    install      Install a Windows Service for the web server
    uninstall    Uninstall the current Windows Service
```
