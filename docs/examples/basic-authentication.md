# Basic HTTP Authentication

Create "user-id:password" pairs using your favourite tool.

```sh
htpasswd -nbBC5 "username" "password"
# username:$2y$05$KYOM0uaMQnEknnu/ckcCuuFyNQbc8BJEUk5X.ixtoCQpjXsc4geHK
```

!!! tip "Tip"
    Speed of the password verification depends on the computing time cost of `bcrypt` algorithm used.
    For example the `htpasswd` tool supports a `-C` argument in order to set the `bcrypt`'s computing time (higher is more secure but slower, default: 5, valid: 4 to 17).



Finally assign the credentails and run the server.

```sh
static-web-server 
    --port 8787
    --root ./my-public-dir
    --basic-auth='username:$2y$05$KYOM0uaMQnEknnu/ckcCuuFyNQbc8BJEUk5X.ixtoCQpjXsc4geHK'
```
