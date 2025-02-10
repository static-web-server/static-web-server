<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.36.0** release `2025-02-10` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.36.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

</div>

See also [the release history](https://github.com/static-web-server/static-web-server/releases) on GitHub.

!!! info "Docker"
    If you are working with Docker containers then check out [the Docker feature page](https://static-web-server.net/features/docker/).

## Installation methods

### Binary installer (Linux/BSDs)

Use the binary installer if your package manager is not supported.

```sh
curl --proto '=https' --tlsv1.2 -sSfL https://get.static-web-server.net | sh
```

`static-web-server` will be installed by default under the `/usr/local/bin` directory.

Alternatively, you can install a specific version of SWS to a custom location by setting environment variables.

```sh
export SWS_INSTALL_VERSION="2.36.0" # full list at https://github.com/static-web-server/static-web-server/tags
export SWS_INSTALL_DIR="~/.local/bin"
curl --proto '=https' --tlsv1.2 -sSfL https://get.static-web-server.net | sh
```

Make sure you set the environment variables for the piped process (`sh` in our case), not the piping process (`curl`).

If you don't want to `export` environment variables then use:

```sh
curl --proto '=https' --tlsv1.2 -sSfL https://get.static-web-server.net | SWS_INSTALL_DIR="~/.local/bin" sh
```

### Arch Linux

Via [Yay](https://github.com/Jguer/yay) or your favorite AUR Helper.

```sh
yay -S static-web-server-bin
```

### Exherbo Linux

Add the `rust` repository and install [the package](https://gitlab.exherbo.org/exherbo/rust/-/tree/master/packages/www-servers/static-web-server) through `cave`:

```
cave sync
cave resolve -x repository/rust
cave resolve -x static-web-server
```

### NixOS

Via [Nix](https://github.com/NixOS/nix) (Linux/MacOS)

```sh
nix-shell -p static-web-server
# or
nix-env -iA nixpkgs.static-web-server
```

- [SWS Nix package](https://search.nixos.org/packages?show=static-web-server&from=0&size=50&sort=relevance&type=packages&query=static-web-server) maintained by [@figsoda](https://github.com/figsoda)
- [SWS Nix module](https://nixos.wiki/wiki/Static_Web_Server) maintained by [@mac-chaffee](https://github.com/mac-chaffee)

### MacOS

Via [Homebrew](https://brew.sh/) (also Linux)

```sh
brew tap static-web-server/static-web-server

# Just the binary
brew install static-web-server-bin

# Or build from source
brew install static-web-server
```

### Windows

Via [Scoop](https://scoop.sh/)

```powershell
scoop install static-web-server
```

### WebAssembly

Via [Wasmer](https://wasmer.io/wasmer/static-web-server/)

```sh
wasmer run wasmer/static-web-server --net --enable-threads --mapdir /public:/my/host/dir -- --port 8787
```

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.36.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `83f5f35197ef9b35475019eba5cd44004f72cecccc61957f2045b26881790c9b`</small>
- [static-web-server-v2.36.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `73992f047066aa39d6d28421429b05bbd877c428420a029e8cca5251fc4b1af7`</small>
- [static-web-server-v2.36.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `2a0071fd3978bd5fbb09a154e0c1d09672cacd6f9356ebeb25e76bb4d7ee1af9`</small>
- [static-web-server-v2.36.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `862741bb4490a2770325b5b08c475c92bf23c7a297c856ed2df2f9ec631d31e6`</small>
- [static-web-server-v2.36.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `170d3b99e5f08c61e38caa8335e91fca0c0156c6f3fba1c00b3a2763e16dc7d4`</small>
- [static-web-server-v2.36.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `2c047e9e58a6c62a31a2a86f15e45543db1bac6c3e0781b656e39e321d94a618`</small>
- [static-web-server-v2.36.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `6d16bd2fea21186e03e641e043acd49fea76d0993a2fee65c9f7153ecf8bccbf`</small>
- [static-web-server-v2.36.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `f45facda3d4164e3f6cd3be9b9b38633d5f36e30ecb3a8100c061e4af9ff26e3`</small>

### ARM64

- [static-web-server-v2.36.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `166fcac258a7d6bd644d427da005bde2212c57031c88a34526f25b859c3ef2a8`</small>
- [static-web-server-v2.36.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `c61e8ecad98309c8a82f101f6a2b43932bcc0fdddaa06231968bd17e56e9f488`</small>
- [static-web-server-v2.36.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `4cb4170bb221edaea86fcdc67152aeb125764054b54da0f1afeec3cd7e737b5e`</small>
- [static-web-server-v2.36.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `f77a638c606ea830fcd14685f27778deda1ffdc6a6e97058e51f9e77acb66833`</small>
- [static-web-server-v2.36.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `db8440d04ab0a2e8a229197da5c61344bb4760b7f1add33ba21443e323b8b8f1`</small>

### x86

- [static-web-server-v2.36.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5cdd9933ebf7c1621744412a83263ecc5cd0fd6a15d75f3aeb12698e72bd7553`</small>
- [static-web-server-v2.36.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `f84d9548f82a044fc4d7a07e341e79afb7c8b8bf5ad01ed4ce584c21493b569a`</small>
- [static-web-server-v2.36.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `9dd732810c6ab9132d17683fac8927811f656c1b45917adfadfa7e96fdc5e89e`</small>
- [static-web-server-v2.36.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `7b5fe36b8ca2affc4c535ca3d2649ba5960b23be5f769005907c376dac504ae0`</small>

### ARM

- [static-web-server-v2.36.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `9ad5935882f4fdea5cd990ba4341662141e6ab22ed86f5df1700be43a3944d6d`</small>
- [static-web-server-v2.36.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `260f17e12c7a23dbd191db596421e76caa1532d29dde0f6861c56b25e2c9f8da`</small>
- [static-web-server-v2.36.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `1066219a8dd30bfc9760b2ba4eeb381effeddc435feaa060f4d3d7f7785c9e5d`</small>

### PowerPC

- [static-web-server-v2.36.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0e0064c7317c549cec0efff824aa0f0f1900e0375777eac7a1ff64633200b542`</small>

### S390X

- [static-web-server-v2.36.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.0/static-web-server-v2.36.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `6e49ff4e2c1eed474212e68927ab26856d92a94ec8adfdceb25abf3dcbc32c67`</small>

## Source files

- [static-web-server-2.36.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.36.0.tar.gz)<br>
<small>**SHA256SUM:** `bb99fd25835050e9572ea4589f66b94a64d1724712a2f4881ab35f29d1d8f2a9`</small>
- [static-web-server-2.36.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.36.0.zip)<br>
<small>**SHA256SUM:** `c03d487ab8b925e482a43fd147106819b2f977ff19fe074ca61f91f2115ccdd4`</small>
