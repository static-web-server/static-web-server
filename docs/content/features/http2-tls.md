# HTTP/2

**`SWS`** provides [HTTP/2](https://en.wikipedia.org/wiki/HTTP/2) protocol support.

This feature is disabled by default and can be activated via the boolean `--http2` option. HTTP/2 requires [TLS](./tls.md) to be enabled; pass `--tls`, `--tls-cert` and `--tls-key` alongside `--http2`.

!!! info "Tips"

    - `--http2` requires TLS. Always pass `--tls --tls-cert <path> --tls-key <path>` together with `--http2`.
    - When HTTP/2 is enabled, [Security Headers](./security-headers.md) are also enabled automatically (via TLS).
    - See the [TLS](./tls.md) page for supported key formats and cipher suite defaults.
    - The server provides [Termination Signal](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html) handling with [Graceful Shutdown](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace) ability by default.

## FIPS-validated Cryptography

For deployments that require FIPS 140-validated cryptography (US federal, regulated industries), SWS can be built with [`aws-lc-rs`](https://github.com/aws/aws-lc-rs) in FIPS mode as the TLS crypto provider, replacing the default [`ring`](https://github.com/briansmith/ring) backend. The underlying cryptographic module is [AWS-LC-FIPS](https://github.com/aws/aws-lc/tree/fips-2024-09-27).

This is opt-in via the `tls-fips` Cargo feature flag, which is mutually exclusive with the default `tls-ring`. Pre-built FIPS binaries and container images are published alongside the regular release artifacts.

The "Safe TLS defaults" listed above describe the `tls-ring` provider. The `tls-fips` provider's defaults are restricted to the subset of FIPS-approved ciphers (no ChaCha20-Poly1305) and FIPS-approved key exchange groups.

!!! info "Build requirements"

    - FIPS builds require `cmake`, `go`, and `libclang` (used by `bindgen` when compiling the FIPS module) at build time. The compiled output is still a single statically-linked binary.
    - Static linking is supported only on Linux x86_64 and aarch64, both `gnu` and `musl` toolchains.
    - The FIPS feature does not change command-line flags, configuration, or the wire protocol; it only swaps the cryptographic backend.

To build from source with FIPS:

```sh
cargo build -v --release --no-default-features \
    --features="tls-fips,compression,directory-listing,directory-listing-download,basic-auth,fallback-page,metrics"
```

Alternatively, in case of build errors with GCC >= 14, try Clang as the C/C++ compiler:

```sh
env CC=clang CXX=clang++ cargo build -v --release --no-default-features \
        --features="tls-fips,compression,directory-listing,directory-listing-download,basic-auth,fallback-page,metrics"
```

Finally, verify that the binary has been compiled with FIPS mode enabled:

```sh
$ static-web-server -V | grep -i "fips"
# FIPS Mode:
#   Module Version:   AWS-LC-FIPS 3.0.x
#   Crypto Provider:  aws-lc-rs (via aws-lc-fips-sys)
```

See the [Cargo features section](../building-from-source.md#cargo-features) for the full list of feature flags.

## Example

Below is an example of how to run the server with HTTP/2 over TLS.

```sh
static-web-server \
    --host 127.0.0.1 \
    --port 8787 \
    --root ./my-public-dir \
    --tls \
    --tls-cert ./my-tls.cert \
    --tls-key ./my-tls.key \
    --http2
```
