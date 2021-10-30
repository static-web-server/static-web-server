# Migration from v1 to v2

The `v2` introduces notable changes including new features, performance improvements and new targets support like ARM64 and OSes like FreeBSD.

This version `v2` was re-written almost from scratch on top of [Hyper](https://github.com/hyperium/hyper) and [Tokio](https://github.com/tokio-rs/tokio) runtime which give us the [Rust asynchronous ability](https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html) by default and latest HTTP/1 - HTTP/2 implementation improvements.
However it still try to keep the same principles of its `v1`: lightness and easy to use. Therefore a migration should not be a big deal.

## v2 breaking changes

This major `v2` has few breaking changes. However a migration should not represent a problem.


!!! tip "Tip"
    It is always worth recommending that you test a major server version upgrade like this first with your application(s) in a development environment or similar.

Please have in mind the following changes in `v2`:

- The server now supports only a root directory path (via `--root` or its equivalent env) so an assets path option is no longer required.
- Cache control headers are applied to assets in an arbitrary manner. See [control headers examples](./examples/cache-control-headers.md) for more details.
- OpenSSL TLS for HTTP/1 is not longer supported instead for the HTTP/2 & TLS (via `--http2` option) the server uses [h2](https://github.com/hyperium/h2) which is on top of [Rustls](https://github.com/ctz/rustls). It means that instead of using a `.p12` or `.pfx` file you can now use only a certificate file along with its private key. See [HTTP/2 & TLS examples](./examples/http2-tls.md) for more details.

The rest of known options are equivalent to `v1` (except the new ones of course).
