# Static Web Server

> A fast web server to static files-serving powered by [Rust Iron](https://github.com/iron/iron). :zap:

**Static Web Server** is an small (`1,6M` static binary) and fast web server to serving static files. Which is also suitable to deploy it into a Docker container.

__Status:__ The status is WIP so feel free to contribute.

## Usage

```
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

## API

Server is configurated via environment variables:

- **SERVER_NAME**: Name for server
- **SERVER_HOST**: Host address (E.g 127.0.0.1). Default `[::]`
- **SERVER_PORT**: Host port
- **SERVER_ROOT**: Root directory path of static files
- **SERVER_ASSETS**: Assets directory path for add cache functionality.

## Contributions

Feel free to send some [Pull request](https://github.com/joseluisq/static-web-server/pulls) or [issue](https://github.com/joseluisq/static-web-server/issues).

## License
MIT license

© 2019 [Jose Quintana](https://git.io/joseluisq)
