# Download and Install

Latest **v2.24.0** release `2023-11-09` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.24.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.24.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `e9b59cca802c56ea240ff0fcbdf546bd3d4ea67462a39b399d39934049712740`</small>
- [static-web-server-v2.24.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `645fc736e1c4e1b9ec61649468670dbb75c0da9f777a8b3fb838dc2d344140dc`</small>
- [static-web-server-v2.24.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5edaf3ece91474ccb94f9daca06ca87eb36d4ae1cea020c7201569d770fcca94`</small>
- [static-web-server-v2.24.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `66961d3d69dcbc24acf92b8932a1a77dab08b4679d4dc10d93921d7f82fe3a62`</small>
- [static-web-server-v2.24.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f65006837467834bfd5b42b0597bd4419f3427885d303ef99f027a1265f3c9a4`</small>
- [static-web-server-v2.24.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `31a6547f4bc0d68fb23b35f49440d6e8a3e83c6711b5a6cf6e2dfb211b22ea12`</small>
- [static-web-server-v2.24.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `a8be1fc8c6a3c3fcaa88b64a20693cd945c12639e9ae59c85738c9a016da0de8`</small>
- [static-web-server-v2.24.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `339c9484c4e90a59c1f26e56abe482d9b7ce99e07e5587d21a6aba5e4da86f54`</small>

### ARM64

- [static-web-server-v2.24.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0d35d3bc27371d02efb99234e3379a37f06452c693f60f21c3dd344947165200`</small>
- [static-web-server-v2.24.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `9ddf0c74c0073bc04a1a09f12c72ee2cced54d77149cd20b6b005fc69582d709`</small>
- [static-web-server-v2.24.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `4d7f410ba80eb9f96f2315e264e9760d95cfd298cec2b352b255dba314b8cb4b`</small>
- [static-web-server-v2.24.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `a97473f67502f24f20ceb77fbb12b7b67bfce0dd44fe7a88f0e3945ae5eb3474`</small>
- [static-web-server-v2.24.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `c26bc69f651465cd922d2a4a74edde3b26a4dd6eb5319d2dffedfecb87b4f261`</small>

### x86

- [static-web-server-v2.24.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `d34e7defe1c4124f566d1364a66cf4dd4ac754837a9c7d3f7d4bd3eebd095e8d`</small>
- [static-web-server-v2.24.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `fa8babe9880c103ce6d0e7352ff36811e7f21aa4325339a77c5be2209a2bcaa4`</small>
- [static-web-server-v2.24.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `d8e21d9700e972e08ced59c74c9a86788fba7a3a874c2b78ba7fa88cd152ff5f`</small>
- [static-web-server-v2.24.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `32900ba12cff18152c1d12286e80067adc9c0193f73f9e06e9e114054d636f5c`</small>

### ARM

- [static-web-server-v2.24.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `39f43769c71461b8bd88329a42ec5867630946581727561246d1a83b84da090d`</small>
- [static-web-server-v2.24.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `7a1a9682da53ee2ed586f05f36fa4c9adb89602573957e7bfd4cbac1e60a1cb6`</small>
- [static-web-server-v2.24.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `8b84ddc5d134cc3b2d69195f80feb028c833f57705e5dcbf08c25d85311b187a`</small>

## PowerPC

- [static-web-server-v2.24.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `aa8a64a82b679073ad71ea323a23200de4b706ee2f745ae12ff2509f8f91a81f`</small>

## S390X

- [static-web-server-v2.24.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.0/static-web-server-v2.24.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `37df2819d5a71914e07a7dbefd56cb27c904379512e6398960cae4b967d6b3cf`</small>

## Source files

- [static-web-server-2.24.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.24.0.tar.gz)<br>
<small>**SHA256SUM:** `11ece513f761cfa01d1793ef95dc5e09cf36a7c1a2cbc837e88affd43a7e1723`</small>
- [static-web-server-2.24.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.24.0.zip)<br>
<small>**SHA256SUM:** `e989d041572a25bbd6e76e7b887f48acde9fab257cabe6a97ff6e196625036de`</small>
