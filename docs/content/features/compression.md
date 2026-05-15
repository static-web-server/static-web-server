# Compression

**`SWS`** provides [`Gzip`](https://datatracker.ietf.org/doc/html/rfc1952), [`Deflate`](https://datatracker.ietf.org/doc/html/rfc1951#section-Abstract), [`Brotli`](https://www.ietf.org/rfc/rfc7932.txt) and [`Zstandard` (zstd)](https://datatracker.ietf.org/doc/html/rfc8878) compression of HTTP responses.

This feature is enabled by default and can be controlled by the boolean `-x, --compression` option or the equivalent [SERVER_COMPRESSION](../configuration/environment-variables.md#server_compression) env.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --compression true
```

## Choice of compression algorithm

The compression algorithm is determined by the [`Accept-Encoding`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header and the compression support built into SWS. By default SWS builds with support for `Gzip`, `Deflate`, `Brotli` and `Zstandard` algorithms.

SWS honors the qualities specified by the client to choose the algorithm. In case of equal quality for several algorithms (or no qualities at all), the internal priority will be selected according to this list:

1. `Zstandard`
2. `Brotli`
3. `Gzip`
4. `Deflate`

## MIME types compressed

Compression is only applied to files with the MIME types listed below, indicating text and similarly well compressing formats.

- `text/*`
- Application types that are essentially text/structured data.
    - `application/csv`
    - `application/graphql`
    - `application/javascript`
    - `application/json`
    - `application/rtf`
    - `application/sql`
    - `application/x-yaml`
    - `application/xml`
    - `application/yaml`
- Binary types that are not text but considered compressible.
    - `application/wasm`
    - `application/font-sfnt`
    - `application/vnd.ms-fontobject`
    - `image/x-icon`
    - `image/vnd.microsoft.icon`

## Compression level

SWS allows selecting the compression level via `--compression-level` command line option or the equivalent [SERVER_COMPRESSION_LEVEL](../configuration/environment-variables.md#server_compression_level) env. The available values are `fastest`, `best` and `default`. `fastest` will result in the lowest CPU load but also the worst compression factor. `best` will attempt to compress the data as much as possible (not recommended with `Brotli` or `Zstandard` compression, will be very slow). `default` tries to strike a balance, choosing a compression level where compression factor is already fairly good but the CPU load is still low.
