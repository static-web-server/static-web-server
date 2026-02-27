# Metrics

SWS provides an optional `/metrics` endpoint that exposes Prometheus-compatible metrics about HTTP traffic. Useful for monitoring request rates, latency, error rates, and connection counts.

This feature is disabled by default and can be controlled by the boolean `--metrics` option or the equivalent [SERVER_METRICS](../configuration/environment-variables.md#server_metrics) env.

## Exposed metrics

| Metric | Type | Labels | Description |
| -- | -- | -- | -- |
| `sws_http_requests_total` | Counter | method, status, host | Total requests by method, status class (2xx, 4xx, etc.), and Host header |
| `sws_http_request_duration_seconds` | Histogram | method, status, host | Request latency with buckets from 50µs to 10s |
| `sws_http_response_bytes_total` | Counter | method, status, host | Total response bytes (from Content-Length) |
| `sws_http_requests_inflight` | Gauge | — | Requests currently being processed |
| `sws_http_connections_active` | Gauge | — | Active HTTP connections |

When built with the `experimental` feature and `RUSTFLAGS="--cfg tokio_unstable"`, Tokio runtime metrics (worker threads, task scheduling, etc.) are also included in the output.

## Example

```sh
static-web-server --root /var/www --metrics
```

```sh
curl http://localhost/metrics
```

```text
# HELP sws_http_requests_total Total HTTP requests by method, status class, and host.
# TYPE sws_http_requests_total counter
sws_http_requests_total{host="localhost",method="GET",status="2xx"} 42
# HELP sws_http_request_duration_seconds HTTP request duration in seconds by method, status class, and host.
# TYPE sws_http_request_duration_seconds histogram
sws_http_request_duration_seconds_bucket{host="localhost",method="GET",status="2xx",le="0.001"} 38
...
# HELP sws_http_connections_active Number of currently active HTTP connections.
# TYPE sws_http_connections_active gauge
sws_http_connections_active 2
```

## Grafana dashboard

An example Grafana dashboard is included in [`contrib/grafana/dashboard.json`](https://github.com/static-web-server/static-web-server/blob/master/contrib/grafana/dashboard.json). Import it into Grafana to get panels for request rates, latency histograms, error rates, and active connections out of the box.

## TOML configuration

```toml
[general]
metrics = true
```
