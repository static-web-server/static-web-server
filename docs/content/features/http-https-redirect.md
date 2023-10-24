# HTTP to HTTPS redirect

**`SWS`** provides support for redirecting HTTP requests to HTTPS via a [301 Moved Permanently](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/301) redirect status response code.

This feature is disabled by default and can be controlled by the boolean `--https-redirect` option or the equivalent [SERVER_HTTPS_REDIRECT](./../configuration/environment-variables.md#server_https_redirect) env.

!!! info "HTTP/2 required"
    HTTPS redirect requires the [HTTP/2](../features/http2-tls.md) feature to be activated.

## HTTPS redirect
The boolean `--https-redirect` is the main option and controls the whole HTTPS redirect feature. If `true` then will tell SWS to redirect all requests with scheme `http` to `https` for the current server instance with a `301 Moved Permanently` redirect status response code. 
This option depends on [`http2`](../features/http2-tls.md) to be enabled.

## HTTPS redirect host
The string `--https-redirect-host` option represents the canonical hostname or IP of the HTTPS (HTTPS/2) server. This is usually associated with the `--host` option, however here this value will be used as the destination for the redirected requests.
It depends on "https-redirect" option to be enabled. The default is `localhost`.

## HTTPS redirect from port
The string `--https-redirect-from-port` option represents the HTTP host port where the redirect server will listen for requests (source) to redirect them to HTTPS. It depends on "https-redirect" option to be enabled. The default is `80`.

## HTTPS redirect from hosts
The string `--https-redirect-from-hosts` option represents a list of hostnames or IPs allowed to redirect from using comma-separated values. Incoming HTTP requests must contain the [HTTP `Host` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Host) and match against this list. It depends on "https-redirect" option to be enabled. The default value is `localhost`.

!!! tip "Tip: define hostnames/IPs to redirect from for increasing security"
    - Via the `--https-redirect-from-hosts` or its env you can tell SWS which hostnames or IPs are allowed to redirect from your SWS server instance to avoid potential spoofing issues.
    - When a hostname or IP is not found in the whitelist then SWS will respond with a `400 Bad Request` status response.

## Example

Below is an example of the feature.

```sh
static-web-server -p 4433 -d public/ -g trace \
    # HTTP/2 + TLS options
    --http2=true \
    --http2-tls-cert=tests/tls/local.dev_cert.ecc.pem \
    --http2-tls-key=tests/tls/local.dev_key.ecc.pem \
\
    # HTTPS redirect options
    --https-redirect=true \
    --https-redirect-host="localhost" \
    --https-redirect-from-port=80 \
    --https-redirect-from-hosts="localhost"
    # or using multiple hostnames/IPs:
    # --https-redirect-from-hosts = "localhost,127.0.0.1"
```

After running the server, the logs should look as follows.

```log
.......
2023-06-01T22:30:17.555338Z  INFO static_web_server::server: http to https redirect: enabled=true
2023-06-01T22:30:17.555349Z  INFO static_web_server::server: http to https redirect host: localhost
2023-06-01T22:30:17.555359Z  INFO static_web_server::server: http to https redirect from port: 80
2023-06-01T22:30:17.555368Z  INFO static_web_server::server: http to https redirect from hosts: localhost
2023-06-01T22:30:17.557507Z  INFO Server::start_server{addr_str="[::]:4433" threads=8}: static_web_server::server: close time.busy=0.00ns time.idle=3.00µs
2023-06-01T22:30:17.557547Z  INFO static_web_server::server: http2 server is listening on https://[::]:4433
2023-06-01T22:30:17.557583Z  INFO Server::start_server{addr=[::]:80 threads=8}: static_web_server::server: close time.busy=0.00ns time.idle=1.92µs
2023-06-01T22:30:17.557596Z  INFO static_web_server::server: http1 redirect server is listening on http://[::]:80
2023-06-01T22:30:17.557768Z  INFO static_web_server::server: press ctrl+c to shut down the servers
```
