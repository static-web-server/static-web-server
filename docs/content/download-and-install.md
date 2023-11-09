# Download and Install

Latest **v2.23.0** release `2023-10-15` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.23.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.23.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `77d8d13d6d188c50c0f8bc92432afd5b5875c1e9c6ded9841d498750788b1ab2`</small>
- [static-web-server-v2.23.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `bfd0ed4961ac859b60c9d83a9fd50af6bc7b72b8ca4102b36bf4990fad5feb2b`</small>
- [static-web-server-v2.23.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `cea58a8b54f33cccd4c287600f40441b89d44d316a7ecb91bda897aa3dce2a8e`</small>
- [static-web-server-v2.23.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `4faa32e984f20cc307320f59ac3663ae7608208af34cb6c63eb21745152c32cc`</small>
- [static-web-server-v2.23.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `6f86fd3d3b6f71b2aa2a71b521aee6f55377e5c5d6a58b92da8d2ee89ecc9587`</small>
- [static-web-server-v2.23.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `289dc019fb756d9feee8394c2971e5aef4509b30354d9995859bd92cade04016`</small>
- [static-web-server-v2.23.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `f96718f3f7c301c60ffa6932bc91b322618386088763b1778636dcf65d238d73`</small>
- [static-web-server-v2.23.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `225ee14497a8710165b916864d6ddc0149c583a8fbd871ad67e802b0aa87e2fb`</small>

### ARM64

- [static-web-server-v2.23.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `84ef76a9921e1945825d546d0c0212d9e639a9f8a132627ae6ba014a5ee737cf`</small>
- [static-web-server-v2.23.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `0f50e86b2f4c29f7dd0176d44ed05d070931e9f215974bd15b179189197e3ff0`</small>
- [static-web-server-v2.23.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `a2db4d15f6a300a6839edeebf4e806bdc0b1943d050c5c5371b83379bd372d3b`</small>
- [static-web-server-v2.23.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `83927990752f9342e470f9537f41e44f60d17c7f41686b722040ee1adc81cf45`</small>

### x86

- [static-web-server-v2.23.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `9731d48033ece21b1a903c60373151c1dc027b1925dcc88584b15e8a96c450dd`</small>
- [static-web-server-v2.23.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `fd486a708c3ea0b45a2596905bab2c79479142d8b0513fd8a8c40b75b70d9940`</small>
- [static-web-server-v2.23.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `2508e39d617a7de0327622ac9d19e8532f42df7bcadd93b470809b8f1d0ba8a5`</small>
- [static-web-server-v2.23.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `4ccaa1e83151cf82e4c9f211e78e04ae5afa9c84f3ec72e73372ea9fc5aa3409`</small>

### ARM

- [static-web-server-v2.23.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `6ec9c47a2ba7d6d51d3cec62dd19bda8347ffb1e907fefac0ab6ad08d1ffa233`</small>
- [static-web-server-v2.23.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `e839965e865c081607ec1311e2218c35c7df2b1bc3bcabced4984d916747a630`</small>
- [static-web-server-v2.23.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.23.0/static-web-server-v2.23.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `bdf4f511280043e65078260382443824e4b24aa5552cda5e8b0fd63a96a387d7`</small>

## Source files

- [static-web-server-2.23.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.23.0.tar.gz)<br>
<small>**SHA256SUM:** `534c957c6d1f6ba9174cfe2a880668da3ca5a3288c665dcb8c0b87b0e08a3b82`</small>
- [static-web-server-2.23.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.23.0.zip)<br>
<small>**SHA256SUM:** `b0ba1f45bbc2b2cf6aed468cddc881ecd29e52acc8c9086dd936378ac687e1ce`</small>
