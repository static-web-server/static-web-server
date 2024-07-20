<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.32.1** release `2024-07-20` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.32.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.31.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.32.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `4849197ca742cec18fc0bd3994e3abcc639225c65af0309a23d8d9ab0206ddbc`</small>
- [static-web-server-v2.32.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `cbb506052535c17a1a042caf25c2ff76354d4de6a96cda605060bee502182cd3`</small>
- [static-web-server-v2.32.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `b5ceba7fb0ebdf930efcc4e1b358dc82ef7b07e5b07f007f118488e79fac2647`</small>
- [static-web-server-v2.32.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `f57320b1b108ef1839f38a470ad92a76e998e78842352e7150dc893e4666b106`</small>
- [static-web-server-v2.32.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `7f10a4601c82f2eda5227abf79edc2df5ba61a221c35a6e45878878c96875cee`</small>
- [static-web-server-v2.32.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `3594e490229b10fafe5c849addc8bcb777f1c69123d9d1a5ed731f6ae57cdf95`</small>
- [static-web-server-v2.32.1-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `4caa4a376055d9678edd9a89ace0693a1d2d323b5330392b2eccd35aac3ea7d0`</small>
- [static-web-server-v2.32.1-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `36ccad543873728b80546b3611a13728d557b15fb652b65ff14e36345f28bfc4`</small>

### ARM64

- [static-web-server-v2.32.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `34571e9390fdcdf7b327870e7d3974b5a61a8d80f565c249487f08db1c81bdf0`</small>
- [static-web-server-v2.32.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `faa86f42798a58ff8a35ffe54f1b8b2be0f767b444ecbbf99bbeae10268eaece`</small>
- [static-web-server-v2.32.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `2b167558c428f55bcc177e40d0707d2bfc9fec4211da787e0824483e1326c386`</small>
- [static-web-server-v2.32.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `deebfc23d5b88db0a29bf29b4cf66f3c6052d758f5ab7755308ffcc9964aa59d`</small>
- [static-web-server-v2.32.1-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `edd940eed3ca8f44e81a6db1cbbaa76900392132fc5435e859daa5d3e6dcc2e2`</small>

### x86

- [static-web-server-v2.32.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `d9f4da156c8c53469c9007c09723cf45f107379c1f81f832e53d3c720890eeb9`</small>
- [static-web-server-v2.32.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `64e41eb1d9802561aebdd7817a9311ec48020933b31f3971e1aeaaf8784eff1f`</small>
- [static-web-server-v2.32.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `571ceaf2a20f786ba10ef81198b5bbbe4a9c8b4db2a02e54e6a6e39ab35f3170`</small>
- [static-web-server-v2.32.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `cab2684753f5c67ab6917b2eb5239c2308b72cd390fc33021720f8a0b5366229`</small>

### ARM

- [static-web-server-v2.32.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `5aaf1b92c565a37666fb474815bd5fd13e529ec3af80a050d79dc48d9cf7dd10`</small>
- [static-web-server-v2.32.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `d42fdedd0bf638b0d2a8ac847e67aba06b6d7686ad2eea0517aeee6e028135e5`</small>
- [static-web-server-v2.32.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `0eabc77293d2beba1e26b847fa7935adcfa7c9a68ef481ea7ba629772a6034d6`</small>

### PowerPC

- [static-web-server-v2.32.1-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `ad0fb7ef05a1db53177a550dace6200e1d53be881edb61b709b6ebe323a0a76e`</small>

### S390X

- [static-web-server-v2.32.1-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.1/static-web-server-v2.32.1-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `a19240d3db07b3a5b5f8f12e73cc2522a4cc1bff69c0cc37dac132ffe46d36ee`</small>

## Source files

- [static-web-server-2.32.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.32.1.tar.gz)<br>
<small>**SHA256SUM:** `771aa43f3bc96334d432e75f31464e1a0d4dcb4aa920c4f58b2339757e929060`</small>
- [static-web-server-2.32.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.32.1.zip)<br>
<small>**SHA256SUM:** `1d06135ce1846749c3a78ec416493024ecda353b927f684115d047a8b3577de6`</small>
