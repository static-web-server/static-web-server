# Logging

**`SWS`** provides logging support by just specifying a log level in lower case. The values allowed are `error`, `warn`, `info`, `debug` and `trace`. The default value is `error`.

This feature is enabled by default and can be controlled by the string `-g, --log-level` option or the equivalent [SERVER_LOG_LEVEL](./../configuration/environment-variables.md#server_log_level) env.

Below is an example of how to adjust the log level.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --log-level "trace"
```

## Log Remote Addresses

SWS provides *Remote Address (IP)* logging for every request via an `INFO` log level.

This feature is disabled by default and can be enabled by the boolean `--log-remote-address` option or the equivalent [SERVER_LOG_REMOTE_ADDRESS](./../configuration/environment-variables.md#server_log_remote_address) env.

If the feature is enabled then log entries for requests will contain a `remote_addr` section with the remote address (IP) value. Otherwise, it will be empty.

Log entry example:

```log
2022-05-23T22:24:50.519540Z  INFO static_web_server::handler: incoming request: method=GET uri=/ remote_addr=192.168.1.126:57625
```

Below is an example of how to enable Remote Address (IP) logging. Note the last two entries.

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
## Log Real Remote IP

When used behind reverse proxy, reported `remote_addr` indicate proxy internal IP address and port, and not client real remote IP.
Proxy server can be configured to provide [X-Forwarded-For header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For), containing comma-separated list of IP addresses, starting with *client real remote IP*, and all following intermediate proxies (if any).

When *Remote Address (IP) logging* [is enabled](#log-remote-addresses), and `X-Forwarded-For` header is present and correctly formated, then log entries for requests will contain a `real_remote_ip` section with IP of remote client, **as reported by this header**. 

We can simulate request as from behind reverse proxy with additional intermediate-proxy with following command:

```sh
curl --header "X-Forwarded-For: 203.0.113.195, 2001:db8:85a3:8d3:1319:8a2e:370:7348" http://0.0.0.0:8080
```

Log entry for such case will look like:

```log
2022-05-23T22:24:50.519540Z  INFO static_web_server::handler: incoming request: method=GET uri=/ remote_addr=192.168.1.126:57625 real_remote_ip=203.0.113.195
```

**`SWS`** will parse `X-Forwarded-For` header, and if format of provided IP is invalid - it will be ignored to prevent log poisoning attacks. In such case `real_remote_ip` section will not be added.

Example from above, but with invalid header:

```sh
curl --header "X-Forwarded-For: <iframe src=//malware.attack>" http://0.0.0.0:8080
```

```log
2022-05-23T22:24:50.519540Z  INFO static_web_server::handler: incoming request: method=GET uri=/ remote_addr=192.168.1.126:57625
```

Be aware, that contents of `X-Forwarded-For` header can be augumented by all proxies in the chain, and as such - remote IP address reported by it may not be trusted.