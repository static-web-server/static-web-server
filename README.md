# Static Web Server [![Build Status](https://ci.joseluisq.net/api/badges/joseluisq/static-web-server/status.svg?branch=develop)](https://ci.joseluisq.net/joseluisq/static-web-server) [![](https://images.microbadger.com/badges/image/joseluisq/static-web-server.svg)](https://microbadger.com/images/joseluisq/static-web-server "Get your own image badge on microbadger.com") [![Docker Cloud Build Status](https://img.shields.io/docker/cloud/build/joseluisq/static-web-server.svg)]

> A fast web server to static files-serving powered by [Rust Iron](https://github.com/iron/iron). :zap:

**Static Web Server** is an small (`1,6M` static binary) and fast web server to serving static files. Which is also suitable to deploy it into a Docker container.

__Status:__ The status is WIP so feel free to contribute.

## Usage

Server is configurated via environment variables:

- **SERVER_NAME**: Name for server. Default `nameless`.
- **SERVER_HOST**: Host address (E.g 127.0.0.1). Default `[::]`.
- **SERVER_PORT**: Host port. Default `80`.
- **SERVER_ROOT**: Root directory path of static files. Default `/public`.
- **SERVER_ASSETS**: Assets directory path for add cache headers functionality. Default `/public/assets`.

## Docker stack

Example using Traefik proxy

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

## Development

```sh
~> make help

Static Web Server
Web Server to static files-serving.

Please use `make <target>` where <target> is one of:
    install           to install dependencies.
    run               to run server in development.
    watch             to run server (watch files mode) in development.
    release           to build a release.
    docker_image      to build a Docker image.
```

## Contributions

Feel free to send some [Pull request](https://github.com/joseluisq/static-web-server/pulls) or [issue](https://github.com/joseluisq/static-web-server/issues).

## License
MIT license

© 2019 [Jose Quintana](https://git.io/joseluisq)
