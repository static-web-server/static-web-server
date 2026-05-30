# Unix Domain Socket

**SWS** can bind to a [Unix Domain Socket](https://en.wikipedia.org/wiki/Unix_domain_socket) (UDS) instead of a TCP host and port. UDS is ideal for **reverse-proxy** setups where SWS and the front-end proxy (e.g. nginx, Caddy, HAProxy) run on the same host: connections bypass the TCP/IP stack entirely (no port allocation, no checksums, no Nagle), and access can be restricted via filesystem permissions.

This feature is **Unix-only** (Linux, macOS, FreeBSD, etc.) and is mutually exclusive with `--host`, `--port`, `--fd`, and TLS options. It serves HTTP/1 over the socket. TLS termination must be handled by the upstream proxy.

## Options

| CLI option | Environment variable | Description |
| -- | -- | -- |
| `--unix-socket <PATH>` | [`SERVER_UNIX_SOCKET`](../configuration/environment-variables.md#server_unix_socket) | Filesystem path to bind the socket (e.g. `/run/sws.sock`). |
| `--unix-socket-mode <OCTAL>` | [`SERVER_UNIX_SOCKET_MODE`](../configuration/environment-variables.md#server_unix_socket_mode) | Permission bits in octal (`660`, `0660`, `0o660`). Default: process umask. |
| `--unix-socket-force` | [`SERVER_UNIX_SOCKET_FORCE`](../configuration/environment-variables.md#server_unix_socket_force) | Remove a stale socket file before binding (socket-type files only; refuses to clobber regular files or directories). |

!!! info "Socket cleanup"

    The socket file is automatically removed on graceful shutdown. If the server is killed abruptly (`SIGKILL`), use `--unix-socket-force` on restart to clean up the stale socket.

## Example

Bind SWS to a UDS at `/run/sws.sock` with `rw-rw----` (660) permissions:

```sh
static-web-server \
    --unix-socket /run/sws.sock \
    --unix-socket-mode 660 \
    --root ./my-public-dir
```

Then point your reverse proxy at the socket. For nginx:

```nginx
upstream sws_backend {
    server unix:/run/sws.sock;
}

server {
    listen 443 ssl;
    server_name example.com;

    location / {
        proxy_pass http://sws_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

!!! note

    Because UDS peers lack an IP address, SWS reports no remote address for connections arriving over the socket. Use the proxy's `X-Real-IP` or `X-Forwarded-For` headers to preserve the original client address.

## TOML configuration

```toml
[general]
unix-socket = "/run/sws.sock"
unix-socket-mode = 660
unix-socket-force = true
```
