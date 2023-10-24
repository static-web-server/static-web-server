# Worker Threads Customization

**SWS** allows customizing the number of worker threads powered by the [Tokio](https://tokio.rs/) runtime.

See [Tokio ` worker_threads` API](https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html#method.worker_threads).

This feature can be controlled by the numeric `-n, --threads-multiplier` option or the equivalent [SERVER_THREADS_MULTIPLIER](./../configuration/environment-variables.md#server_threads_multiplier) env.

## Worker threads multiplier

The value of `-n, --threads-multiplier` works as multiplier digits to determine the number of worker threads used by the server.

Multiplying this input number by the number of system CPUs.

The formula used is the following:

> worker threads = number of CPUs * n

*Where `n` is the input value of `-n, --threads-multiplier`.*

**For example:** If there are `4` available CPUs and the `--threads-multiplier` is `8` then the total of *worker threads* to use will be `32`.

!!! tip "Tip"
    When the `--threads-multiplier` input value is `0` or `1` then one thread per core is used (default value).

!!! info "WebAssembly"
    We use `2` threads per core in [Wasm](https://webassembly.org/) and `1` in native environments by default.

!!! warn "Warn"
    The number of worker threads resulted should be a number between `1` and `32,768` though it is advised to keep this value on the smaller side. See [Tokio ` worker_threads` API](https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html#method.worker_threads) for more details.

Below is an example of how to adjust the number of worker threads.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    # NOTE: "8" gets multiplied by the number of the available cores.
    --threads-multiplier 8
```
