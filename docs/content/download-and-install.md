<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.33.0** release `2024-09-17` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.33.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.33.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

### TrueNAS SCALE

If you use [TrueNAS SCALE](https://www.truenas.com/truenas-scale/) then visit [TrueCharts Community Website](https://truecharts.org/charts/stable/static-web-server/) and its [Introduction to SCALE](https://truecharts.org/manual/SCALE/guides/scale-intro) page to install SWS application in your instance.  

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.33.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `1bf93cd1b5e970180967ea9e6a4f432ee1018aba0029d7c12efbf6a8693c9b0d`</small>
- [static-web-server-v2.33.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `fea20214e7c9b14e6cd810613f27f0f3bd99dda9f6779c03141691f929c10d66`</small>
- [static-web-server-v2.33.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `9b2e7943ba03efe19754cc6ffeee9b7170e6c4643166f8000ebed7e63435557c`</small>
- [static-web-server-v2.33.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `da9a67b67401773e5512866a9dcb87bb61fb00d274169272e56e2669451481bd`</small>
- [static-web-server-v2.33.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `27fc54ad885e193ce936994716104cf8a629492f2b28b8a426bcca1b6142ef75`</small>
- [static-web-server-v2.33.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `d4a8afb51961d41c2a245c5be03a80f48dad29c89b6cabc5677645b1f55c9f3b`</small>
- [static-web-server-v2.33.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `8375efdbd9fdd0d740ecbd56463016c50b889ea8c96ba9b0c1d3560da6af6187`</small>
- [static-web-server-v2.33.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `a0d9a274cde89793ac731b9b5510a4e3046225d88eb485b8951a6fc0c54ac90b`</small>

### ARM64

- [static-web-server-v2.33.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `d01fbc9c92ce0d83f31073d446acec11ab4e1d2221c7bc88d77de7792a5d1c46`</small>
- [static-web-server-v2.33.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `807ff371f1e3a9775da858eca2a6a56468933b88cd6374e1e8886d3fa6ffacee`</small>
- [static-web-server-v2.33.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `d6d8c6885ab53f9f74bd29535fc7cc5adfd622330953cf8af589c22e2335b16c`</small>
- [static-web-server-v2.33.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `262b8196ec2633c53038f27bb44af395c64b869647350e57e43e175d6710007f`</small>
- [static-web-server-v2.33.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `dc4b2a05a88739cfdd69de327f52f07831fc21e85581be169c7490f5d0734895`</small>

### x86

- [static-web-server-v2.33.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `b587be7cd9e81fec731c1772f8d5dc8b9455e61d0868a0ba533fb8e151c01b6f`</small>
- [static-web-server-v2.33.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `d3ead012d973b83e2d5393660913a006f8f51f53e4e9a713c47ab3861db3ea6d`</small>
- [static-web-server-v2.33.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f0b736cacf4e22e69382f1f9f34ab6d731ea5b535bb904e944b3df52dd9e7379`</small>
- [static-web-server-v2.33.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `33d27337867a20176804f5fff569db8a355ad0f0203887981453e1bc2ffc27fb`</small>

### ARM

- [static-web-server-v2.33.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `2f12a9d8b46091a737d2a07fb0801e9dfe5f5f0c3680c107ffb3b31c25d1018d`</small>
- [static-web-server-v2.33.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `eadee7de51ccb28e69970524672f2d870ee2b4890335bfdbb6531864d4948045`</small>
- [static-web-server-v2.33.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `ac5693f06f834f7f7658abdd533f18e25dfc298462db67ac41dcd3eb06e99793`</small>

### PowerPC

- [static-web-server-v2.33.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `bcbef68b691cc932c4027c3a7fe6377da3d67f8649fb91d466a11c33a6d95efb`</small>

### S390X

- [static-web-server-v2.33.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.0/static-web-server-v2.33.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `3adc19dd1166e1118e3add91e65d031649c84cb1be64caa8ed20f947351f3a35`</small>

## Source files

- [static-web-server-2.33.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.33.0.tar.gz)<br>
<small>**SHA256SUM:** `3858b355bfc67cd961a665650af2c0507554497b8bd7ae3ca40451133edd792d`</small>
- [static-web-server-2.33.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.33.0.zip)<br>
<small>**SHA256SUM:** `f4f23f405d6bea4072ed3a6e2349e614cf5a796e28b5f1f66c79133a3564382f`</small>
