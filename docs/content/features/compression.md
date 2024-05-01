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

## MIME types compressed

Compression is only applied to files with the MIME types listed below, indicating text and similarly well compressing formats. The asterisk `*` is a placeholder indicating an arbitrary MIME type part.

```txt
text/*
*+xml
*+json
application/rtf
application/javascript
application/json
application/xml
font/ttf
application/font-sfnt
application/vnd.ms-fontobject
application/wasm
```

## Compression level

SWS allows selecting the compression level via `--compression-level` command line option or the equivalent [SERVER_COMPRESSION_LEVEL](../configuration/environment-variables.md#server_compression_level) env. The available values are `fastest`, `best` and `default`. `fastest` will result in the lowest CPU load but also the worst compression factor. `best` will attempt to compress the data as much as possible (not recommended with `Brotli` or `Zstandard` compression, will be very slow). `default` tries to strike a balance, choosing a compression level where compression factor is already fairly good but the CPU load is still low.
