# Download and Install

Latest **v2.18.0** release `2023-06-08` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.18.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.18.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `2e0bea73a3a09b5d08da163745824bb71f9ec8ddc5d596e73cdf613d3d8c3ee4`</small>
- [static-web-server-v2.18.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `852ccfca4ac922705040cad0fd71cf42c81aa18b44e6b21132cd9e37b5f07def`</small>
- [static-web-server-v2.18.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `01d36e2f73940dc61665f6525e2a35486d882f1c9c929646db85f5a9fe59a851`</small>
- [static-web-server-v2.18.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `728f3f1df9d672ed353006bd7ecc5df7cc4cac6cea4907057899777e98f18fee`</small>
- [static-web-server-v2.18.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `56beca2ea83df796ad005961d1b319c0d8ca46cc31760335de931cbfc3617066`</small>
- [static-web-server-v2.18.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `014911108a1d308aa554093c427f7c9afbbfd4a3329b111e99179e807c78440c`</small>

### ARM64

- [static-web-server-v2.18.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `dc34b49a7a18db2eb7913559ac490eecf9ab8bed1ac9df12495cc6daf2e694e9`</small>
- [static-web-server-v2.18.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `b94f804d56356caddba0df6eb0108c7bfde75b78ff3c1200ad654e0045eeadd8`</small>
- [static-web-server-v2.18.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `08cc27e5eb1a2acfee93da85d464d9448ddf1e76372d3faefd9db2c02215421d`</small>
- [static-web-server-v2.18.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `e0bb4c5973e0235fe12f0bd2fde03e375ed7b0224633b490a299392b722d0778`</small>

### x86

- [static-web-server-v2.18.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `fb12e445208d7a4403dbf72fbdf36d3f795a39bcc2a5b115a8ebb04f7c724514`</small>
- [static-web-server-v2.18.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `9a41d07a81028d4a1b4c9b17f3cad2f1bd3c3c9507ef7c9fccf8559499cc75eb`</small>
- [static-web-server-v2.18.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `13423dcdd82b6ee70bb821c7b88ffae466f3dcd2f8b7d8aa7cdd142f057de335`</small>
- [static-web-server-v2.18.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `c14f31b38a325fa71acc240e7106d3dd85fd65ec537a93c99edbd9b8691f1183`</small>

### ARM

- [static-web-server-v2.18.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `66352fd8118b5230203bba80acbf4a558423c93ea4921b918f064797e36e6d13`</small>
- [static-web-server-v2.18.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `a34b8575d94b82953afa1d3befbc86be4f1aa0242ce843f9a0edb01ea8b633f4`</small>
- [static-web-server-v2.18.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.18.0/static-web-server-v2.18.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `74ec3127e0af012360f1e1f7d26b70010ce2add806404823155510d19693b994`</small>

## Source files

- [static-web-server-2.18.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.18.0.tar.gz)<br>
<small>**SHA256SUM:** `c966f8a5f0dc2fec42dcbc800af46646a61955332da9eceb1be95c5e634f12f2`</small>
- [static-web-server-2.18.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.18.0.zip)<br>
<small>**SHA256SUM:** `942e50ec5a24961aa09cfa6704378a6154bba103c76844823b063064ac959a26`</small>
