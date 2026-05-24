# Include Hidden Files

**`SWS`** *does not* serve hidden files (dotfiles) by default for security reasons.

As a result, SWS will respond with a `404 Not Found` status for hidden files, and they will not be shown in the directory listing if enabled.

This behaviour is controlled by the boolean `--include-hidden` option (disabled by default) or the equivalent [SERVER_INCLUDE_HIDDEN](./../configuration/environment-variables.md#server_include_hidden) env.

Here is an example of how to enable serving of hidden files if wanted:

```sh
static-web-server \
    -p=8787 -d=./public -g=trace \
    --directory-listing \
    --include-hidden=true
```
