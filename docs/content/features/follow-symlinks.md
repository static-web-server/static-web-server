# Follow Symlinks

**`SWS`** *does not* follow symlinks by default for security reasons.

As a result, SWS will respond with a `403 Forbidden` status if a symlink is requested, and the symlink will not be shown in the directory listing if enabled.

This behaviour is controlled by the boolean `--follow-symlinks` option (disabled by default) or the equivalent [SERVER_FOLLOW_SYMLINKS](./../configuration/environment-variables.md#server_follow_symlinks) env.

!!! info "Possible performance impact for large paths"

    When `--follow-symlinks` is disabled (the default), the server checks whether the whole requested path is a symlink or contains intermediate symlink components.
    This involves filesystem access (syscall) for verification of each path component. Therefore, depending on the depth of the path, it may require multiple file system calls.

    If this check results in a _noticeable_ negative performance impact (e.g. very deep paths), you can enable `--follow-symlinks=true`. Remember that SWS will still refuse symlinks that resolve outside the webroot, regardless of this setting.

Here is an example of how to enable symlink following if wanted:

```sh
static-web-server \
    -p=8787 -d=./public -g=trace \
    --directory-listing \
    --follow-symlinks=true
```
