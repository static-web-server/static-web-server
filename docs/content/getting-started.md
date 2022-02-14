# Getting Started

[Download](./download-and-install.md) the binary for your platform and then just type

```sh
static-web-server --port 8787 --root ./my-public-dir
```

Or if you use [Docker](https://www.docker.com/) just try

```sh
docker run --rm -it -p 8787:80 joseluisq/static-web-server:2
```

!!! info "Docker Tip"
    You can specify a Docker volume like `-v $HOME/my-public-dir:/public` in order to overwrite the default root directory. See [Docker examples](features/docker.md).

To see the available options type `static-web-server -h` or go to the [Command-line arguments](./configuration/command-line-arguments.md) section.

Or if you are looking for more advanced examples then have a look at [the features](./features/http1.md) section.
