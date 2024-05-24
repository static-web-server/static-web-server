<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.31.1** release `2024-05-23` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.31.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.31.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `02de9d929d69bc46d14fd4bcc8bff7f158fdbf714d49d3826dc09d9216e870b2`</small>
- [static-web-server-v2.31.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `2a246d41e584f42c68e0d71eb14564620028e0be664916adf00e7df3957f206d`</small>
- [static-web-server-v2.31.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `e2fb31126829209baaa9ac70c142325990455d3939fe0e5aaf62e18be1164337`</small>
- [static-web-server-v2.31.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `cb6f8919f816591e5002673885e4a3d0daa12ddd8e82b5f668b8eeae10401eb3`</small>
- [static-web-server-v2.31.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `8ddc30d11e35ea59ac97e8e89729c8846d95a581f6c28090decb7459d10ccc3f`</small>
- [static-web-server-v2.31.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `55cfc8ded7cdaa2f43831e42ffe7b723536090cfebf5c86a55101b02a8d75894`</small>
- [static-web-server-v2.31.1-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `df47da8e237427bb26cd8a81312ca12e3571ecd2b419589c97e5094b8e376624`</small>
- [static-web-server-v2.31.1-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `655e7bfb9b4e1fe39989ef6f9923d7d8d2d534a82b63a4e51e7e9fe1fb1ccb0f`</small>

### ARM64

- [static-web-server-v2.31.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `534b8091cc206618d4fb3f6cd0a669e2bbc3d8fab46db096f91b70f373c224f1`</small>
- [static-web-server-v2.31.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `c76fea34b8406297d64062a5e295280dcb871de3f3e9c43a49113a1b5ca5318e`</small>
- [static-web-server-v2.31.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `c0997e43211fc4879afd90485a75c15a902ca86cb536ed6300ee6e27199b1df0`</small>
- [static-web-server-v2.31.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `77117ef0553189e79dd90ebd0fe7da123841db105fd058d1553ef85ac2dc55f5`</small>
- [static-web-server-v2.31.1-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `57af06a0f219d264c43fe2695e3d1b45c041cfde99758d8678dde375313315a5`</small>

### x86

- [static-web-server-v2.31.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `53e14a56d6c685d68faa51c1515d2ea843209b4aadfacc47e9e099cfba8c88eb`</small>
- [static-web-server-v2.31.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `4b8911fe1dd95685a7ea641f4921333229304e75c60df081ddec6cb3a6525d1b`</small>
- [static-web-server-v2.31.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `92f16a46996dff4b53dcad8dafe93528f270f711f6daac928c4475a0b6cbe104`</small>
- [static-web-server-v2.31.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `4cb102120c4a64105b4c8bd927e429270b035de95fc980113606760c8fd5f7a6`</small>

### ARM

- [static-web-server-v2.31.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `eb1bcf695f9512dfef5ec14d906e9efad1bb86937ad071a0da0bc8d692648948`</small>
- [static-web-server-v2.31.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `60c4349e4fe91dde2439f345cfa868e50714027b152339ebcf16154aebabf21c`</small>
- [static-web-server-v2.31.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `0aaeae8a6e4244b34540c6fdc2be6ab3120ce54e7f8cac21ba0117c18b78adbc`</small>

### PowerPC

- [static-web-server-v2.31.1-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `85f400c5bf4a8604d76eafaa16540508f8ec9da8bd39f7136080c5d9b8d0d85e`</small>

### S390X

- [static-web-server-v2.31.1-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.1/static-web-server-v2.31.1-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `ae96c62b12f55c05bf3c500a96bb5fa57146273a4f67f5556c0cbd0ece71ca00`</small>

## Source files

- [static-web-server-2.31.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.31.1.tar.gz)<br>
<small>**SHA256SUM:** `63cae6bd2bce4d36907805ea54a1e40e9af93a9fb72accbce6445589a083febc`</small>
- [static-web-server-2.31.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.31.1.zip)<br>
<small>**SHA256SUM:** `2189129d1c00a96973f32b583010ccd9df063fcee8d869fa277480e248b52906`</small>
