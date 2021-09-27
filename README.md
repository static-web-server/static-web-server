# Static Web Server [![CI](https://github.com/joseluisq/static-web-server/workflows/CI/badge.svg?branch=1.x)](https://github.com/joseluisq/static-web-server/actions?query=workflow%3ACI) [![Docker Image Version (tag latest semver)](https://img.shields.io/docker/v/joseluisq/static-web-server/1)](https://hub.docker.com/r/joseluisq/static-web-server/) [![Docker Image Size (tag)](https://img.shields.io/docker/image-size/joseluisq/static-web-server/1)](https://hub.docker.com/r/joseluisq/static-web-server/tags) [![Docker Image](https://img.shields.io/docker/pulls/joseluisq/static-web-server.svg)](https://hub.docker.com/r/joseluisq/static-web-server/)

*This is the stable `v1`. For the `v2` refer to [master](https://github.com/joseluisq/static-web-server) branch.*

_To migrate to `v2` please take a look at [v2.0.0](https://github.com/joseluisq/static-web-server/releases/tag/v2.0.0) release._

> A blazing fast static files-serving web server powered by [Rust Iron](https://github.com/iron/iron). ⚡

**Static Web Server** is a very small and fast production-ready web server to serving static web files or assets.

## Features

- Built with [Rust](https://rust-lang.org) which is focused on [safety, speed, and concurrency](https://kornel.ski/rust-c-speed).
- Memory safety and very reduced CPU and RAM overhead.
- Blazing fast static files-serving thanks to [Rust Iron](https://github.com/iron/iron) and [Hyper](https://github.com/hyperium/hyper).
- No dependencies. Just a single __4MB__ and fully static binary ([Musl libc](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)) which makes it suitable for running on [any Linux distro](https://en.wikipedia.org/wiki/Linux_distribution) or [Docker container](https://hub.docker.com/r/joseluisq/static-web-server/tags).
- Gzip compression on demand via [accept-encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header.
- [Partial Content Delivery](https://en.wikipedia.org/wiki/Byte_serving) support for byte-serving of large files.
- [Cache control headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) for assets.
- [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) support.
- [HEAD](https://tools.ietf.org/html/rfc7231#section-4.3.2) responses support.
- [TLS](https://www.openssl.org/) support via [Rust Native TLS](https://docs.rs/native-tls/0.2.3/native_tls/) crate.
- Optional directory listing.
- Lightweight and configurable logging.
- Optional HTTP to HTTPS redirection.
- First-class [Docker](https://docs.docker.com/get-started/overview/) support. [Scratch](https://hub.docker.com/_/scratch) and latest [Alpine Linux](https://hub.docker.com/_/alpine) Docker images.
- Server configurable via CLI arguments or their equivalent environment variables.
- Cross-platform. Binaries available for Linux, macOS and Windows x86_64/ARM64.

## Releases

Available for download/install via following methods:

### Docker Images (Linux x86_64)

[hub.docker.com/r/joseluisq/static-web-server/](https://hub.docker.com/r/joseluisq/static-web-server/)

### Release binaries

[github.com/joseluisq/static-web-server/releases](https://github.com/joseluisq/static-web-server/releases).

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
| `SERVER_NAME` | Name for server. | Default `my-static-server`. |
| `SERVER_HOST` | Host address (E.g 127.0.0.1). | Default `[::]`. |
| `SERVER_PORT` | Host port. | Default `80`. |
| `SERVER_ROOT` | Root directory path of static files. | Default `./public`. |
| `SERVER_ASSETS` | Assets directory path for add cache headers functionality. | Default `./public/assets`. |
| `SERVER_LOG_LEVEL` | Specify a logging level in lower case (see [log::LevelFilter](https://docs.rs/log/0.4.10/log/enum.LevelFilter.html)). | Default `error` |
| `SERVER_ERROR_PAGE_404` | HTML file path for 404 errors. | If path is not specified or simply don't exists then server will use a generic HTML error message. Default file path `./public/404.html`. |
| `SERVER_ERROR_PAGE_50X` | HTML file path for 50x errors. | If path is not specified or simply don't exists then server will use a generic HTML error message. Default file path `./public/50x.html` |
| `SERVER_TLS` | Enables TLS/SSL support. Make sure also to adjust current server port. | Default `false` |
| `SERVER_TLS_PKCS12` | A cryptographic identity [PKCS #12](https://docs.rs/native-tls/0.2.3/native_tls/struct.Identity.html#method.from_pkcs12) bundle file path containing a [X509 certificate](https://en.wikipedia.org/wiki/X.509) along with its corresponding private key and chain of certificates to a trusted root. | Default empty |
| `SERVER_TLS_PKCS12_PASSWD` | A specified password to decrypt the private key. | Default empty |
| `SERVER_TLS_REDIRECT_FROM` | Host port for redirecting HTTP requests to HTTPS. This option enables the HTTP redirect feature | Default empty (disabled) |
| `SERVER_TLS_REDIRECT_HOST` | Host name of HTTPS site for redirecting HTTP requests to. | Default host address |
| `SERVER_CORS_ALLOW_ORIGINS` | Specify a CORS list of allowed origin hosts separated by comas. Host ports or protocols aren't being checked. Use an asterisk (*) to allow any host. See [Iron CORS crate](https://docs.rs/iron-cors/0.8.0/iron_cors/#mode-1-whitelist). | Default empty (disabled) |
| `SERVER_DIRECTORY_LISTING` | Enable directory listing for all requests ending with the slash character (‘/’) | Default `false` (disabled) |

### Command-line arguments

CLI arguments listed with `static-web-server -h`.

```
static-web-server 1.18.0
A blazing fast static files-serving web server powered by Rust Iron

USAGE:
    static-web-server [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --assets <assets>
            Assets directory path for add cache headers functionality [env: SERVER_ASSETS=]  [default: ./public/assets]

    -c, --cors-allow-origins <cors-allow-origins>
            Specify a CORS list of allowed origin hosts separated by comas. Host ports or protocols aren't being
            checked. Use an asterisk (*) to allow any host [env: SERVER_CORS_ALLOW_ORIGINS=]
    -i, --directory-listing <directory-listing>
            Enable directory listing for all requests ending with the slash character (‘/’) [env:
            SERVER_DIRECTORY_LISTING=]
    -a, --host <host>                                Host address (E.g 127.0.0.1) [env: SERVER_HOST=]  [default: [::]]
    -g, --log-level <log-level>
            Specify a logging level in lower case [env: SERVER_LOG_LEVEL=]  [default: error]

    -l, --name <name>                                Name for server [env: SERVER_NAME=]
        --page404 <page404>
            HTML file path for 404 errors. If path is not specified or simply don't exists then server will use a
            generic HTML error message [env: SERVER_ERROR_PAGE_404=]  [default: ./public/404.html]
        --page50x <page50x>
            HTML file path for 50x errors. If path is not specified or simply don't exists then server will use a
            generic HTML error message [env: SERVER_ERROR_PAGE_50X=]  [default: ./public/50x.html]
    -p, --port <port>                                Host port [env: SERVER_PORT=]  [default: 80]
    -d, --root <root>
            Root directory path of static files [env: SERVER_ROOT=]  [default: ./public]

    -t, --tls <tls>                                  Enables TLS/SSL support [env: SERVER_TLS=]
        --tls-pkcs12 <tls-pkcs12>
            A cryptographic identity PKCS #12 bundle file path containing a X509 certificate along with its
            corresponding private key and chain of certificates to a trusted root [env: SERVER_TLS_PKCS12=]
        --tls-pkcs12-passwd <tls-pkcs12-passwd>
            A specified password to decrypt the private key [env: SERVER_TLS_PKCS12_PASSWD=]

        --tls-redirect-from <tls-redirect-from>
            Host port for redirecting HTTP requests to HTTPS. This option enables the HTTP redirect feature [env:
            SERVER_TLS_REDIRECT_FROM=]
        --tls-redirect-host <tls-redirect-host>
            Host name of HTTPS site for redirecting HTTP requests to. Defaults to host address [env:
            SERVER_TLS_REDIRECT_HOST=]
```

## TLS/SSL

TLS/SSL support is provided by [Rust Native TLS](https://docs.rs/native-tls/0.2.3/native_tls/struct.Identity.html#method.from_pkcs12) crate which supports [PKCS #12 cryptographic identity](https://en.wikipedia.org/wiki/PKCS_12).
An identity is an [X509 certificate](https://en.wikipedia.org/wiki/X.509) certificate along with its corresponding private key and chain of certificates to a trusted root.

For instance, identity files (`.p12` or `.pfx`) can be generated using the [OpenSSL SSL/TLS Toolkit](https://www.openssl.org/docs/manmaster/man1/pkcs12.html):

Generate a self-signed certificate (optional):

```sh
openssl req -x509 -newkey rsa:4096 -nodes -keyout local.key -out local.crt -days 3650
```

Generate a PKCS #12 indentity file (using an existing certificate and private key):

```sh
openssl pkcs12 -export -out identity.p12 -inkey local.key -in local.crt -password pass:my_password
```

## Docker stack

Example using [Traefik Proxy](https://traefik.io/):

```yaml
version: "3.3"

services:
  web:
    image: joseluisq/static-web-server:1
    environment:
        - SERVER_NAME=my-server
        - SERVER_HOST=127.0.0.1
        - SERVER_PORT=80
        - SERVER_ROOT=/public
        # NOTE:
        #   For the server, assets directory is not relative to root.
        #   That's why, it's necessary to be explicit (prefer absolute paths).
        #   See release v1.8.0+ for more details.
        - SERVER_ASSETS=/public/assets
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
