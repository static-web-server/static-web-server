# HTTP Methods Supported

**`SWS`** only supports [`GET`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/GET), [`HEAD`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/HEAD) and [`OPTIONS`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/OPTIONS) HTTP methods.

## OPTIONS Method

### Identifying allowed request methods

The HTTP [OPTIONS](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/OPTIONS) method can be used to send a request to check for permitted communication options for either a given URL or server.

Example using an HTTP client.

```sh
curl -I -X OPTIONS http://localhost:8787/assets/main.js
# HTTP/1.1 204 No Content
# allow: OPTIONS, HEAD, GET
# accept-ranges: bytes
# cache-control: public, max-age=31536000
# date: Thu, 10 Mar 2022 21:26:01 GMT
```

### Preflight requests in CORS

The HTTP [OPTIONS](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/OPTIONS) method can also be used to send a request asking if it is acceptable to send requests to the server and if it is aware of using specific methods and headers.

!!! info "Tip"
    If an `Access-Control-Request-Method` or `Access-Control-Request-Headers` value is not allowed then the server replies with a `403 Forbidden` HTTP error. See [CORS](./cors.md) feature for more details.

Example using an HTTP client.

```sh
curl http://localhost:8787/assets/main.js \
    -I -X OPTIONS \
    -H "Access-Control-Request-Method: HEAD" \
    -H "Access-Control-Request-Headers: content-type" \
    -H "Origin: http://localhost:8787"
# HTTP/1.1 204 No Content
# access-control-allow-origin: http://localhost:8787
# accept-ranges: bytes
# access-control-allow-headers: content-type, origin
# access-control-allow-methods: GET, OPTIONS, HEAD
# cache-control: public, max-age=31536000
# date: Thu, 10 Mar 2022 21:45:55 GMT
```
