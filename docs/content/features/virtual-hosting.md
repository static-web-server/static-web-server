# Virtual Hosting

**SWS** provides rudimentary support for name-based [virtual hosting](https://en.wikipedia.org/wiki/Virtual_hosting#Name-based). This allows you to serve files from different root directories depending on the ["Host" header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/host) of the request, with all other settings staying the same.

!!! warning "All other settings are the same!"
    Each virtual host has to have all the same settings (aside from `root`). If using TLS, your certificates will have to cover all virtual host names as Subject Alternative Names (SANs). Also, beware of other conflicting settings like redirects and rewrites. If you find yourself needing different settings for different virtual hosts, it is recommended to run multiple instances of SWS.

Virtual hosting can be useful for serving more than one static website from the same SWS instance, if it's not otherwise feasible to run multiple instances of SWS. Browsers will automatically send a `Host` header which matches the hostname in the URL bar, which is how HTTP servers are able to tell which "virtual" host that the client is accessing.

By default, SWS will always serve files from the main `root` directory. If you configure virtual hosting and the "Host" header matches, SWS will instead look for files in an alternate root directory you specify.

## Examples

```toml
# By default, all requests are served from here
root = "/var/www/html"

[advanced]

[[advanced.virtual-hosts]]
# But if the "Host" header matches this...
host = "sales.example.com"
# ...then files will be served from here instead
root = "/var/sales/html"

[[advanced.virtual-hosts]]
host = "blog.example.com"
root = "/var/blog/html"
```
