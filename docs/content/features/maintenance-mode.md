# Maintenance Mode

**`SWS`** provides a way to put a server into a maintenance mode. Allowing the server to respond with a custom HTTP status code and HTML content always by default.

This is useful to allow the server to be taken offline without disrupting the service.

The feature is disabled by default and can be controlled by the boolean `--maintenance-mode` option or the equivalent [SERVER_MAINTENANCE_MODE](./../configuration/environment-variables.md#server_maintenance_mode) env.

## How it works

When the feature is enabled, SWS will respond *always* with the specified (or default) status code and HTML content to every request ignoring all SWS features. Except the [Health check](./health-endpoint.md), [CORS](./cors.md) and [Basic Authentication](./basic-authentication.md) features.

## HTTP Status Code

The `--maintenance-mode-status` or the equivalent [SERVER_MAINTENANCE_MODE_STATUS](./../configuration/environment-variables.md#server_maintenance_mode_status) env variable can be used to tell SWS to reply with a specific status code.

When not specified, the server will reply with the `503 Service Unavailable` status.

## HTML Page

The `--maintenance-mode-file`  or the equivalent [SERVER_MAINTENANCE_MODE_FILE](./../configuration/environment-variables.md#server_maintenance_mode_file) env variable can be also used to customize the response content.

The value should be an existing local HTML file path. When not provided a generic message will be displayed.

!!! tip "Optional"
    Remember that either `--maintenance-mode-status` and `--maintenance-mode-file` are optional and can be omitted as needed.

!!! info "Independent path"
    The `--maintenance-mode-file` is an independent file path and not relative to the root.

## Example

For instance, the server will respond with a `503 Service Unavailable` status code and a custom message.

```sh
static-web-server -p 8787 -d ./public \
    --maintenance-mode \
    # optional status code, `503` by default
    --maintenance-mode-status=503 \
    # optional HTML page, generic message by default
    --maintenance-mode-file="./maintenance.html"
```
