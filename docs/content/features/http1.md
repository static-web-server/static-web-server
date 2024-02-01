# HTTP/1

The HTTP/1 is the protocol by default and can be used by specifying a host address via the `-a, --host` ([SERVER_HOST](./../configuration/environment-variables.md#server_host)) argument, the port of the host via `-p, --port` ([SERVER_PORT](./../configuration/environment-variables.md#server_port)) and the directory of the static files using the `-d, --root` ([SERVER_ROOT](./../configuration/environment-variables.md#server_root)) argument.

!!! info "Tips"
    - Either `--host`, `--port` and `--root` have defaults (optional values) so they can be specified or omitted as required.
    - The server provides [Termination Signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling with [Graceful Shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) ability by default.

Below is an example of how to run the server using HTTP/1.

```sh
static-web-server \
    --host 127.0.0.1 \
    --port 8787 \
    --root ./my-public-dir
```
