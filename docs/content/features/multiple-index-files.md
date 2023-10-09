# Multiple index files

**`SWS`** allows to provide a list of files that will be used as an index for requests ending with the slash character (‘/’).

!!! info "Notes"
    - Files are checked in the specified order from left to right.
    - The option value can be a single index or comma-separated when multiple values.
    - The default value is `index.html`.

This feature is disabled by default and can be controlled by the string list `--index-files` option or the equivalent [SERVER_INDEX_FILES](./../configuration/environment-variables.md#server_index_files) env.

Here is an example:

```sh
static-web-server -p 8787 -d ./public \
    --index-files="index.html, index.htm, default.html"
```
