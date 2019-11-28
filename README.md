# Static Web Server [![Build Status](https://travis-ci.com/joseluisq/static-web-server.svg?branch=master)](https://travis-ci.com/joseluisq/static-web-server) [![](https://images.microbadger.com/badges/image/joseluisq/static-web-server.svg)](https://microbadger.com/images/joseluisq/static-web-server "Get your own image badge on microbadger.com") [![Docker Image](https://img.shields.io/docker/pulls/joseluisq/static-web-server.svg?maxAge=2592000)](https://hub.docker.com/r/joseluisq/static-web-server/)

> A blazing fast static files-serving web server powered by [Rust Iron](https://github.com/iron/iron). :zap:

**Static Web Server** is a small (`1,6M` static binary) and fast web server to serving static files like html files or assets.

It's also an option to deploy it inside a [Docker container](https://hub.docker.com/r/joseluisq/static-web-server).

__Status:__ It's in beta yet. Feel free to contribute.

## Usage

Server is configured via environment variables:

- **SERVER_NAME**: Name for server. Default `my-static-server`.
- **SERVER_HOST**: Host address (E.g 127.0.0.1). Default `[::]`.
- **SERVER_PORT**: Host port. Default `80`.
- **SERVER_ROOT**: Root directory path of static files. Default `./public`.
- **SERVER_ASSETS**: Assets directory path for add cache headers functionality. Default `./assets` but relative to the root.

## Docker stack

Example using [Traefik proxy](https://traefik.io/):

```yaml
version: "3.3"

services:
  web:
    image: joseluisq/static-web-server:latest
    environment:
        - SERVER_NAME=my-server
        - SERVER_HOST=127.0.0.1
        - SERVER_PORT=8080
        - SERVER_ROOT=/html
        - SERVER_ASSETS=./assets
    volumes:
        - ./some-dir-path:/html
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

© 2019 [Jose Quintana](https://git.io/joseluisq)
