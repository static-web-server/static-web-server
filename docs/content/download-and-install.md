# Download and Install

Latest **v2.20.0** release `2023-07-12` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.20.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.20.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `1ae4f2f84f32a8d9e7d7a472e75d32446a7b0aa85b37076d119b2f1da73a7df8`</small>
- [static-web-server-v2.20.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `4a54022cb75c46fce5702fa4fd8b0127cb1c079cdd11d278eace8e62ae86d224`</small>
- [static-web-server-v2.20.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `2560cc18c293bd8f7818332fbe8f5fe2deac2ed4b0f3f5df15bbef231ed859da`</small>
- [static-web-server-v2.20.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `f46b1ee3d786a7b4b114096162c56cfc65570dc7bfdbb46e8cf2bb53dc418acf`</small>
- [static-web-server-v2.20.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `c3207e53ac1b66024228ad1018264921314d58ae0138cd331736af5f7e52fdf2`</small>
- [static-web-server-v2.20.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `73ed5e19d865ac9359102764b10398f5f1a671b6f70183b4b1bae9c6206e36d3`</small>

### ARM64

- [static-web-server-v2.20.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `b9ba8344f61dcc876d047ec309d0e0103ed03c4caba875529b3317adae749a84`</small>
- [static-web-server-v2.20.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `af6aa87c46a49729585924eff54d6f598369770c600098d2cd8c955819e5c4bd`</small>
- [static-web-server-v2.20.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `54a6654562240af1b99e35ee5e7aee56f803853ed28db7b983d2270bef206134`</small>
- [static-web-server-v2.20.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `21620f479c1f06496bfab069178dcffd496069dd910a52495aeebf4e697d0f73`</small>

### x86

- [static-web-server-v2.20.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `2ceaaea8a10c995c5314604a10ce434ba2a24028543f717650338d57ae56a594`</small>
- [static-web-server-v2.20.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `553263f923ca8f7f2ec0d113fe10f075262f99a10a96b16200c951ccde423e0f`</small>
- [static-web-server-v2.20.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `4b675808c03518f24520e9fe660f42292f06f7fb027080222fb7ae0f29ca05fa`</small>
- [static-web-server-v2.20.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `394da37c3f3e0dcce707d196d79428a5829c4c4e8e8389f4a357d50f655e418a`</small>

### ARM

- [static-web-server-v2.20.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `cd6e972aa0bc67129b14550003ec89893714d29b323f58280d7ca06100bf2031`</small>
- [static-web-server-v2.20.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `4b788fa79ac4dffd4b6c835b321fdeb834cabf77faa1eed1b300377b0b074091`</small>
- [static-web-server-v2.20.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.0/static-web-server-v2.20.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `126ab47e67402dace8f95887cbab1bfa3779faf94517b8d3d2bdca2018915619`</small>

## Source files

- [static-web-server-2.20.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.20.0.tar.gz)<br>
<small>**SHA256SUM:** `e4b01785086c055cdfafa475b1a0e18c9d47ded3a5ee0bf3b708bb39fd905e5f`</small>
- [static-web-server-2.20.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.20.0.zip)<br>
<small>**SHA256SUM:** `16d707da093551af60ad8b9193a103b5dc2461a3d85075e7bc128a5289aba387`</small>
