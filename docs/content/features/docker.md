# Docker

`SWS` has first-class [Docker](https://docs.docker.com/get-started/overview/) support. It provides a [Scratch](https://hub.docker.com/_/scratch) as well as the latest [Alpine Linux](https://hub.docker.com/_/alpine) Docker images.

## OS/Arch

Only the following operating systems and architectures are supported.

- `linux/386`
- `linux/amd64`
- `linux/arm/v6`
- `linux/arm/v7`
- `linux/arm64`

View all images on [Docker Hub](https://hub.docker.com/r/joseluisq/static-web-server/).

## Run a container

To give the server a quick try just run the following commands.

!!! info "Tips"
    - [Server CLI arguments](/configuration/command-line-arguments/) can be provided directly to the container or omitted as shown below.
    - It can specify a Docker volume like `-v $HOME/my-public-dir:/public` to overwrite the default root directory.

Run the scratch Docker image (just the binary)

```sh
docker run --rm -it -p 8787:80 joseluisq/static-web-server:2 -g info
```

Or run the Alpine Docker image variant

```sh
docker run --rm -it -p 8787:80 joseluisq/static-web-server:2-alpine -g info
```

## Dockerfile

SWS Docker images can be extended as needed.

Extending the scratch Docker image (just the binary)

```Dockerfile
FROM joseluisq/static-web-server:2
# do stuff...
```

Or the Alpine Docker image variant

```Dockerfile
FROM joseluisq/static-web-server:2-alpine
# do stuff...
```

## Docker Compose

Below a [Docker Compose](https://docs.docker.com/compose/) example using the [Traefik Proxy](https://traefik.io/traefik/).

```yaml
version: "3.3"

services:
  web:
    image: joseluisq/static-web-server:2
    environment:
      # Note: those envs are customizable but also optional
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
