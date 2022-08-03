# Getting Started

[Download and install](./download-and-install.md) the binary for your specific platform and then type

```sh
static-web-server --port 8787 --root ./my-public-dir
```

Or if you use [Docker](https://www.docker.com/) just try

```sh
docker run --rm -it -p 8787:80 joseluisq/static-web-server:2
```

!!! info "Docker Tip"
    You can specify a Docker volume like `-v $HOME/my-public-dir:/public` to overwrite the default root directory. See [Docker examples](features/docker.md).

- Type `static-web-server --help` or see the [Command-line arguments](./configuration/command-line-arguments.md) section.
- See how to configure the server using a [configuration file](configuration/config-file.md).
- Have a look at [the features](./features/http1.md) section for more advanced examples.
