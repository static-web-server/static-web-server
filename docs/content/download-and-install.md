<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.39.0** release `2025-10-26` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.39.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.39.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

Using [Homebrew Formulae](https://formulae.brew.sh/formula/static-web-server) (also Linux)

```sh
# Build from source
brew install static-web-server
```

Or using the [SWS Homebrew Tap](https://github.com/static-web-server/homebrew-tap) (also Linux)

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

- [static-web-server-v2.39.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `19ca53266b11f4792d7c1228dafff5b3acaed2664acee6cb5b9697ee040899ba`</small>
- [static-web-server-v2.39.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `69c475bc778e8a78022b50b77ac18a16442829f912ed4f21b7e5b2e63a0ca4f4`</small>
- [static-web-server-v2.39.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `a47ed9d1f315bfd9fb3839ecfe8329487d1aee4c08e6ea5a57d236984973d518`</small>
- [static-web-server-v2.39.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `8ed7e75844f952ad9f20f303187084af4c433ecac4558b2c3268af7f36d0983c`</small>
- [static-web-server-v2.39.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `1112412c1b8374792cfaaae18cf9812ab1ed49e6bf835a75c7f57ab2f34f1b12`</small>
- [static-web-server-v2.39.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `3b7625bd7212f5886637dda994520705ac8c423dd217fdd0ad90e7bf07abb69b`</small>
- [static-web-server-v2.39.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `a99c14d34f2b3402de6d0248ccc622fa1558d9485fd0f41a34e834784575fdcc`</small>
- [static-web-server-v2.39.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `aea533e9d11875f8d11138d9f410f506a930596ba48476916f8c8ab7d83eb66f`</small>

### ARM64

- [static-web-server-v2.39.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `078bd8b48814c9b8e3c0f8ebc71f173ada87c00fd6d776f04a42a25b9d87ec36`</small>
- [static-web-server-v2.39.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `fc40bcee1c68a51b4169fa7d08ecca15e91b0dc7d7e58391329c5f1998a33481`</small>
- [static-web-server-v2.39.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `19c0f55c0ce17020d472db5d726bd53f5cb098814f4b674d62a7d329063c2b71`</small>
- [static-web-server-v2.39.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `db0f3bc158dd9612223684a8a3f7de0665975a0c236cc14af89ec87c5de1c9da`</small>
- [static-web-server-v2.39.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5083be6267c8807396564a971fd094eb4c3bc360db7718f0451e53d7633ebf6b`</small>

### x86

- [static-web-server-v2.39.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `8e34ed470a78bb9b3f0b378af0c6ad44673c21264a64a01d8e2fc3b735e3b0b0`</small>
- [static-web-server-v2.39.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `396801d3be49587a143ae56717c2159b8c18e1e03b4be43e15d3f58c380374a6`</small>
- [static-web-server-v2.39.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0e6591af6e6525ed5d0b4852bdfa44a3fb2f7f7cef78b09166eb23f3e2ee4221`</small>
- [static-web-server-v2.39.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `2fdb9c7af6df78871f850ed3b0da359b3d64a7a52414db0b25c38b4ff3befee4`</small>

### ARM

- [static-web-server-v2.39.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `bedd6f3b01e6bb7c8328a1aebef87cc2861d116204f29b14fe544e983e4c1649`</small>
- [static-web-server-v2.39.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `299eec92abd60f6c0d14f6114caf24fe0efe431c09bfd9f0a1a74f00b38b95b1`</small>
- [static-web-server-v2.39.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `9f2f6bc22aabd5e5b23922e479f9737b6d43388e69719cba600cadc0a1cd6777`</small>

### PowerPC

- [static-web-server-v2.39.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `72c5bca530307d2ec10cbb743f96f61f860bb22994a759236da7efaf819894c5`</small>

### S390X

- [static-web-server-v2.39.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.39.0/static-web-server-v2.39.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `b347661f9c6cedc19799648e7e876f9e9e9f7f5834b5cc4679c4eed307d5faf9`</small>

## Source files

- [static-web-server-2.39.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.39.0.tar.gz)<br>
<small>**SHA256SUM:** `da5906ed28eb47ebe6fae782b20a2f99f69c094e7885c66612e2c03d7911631a`</small>
- [static-web-server-2.39.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.39.0.zip)<br>
<small>**SHA256SUM:** `f47cd4956789bd14400991b506aa75fc72ab6b73fa0e7bf5cde5fcd317b37989`</small>
