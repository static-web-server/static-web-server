static-web-server(1)
====================

Name
----

static-web-server - a blazing fast and asynchronous web server for static files-serving.

SYNOPSIS
--------

*static-web-server* [OPTIONS]

*static-web-server* [_OPTIONS_] *--help*

*static-web-server* [_OPTIONS_] *--version*

FLAGS
-----

-h, --help::
        Prints help information
-V, --version::
        Prints version information

OPTIONS
-------

--basic-auth <basic-auth>::
It provides The "Basic" HTTP Authentication scheme using credentials as "user-id:password" pairs. Password must be encoded using the "BCrypt" password-hashing function [env: SERVER_BASIC_AUTH=]  [default: ]

-e, --cache-control-headers <cache-control-headers>::
Enable cache control headers for incoming requests based on a set of file types. The file type list can be found on ``src/control_headers.rs`` file [env: SERVER_CACHE_CONTROL_HEADERS=]  [default: true]

-x, --compression <compression>::
Gzip, Deflate or Brotli compression on demand determined by the Accept-Encoding header and applied to text-based web file types only [env: SERVER_COMPRESSION=]  [default: true]

-w, --config-file <config-file>::
Server TOML configuration file path [env: SERVER_CONFIG_FILE=]

-j, --cors-allow-headers <cors-allow-headers>::
Specify an optional CORS list of allowed headers separated by comas. Default "origin, content-type". It requires ``--cors-allow-origins`` to be used along with [env: SERVER_CORS_ALLOW_HEADERS=]  [default: origin, content-type]

-c, --cors-allow-origins <cors-allow-origins>::
Specify an optional CORS list of allowed origin hosts separated by comas. Host ports or protocols aren't being checked. Use an asterisk (*) to allow any host [env: SERVER_CORS_ALLOW_ORIGINS=]  [default: ]

-z, --directory-listing <directory-listing>::
Enable directory listing for all requests ending with the slash character (‘/’) [env: SERVER_DIRECTORY_LISTING=]  [default: false]

--directory-listing-order <directory-listing-order>::
Specify a default code number to order directory listing entries per ``Name``, ``Last modified`` or ``Size`` attributes (columns). Code numbers supported: 0 (Name asc), 1 (Name desc), 2 (Last modified asc), 3 (Last modified desc), 4 (Size asc), 5 (Size desc). Default 6 (unordered) [env: SERVER_DIRECTORY_LISTING_ORDER=] [default: 6]

-f, --fd <fd>::
Instead of binding to a TCP port, accept incoming connections to an already-bound TCP socket listener on the specified file descriptor number (usually zero). Requires that the parent process (e.g. inetd, launchd, or systemd) binds an address and port on behalf of static-web-server, before arranging for the resulting file descriptor to be inherited by static-web-server. Cannot be used in conjunction with the port and host arguments. The included systemd unit file utilises this feature to increase security by allowing the static-web-server to be sandboxed more completely [env: SERVER_LISTEN_FD=]

-q, --grace-period <grace-period>::
Defines a grace period in seconds after a ``SIGTERM`` signal is caught which will delay the server before to shut it down gracefully. The maximum value is 255 seconds [env: SERVER_GRACE_PERIOD=]  [default: 0]

-a, --host <host>::
Host address (E.g 127.0.0.1 or ::1) [env: SERVER_HOST=]  [default: ::]

-t, --http2 <http2>::
Enable HTTP/2 with TLS support [env: SERVER_HTTP2_TLS=]  [default: false]

--http2-tls-cert <http2-tls-cert>::
Specify the file path to read the certificate [env: SERVER_HTTP2_TLS_CERT=]

--http2-tls-key <http2-tls-key>::
Specify the file path to read the private key [env: SERVER_HTTP2_TLS_KEY=]

-g, --log-level <log-level>::
Specify a logging level in lower case. Values: error, warn, info, debug or trace [env: SERVER_LOG_LEVEL=] [default: error]

--log-remote-address <log-remote-address>::
Log incoming requests information along with its remote address if available using the ``info`` log level [env: SERVER_LOG_REMOTE_ADDRESS=]  [default: false]

--page-fallback <page-fallback>::
HTML file path that is used for GET requests when the requested path doesn't exist. The fallback page is served with a 200 status code, useful when using client routers. If the path is not specified or simply doesn't exist then this feature will not be active [env: SERVER_FALLBACK_PAGE=]

--page404 <page404>::
HTML file path for 404 errors. If the path is not specified or simply doesn't exist then the server will use a generic HTML error message [env: SERVER_ERROR_PAGE_404=]  [default: ./public/404.html]

--page50x <page50x>::
HTML file path for 50x errors. If the path is not specified or simply doesn't exist then the server will use a generic HTML error message [env: SERVER_ERROR_PAGE_50X=]  [default: ./public/50x.html]

-p, --port <port>::
Host port [env: SERVER_PORT=]  [default: 80]

-d, --root <root>::
Root directory path of static files [env: SERVER_ROOT=]  [default: ./public]

--security-headers <security-headers>::
Enable security headers by default when HTTP/2 feature is activated. Headers included: "Strict-Transport- Security: max-age=63072000; includeSubDomains; preload" (2 years max-age), "X-Frame-Options: DENY", "X-XSS-Protection: 1; mode=block" and "Content-Security-Policy: frame-ancestors 'self'" [env: SERVER_SECURITY_HEADERS=]  [default: false]

-n, --threads-multiplier <threads-multiplier>::
Number of worker threads multiplier that'll be multiplied by the number of system CPUs using the formula: ``worker threads = number of CPUs * n`` where ``n`` is the value that changes here. When multiplier value is 0 or 1 then one thread per core is used. Number of worker threads result should be a number between 1 and 32,768 though it is advised to keep this value on the smaller side [env: SERVER_THREADS_MULTIPLIER=] [default: 1]


VERSION
-------
2.9.0


HOMEPAGE
--------
https://sws.joseluisq.net


REPORTING BUGS
--------------

Report bugs and feature requests in the issue tracker. Please do your best to provide a reproducible test case for bugs.

https://github.com/joseluisq/static-web-server/issues

AUTHORS
-------
Jose Quintana <joseluisq.net>
