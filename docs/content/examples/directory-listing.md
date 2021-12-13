# Directory Listing

**`SWS`** provides a directory listing feature to display content of directories.

This feature is disabled by default and can be controlled by the boolean `-z, --directory-listing` option or the equivalent [SERVER_DIRECTORY_LISTING](./../configuration/environment-variables.md#server_directory_listing) env.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --directory-listing true
```

And here an example of how the directory listing looks like.

<img title="SWS - Directory Listing" src="https://user-images.githubusercontent.com/1700322/145420578-5a508d2a-773b-4239-acc0-197ea2062ff4.png" width="400">

## Sorting

Sorting by `Name`, `Last modified` and `Size` is enabled as clickable columns when directory listing is activated via the `--directory-listing=true` option.

You can also use the `sort` query parameter to sort manually by certain attribute from URI. E.g `https://localhost/?sort=5`.

## Sorting by default

Some times one wants to sort by certain attribute but by **default**. In that case default ascending or descending ordering of files/dirs by their attributes is provided by the numeric `--directory-listing-order` option or the equivalent [SERVER_DIRECTORY_LISTING_ORDER](./../configuration/environment-variables.md#server_directory_listing_order) env.

To do so you have to pass a [code sorting number](#code-numbers-for-sorting). E.g `--directory-listing-order=2`.

## Code numbers for sorting

Below the possible number code values for sorting/ordering which are grouped by attribute.

### Name

- `0`: Ascending
- `1`: Descending

### Last modified

- `2`: Ascending
- `3`: Descending

### Size

- `4`: Ascending
- `5`: Descending

### Default

- `6`: Unordered

!!! info "Tips"
    - The `--directory-listing-order` option depends on `--directory-listing` to be enabled.
    - Use the query `?sort=NUMBER` to customize the sorting. E.g `https://localhost/?sort=5` (sort by Size in descending order)

Example:

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --directory-listing true \
    # E.g Sorting file/dir names in descending order
    --directory-listing-order 1
```
