# Download and Install

Latest **v2.19.0** release `2023-06-16` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.19.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

</div>

See also [the release history](https://github.com/static-web-server/static-web-server/releases) on GitHub.

!!! info "Docker"
    If you are working with Docker containers then check out [the Docker feature page](https://static-web-server.net/features/docker/).

## Installation methods

### Binary installer (Linux/BSDs)

Use our binary installer if your package manager is not supported.

```sh
curl --proto '=https' --tlsv1.2 -sSfL https://get.static-web-server.net | sh
```

`static-web-server` should be installed under the `/usr/local/bin` directory.

### Arch Linux

Via [Yay](https://github.com/Jguer/yay) or your favorite AUR Helper.

```sh
yay -S static-web-server-bin
```

### NixOS

Via [Nix](https://github.com/NixOS/nix) (Linux/MacOS)

```sh
nix-env -iA nixpkgs.static-web-server
```

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

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.19.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `907532f456c7235fcffd59d84f1689fdd887994d59c895b0d7edbfd20db30787`</small>
- [static-web-server-v2.19.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `c196c55e2151262e6741de506abef17ff1d90049663c1658ea6a04d490f66797`</small>
- [static-web-server-v2.19.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5beceb806bf5b7f1fbf07033582e41a7623333db785708d71363fe2ca4584b8a`</small>
- [static-web-server-v2.19.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `3656ecd81362057f958a76b58a4461dc6af90ab3174b5a226724c3039e87e7d3`</small>
- [static-web-server-v2.19.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `4d4090fc4995c641d16c9b59bfecb30b525edb6af89ff88d8ca2d32461ad00b9`</small>
- [static-web-server-v2.19.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `0bed74543e4c092a40911310602224d6796d8e05df05a8a1be0b49961c0f5f9a`</small>

### ARM64

- [static-web-server-v2.19.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `e44bf9fa3c78469743c85131ec3fa42ff647eb9e6d087bfa6a9cf1dde74787ff`</small>
- [static-web-server-v2.19.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `009c9887868208bc51d621bb65bd5913325ee55e2a0c68f9943985ae11c088b2`</small>
- [static-web-server-v2.19.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `714ecb86d5c895fd737074cf7cd56019a9c0cc7d04567d91f946f34eebb88e0c`</small>
- [static-web-server-v2.19.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `0ff332a165e3f0e7078ce02fa1c7f628c570e97df22e3059a95110c705194142`</small>

### x86

- [static-web-server-v2.19.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `affac7b3df9fba7188f2b623109bfc4da5f22ccf936b72bcd32951cc09c1f616`</small>
- [static-web-server-v2.19.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `93410e6104e69ddb1bfb59c99a3b4f2b748ec9b9bbc6b8a3132b3396ef8eb5d6`</small>
- [static-web-server-v2.19.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0071e9d53ab0ad34189d901866978cf82a3703b9bfa286ee0a5a5f1946567ec5`</small>
- [static-web-server-v2.19.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `b52ae2d25d077e8bb01006ec11501e16769273267ce89297240494c18d7669e2`</small>

### ARM

- [static-web-server-v2.19.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `ef7de111bd448ea59519e3670fc59c32591b70fb6868758ef25ea27a989d476a`</small>
- [static-web-server-v2.19.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `d273c3edd915a1a10ac46442e235dd1b83c6b104d26565cf9887fdf2289e8299`</small>
- [static-web-server-v2.19.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.19.0/static-web-server-v2.19.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `370a2b1945a0fd7556bb1e9a1104019fc1b238cc8c9f5c41aba6bdb604e09a6b`</small>

## Source files

- [static-web-server-2.19.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.19.0.tar.gz)<br>
<small>**SHA256SUM:** `f5d5f19ab8d5fd9d33ef18e7c1cf673e7fae105e7a358930752deeaafd59cdf4`</small>
- [static-web-server-2.19.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.19.0.zip)<br>
<small>**SHA256SUM:** `471b428fd47c8e7205787934fa046e1afd964f229f38b88f3b60c96cb2d3b81e`</small>
