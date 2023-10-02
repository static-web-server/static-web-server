# Graceful Shutdown

**SWS** can terminate gracefully in what is known as a [graceful shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) process.

It means that when a `SIGTERM` [termination signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) is caught the server will stop receiving more requests immediately but in turn, it will continue processing all existing requests until they are completed (or closed).

!!! tip "Tips"
    - In **BSD/Unix-like** systems, SWS will start the graceful shutdown process when a `SIGTERM`, `SIGINT` or `SIGQUIT` [termination signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) is caught.
    - In **Windows** systems otherwise, SWS will start the graceful shutdown process right after a <kbd>CTRL + C</kbd>. This is used to abort the current task.

## Grace Period

Sometimes one wants to control the graceful shutdown process for different reasons. For example during [Kubernetes rollouts](https://github.com/static-web-server/static-web-server/issues/79).

In these situations, SWS allows delaying the graceful shutdown process right after a `SIGTERM` providing a *grace period* in seconds.

The feature is disabled by default and can be controlled by the numeric `-q, --grace-period` option or its equivalent [SERVER_GRACE_PERIOD](./../configuration/environment-variables.md#server_grace_period) env.

!!! tip "Tip"
    The maximum grace period value is `255` seconds (4.25 min). The default value is `0` (no delay).

Here is an example of delaying the graceful shutdown process by `10` seconds after a `SIGTERM`.

```sh
static-web-server -p 8787 -d ./public/ -g trace --grace-period 10
```
