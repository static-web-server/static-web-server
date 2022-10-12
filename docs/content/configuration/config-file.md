# TOML Configuration File

**`SWS`** can be configured using a [TOML](https://toml.io/en/) file to adjust the general server features as well as other advanced ones.

It's disabled by default and can be enabled by passing a _string file path_ via the `-w, --config-file` option or its equivalent [SERVER_CONFIG_FILE](./../configuration/environment-variables.md#server_config_file) env.

## TOML File (Manifest)

Below is just an example showing all features with their default values.

```toml
[general]

#### Address & Root dir
host = "::"
port = 80
root = "./public"

#### Logging
log-level = "error"

#### Cache Control headers
cache-control-headers = true

#### Auto Compression
compression = true

#### Error pages
page404 = "./public/404.html"
page50x = "./public/50x.html"

#### HTTP/2 + TLS
http2 = false
# http2-tls-cert = "some.cert"
# http2-tls-key = "some.key"

#### Security headers
security-headers = true

#### CORS
cors-allow-origins = ""
cors-allow-headers = ""

#### Directory listing
directory-listing = false

#### Directory listing sorting code
directory-listing-order = 6

#### Directory listing content format ("html" or "json")
directory-listing-format = "html"

#### Basich Authentication
basic-auth = ""

#### File descriptor binding
# fd = ""

#### Worker threads
threads-multiplier = 1

#### Grace period after a graceful shutdown
grace-period = 0

#### Page fallback for 404s
# page-fallback = "some_page.html"

#### Log request Remote Address if available
log-remote-address = false

#### Redirect to trailing slash in the requested directory uri
redirect-trailing-slash = true

#### Check for existing pre-compressed files
compression-static = false


### Windows Only

#### Windows Service support.
#### NOTE: this doesn't create a Windows Service per se,
#### instead, it just tells SWS to run in a Windows Service context,
#### so it's necessary to install the SWS Windows Service first
#### using the `static-web-server.exe -w config.toml install` command.
#### More details on https://sws.joseluisq.net/features/windows-service/
# windows-service = false


[advanced]

#### ....
```

### General options

The TOML `[general]` section allows adjusting the current options available via the CLI/ENV ones.

So they are equivalent to each other **except** for the `-w, --config-file` option which is omitted and can not be used for obvious reasons.

!!! info "Config file-based features are optional"
    All server feature options via the configuration file are optional and can be omitted as needed.

### Advanced options

The TOML `[advanced]` section is intended for more complex features.

For example [Custom HTTP Headers](../features/custom-http-headers.md) or [Custom URL Redirects](../features/url-redirects.md).

### Precedence

Whatever config file-based feature option will take precedence over its CLI or ENV equivalent.

## Usage

The following command runs the server using a specific `config.toml` file.

```sh
static-web-server -w config.toml
```
