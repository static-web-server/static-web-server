# Download and Install

Latest **v2.17.0** release `2023-06-03` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.17.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

</div>

See also [the release history](https://github.com/static-web-server/static-web-server/releases) on GitHub.

!!! info "Docker"
    If you are working with Docker containers then check out [the Docker page](https://static-web-server.net/features/docker/).

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

- [static-web-server-v2.17.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `02af0a207835d6936491707739c77b9cbc387811217d3760a5fda10b7d40804e`</small>
- [static-web-server-v2.17.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `5d41f180d0930684527a8d1b7c596005c3b54bd7d0e86ccb9fc898393fc26793`</small>
- [static-web-server-v2.17.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `cf3e280169616ec1b982e781837ec7e9e1f69463b3c5e94499236efc7e25eca8`</small>
- [static-web-server-v2.17.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `5e31820e4e7ba97ed94887e107d62dfb651896fc81b90626db6ebe631502edd4`</small>
- [static-web-server-v2.17.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `29383643eeb8c575497e111e48018f131c5e008b9c7019f2598662ac5aef0940`</small>
- [static-web-server-v2.17.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `818358c5befb795b5890e0f00887c37af3971d4bd3032969d608284dc485c12d`</small>

### ARM64

- [static-web-server-v2.17.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `2b6c2cb396ee0bc509eb175e48006754cc6569a56d70511f0ac7696acff28e20`</small>
- [static-web-server-v2.17.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `d4eea52af6b30f7c9caa50a8af5858d8433843ba9bd14f8eacce1e8801112be3`</small>
- [static-web-server-v2.17.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `595018f81dbc3278367c276cb85edb93765958c501de77880082dc4260a10628`</small>
- [static-web-server-v2.17.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `7a6ede773dc0662e9adb0595911b212857f9b06acfdf8bb0c31e373b7b0edf91`</small>

### x86

- [static-web-server-v2.17.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `16f857613a977d2b80131329a526fce6fb89b0a8ca5760ce9a80cfba10328605`</small>
- [static-web-server-v2.17.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `499226ba87fdcad55d2f48090079eda56396ce02e8833df6c9ab713dfd80cbea`</small>
- [static-web-server-v2.17.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `3b522d5fddcfda2473266cca34f509615a19c71a977335379f97202ffdd71def`</small>
- [static-web-server-v2.17.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `c72629fc2ad6d8bc002fad87b0c101befae9908a4d151b8e7989875bc229d9a6`</small>

### ARM

- [static-web-server-v2.17.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `704070f7a92723eeb8c4d9bb80d5f4a6fc827be9a08b4fce557d5f9dc02a0267`</small>
- [static-web-server-v2.17.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `356c1177e59fa518bda6a6a93c1db6290422e07c2590dcae606946e8efc45489`</small>
- [static-web-server-v2.17.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.17.0/static-web-server-v2.17.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `5d69a5fa1f8b2bc5231191dc8db7fe953d63cc09991e18ab56f300fe1f27d0ca`</small>

## Source files

- [static-web-server-2.17.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.17.0.tar.gz)<br>
<small>**SHA256SUM:** `d61e655e32cb0089d73a4c5687658f3993695062a42432362020ecd888ded441`</small>
- [static-web-server-2.17.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.17.0.zip)<br>
<small>**SHA256SUM:** `2e67688e3e37720b1bdd9433a447cdc119c67b5b1ccb94bab3ff014a6dc0c2e2`</small>
