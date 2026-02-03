# Disable Symlinks

**`SWS`** *does not* follow symlinks by default for security reasons.

As a result, SWS will respond with a `403 Forbidden` status if a symlink is requested as well as it won't be shown in the directory listing if enabled.

This feature is enabled by default and can be controlled by the boolean `--disable-symlinks` option or the equivalent [SERVER_DISABLE_SYMLINKS](./../configuration/environment-variables.md#server_disable_symlinks) env.

Here is an example of how to enable symlinks following if wanted:

```sh
static-web-server \
    -p=8787 -d=./public -g=trace \
    --directory-listing \
    --disable-symlinks=false
```
