# Blocking Threads Customization

**SWS** allows limiting the number of blocking threads powered by the [Tokio](https://tokio.rs/) runtime.

This feature can be controlled by the numeric `-b, --max-blocking-threads` option or the equivalent [SERVER_MAX_BLOCKING_THREADS](./../configuration/environment-variables.md#server_max_blocking_threads) env.

!!! info "WebAssembly"
    We use `20` in [Wasm](https://webassembly.org/) by default and `512` in native environments (Tokio's default). See [Tokio ` max_blocking_threads` API](https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html#method.max_blocking_threads) for more details.

Below is an example.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --max-blocking-threads 20
```
