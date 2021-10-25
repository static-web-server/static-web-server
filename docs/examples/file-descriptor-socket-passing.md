# File Descriptor Socket Passing

A **systemd** unit files example for [socket activation](http://0pointer.de/blog/projects/socket-activation.html) can be found under the [`systemd/`](https://github.com/joseluisq/static-web-server/tree/master/systemd) directory of the Git repository.
If you are using `inetd`, its "`wait`" option should be used in conjunction with static-web-server's `--fd 0`
option.

Alternatively, the light-weight [`systemfd`](https://github.com/mitsuhiko/systemfd) utility may be
useful. Especially for testing.

For example using `systemfd` utility as follow:

```sh
systemfd --no-pid -s http::8091 -- path/to/static-web-server --fd 0
```
