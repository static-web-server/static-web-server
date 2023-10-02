# File Descriptor Socket Passing

**SWS** provides the ability to accept a socket listener as a file descriptor for use in sandboxing and on-demand applications via `systemd` (Linux), `launchd` (Macos) or similar.

!!! tip "Tip"
    The [Socket Activation](http://0pointer.de/blog/projects/socket-activation.html) model is an alternative to TCP port binding.

Socket activation is supported by the `-f, --fd` option or the equivalent [SERVER_LISTEN_FD](./../configuration/environment-variables.md#server_listen_fd) env.

If you are using `inetd`, its "`wait`" option should be used in conjunction with static-web-server's `--fd 0`
option.

## Systemd

If you're using `systemd` on Linux, there is a fully working example in the SWS Git repository under the [.`/systemd`](https://github.com/static-web-server/static-web-server/tree/master/systemd) directory.

### Service example

Below is a `systemd` service example. Follow the steps to create an SWS service using HTTP2 (`static-web-server.service`).
The service will bind SWS to a TCP `443` privileged port without running the server as root.

If you want to change the server port used by the service, edit the value of `ListenStream` in the `static-web-server.socket` file.

The template files can be found in [.`/systemd`](https://github.com/static-web-server/static-web-server/tree/master/systemd) directory.

```sh
# 1. Copy environment file template
#    Use an environment variable file, add/modify the values if necessary and
#    assign the proper owner/permissions to the environment variable file.
#    TIP: you could skip this step and use a config file if you prefer.
cp systemd/etc_default_static-web-server /etc/default/static-web-server

# TIP: For example, you could create a `nologin` user with specific privileges.

# 2. Copy service file templates
cp systemd/static-web-server.s* /etc/systemd/system/

# 3. Make sure that the `EnvironmentFile` and `ExecStart` values
#    of the service match to the corresponding file paths in the `static-web-server.service` file.
#    TIP: Use absolute paths. 
# EnvironmentFile=/etc/default/static-web-server
# ExecStart=/usr/local/bin/static-web-server --fd 0

# 4. Make sure to change this value with an existing user editing the `static-web-server.service` file.
# SupplementaryGroups=www-data

# 5. Create/reuse a directory for placing the certificate and private key.
#    TIP: this is an example, you can create/reuse your own dirs.
sudo mkdir /etc/static-web-server

# 6. For example purposes, copy the testing cert/key files.
#    TIP: Use your own cert/key files instead.
sudo cp tests/tls/local.dev_cert.ecc.pem /etc/static-web-server/
sudo cp tests/tls/local.dev_key.ecc.pem /etc/static-web-server/

# 7. Create/reuse a root directory (example only)
sudo mkdir -p /var/www/html
sudo sh -c 'echo "<h1>Static Web Server is running!</h1>" > /var/www/html/index.html'

# 8. Reload systemd manager configuration
sudo systemctl daemon-reload

# 9. Start the SWS service
sudo systemctl start static-web-server.service

# 10. Show the status of the SWS service running
sudo systemctl status static-web-server.service

# 11. Enable the service to start automatically at boot (optional)
sudo systemctl enable static-web-server.service

# 12. Analyze and debug the SWS service security
sudo systemd-analyze security static-web-server.service
#    If the service was successfully created then you should get something like:
#    → Overall exposure level for static-web-server.service: 0.6 SAFE 😀
```

## Testing

Alternatively, the lightweight [`systemfd`](https://github.com/mitsuhiko/systemfd) utility may be useful, especially for testing purposes.

For example, using `systemfd` utility as follows:

```sh
sudo systemfd --no-pid -s http::8091 -- path/to/static-web-server --fd 0
```

Or if you want to test using an environment variables file then you could use [Enve](https://github.com/joseluisq/enve).

```sh
sudo enve -f /path/to/environment.env systemfd --no-pid -s http::443 -- path/to/static-web-server --fd 0
```
