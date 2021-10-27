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

<img title="SWS - Directory Listing" src="https://user-images.githubusercontent.com/1700322/118666481-81f22c80-b7f3-11eb-8c10-d530304e0e34.png" width="400">
