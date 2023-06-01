# HTTP/2 and TLS

**`SWS`** provides [HTTP/2](https://en.wikipedia.org/wiki/HTTP/2) protocol and [TLS](https://en.wikipedia.org/wiki/Transport_Layer_Security) support.

This feature is disabled by default and can be activated via the boolean `-t, --http2` option as well as string arguments `--http2-tls-cert` (TLS certificate file path) and `--http2-tls-key` (private key file path).

## Safe TLS defaults

SWS comes with safe TLS defaults for underlying cryptography.

- **Cipher suites:**
    - TLS1.3:
      ```
      TLS13_AES_256_GCM_SHA384
      TLS13_AES_128_GCM_SHA256
      TLS13_CHACHA20_POLY1305_SHA256
      ```
    - TLS1.2:
      ```
      TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
      TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
      TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
      TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
      TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
      TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
      ```
- **Key exchange groups:**
    - `X25519`, `SECP256R1` and `SECP384R1`
- **Protocol versions:**
    - TLS `1.2` and `1.3`

These defaults are safe and useful for most use cases. See [Rustls safe defaults](https://docs.rs/rustls/0.21.1/rustls/struct.ConfigBuilder.html#method.with_safe_defaults) for more details.

## Private key file formats

Only the following private key file formats are supported:

- **RSA Private Key:** A DER-encoded plaintext RSA private key as specified in [PKCS#1/RFC3447](https://datatracker.ietf.org/doc/html/rfc3447).
- **PKCS8 Private Key:** A DER-encoded plaintext private key as specified in [PKCS#8/RFC5958](https://datatracker.ietf.org/doc/rfc5958/).
- **EC Private Key:** A Sec1-encoded plaintext private key as specified in [RFC5915](https://www.rfc-editor.org/rfc/rfc5915).

## Example

!!! info "Tips"
    - Either `--host`, `--port` and `--root` have defaults (optional values) so they can be specified or omitted as required.
    - Don't forget to adjust the proper `--port` value for the HTTP/2 & TLS feature.
    - When this feature is enabled (`--http2=true`) then the [security headers](./security-headers.md) are also enabled automatically.
    - The server provides [Termination Signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling with [Graceful Shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) ability by default.

```sh
static-web-server \
    --host 127.0.0.1 \
    --port 8787 \
    --root ./my-public-dir \
    --http2 true \
    --http2-tls-cert ./my-tls.cert \
    --http2-tls-key ./my-tls.key
```
