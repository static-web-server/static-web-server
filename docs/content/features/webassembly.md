# WebAssembly

**SWS** can run in a [WebAssembly](https://webassembly.org/) context.

!!! info "Wasm/Wasix targets are not officially supported by SWS yet"
    We do not officially support Wasm or Wasix targets yet. But SWS project will eventually support Wasix as a target in a not remote future.<br>
    In the meantime, [Wasmer](https://wasmer.io/) folks made it possible to [run SWS via Wasix today](https://wasmer.io/posts/announcing-wasix) via a series of patches.

## Wasix

You can run SWS using [The Wasmer Runtime](https://wasmer.io/wasmer/static-web-server/) with [Wasix](https://wasix.org/). See the [wasmer/static-web-server](https://wasmer.io/wasmer/static-web-server) package.

To run SWS, make sure to install Wasmer first and then enable its `net` and `threads` features as well as map your host directory via the `mapdir` option before starting the server.

Here is an example.

```sh
wasmer run wasmer/static-web-server \
    --net --enable-threads --mapdir /public:/my/host/dir -- --port 8787
```

See [The WASIX with Axum Tutorial](https://wasix.org/docs/language-guide/rust/tutorials/wasix-axum) for more details.
