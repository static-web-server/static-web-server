# Pre-compressed files serving

**`SWS`** provides support to serve pre-compressed [`Gzip`](https://datatracker.ietf.org/doc/html/rfc1952), [`Brotli`](https://www.ietf.org/rfc/rfc7932.txt) and [`Zstandard` (zstd)](https://datatracker.ietf.org/doc/html/rfc8878) files directly from the disk.

SWS can look up existing pre-compressed file variants (`.gz`, `.br` or `.zst`) on disk and serve them directly.

The feature is disabled by default and can be controlled by the boolean `--compression-static` option or the equivalent [SERVER_COMPRESSION_STATIC](./../configuration/environment-variables.md#server_compression_static) env.

When the `compression-static` option is enabled and the pre-compressed file is found on the file system then it's served directly.
Otherwise, if the pre-compressed file is not found then SWS just continues the normal workflow (trying to serve the original file requested instead). Additionally, if for example the [compression](../features/compression.md) option was also enabled then the requested file can be compressed on the fly right after.

!!! info "Compressed file type"
    The pre-compressed file type is determined by the [`Accept-Encoding`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header value.

Here is an example:

```sh
static-web-server -p=8787 -d=/var/www --compression-static=true -g=trace
```

Below are some relevant log entries to show how the feature works.

```log
2022-09-22T21:30:12.904102Z  INFO static_web_server::handler: incoming request: method=GET uri=/downloads/Capture5.png
2022-09-22T21:30:12.904218Z TRACE static_web_server::static_files: dir: base="/var/www", route="downloads/Capture5.png"
2022-09-22T21:30:12.904295Z TRACE static_web_server::compression_static: preparing pre-compressed file path variant of /var/www/downloads/Capture5.png
2022-09-22T21:30:12.904509Z TRACE static_web_server::compression_static: getting metadata for pre-compressed file variant /var/www/downloads/Capture5.png.gz
2022-09-22T21:30:12.904746Z TRACE hyper::proto::h1::conn: flushed({role=server}): State { reading: KeepAlive, writing: Init, keep_alive: Busy }
2022-09-22T21:30:12.904932Z TRACE static_web_server::static_files: file found: "/var/www/downloads/Capture5.png.gz"
2022-09-22T21:30:12.904983Z TRACE static_web_server::compression_static: pre-compressed file variant found, serving it directly
2022-09-22T21:30:12.905095Z TRACE hyper::proto::h1::conn: flushed({role=server}): State { reading: KeepAlive, writing: Init, keep_alive: Busy }
2022-09-22T21:30:12.905836Z TRACE encode_headers: hyper::proto::h1::role: Server::encode status=200, body=Some(Unknown), req_method=Some(GET)
2022-09-22T21:30:12.905965Z TRACE encode_headers: hyper::proto::h1::role: close time.busy=138µs time.idle=35.4µs
2022-09-22T21:30:12.906236Z DEBUG hyper::proto::h1::io: flushed 242 bytes
```
