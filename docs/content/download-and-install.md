# Download and Install

Latest **v2.20.2** release `2023-08-03` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.20.2), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.20.2-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `e9ea0d977896a42211743ce24c92874da8e5bd5e6a1767ef52d573277c462253`</small>
- [static-web-server-v2.20.2-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `a7f250baac1e1d3e52f8afc3c01b90ac4d848a04227859d39e14424102f03f9f`</small>
- [static-web-server-v2.20.2-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `bc3aae99cbd6acce2564d0042ca44cf3425ac1724d7d5885b2cc4c2e0bed8f04`</small>
- [static-web-server-v2.20.2-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `470acc41ea911d7358483252e705882209575f6d334e18037589d0a5ba923cba`</small>
- [static-web-server-v2.20.2-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `4ec59ce8dc4c6f7d56758d19316fefc2ccad5b3efb54bca03746c4ff6ddc0815`</small>
- [static-web-server-v2.20.2-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `f73586d4f16758f05e905ce956415e3d171c0a0cb7f7cd543df0a1d668598874`</small>

### ARM64

- [static-web-server-v2.20.2-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `9cdc969f0935e49c8a03da34ad3604e7cb91c610970f500389149a57be2912bb`</small>
- [static-web-server-v2.20.2-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `8bdc374660b349178ea8e4de8e2a301e706ede56976954ef5e85f5834787fed1`</small>
- [static-web-server-v2.20.2-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `d9b1b594ae9b7b4f67dc52bbc42e5be7cc109480f8de56fb88992163d227a2ce`</small>
- [static-web-server-v2.20.2-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `7c40c5362b7ebd4acae69d575933a2981deb5eadfbb99752e0f934501ce1249c`</small>

### x86

- [static-web-server-v2.20.2-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `eff9b319aaa374ec0ba20c6dfc04f23e0a98954b4d599c03562f4317b0f4aea7`</small>
- [static-web-server-v2.20.2-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `29427a9aaf65b5a4fd05fdbcaeffaad203c1b66cbc944b3b3eadceaf8010e1e4`</small>
- [static-web-server-v2.20.2-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `8053f8a96bd61ac236875f987d15a4fd41421be899961c5bddbcb7ad7db3d3f1`</small>
- [static-web-server-v2.20.2-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `ee265a08e4189bc98fc332705042a0eeb5b9a75b86c7aa234bd86d7bb9cdb229`</small>

### ARM

- [static-web-server-v2.20.2-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `4ef64d06ee13f00717c0fc5a4acee547b83fc126d963a07b4665be99a02056ed`</small>
- [static-web-server-v2.20.2-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `243910910732e43e75d71378e9c75125324483d1b5232c30c9fd041b392f5dc1`</small>
- [static-web-server-v2.20.2-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.2/static-web-server-v2.20.2-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `e05f1ff2c18dc1a131737e195675f99e3e78f6d8911b07048525d43d625db1e1`</small>

## Source files

- [static-web-server-2.20.2.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.20.2.tar.gz)<br>
<small>**SHA256SUM:** `bc9ed7a6b0eebe8999a7d8e8ebd372e902d2a2d7047729772ca9097a4a9d1e78`</small>
- [static-web-server-2.20.2.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.20.2.zip)<br>
<small>**SHA256SUM:** `cd0e8f49700d35a04704c2e27ed4a61ac4706a9f9de32c13ad77371b30e40e94`</small>
