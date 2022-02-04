# HTTP/2 and TLS

**`SWS`** provides [HTTP/2](https://en.wikipedia.org/wiki/HTTP/2) protocol and [TLS](https://en.wikipedia.org/wiki/Transport_Layer_Security) support.

This feature is disabled by default and can be activated via the boolean `-t, --http2` option as well as string arguments `--http2-tls-cert` (TLS certificate file path) and `--http2-tls-key` (private key file path).

!!! info "Tips"
    - Either `--host`, `--port` and `--root` have defaults (optional values) so they can be specified or omitted as required.
    - Don't forget to adjust the proper `--port` value for the HTTP/2 & TLS feature.
    - When this feature is enabled (`--http2=true`) then the [security headers](./security-headers.md) are also enabled automatically.
    - The server provides [Termination Signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling with [Graceful Shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) ability by default.

```sh
static-web-server \
    --host 127.0.0.1 \
    --port 8787 \
    --root ./my-public-dir \
    --http2 true \
    --http2-tls-cert ./my-tls.cert \
    --http2-tls-key ./my-tls.key
```
