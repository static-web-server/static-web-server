# Docker

`SWS` has first-class [Docker](https://docs.docker.com/get-started/overview/) support.

It is provided in three Docker image variants such as [Scratch](https://hub.docker.com/_/scratch), [Alpine](https://hub.docker.com/_/alpine) and [Debian](https://hub.docker.com/_/debian) images.

All images are available on [Docker Hub](https://hub.docker.com/r/joseluisq/static-web-server/) and [GitHub Container Registry](https://github.com/static-web-server/static-web-server/pkgs/container/static-web-server)

## OS/Arch

All Docker images are [Multi-Arch](https://www.docker.com/blog/how-to-rapidly-build-multi-architecture-images-with-buildx/) and the following operating systems and architectures are supported.

- `linux/386`
- `linux/amd64`
- `linux/arm/v6`
- `linux/arm/v7`
- `linux/arm64`

!!! tip "SWS statically-linked binary"
    All the Docker images use the SWS statically-linked binary, meaning that the binary is highly-optimized, performant, and dependency-free thanks to [musl libc](https://www.musl-libc.org/).

## Run a container

To give the server a quick try then just run the following commands.

!!! tip "Tips"
    - [The SWS CLI arguments](../configuration/command-line-arguments.md) can be provided directly to the container or omitted as shown below.
    - A Docker volume like `-v $HOME/my-public-dir:/public` can be specified to overwrite the default root directory.

To run SWS, there are several Docker image variants that you can use.

**Scratch** (just the binary)

```sh
docker run --rm -it -p 8787:80 joseluisq/static-web-server:2 -g info
# or
docker run --rm -it -p 8787:80 ghcr.io/static-web-server/static-web-server:2 -g info
```

**Alpine**

```sh
docker run --rm -it -p 8787:80 joseluisq/static-web-server:2-alpine -g info
# or
docker run --rm -it -p 8787:80 ghcr.io/static-web-server/static-web-server:2-alpine -g info
```

**Debian**

```sh
docker run --rm -it -p 8787:80 joseluisq/static-web-server:2-debian -g info
# or
docker run --rm -it -p 8787:80 ghcr.io/static-web-server/static-web-server:2-debian -g info
```

## Dockerfile

SWS Docker images can be extended as needed.

Extending the **Scratch** Docker image (just the binary)

```Dockerfile
FROM joseluisq/static-web-server:2
# or
FROM ghcr.io/static-web-server/static-web-server:2
# do stuff...
```

Or the **Alpine**

```Dockerfile
FROM joseluisq/static-web-server:2-alpine
# or
FROM ghcr.io/static-web-server/static-web-server:2-alpine
# do stuff...
```

Or the **Debian**

```Dockerfile
FROM joseluisq/static-web-server:2-debian
# or
FROM ghcr.io/static-web-server/static-web-server:2-debian
# do stuff...
```

## Docker Compose

Below is a [Docker Compose](https://docs.docker.com/compose/) example using the [Traefik Proxy](https://traefik.io/traefik/).

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
