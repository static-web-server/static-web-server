<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.32.2** release `2024-08-13` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.32.2), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.32.2" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.32.2-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `4f4d987e9e4d9b30657837682c871ef9d62b17c0e3b6e1b3b4a95135c3f187c4`</small>
- [static-web-server-v2.32.2-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `a6ede027164565750d4b6281264af78447e55c293d371e03293d0ba81318ac61`</small>
- [static-web-server-v2.32.2-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `c4a213af197961a316884a5b1078feb50b130c5ed352545f7f7eeba58adcc6a4`</small>
- [static-web-server-v2.32.2-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `c997fe2fdaef886201a4b741c18ce878dbecfa8b73add8c75afe3dc6a6ab5d44`</small>
- [static-web-server-v2.32.2-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `88bfbc1b807967a92b51f48516b2f1cd10e43f4d2be0dd0cd8b20f791bb02870`</small>
- [static-web-server-v2.32.2-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `06716f1e5e488977cdb2f3af37bfcf2e9f2c51efc8e8daaa07a5267b72e3d563`</small>
- [static-web-server-v2.32.2-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `94e80dcda88be7e5f787c84f376a5f3824c01af4f8e71b80ab8b97642ecea747`</small>
- [static-web-server-v2.32.2-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `a4ed22683dcb3ca9aee9f538caa16ee136b686def208bae63a9c3218e25fbf8b`</small>

### ARM64

- [static-web-server-v2.32.2-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f27b50708f90cfadf7881ec27eca4c1c62de9fadf28d579ebdf511bd86ad96f9`</small>
- [static-web-server-v2.32.2-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `60d0bef8be23cb7c5d102876fc21a21c06191b43a8412a9e9da1aaa209ecc556`</small>
- [static-web-server-v2.32.2-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `805478b61561f50072f9921c47f1ff4f860cfc926ae62197fa1c2707c02cc356`</small>
- [static-web-server-v2.32.2-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `c0b869be3ae494f8031aba691be5da69a60761c035652c40a90c6abba62389b0`</small>
- [static-web-server-v2.32.2-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `7be87e187c1f04ae4f925c08a1e422147bc7a1d000ce1d7cefb68024546ce1b0`</small>

### x86

- [static-web-server-v2.32.2-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `6d385fd24a1577e79fd39870da7187a314a2813967a8a7920d76627a952a88d1`</small>
- [static-web-server-v2.32.2-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `fc33922c509c77aed0d2fe4f75a3a9ec41d42c21af22ae5e6cad30af5c54608f`</small>
- [static-web-server-v2.32.2-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `cc21fa05b39aba8241a6da491d5f659fd0e46ed3a8ba3a3767bfcf949b77756a`</small>
- [static-web-server-v2.32.2-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `8b8d1a8e629fb6130b58d924da60e7d8054b15da7e6443fff4c1af0c8a4cdeed`</small>

### ARM

- [static-web-server-v2.32.2-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `0ccf4930bfe094b0a08fc89962c1f094e5915335c11fa15f67c7acd6d210d234`</small>
- [static-web-server-v2.32.2-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `130e6211f335efef402562b503e600789826bfd0a2c936fa125a746afcc2c345`</small>
- [static-web-server-v2.32.2-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `ca8cdd85a43f85851b889be1982827f6c373e24cd64bf41c2234b0d01d6551a6`</small>

### PowerPC

- [static-web-server-v2.32.2-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0c992759c149f2d40cad21eb9df6e9e6fd2a815afd2ed33de42f2c06ca6c1d22`</small>

### S390X

- [static-web-server-v2.32.2-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.2/static-web-server-v2.32.2-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `bf94aeb75bfc4c995007b4f8821ea700b98b9b7c05cd564fa1702590c20a4752`</small>

## Source files

- [static-web-server-2.32.2.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.32.2.tar.gz)<br>
<small>**SHA256SUM:** `191a014f2f30fa145fbac727fb930e2a7063f3c27b8e72f33c21a8814969a641`</small>
- [static-web-server-2.32.2.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.32.2.zip)<br>
<small>**SHA256SUM:** `49575bf34f583bd284575a0737fc0715671f0c9addf061949f252ec92505f374`</small>
