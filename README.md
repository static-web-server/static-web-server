# static-web-server

> Web Server to static file-serving. 

## Usage

```
Static Web Server
Web Server to static file-serving.

Please use `make <target>` where <target> is one of:
    install           to install dependencies.
    run               to run server in development.
    watch             to run server (watch files mode) in development.
    release           to build a release.
    docker_image      to build a Docker image.
```

## API

Server is configurated via environment variables:

- __SERVER_NAME__: Name for server
- __SERVER_HOST__: Host address (E.g 127.0.0.1). Default `[::]`
- __SERVER_PORT__: Host port
- __SERVER_ROOT__: Root directory path of static files
- __SERVER_ASSETS__: Assets directory path for add cache functionality.
