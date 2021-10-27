# Basic HTTP Authentication

**`SWS`** provides "Basic" HTTP Authentication Scheme using a `user-id/password` pairs encoded with `Base64`.

This feature is disabled by default and can be controlled by the string `--basic-auth` option or the equivalent [SERVER_BASIC_AUTH](./../configuration/environment-variables.md#server_basic_auth) env.

First, create a `user-id/password` pair using your favourite tool.

!!! info "Note"
    Only the password must be encoded using the [`BCrypt`](https://en.wikipedia.org/wiki/Bcrypt) password-hashing function.

In this example we are using the Apache [`htpasswd`](https://httpd.apache.org/docs/2.4/programs/htpasswd.html) tool.

```sh
htpasswd -nbBC5 "username" "password"
# username:$2y$05$KYOM0uaMQnEknnu/ckcCuuFyNQbc8BJEUk5X.ixtoCQpjXsc4geHK
```

!!! tip "Tip"
    The password verification happens at runtime but its verification speed depends on the computing time cost of `bcrypt` algorithm used.

    For example the `htpasswd` tool supports a `-C` argument in order to adjust the `bcrypt`'s computing time.
    
    Using a higher value is more secure but slower. The default values is `5` and the possible values are ranging from `4` to `17`.

Finally assign the credentails and run the server.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --basic-auth 'username:$2y$05$KYOM0uaMQnEknnu/ckcCuuFyNQbc8BJEUk5X.ixtoCQpjXsc4geHK'
```
