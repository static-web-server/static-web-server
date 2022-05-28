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

## Log Remote Addresses

SWS provides **Remote Address (IP)** logging for every request via an `INFO` log level.

This feature is disabled by default and can be enabled by the boolean `--log-remote-address` option or the equivalent [SERVER_LOG_REMOTE_ADDRESS](./../configuration/environment-variables.md#server_log_remote_address) env.

If the feature is enabled then log entries for requests will contain a `remote_addr` section with the remote address (IP) value. Otherwise it will be empty.

Log entry example:

```log
2022-05-23T22:24:50.519540Z  INFO static_web_server::handler: incoming request: method=GET uri=/ remote_addr=192.168.1.126:57625
```

Below an example of how to enable Remote Address (IP) logging. Note the last two entries.

```sh
static-web-server -a "0.0.0.0" -p 8080 -d docker/public/ -g info --log-remote-address=true
# 2022-05-23T22:24:44.523057Z  INFO static_web_server::logger: logging level: info
# 2022-05-23T22:24:44.523856Z  INFO static_web_server::server: server bound to TCP socket 0.0.0.0:8080
# 2022-05-23T22:24:44.523962Z  INFO static_web_server::server: runtime worker threads: 4
# 2022-05-23T22:24:44.523989Z  INFO static_web_server::server: security headers: enabled=false
# 2022-05-23T22:24:44.524006Z  INFO static_web_server::server: auto compression: enabled=true
# 2022-05-23T22:24:44.524061Z  INFO static_web_server::server: directory listing: enabled=false
# 2022-05-23T22:24:44.524097Z  INFO static_web_server::server: directory listing order code: 6
# 2022-05-23T22:24:44.524133Z  INFO static_web_server::server: cache control headers: enabled=true
# 2022-05-23T22:24:44.524191Z  INFO static_web_server::server: basic authentication: enabled=false
# 2022-05-23T22:24:44.524210Z  INFO static_web_server::server: grace period before graceful shutdown: 0s
# 2022-05-23T22:24:44.524527Z  INFO Server::start_server{addr_str="0.0.0.0:8080" threads=4}: static_web_server::server: close time.busy=0.00ns time.idle=10.6µs
# 2022-05-23T22:24:44.524585Z  INFO static_web_server::server: listening on http://0.0.0.0:8080
# 2022-05-23T22:24:44.524614Z  INFO static_web_server::server: press ctrl+c to shut down the server
# 2022-05-23T22:24:50.519540Z  INFO static_web_server::handler: incoming request: method=GET uri=/ remote_addr=192.168.1.126:57625
# 2022-05-23T22:25:26.516841Z  INFO static_web_server::handler: incoming request: method=GET uri=/favicon.ico remote_addr=192.168.1.126:57625
```
