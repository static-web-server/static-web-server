# Ignore files

**`SWS`** provides some options to ignore files or directories from being served and displayed if the directory listing is enabled. 

## Ignore hidden files (dotfiles)

SWS doesn't ignore dotfiles (hidden files) by default.
However, it's possible to ignore those files as shown below. As a result, SWS will respond with a `404 Not Found` status.

This feature is disabled by default and can be controlled by the boolean `--ignore-hidden-files` option or the equivalent [SERVER_IGNORE_HIDDEN_FILES](./../configuration/environment-variables.md#server_ignore_hidden_files) env.

Here is an example of how to ignore hidden files:

```sh
static-web-server \
    -p=8787 -d=tests/fixtures/public -g=trace \
    --directory-listing=true \
    --ignore-hidden-files true
```
