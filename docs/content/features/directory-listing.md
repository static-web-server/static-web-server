# Directory Listing

**`SWS`** provides a directory listing feature to display the content of directories.

This feature is disabled by default and can be controlled by the boolean `-z, --directory-listing` option or the equivalent [SERVER_DIRECTORY_LISTING](./../configuration/environment-variables.md#server_directory_listing) env.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --directory-listing true
```

And here is an example of how the directory listing looks like.

<img title="SWS - Directory Listing" src="https://user-images.githubusercontent.com/1700322/145420578-5a508d2a-773b-4239-acc0-197ea2062ff4.png" width="400">

## Relative paths for entries

SWS uses relative paths for the directory listing entries (file or directory) and is used regardless of the [redirect trailing slash](../features/trailing-slash-redirect.md) feature.

However, when the *"redirect trailing slash"* feature is disabled and a directory request URI doesn't contain a trailing slash then the entries will contain the path `parent-dir/entry-name` as the link value. Otherwise, just an `entry-name` link value is used (default behavior).

Note also that in both cases, SWS will append a trailing slash to the entry if is a directory.

## Sorting

Sorting by `Name`, `Last modified` and `Size` is enabled as clickable columns when the directory listing is activated via the `--directory-listing=true` option.

You can also use the `sort` query parameter to sort manually by certain attributes from URI. E.g `https://localhost/?sort=5`.

### Sorting by default

Sometimes one wants to sort by a certain attribute but by **default**. In that case, the default ascending or descending ordering of files/dirs by their attributes is provided by the numeric `--directory-listing-order` option or the equivalent [SERVER_DIRECTORY_LISTING_ORDER](./../configuration/environment-variables.md#server_directory_listing_order) env.

To do so you have to pass a [code sorting number](#code-numbers-for-sorting). E.g `--directory-listing-order=2`.

### Code numbers for sorting

Below are the possible number code values for sorting or ordering which are grouped by attribute.

#### Name

- `0`: Ascending
- `1`: Descending

#### Last modified

- `2`: Ascending
- `3`: Descending

#### Size

- `4`: Ascending
- `5`: Descending

#### Default

- `6`: Unordered

!!! tip "Tips"
    - The `--directory-listing-order` option depends on `--directory-listing` to be enabled.
    - Use the query `?sort=NUMBER` to customize the sorting via the URI. E.g `https://localhost/?sort=5` (sort by size in descending order)

Example:

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --directory-listing true \
    # E.g Sorting file/dir names in descending order
    --directory-listing-order 1
```

## Output format

**`SWS`** provides support for specifying an output format either HTML (default) or JSON for the directory listing entries via the string `--directory-listing-format` option or the equivalent [SERVER_DIRECTORY_LISTING_FORMAT](./../configuration/environment-variables.md#server_directory_listing_format) env.

!!! tip "Tips"
    - The `--directory-listing-format` option depends on `--directory-listing` to be enabled.

### HTML format

This is the default format when `--directory-listing` is enabled.

### JSON format

The JSON format used is shown below for directories and files. Note that the `size` attribute is only available for files and the `mtime` value is UTC-based.

```json
[
  {
    "name": "my-directory",
    "type": "directory",
    "mtime": "2022-10-07T00:53:50Z"
  },
  {
    "name": "my_file.tar.gz",
    "type": "file",
    "mtime": "2022-09-27T22:44:34Z",
    "size": 332
  }
]
```

Here is an example of serving the directory listing in JSON format.


```sh
static-web-server \
    -p=8787 -d=tests/fixtures/public -g=trace \
    --directory-listing=true \
    --directory-listing-format="json"
```

And below is a client request example to illustrate how the feature works.

```sh
curl -iH "content-type: application/json" http://localhost:8787
# HTTP/1.1 200 OK
# content-type: application/json
# content-length: 163
# cache-control: public, max-age=86400
# date: Tue, 11 Oct 2022 23:24:55 GMT

# [{"name":"spécial directöry","type":"directory","mtime":"2022-10-07T00:53:50Z"},{"name":"index.html.gz","type":"file","mtime":"2022-09-27T22:44:34Z","size":332}]⏎
```
