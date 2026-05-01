# HTTP/2

**`SWS`** provides [HTTP/2](https://en.wikipedia.org/wiki/HTTP/2) protocol support.

This feature is disabled by default and can be activated via the boolean `--http2` option. HTTP/2 requires [TLS](./tls.md) to be enabled; pass `--tls`, `--tls-cert` and `--tls-key` alongside `--http2`.

!!! info "Tips"

    - `--http2` requires TLS. Always pass `--tls --tls-cert <path> --tls-key <path>` together with `--http2`.
    - When HTTP/2 is enabled, [Security Headers](./security-headers.md) are also enabled automatically (via TLS).
    - See the [TLS](./tls.md) page for supported key formats and cipher suite defaults.
    - The server provides [Termination Signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling with [Graceful Shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) ability by default.

## Example

Below is an example of how to run the server with HTTP/2 over TLS.

```sh
static-web-server \
    --host 127.0.0.1 \
    --port 8787 \
    --root ./my-public-dir \
    --tls \
    --tls-cert ./my-tls.cert \
    --tls-key ./my-tls.key \
    --http2
```
