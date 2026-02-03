# Ignore files

**`SWS`** provides some options to ignore files or directories from being served and displayed if the directory listing is enabled.

## Ignore hidden files (dotfiles)

SWS *does ignore* dotfiles (hidden files) by default for security reasons.

As a result, SWS will respond with a `404 Not Found` status as well as hidden files won't be shown in the directory listing if enabled.

This feature is enabled by default and can be controlled by the boolean `--ignore-hidden-files` option or the equivalent [SERVER_IGNORE_HIDDEN_FILES](./../configuration/environment-variables.md#server_ignore_hidden_files) env.

Here is an example of how to disable hidden files ignoring if wanted:

```sh
static-web-server \
    -p=8787 -d=./public -g=trace \
    --directory-listing \
    --ignore-hidden-files=false
```
