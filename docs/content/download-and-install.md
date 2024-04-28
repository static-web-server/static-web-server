# Download and Install

Latest **v2.30.0** release `2024-04-29` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.30.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.30.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `0b5aef1f4593d3df33dda7a79b74867ea0a393c829d0fac837c1f55192a0d405`</small>
- [static-web-server-v2.30.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `2affd9c129194f99b82ad7a1f262ec8ce80666223a2d092836554c9596d133ea`</small>
- [static-web-server-v2.30.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `9f246c9dbb61434241bf9fac48d01cd7c5ff89ed839f0597425f8ffd7f118322`</small>
- [static-web-server-v2.30.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `3c29a3e2deaa310a425e471eaa5e222cd73f0e7c1dc0435765d4bc6023e5cb0f`</small>
- [static-web-server-v2.30.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `5a3283efc33f3ffe8b16c98dfef8130a7fbf77bd23cfe5275f86215f4ffdb600`</small>
- [static-web-server-v2.30.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `b35cfbff085ddb1eab66cc7d90a017009fd6cbfd7aeb58ada88860d5a9bab18b`</small>
- [static-web-server-v2.30.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `ccc249ae0b13a8c1a04c61fd4e82b126497a1cb6d533f8581569113d750620c7`</small>
- [static-web-server-v2.30.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `cabd55e36bdc61d77664aa314d5e1e6f567be40fae0874211008a820fb4f1e0f`</small>

### ARM64

- [static-web-server-v2.30.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f31efb4838222335c4a1d9aec1420c67574c7babce3cc28956c502cd2523882b`</small>
- [static-web-server-v2.30.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `cb59a62373555d85f04f6482ad182cb38b14132f742031305346e78b6bd614d5`</small>
- [static-web-server-v2.30.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `6c63a388e421e3c533bb1efc691a1ba08b9b2a93a1f85824979c2a93c93ef3d6`</small>
- [static-web-server-v2.30.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `797127a587e62d885ec8677aae72548ff8566cafbfcc5f91c0fffa3a191525a6`</small>
- [static-web-server-v2.30.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5f5cf19dbab8cf4aa97dc70f6f9af7b888ea7112b26bb95cfec9c25b82639d41`</small>

### x86

- [static-web-server-v2.30.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `1e6c91dacff7403338adad458491f54e05b7109b01a1fb4cdec312abf285c3da`</small>
- [static-web-server-v2.30.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `d6ba3c180cf2f63ffe09e88eef450e79dcab888ec4e80cf2267d692b48f482ae`</small>
- [static-web-server-v2.30.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `d41577e62bdcb926d7a0a17d6f7365c15b77f58b2590371d0819bf7a634bad78`</small>
- [static-web-server-v2.30.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `f54fa98f2a88eb7352c8c5f8c04f6ac8956430baf501cec0879887df20998084`</small>

### ARM

- [static-web-server-v2.30.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `0ad66384e9aeb10d7ab4a527e0183196fdfc9ef8225ae7e0727e15e4130ef443`</small>
- [static-web-server-v2.30.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `b2e4cae63b46930bfbcbd095e1508d862c75f135b4dd1234590ba430c587299f`</small>
- [static-web-server-v2.30.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `3c1fa363dc1732f2d8ea5ae1a8be3a70625a4a3b72dde6b095a4a939e982c315`</small>

### PowerPC

- [static-web-server-v2.30.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `6dc52ffd948902b258c1d630bc2d141e0cb1ee72d15303b56d979c0ad8e66b3c`</small>

### S390X

- [static-web-server-v2.30.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.30.0/static-web-server-v2.30.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `7b10d2dedf59f492d0027e479652bc1f429f1e7908578d2224ebc612e3cb7d14`</small>

## Source files

- [static-web-server-2.30.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.30.0.tar.gz)<br>
<small>**SHA256SUM:** `c141d2d9db0d2bc0718bf860558aab0afc2dca0dc0cdedb70f8e01fcb3f867b8`</small>
- [static-web-server-2.30.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.30.0.zip)<br>
<small>**SHA256SUM:** `4c53ca11d13659927ba458fdd4c1323ab0507f3384e3bd1246b3d2d8a00ba0f9`</small>
