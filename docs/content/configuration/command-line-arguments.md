# Command-Line Arguments

The server can be configured via the following command-line arguments.

!!! tip "Remember"
    - Command-line arguments are equivalent to their [environment variables](./environment-variables.md).
    - Command-line arguments take precedence over their equivalent environment variables.


```
$ static-web-server -h

A cross-platform, high-performance and asynchronous web server for static files-serving.

Usage: static-web-server [OPTIONS] [COMMAND]

Commands:
  generate  Generate man pages and shell completions
  help      Print this message or the help of the given subcommand(s)

Options:
  -a, --host <HOST>
          Host address (E.g 127.0.0.1 or ::1) [env: SERVER_HOST=] [default: ::]
  -p, --port <PORT>
          Host port [env: SERVER_PORT=] [default: 80]
  -f, --fd <FD>
          Instead of binding to a TCP port, accept incoming connections to an already-bound TCP socket listener on the specified file descriptor number (usually zero). Requires that the parent process (e.g. inetd, launchd, or systemd) binds an address and port on behalf of static-web-server, before arranging for the resulting file descriptor to be inherited by static-web-server. Cannot be used in conjunction with the port and host arguments. The included systemd unit file utilises this feature to increase security by allowing the static-web-server to be sandboxed more completely [env: SERVER_LISTEN_FD=]
  -n, --threads-multiplier <THREADS_MULTIPLIER>
          Number of worker threads multiplier that'll be multiplied by the number of system CPUs using the formula: `worker threads = number of CPUs * n` where `n` is the value that changes here. When multiplier value is 0 or 1 then one thread per core is used. Number of worker threads result should be a number between 1 and 32,768 though it is advised to keep this value on the smaller side [env: SERVER_THREADS_MULTIPLIER=] [default: 1]
  -b, --max-blocking-threads <MAX_BLOCKING_THREADS>
          Maximum number of blocking threads [env: SERVER_MAX_BLOCKING_THREADS=] [default: 512]
  -d, --root <ROOT>
          Root directory path of static files [env: SERVER_ROOT=] [default: ./public]
      --page50x <PAGE50X>
          HTML file path for 50x errors. If the path is not specified or simply doesn't exist then the server will use a generic HTML error message. If a relative path is used then it will be resolved under the root directory [env: SERVER_ERROR_PAGE_50X=] [default: ./50x.html]
      --page404 <PAGE404>
          HTML file path for 404 errors. If the path is not specified or simply doesn't exist then the server will use a generic HTML error message. If a relative path is used then it will be resolved under the root directory [env: SERVER_ERROR_PAGE_404=] [default: ./404.html]
      --page-fallback <PAGE_FALLBACK>
          HTML file path that is used for GET requests when the requested path doesn't exist. The fallback page is served with a 200 status code, useful when using client routers. If the path is not specified or simply doesn't exist then this feature will not be active [env: SERVER_FALLBACK_PAGE=] [default: ]
  -g, --log-level <LOG_LEVEL>
          Specify a logging level in lower case. Values: error, warn, info, debug or trace [env: SERVER_LOG_LEVEL=] [default: error]
  -c, --cors-allow-origins <CORS_ALLOW_ORIGINS>
          Specify an optional CORS list of allowed origin hosts separated by commas. Host ports or protocols aren't being checked. Use an asterisk (*) to allow any host [env: SERVER_CORS_ALLOW_ORIGINS=] [default: ]
  -j, --cors-allow-headers <CORS_ALLOW_HEADERS>
          Specify an optional CORS list of allowed headers separated by commas. Default "origin, content-type". It requires `--cors-allow-origins` to be used along with [env: SERVER_CORS_ALLOW_HEADERS=] [default: "origin, content-type, authorization"]
      --cors-expose-headers <CORS_EXPOSE_HEADERS>
          Specify an optional CORS list of exposed headers separated by commas. Default "origin, content-type". It requires `--cors-expose-origins` to be used along with [env: SERVER_CORS_EXPOSE_HEADERS=] [default: "origin, content-type"]
  -t, --http2 [<HTTP2>]
          Enable HTTP/2 with TLS support [env: SERVER_HTTP2_TLS=] [default: false] [possible values: true, false]
      --http2-tls-cert <HTTP2_TLS_CERT>
          Specify the file path to read the certificate [env: SERVER_HTTP2_TLS_CERT=]
      --http2-tls-key <HTTP2_TLS_KEY>
          Specify the file path to read the private key [env: SERVER_HTTP2_TLS_KEY=]
      --https-redirect [<HTTPS_REDIRECT>]
          Redirect all requests with scheme "http" to "https" for the current server instance. It depends on "http2" to be enabled [env: SERVER_HTTPS_REDIRECT=] [default: false] [possible values: true, false]
      --https-redirect-host <HTTPS_REDIRECT_HOST>
          Canonical host name or IP of the HTTPS (HTTPS/2) server. It depends on "https_redirect" to be enabled [env: SERVER_HTTPS_REDIRECT_HOST=] [default: localhost]
      --https-redirect-from-port <HTTPS_REDIRECT_FROM_PORT>
          HTTP host port where the redirect server will listen for requests to redirect them to HTTPS. It depends on "https_redirect" to be enabled [env: SERVER_HTTPS_REDIRECT_FROM_PORT=] [default: 80]
      --https-redirect-from-hosts <HTTPS_REDIRECT_FROM_HOSTS>
          List of host names or IPs allowed to redirect from. HTTP requests must contain the HTTP 'Host' header and match against this list. It depends on "https_redirect" to be enabled [env: SERVER_HTTPS_REDIRECT_FROM_HOSTS=] [default: localhost]
      --index-files <INDEX_FILES>
          List of files that will be used as an index for requests ending with the slash character (‘/’). Files are checked in the specified order [env: SERVER_INDEX_FILES=] [default: index.html]
  -x, --compression [<COMPRESSION>]
          Gzip, Deflate, Brotli or Zstd compression on demand determined by the Accept-Encoding header and applied to text-based web file types only [env: SERVER_COMPRESSION=] [default: true] [possible values: true, false]
      --compression-level <COMPRESSION_LEVEL>
          Compression level to apply for Gzip, Deflate, Brotli or Zstd compression [env: SERVER_COMPRESSION_LEVEL=] [default: default] [possible values: fastest, best, default]
      --compression-static [<COMPRESSION_STATIC>]
          Look up the pre-compressed file variant (`.gz`, `.br` or `.zst`) on disk of a requested file and serves it directly if available. The compression type is determined by the `Accept-Encoding` header [env: SERVER_COMPRESSION_STATIC=] [default: false] [possible values: true, false]
  -z, --directory-listing [<DIRECTORY_LISTING>]
          Enable directory listing for all requests ending with the slash character (‘/’) [env: SERVER_DIRECTORY_LISTING=] [default: false] [possible values: true, false]
      --directory-listing-order <DIRECTORY_LISTING_ORDER>
          Specify a default code number to order directory listing entries per `Name`, `Last modified` or `Size` attributes (columns). Code numbers supported: 0 (Name asc), 1 (Name desc), 2 (Last modified asc), 3 (Last modified desc), 4 (Size asc), 5 (Size desc). Default 6 (unordered) [env: SERVER_DIRECTORY_LISTING_ORDER=] [default: 6]
      --directory-listing-format <DIRECTORY_LISTING_FORMAT>
          Specify a content format for directory listing entries. Formats supported: "html" or "json". Default "html" [env: SERVER_DIRECTORY_LISTING_FORMAT=] [default: html] [possible values: html, json]
      --security-headers [<SECURITY_HEADERS>]
          Enable security headers by default when HTTP/2 feature is activated. Headers included: "Strict-Transport-Security: max-age=63072000; includeSubDomains; preload" (2 years max-age), "X-Frame-Options: DENY" and "Content-Security-Policy: frame-ancestors 'self'" [env: SERVER_SECURITY_HEADERS=] [default: false] [possible values: true, false]
  -e, --cache-control-headers [<CACHE_CONTROL_HEADERS>]
          Enable cache control headers for incoming requests based on a set of file types. The file type list can be found on `src/control_headers.rs` file [env: SERVER_CACHE_CONTROL_HEADERS=] [default: true] [possible values: true, false]
      --basic-auth <BASIC_AUTH>
          It provides The "Basic" HTTP Authentication scheme using credentials as "user-id:password" pairs. Password must be encoded using the "BCrypt" password-hashing function [env: SERVER_BASIC_AUTH=] [default: ]
  -q, --grace-period <GRACE_PERIOD>
          Defines a grace period in seconds after a `SIGTERM` signal is caught which will delay the server before to shut it down gracefully. The maximum value is 255 seconds [env: SERVER_GRACE_PERIOD=] [default: 0]
  -w, --config-file <CONFIG_FILE>
          Server TOML configuration file path [env: SERVER_CONFIG_FILE=] [default: ./config.toml]
      --log-remote-address [<LOG_REMOTE_ADDRESS>]
          Log incoming requests information along with its remote address if available using the `info` log level [env: SERVER_LOG_REMOTE_ADDRESS=] [default: false] [possible values: true, false]
      --log-forwarded-for [<LOG_FORWARDED_FOR>]
          Log real IP from X-Forwarded-For header [env: SERVER_LOG_FORWARDED_FOR] [default: false] [possible values: true, false]
      --trusted-proxies <TRUSTED_PROXIES>
          A comma separated list of IP addresses to accept the X-Forwarded-For header from. Empty means trust all IPs [env: SERVER_TRUSTED_PROXIES] [default: ""]
      --redirect-trailing-slash [<REDIRECT_TRAILING_SLASH>]
          Check for a trailing slash in the requested directory URI and redirect permanently (308) to the same path with a trailing slash suffix if it is missing [env: SERVER_REDIRECT_TRAILING_SLASH=] [default: true] [possible values: true, false]
      --ignore-hidden-files [<IGNORE_HIDDEN_FILES>]
          Ignore hidden files/directories (dotfiles), preventing them to be served and being included in auto HTML index pages (directory listing) [env: SERVER_IGNORE_HIDDEN_FILES=] [default: false] [possible values: true, false]
      --disable-symlinks [<DISABLE_SYMLINKS>]
          Prevent following files or directories if any path name component is a symbolic link [env: SERVER_DISABLE_SYMLINKS=] [default: false] [possible values: true, false]
      --health [<HEALTH>]
          Add a /health endpoint that doesn't generate any log entry and returns a 200 status code. This is especially useful with Kubernetes liveness and readiness probes [env: SERVER_HEALTH=] [default: false] [possible values: true, false]
      --maintenance-mode [<MAINTENANCE_MODE>]
          Enable the server's maintenance mode functionality [env: SERVER_MAINTENANCE_MODE=] [default: false] [possible values: true, false]
      --maintenance-mode-status <MAINTENANCE_MODE_STATUS>
          Provide a custom HTTP status code when entering into maintenance mode. Default 503 [env: SERVER_MAINTENANCE_MODE_STATUS=] [default: 503]
      --maintenance-mode-file <MAINTENANCE_MODE_FILE>
          Provide a custom maintenance mode HTML file. If not provided then a generic message will be displayed [env: SERVER_MAINTENANCE_MODE_FILE=] [default: ]
  -V, --version
          Print version info and exit
  -h, --help
          Print help (see more with '--help')
```

## Windows

The following options and commands are Windows platform-specific.

```
 -s, --windows-service <windows-service>
            Run the web server as a Windows Service [env: SERVER_WINDOWS_SERVICE=]  [default: false]

SUBCOMMANDS:
    help         Prints this message or the help of the given subcommand(s)
    install      Install a Windows Service for the web server
    uninstall    Uninstall the current Windows Service
```
