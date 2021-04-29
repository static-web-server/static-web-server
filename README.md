# Static Web Server [![CI](https://github.com/joseluisq/static-web-server/workflows/CI/badge.svg)](https://github.com/joseluisq/static-web-server/actions?query=workflow%3ACI) [![Docker Image Version (tag latest semver)](https://img.shields.io/docker/v/joseluisq/static-web-server/1)](https://hub.docker.com/r/joseluisq/static-web-server/) [![Docker Image Size (tag)](https://img.shields.io/docker/image-size/joseluisq/static-web-server/1)](https://hub.docker.com/r/joseluisq/static-web-server/tags) [![Docker Image](https://img.shields.io/docker/pulls/joseluisq/static-web-server.svg)](https://hub.docker.com/r/joseluisq/static-web-server/)

**Status:** WIP `v2` release under **active** development. For the stable `v1` and contributions please refer to [1.x](https://github.com/joseluisq/static-web-server/tree/1.x) branch.

> A blazing fast static files-serving web server. ⚡

**Static Web Server** is a very small and fast production-ready web server to serving static web files or assets.

## Features

- Built with [Rust](https://rust-lang.org) which is focused on [safety, speed, and concurrency](https://kornel.ski/rust-c-speed).
- Memory safety and very reduced CPU and RAM overhead.
- Blazing fast static files-serving and asynchronous powered by [Hyper](https://github.com/hyperium/hyper/) `v0.14`, [Tokio](https://github.com/tokio-rs/tokio) `v1` and a set of [awesome crates](./Cargo.toml).
- Suitable for lightweight [GNU/Linux Docker containers](https://hub.docker.com/r/joseluisq/static-web-server/tags). It's a fully __5MB__ static binary thanks to [Rust and Musl libc](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html).
- GZip, Deflate or Brotli compression for text-based web files only.
- Compression on demand via [Accept-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header.
- [Partial Content Delivery](https://en.wikipedia.org/wiki/Byte_serving) support for byte-serving of large files.
- [Cache Control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) headers for assets.
- [HEAD](https://tools.ietf.org/html/rfc7231#section-4.3.2) responses.
- Lightweight and configurable logging via [tracing](https://github.com/tokio-rs/tracing) crate.
- [Termination signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling.
- [HTTP/2](https://tools.ietf.org/html/rfc7540) + TLS support.
- Customizable number of worker threads.
- Default and custom error pages.
- [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) support.
- Configurable using CLI arguments or environment variables.
- First-class [Docker](https://docs.docker.com/get-started/overview/) support. [Scratch](https://hub.docker.com/_/scratch) and latest [Alpine Linux](https://hub.docker.com/_/alpine) Docker images available.
- MacOs binary support thanks to [Rust Linux / Darwin Builder](https://github.com/joseluisq/rust-linux-darwin-builder).

## Releases

Available for download/install via following methods:

- **Docker Image** on [hub.docker.com/r/joseluisq/static-web-server/](https://hub.docker.com/r/joseluisq/static-web-server/)
- **Release binaries** for `GNU/Linux` and `MacOS` x86_64 on [github.com/joseluisq/static-web-server/releases](https://github.com/joseluisq/static-web-server/releases).

## Usage

Server can be configured either via environment variables or their equivalent command-line arguments.

### Environment Variables

| Variable                    | Description                                                                                                                                                                                                                                                                                                                                                                                        | Default                                                                                                                         |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `SERVER_HOST`               | Host address (E.g 127.0.0.1).                                                                                                                                                                                                                                                                                                                                                                      | Default `[::]`.                                                                                                                 |
| `SERVER_PORT`               | Host port.                                                                                                                                                                                                                                                                                                                                                                                         | Default `80`.                                                                                                                   |
| `SERVER_ROOT`               | Root directory path of static                                                                                                                                                                                                                                                                                                                                                                      | Default `./public`.                                                                                                             |
| `SERVER_LOG_LEVEL`          | Specify a logging level in lower case. (Values `error`, `warn`, `info`, `debug`, `trace`).                                                                                                                                                                                                                                                                                                         | Default `error`                                                                                                                 |
| `SERVER_ERROR_PAGE_404`     | HTML file path for 404 errors.                                                                                                                                                                                                                                                                                                                                                                     | If path is not specified or simply don't exists then server will use a generic HTML error message. Default `./public/404.html`. |
| `SERVER_ERROR_PAGE_50X`     | HTML file path for 50x errors.                                                                                                                                                                                                                                                                                                                                                                     | If path is not specified or simply don't exists then server will use a generic HTML error message. Default `./public/50x.html`  |
| `SERVER_THREADS_MULTIPLIER` | Number of worker threads multiplier that'll be multiplied by the number of system CPUs using the formula: `worker threads = number of CPUs * n` where `n` is the value that changes here. When multiplier value is 0 or 1 then the `number of CPUs` is used.  Number of worker threads result should be a number between 1 and 32,768 though it is advised to keep this value on the smaller side. | Default `8`                                                                                                                     |
| `SERVER_HTTP2_TLS`          | Enable HTTP/2 with TLS support. Make sure also to adjust current server port.                                                                                                                                                                                                                                                                                                                      | Default `false`                                                                                                                 |
| `SERVER_HTTP2_TLS_CERT`     | Specify the file path to read the certificate.                                                                                                                                                                                                                                                                                                                                                     | Default empty                                                                                                                   |
| `SERVER_HTTP2_TLS_KEY`      | Specify the file path to read the private key.                                                                                                                                                                                                                                                                                                                                                     | Default empty                                                                                                                   |
| `SERVER_CORS_ALLOW_ORIGINS` | Specify a optional CORS list of allowed origin hosts separated by comas. Host ports or protocols aren't being checked. Use an asterisk (*) to allow any host.                                                                                                                                                                                                                                      | Default empty (which means CORS is disabled)                                                                                    |

### Command-line arguments

CLI arguments listed with `static-web-server -h`.

```
static-web-server 2.0.0-beta.3
A blazing fast static files-serving web server powered by Rust

USAGE:
    static-web-server [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cors-allow-origins <cors-allow-origins>
            Specify an optional CORS list of allowed origin hosts separated by comas. Host ports or protocols aren't
            being checked. Use an asterisk (*) to allow any host [env: SERVER_CORS_ALLOW_ORIGINS=]  [default: ]
    -a, --host <host>
            Host address (E.g 127.0.0.1 or ::1) [env: SERVER_HOST=]  [default: ::]

    -t, --http2 <http2>                              Enable HTTP/2 with TLS support [env: SERVER_HTTP2_TLS=]
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

    -n, --threads-multiplier <threads-multiplier>
            Number of worker threads multiplier that'll be multiplied by the number of system CPUs using the formula:
            `worker threads = number of CPUs * n` where `n` is the value that changes here. When multiplier value is 0
            or 1 then the `number of CPUs` is used. Number of worker threads result should be a number between 1 and
            32,768 though it is advised to keep this value on the smaller side [env: SERVER_THREADS_MULTIPLIER=]
            [default: 8]
```

## Docker stack

Example using [Traefik Proxy](https://traefik.io/):

```yaml
version: "3.3"

services:
  web:
    image: joseluisq/static-web-server:2.0.0-beta.3
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
