# Disable Symlinks

**`SWS`** does follow symlinks by default. However, it's possible to disable all symlinks (deny access) by preventing to following files or directories if any path name component is a symbolic link. This applies to direct requests (URL) or those using the directory listing.

As a result, SWS will respond with a `403 Forbidden` status if a symlink is requested or it won't be shown in the directory listing if enabled.

This feature is disabled by default and can be controlled by the boolean `--disable-symlinks` option or the equivalent [SERVER_DISABLE_SYMLINKS](./../configuration/environment-variables.md#server_disable_symlinks) env.

Here is an example of how to disable symlinks:

```sh
static-web-server \
    -p=8787 -d=./public -g=trace \
    --directory-listing \
    --disable-symlinks
```
