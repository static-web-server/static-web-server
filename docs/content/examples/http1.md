# HTTP/1

The HTTP/1 is the protocol by default and can be used specifying a host address via the `-a, --host` ([SERVER_HOST](./../configuration/environment-variables.md#server_host)) argument, the host port via `-p, --port` ([SERVER_PORT](./../configuration/environment-variables.md#server_port)) and the directory of the static files via `-d, --root` ([SERVER_ROOT](./../configuration/environment-variables.md#server_root)) argument.

!!! info "Tip"
    Note that either `--host`, `--port` and `--root` have defaults (optional values) so they can be specified or omitted as required.

Below an example of how to run the server using HTTP/1.

```sh
static-web-server \
    --host 127.0.0.1 \
    --port 8787 \
    --root ./my-public-dir
```
