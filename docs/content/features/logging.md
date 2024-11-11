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

Below is an example of how to enable Remote Address (IP) logging.

```sh
static-web-server -a "0.0.0.0" -p 8080 -d docker/public/ -g info --log-remote-address=true
```

The relevant log output:
```log
INFO static_web_server::logger: logging level: info
<...>
INFO static_web_server::info: log requests with remote IP addresses: enabled=true
<...>
INFO static_web_server::handler: incoming request: method=GET uri=/ remote_addr=192.168.1.126:57625
INFO static_web_server::handler: incoming request: method=GET uri=/favicon.ico remote_addr=192.168.1.126:57625
```
## Log Real Remote IP

When used behind a reverse proxy the reported `remote_addr` indicates the proxies IP address and port, not the clients real IP.
The Proxy server can be configured to provide the [X-Forwarded-For header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For), containing a comma-separated list of IP addresses, starting with the *real remote client IP*, and all following intermediate proxies (if any).


To enable logging of the real remote IP, enable the `--log-forwarded-for` option or the equivalent [SERVER_LOG_FORWARDED_FOR](/docs/content/configuration/environment-variables.md#serverlogforwardedfor) env. By default this will log all requests which have a correctly formatted `X-Forwarded-For` header. 

Since the content of the `X-Forwarded-For` header can be changed by all proxies in the chain, the remote IP address reported may not be trusted.

To restrict the logging to only trusted proxy IPs, you can use the `--trusted-proxies` option, or the equivalent [SERVER_TRUSTED_PROXIES](/docs/content/configuration/environment-variables.md#servertrustedproxies) env. This should be a list of IPs, separated by commas. An empty list (the default) indicates that all IPs should be trusted.

Command used for the following examples:
```sh
static-web-server -a "::" --log-forwarded-for=true --trusted-proxies="::1" -p 8080 -d docker/public/ -g info
```

Look for these lines in the log output:
```log
<...>
INFO static_web_server::info: log level: info
INFO static_web_server::info: log requests with remote IP addresses: enabled=false
INFO static_web_server::info: log X-Forwarded-For real remote IP addresses: enabled=true
INFO static_web_server::info: trusted IPs for X-Forwarded-For: [::1]
<...>
```

We can simulate request as from behind reverse proxy with additional intermediate-proxy with following command:

```sh
curl "http://[::1]:8080" --header "X-Forwarded-For: 203.0.113.195, 2001:db8:85a3:8d3:1319:8a2e:370:7348"
```

Log entry for this request will look like:

```log
INFO static_web_server::handler: incoming request: method=GET uri=/ real_remote_ip=203.0.113.195
```

---

If we send the request from `127.0.0.1` instead:
```sh
curl "http://127.0.0.1:8080" --header "X-Forwarded-For: 203.0.113.195, 2001:db8:85a3:8d3:1319:8a2e:370:7348"
```

we get the following log output:
```log
INFO static_web_server::handler: incoming request: method=GET uri=/
```
`127.0.0.1` is not in the `trusted_proxies`, so we dont get a `real_remote_address` in the log.

Note the absence of the proxies remote address in these examples. If you want to log the remote address and the real remote address, you need to specify both `--log-remote-address` and `--log-forwarded-for`.

---

**`SWS`** will parse the `X-Forwarded-For` header and if the provided client IP is invalid, it will be ignored to prevent log poisoning attacks. In such cases the `real_remote_ip` section will not be added.

Example from above, but with invalid header:

```sh
curl "http://[::1]:8080" --header "X-Forwarded-For: <iframe src=//malware.attack>"
```

```log
2022-05-23T22:24:50.519540Z  INFO static_web_server::handler: incoming request: method=GET uri=/
```
