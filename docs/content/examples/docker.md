# Docker

`SWS` has first-class [Docker](https://docs.docker.com/get-started/overview/) support. It provides a [Scratch](https://hub.docker.com/_/scratch) as well as the latest [Alpine Linux](https://hub.docker.com/_/alpine) Docker images.

🐋 View on [Docker Hub](https://hub.docker.com/r/joseluisq/static-web-server/).

## Run a container

```sh
# Scratch image (just the binary)
docker run --rm -it -p 8787:80 joseluisq/static-web-server:2

# Or Alpine image
docker run --rm -it -p 8787:80 joseluisq/static-web-server:2-alpine
```

## Dockerfile

```Dockerfile
# Scratch image (just the binary)
FROM joseluisq/static-web-server:2

# Or Alpine image
FROM joseluisq/static-web-server:2-alpine
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
