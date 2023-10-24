# Basic HTTP Authentication

**`SWS`** provides ['Basic' HTTP Authentication Scheme](https://datatracker.ietf.org/doc/html/rfc7617) using an `user:password` pair.

This feature is disabled by default and can be controlled by the string `--basic-auth` option or the equivalent [SERVER_BASIC_AUTH](./../configuration/environment-variables.md#server_basic_auth) env.

The format to use is the following:

> `username:encrypted_password`

Both are separated by a `:` (punctuation mark) character.

!!! info "Password Encryption"
    Only the password must be encoded using the [`BCrypt`](https://en.wikipedia.org/wiki/Bcrypt) password-hashing function.

As an example, we will use the [Apache `htpasswd`](https://httpd.apache.org/docs/2.4/programs/htpasswd.html) tool to generate the `username:encrypted_password` pair.

```sh
htpasswd -nBC10 "username"
# New password: 
# Re-type new password: 
# username:$2y$10$8phm28BB4YpKPDjOpdTT8eUcfVDw0xc85VZPxg2zae1GR8EQqus3i
```

!!! tip "Password Security Advice"
    The password verification happens at runtime but its verification speed depends on the computing time cost of `bcrypt` algorithm used.

    For example, the `htpasswd` tool supports a `-C` argument to adjust the `bcrypt`'s computing time.
    
    Using a higher value is more secure but slower. The default value is `5` and the possible values are ranging from `4` to `17`.

!!! tip "Docker Compose Advice"
    If you are using `SERVER_BASIC_AUTH` env via a `docker-compose.yml` file don't forget to replace the single `$` (dollar sign) with a `$$` (double-dollar sign) if you want those individual `$` dollar signs in your configuration to be treated by Docker as literals.<br>
    More details in [the Docker Compose file: variable substitution](https://docs.docker.com/compose/compose-file/compose-file-v2/#variable-substitution) page.

Finally, assign the credentials and run the server.

```sh
static-web-server \
    --port 8787 \
    --root ./my-public-dir \
    --basic-auth 'username:$2y$10$8phm28BB4YpKPDjOpdTT8eUcfVDw0xc85VZPxg2zae1GR8EQqus3i'
```
