# Download and Install

Latest **v2.24.2** release `2023-12-28` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.24.2), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

### WebAssembly

Via [Wasmer](https://wasmer.io/wasmer/static-web-server/)

```sh
wasmer run wasmer/static-web-server --net --enable-threads --mapdir /public:/my/host/dir -- --port 8787
```

### TrueNAS SCALE

If you use [TrueNAS SCALE](https://www.truenas.com/truenas-scale/) then visit [TrueCharts Community Website](https://truecharts.org/charts/stable/static-web-server/) and its [Introduction to SCALE](https://truecharts.org/manual/SCALE/guides/scale-intro) page to install SWS application in your instance.  

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.24.2-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `5b78b23c25a991737159ecf34bd894321d66d950947ed6f33116b988f96d4ca8`</small>
- [static-web-server-v2.24.2-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `631c6cc3ee32cb4d55f0092c792a480d97a2d1c6cc947dc2ea03add02a50d8f8`</small>
- [static-web-server-v2.24.2-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `171dbc6452eb53dee5f37f5040531f2082da5ffc7d3bd6f66ac33a9244be38ac`</small>
- [static-web-server-v2.24.2-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `0dbac3b28b53f530f6395e4ef32d3d38c3c6bd753ec497ac6118bcbc878c23f8`</small>
- [static-web-server-v2.24.2-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `2df323008f35b0eb4ce4e3499cbd67d01518fefe03fe90f563811072470cea69`</small>
- [static-web-server-v2.24.2-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `efc1eb614e4f9d66b2b79cf2ae474e0157613824176cd8ad2a7af27d439bedba`</small>
- [static-web-server-v2.24.2-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `22738032b859a1ec81a26b6412f816b7036cf84da7e94785df9913d169bcbb84`</small>
- [static-web-server-v2.24.2-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `b611dd089aadb76d21aee00053b5d1ea123f7d391fba82320bd62564f2546b1a`</small>

### ARM64

- [static-web-server-v2.24.2-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f3d53785d8f2e193a4c3a106ca3851dc3921b197fbb16388626746357ced6ca7`</small>
- [static-web-server-v2.24.2-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `2d23dce6de68af5228b1ad3708bdc746f2bc96710f939a908e1bca86b11b4fd2`</small>
- [static-web-server-v2.24.2-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `5a28276ca957a1d33eb9d37ab4ade05230570bc174375fb07f1016aa66147fa6`</small>
- [static-web-server-v2.24.2-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `1281b1f106d1f98934d5c826242ffa99330a8d589717c8889686de5c4192e6f7`</small>
- [static-web-server-v2.24.2-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `c667aa92d10b42c670d1a6e41dcb2bbd514aeada2069dc142aa51ac00a73b630`</small>

### x86

- [static-web-server-v2.24.2-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `9488a2611d96b8430403a0fac6c3fbedc43c9d66cf11c53ba8f652c5be96089a`</small>
- [static-web-server-v2.24.2-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `8f4f7e449f2d445d63abc5d11126364501dbb4c0b07f3ce832e737b5434ba8cc`</small>
- [static-web-server-v2.24.2-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `2dfeb059844140c46a3e609543bd253fbec266a20e553d46ab860194c747ed9a`</small>
- [static-web-server-v2.24.2-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `50355aaea32505106080610e5fccdbd8bcb8bb1c177cd3bdff0c3b0acea84f0f`</small>

### ARM

- [static-web-server-v2.24.2-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `38e3e378f897ca55abc70d8e9fd97d5da49cd4bf586ad9e86b0074a8e2837f98`</small>
- [static-web-server-v2.24.2-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `01fc0e58599a1d4c20110da5222eb51d95078d59625fb97d7e83f4676bacabe5`</small>
- [static-web-server-v2.24.2-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `a002de02d18422e096870736ebbc7f24404c37328d18a19068c67dbdd7c95d24`</small>

### PowerPC

- [static-web-server-v2.24.2-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `3f3efb24e55239193c703c6b5c2e81260ef0706c59b89ce324721848665bef3f`</small>

### S390X

- [static-web-server-v2.24.2-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.2/static-web-server-v2.24.2-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `6de01c70ba0dec6fc55ddc3bf4ded3b62b1d778b8daa7776c802700f60907d18`</small>

## Source files

- [static-web-server-2.24.2.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.24.2.tar.gz)<br>
<small>**SHA256SUM:** `9b56a901d31b7d3892f3203dd998ac4c82d7462297ed565cddde5175a9157868`</small>
- [static-web-server-2.24.2.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.24.2.zip)<br>
<small>**SHA256SUM:** `a57eea8325ef708407741f373019c07a1b893ceea63107c457ea91cd35c08152`</small>
