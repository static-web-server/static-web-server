# Logging

**`SWS`** provides logging support just specifying a log level in lower case. The values allowed are `error`, `warn`, `info`, `debug` and `trace`. The default value is `error`.

This feature is enabled by default and can be controlled by the string `-g, --log-level` option or the equivalent [SERVER_LOG_LEVEL](./../configuration/environment-variables.md#server_log_level) env.

Below an example of how to adjust the log level.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --log-level "trace"
```
