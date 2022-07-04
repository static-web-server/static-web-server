# File Descriptor Socket Passing

**SWS** provides the ability to accept a socket listener as a file descriptor for use in sandboxing and on-demand applications via `systemd` (Linux), `launchd` (Macos) or similar.

!!! tip "Tip"
    The [Socket Activation](http://0pointer.de/blog/projects/socket-activation.html) model is an alternative to TCP port binding.

Socket activation is supported by the `-f, --fd` option or the equivalent [SERVER_LISTEN_FD](./../configuration/environment-variables.md#server_listen_fd) env.

If you are using `inetd`, its "`wait`" option should be used in conjunction with static-web-server's `--fd 0`
option.

## Systemd

If you're using `systemd` on Linux, there is a fully working example in the SWS Git repository under the [.`/systemd`](https://github.com/joseluisq/static-web-server/tree/master/systemd) directory.

## Testing

Alternatively, the lightweight [`systemfd`](https://github.com/mitsuhiko/systemfd) utility may be useful, especially for testing purposes.

For example, using `systemfd` utility as follow:

```sh
systemfd --no-pid -s http::8091 -- path/to/static-web-server --fd 0
```
