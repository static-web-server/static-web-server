# Docker

`SWS` has first-class [Docker](https://docs.docker.com/get-started/overview/) support.

It is provided in three Docker image variants such as [Scratch](https://hub.docker.com/_/scratch), [Alpine](https://hub.docker.com/_/alpine) and [Debian](https://hub.docker.com/_/debian) images.

All images are available on [Docker Hub](https://hub.docker.com/r/joseluisq/static-web-server/) and [GitHub Container Registry](https://github.com/static-web-server/static-web-server/pkgs/container/static-web-server).

## OS/Arch

All Docker images are [Multi-Arch](https://www.docker.com/blog/how-to-rapidly-build-multi-architecture-images-with-buildx/) and the following operating systems and architectures are supported.

- `linux/386`
- `linux/amd64`
- `linux/arm/v6`
- `linux/arm/v7`
- `linux/arm64`
- `linux/ppc64le` (Debian only)
- `linux/s390x` (Debian only)

!!! tip "SWS statically-linked binary"
    All the Docker images use the SWS statically-linked binary, meaning that the binary is highly optimized, performant, and dependency-free thanks to [musl libc](https://www.musl-libc.org/).

## Run a container

To give the server a quick try just run the following commands.

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

Example using [Docker Compose](https://docs.docker.com/compose/).

```yaml
version: "3.3"

services:
  website:
    image: joseluisq/static-web-server:2-alpine
    container_name: "website"
    ports:
      - 80:80
    restart: unless-stopped
    environment:
      # Note: those envs are customizable but also optional
      - SERVER_ROOT=/var/public
      - SERVER_CONFIG_FILE=/etc/config.toml
    volumes:
      - ./public:/var/public
      - ./config.toml:/etc/config.toml
```

## Traefik Proxy

Example using [Docker Swarm](https://docs.docker.com/engine/swarm/) and [Traefik Proxy](https://traefik.io/traefik/).

1. Create an external `traefik_net` Docker attachable network for Traefik:
    - `docker network create --driver=overlay --attachable traefik_net`
2. Map a host directory like `/var/www/website` to the service container or create an external `website_data` Docker volume if you prefer:
    - `docker volume create website_data`

```yaml
version: "3.3"

services:
  traefik:
    image: "traefik:v2.11"
    command:
      #- "--log.level=DEBUG"
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.web.address=:80"
    ports:
      - "80:80"
      - "8080:8080"
    volumes:
      - "/var/run/docker.sock:/var/run/docker.sock:ro"

  website:
    image: joseluisq/static-web-server:2
    environment:
      # Note: those envs are customizable but also optional
      - SERVER_ROOT=/public
    volumes:
      - /var/www/website:/public
      # Or use an existing Docker volume
      # - website_data:/public
    labels:
      - "traefik.enable=true"
      - "traefik.docker.network=traefik_net"
      - "traefik.http.routers.website.entrypoints=web"
      - "traefik.http.routers.website.rule=Host(`website.localhost`)"
      - "traefik.http.routers.website.priority=1"
      - "traefik.http.services.website.loadbalancer.server.port=80"
    networks:
      - traefik_net

# volumes:
#   website_data:
#     external: true

networks:
  traefik_net:
    external: true
```

## Kubernetes

Example using [Kubernetes](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/) pod with liveness probe.

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: website
spec:
  containers:
    - name: sws
      image: ghcr.io/static-web-server/static-web-server
      command:
        - static-web-server
        - --root=/public
        - --health
      ports:
      - containerPort: 80
      livenessProbe:
        httpGet:
          path: /health
          port: http
```

### TrueCharts
The [TrueCharts Community](https://truecharts.org) also provides a ready-to-use [Static Web Server Helm-Chart](https://truecharts.org/charts/stable/static-web-server/) that you can easily deploy in your Kubernetes.
