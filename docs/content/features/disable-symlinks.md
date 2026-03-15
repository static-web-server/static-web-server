# Disable Symlinks

**`SWS`** *does not* follow symlinks by default for security reasons.

As a result, SWS will respond with a `403 Forbidden` status if a symlink is requested as well as it won't be shown in the directory listing if enabled.

This feature is enabled by default and can be controlled by the boolean `--disable-symlinks` option or the equivalent [SERVER_DISABLE_SYMLINKS](./../configuration/environment-variables.md#server_disable_symlinks) env.

!!! info "Possible performance impact for large paths"

    Take into account that when `disable-symlinks` is enabled (`--disable-symlinks=true` by default in SWS), the server will check if the whole requested path is a symlink or contains intermediate symlink components.
    This involves filesystem access (syscall) for verification of each path component. Therefore, depending on the depth of the path, it may require multiple file system calls.

    If this feature results in a _noticeable_ negative performance impact (e.g., large paths), then you could consider turning the feature off. If doing so, remember that SWS will allow symlinks in the webroot, but it won't follow them if they resolve outside of it, which is SWS's behaviour regardless of this feature.

Here is an example of how to enable symlinks following if wanted:

```sh
static-web-server \
    -p=8787 -d=./public -g=trace \
    --directory-listing \
    --disable-symlinks=false
```
