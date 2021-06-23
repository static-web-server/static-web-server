# Static Web Server [![CI](https://github.com/joseluisq/static-web-server/workflows/CI/badge.svg)](https://github.com/joseluisq/static-web-server/actions?query=workflow%3ACI) [![Docker Image Version (tag latest semver)](https://img.shields.io/docker/v/joseluisq/static-web-server/1)](https://hub.docker.com/r/joseluisq/static-web-server/) [![Docker Image Size (tag)](https://img.shields.io/docker/image-size/joseluisq/static-web-server/1)](https://hub.docker.com/r/joseluisq/static-web-server/tags) [![Docker Image](https://img.shields.io/docker/pulls/joseluisq/static-web-server.svg)](https://hub.docker.com/r/joseluisq/static-web-server/)

**Status:** `v2` under **active** development. For the stable `v1` please refer to [1.x](https://github.com/joseluisq/static-web-server/tree/1.x) branch.

> A blazing fast and asynchronous web server for static files-serving. ⚡

**Static Web Server** is a very small and fast production-ready web server to serving static web files or assets.

## Features

- Built with [Rust](https://rust-lang.org) which is focused on [safety, speed, and concurrency](https://kornel.ski/rust-c-speed).
- Memory safety and very reduced CPU and RAM overhead.
- Blazing fast static files-serving and asynchronous powered by latest [Hyper](https://github.com/hyperium/hyper/), [Tokio](https://github.com/tokio-rs/tokio) and a set of [awesome crates](./Cargo.toml).
- Single __4MB__ and fully static binary with no dependencies ([Musl libc](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)). Suitable for running on [any Linux distro](https://en.wikipedia.org/wiki/Linux_distribution) or [Docker container](https://hub.docker.com/r/joseluisq/static-web-server/tags).
- GZip, Deflate or Brotli compression for text-based web files only.
- Compression on demand via [Accept-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header.
- [Partial Content Delivery](https://en.wikipedia.org/wiki/Byte_serving) support for byte-serving of large files.
- [Cache Control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) headers for assets.
- [HEAD](https://tools.ietf.org/html/rfc7231#section-4.3.2) responses.
- Lightweight and configurable logging via [tracing](https://github.com/tokio-rs/tracing) crate.
- [Termination signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling.
- [HTTP/2](https://tools.ietf.org/html/rfc7540) + TLS support.
- [Security headers](https://web.dev/security-headers/) for HTTP/2 by default.
- Customizable number of worker threads.
- Optional directory listing.
- [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) support.
- Default and custom error pages.
- Configurable using CLI arguments or environment variables.
- First-class [Docker](https://docs.docker.com/get-started/overview/) support. [Scratch](https://hub.docker.com/_/scratch) and latest [Alpine Linux](https://hub.docker.com/_/alpine) Docker images available.
- The ability to accept a socket listener as a file descriptor for use in sandboxing and on-demand applications (E.g [systemd](http://0pointer.de/blog/projects/socket-activation.html)).
- Cross-platform. Binaries available for Linux, macOS and Windows\* x86_64/ARM64.

## Releases

### Docker image

Available on [hub.docker.com/r/joseluisq/static-web-server](https://hub.docker.com/r/joseluisq/static-web-server/)

### Release binaries

Available to download on [github.com/joseluisq/static-web-server/releases](https://github.com/joseluisq/static-web-server/releases).

Below the current supported targets.

#### Linux

- x86_64-unknown-linux-gnu (64-bit)
- x86_64-unknown-linux-musl (64-bit)
- aarch64-unknown-linux-musl (ARM64)
- aarch64-unknown-linux-gnu (ARM64)
- arm-unknown-linux-gnueabihf (ARM)

#### macOS

- x86_64-apple-darwin (64-bit)
- aarch64-apple-darwin (ARM64)

#### Windows

- x86_64-pc-windows-msvc (64-bit)
- aarch64-pc-windows-msvc (ARM64)

## Usage

Server can be configured either via environment variables or their equivalent command-line arguments.

### Environment Variables

| Variable | Description | Default |
| --- | --- | --- |
| `SERVER_HOST` | Host address (E.g 127.0.0.1). | Default `[::]`. |
| `SERVER_PORT` | Host port. | Default `80`. |
| `SERVER_LISTEN_FD` | Optional file descriptor number (e.g. `0`) to inherit an already-opened TCP listener on (instead of using `SERVER_HOST` and/or `SERVER_PORT` ). |
| `SERVER_ROOT` | Root directory path of static | Default `./public`. |
| `SERVER_LOG_LEVEL`          | Specify a logging level in lower case. (Values `error`, `warn`, `info`, `debug`, `trace`). | Default `error` |
| `SERVER_ERROR_PAGE_404`     | HTML file path for 404 errors. | If path is not specified or simply don't exists then server will use a generic HTML error message. Default `./public/404.html`. |
| `SERVER_ERROR_PAGE_50X`     | HTML file path for 50x errors. | If path is not specified or simply don't exists then server will use a generic HTML error message. Default `./public/50x.html`  |
| `SERVER_THREADS_MULTIPLIER` | Number of worker threads multiplier that'll be multiplied by the number of system CPUs using the formula: `worker threads = number of CPUs * n` where `n` is the value that changes here. When multiplier value is 0 or 1 then the `number of CPUs` is used. Number of worker threads result should be a number between 1 and 32,768 though it is advised to keep this value on the smaller side. | Default one thread per core. |
| `SERVER_HTTP2_TLS` | Enable HTTP/2 with TLS support. Make sure also to adjust current server port. | Default `false` |
| `SERVER_HTTP2_TLS_CERT`     | Specify the file path to read the certificate. | Default empty |
| `SERVER_HTTP2_TLS_KEY`      | Specify the file path to read the private key. | Default empty |
| `SERVER_CORS_ALLOW_ORIGINS` | Specify a optional CORS list of allowed origin hosts separated by comas. Host ports or protocols aren't being checked. Use an asterisk (*) to allow any host. | Default empty (which means CORS is disabled) |
| `SERVER_COMPRESSION`  | Gzip, Deflate or Brotli compression on demand determined by the *Accept-Encoding* header and applied to text-based web file types only. See [ad-hoc mime-type list](https://github.com/joseluisq/static-web-server/blob/master/src/compression.rs#L20) | Default `true` (enabled) |
| `SERVER_DIRECTORY_LISTING`  | Enable directory listing for all requests ending with the slash character (‘/’) | Default `false` (disabled) |
| `SERVER_SECURITY_HEADERS` | Enable security headers by default when HTTP/2 feature is activated. Headers included: `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload` (2 years max-age), `X-Frame-Options: DENY`, `X-XSS-Protection: 1; mode=block` and `Content-Security-Policy: frame-ancestors 'self'` | Default `false` (disabled) |

### Command-line arguments

CLI arguments listed with `static-web-server -h`.

```
static-web-server 2.0.0-beta.4
A blazing fast static files-serving web server powered by Rust

USAGE:
    static-web-server [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -x, --compression <compression>
            Gzip, Deflate or Brotli compression on demand determined by the Accept-Encoding header and applied to text-
            based web file types only [env: SERVER_COMPRESSION=]  [default: true]
    -c, --cors-allow-origins <cors-allow-origins>
            Specify an optional CORS list of allowed origin hosts separated by comas. Host ports or protocols aren't
            being checked. Use an asterisk (*) to allow any host [env: SERVER_CORS_ALLOW_ORIGINS=]  [default: ]
    -z, --directory-listing <directory-listing>
            Enable directory listing for all requests ending with the slash character (‘/’) [env:
            SERVER_DIRECTORY_LISTING=]  [default: false]
    -f, --fd <fd>
            Instead of binding to a TCP port, accept incoming connections to an already-bound TCP socket listener on the
            specified file descriptor number (usually zero). Requires that the parent process (e.g. inetd, launchd, or
            systemd) binds an address and port on behalf of static-web-server, before arranging for the resulting file
            descriptor to be inherited by static-web-server. Cannot be used in conjunction with the port and host
            arguments. The included systemd unit file utilises this feature to increase security by allowing the static-
            web-server to be sandboxed more completely [env: SERVER_LISTEN_FD=]
    -a, --host <host>
            Host address (E.g 127.0.0.1 or ::1) [env: SERVER_HOST=]  [default: ::]

    -t, --http2 <http2>
            Enable HTTP/2 with TLS support [env: SERVER_HTTP2_TLS=]  [default: false]

        --http2-tls-cert <http2-tls-cert>
            Specify the file path to read the certificate [env: SERVER_HTTP2_TLS_CERT=]  [default: ]

        --http2-tls-key <http2-tls-key>
            Specify the file path to read the private key [env: SERVER_HTTP2_TLS_KEY=]  [default: ]

    -g, --log-level <log-level>
            Specify a logging level in lower case. Values: error, warn, info, debug or trace [env: SERVER_LOG_LEVEL=]
            [default: error]
        --page404 <page404>
            HTML file path for 404 errors. If path is not specified or simply don't exists then server will use a
            generic HTML error message [env: SERVER_ERROR_PAGE_404=]  [default: ./public/404.html]
        --page50x <page50x>
            HTML file path for 50x errors. If path is not specified or simply don't exists then server will use a
            generic HTML error message [env: SERVER_ERROR_PAGE_50X=]  [default: ./public/50x.html]
    -p, --port <port>                                Host port [env: SERVER_PORT=]  [default: 80]
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

## Use of file descriptor socket passing

Example `systemd` unit files for socket activation are included in the [`systemd/`](systemd/) directory. If
using `inetd`, its "`wait`" option should be used in conjunction with static-web-server's `--fd 0`
option.

Alternatively, the light-weight [`systemfd`](https://github.com/mitsuhiko/systemfd) utility may be
useful - especially for testing e.g. `systemfd --no-pid -s http::8091 -- path/to/static-web-server --fd 0`

## Docker stack

Example using [Traefik Proxy](https://traefik.io/):

```yaml
version: "3.3"

services:
  web:
    image: joseluisq/static-web-server:2.0.0-beta.4
    environment:
        - SERVER_HOST=127.0.0.1
        - SERVER_PORT=80
        - SERVER_ROOT=/public
    volumes:
        - ./some-dir-path:/public
    labels:
        - "traefik.enable=true"
        - "traefik.frontend.entryPoints=https"
        - "traefik.backend=localhost_dev"
        - "traefik.frontend.rule=Host:localhost.dev"
        - "traefik.port=80"
    networks:
        - traefik_net

networks:
    traefik_net:
        external: true
```

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in current work by you, as defined in the Apache-2.0 license, shall be dual licensed as described below, without any additional terms or conditions.

Feel free to send some [Pull request](https://github.com/joseluisq/static-web-server/pulls) or [issue](https://github.com/joseluisq/static-web-server/issues).

## License

This work is primarily distributed under the terms of both the [MIT license](LICENSE-MIT) and the [Apache License (Version 2.0)](LICENSE-APACHE).

© 2019-present [Jose Quintana](https://git.io/joseluisq)
