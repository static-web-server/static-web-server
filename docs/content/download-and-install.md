<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.34.0** release `2024-12-04` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.34.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.34.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.34.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `c11bb6315d897f0dbd4be461694062e73b968141e208159e6af85ee4707b7a11`</small>
- [static-web-server-v2.34.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `34fb2a3d34846330946e2e962483f747edec520d4e740d8346df9b4955cf871d`</small>
- [static-web-server-v2.34.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `ea8199311e4564bd481f6807d8cfa5e5ed2776fa0748a1f18b3edfbf83c7d7e8`</small>
- [static-web-server-v2.34.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `62000f4ccdde3336f5c4da97f3386ac214e8152f236daef03b787fe4c504baf6`</small>
- [static-web-server-v2.34.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `ea19d9be9abcdea29c4eaad182c62689808db654b01b5aac1baff07329b38514`</small>
- [static-web-server-v2.34.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `b5360c8332f3e7f15735321388e1a5c60be18dd66c73b51edd35ee7880f5bdfa`</small>
- [static-web-server-v2.34.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `407f465e0b7caa493a639b7d90b1a63e865791b2fc20e8c432607c1140eb1da2`</small>
- [static-web-server-v2.34.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `232b8006f64dcb75f9ee04174957e9ec07deba9cb81137f82fd630eaf8f56146`</small>

### ARM64

- [static-web-server-v2.34.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `846685f2875fb2dbe3dac49845737f964fc8bf7f0f6a28756e10a4b7850e00a9`</small>
- [static-web-server-v2.34.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `17b6d321a368d8777b54a932345fe9b157d357d60edae30341be1457f2f713d4`</small>
- [static-web-server-v2.34.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `c7a9328ae913fbc110bddefbce8a98748329a977f9793aae6e8c40b58efbd19a`</small>
- [static-web-server-v2.34.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `1316896add4e3727b47efa346ed73a2edde24e6d83b669988b7eeba0785bdade`</small>
- [static-web-server-v2.34.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `741e8cd3f78f6e24951ff532837b16665f1956d02ae5bdb64555b02a5e9b03fd`</small>

### x86

- [static-web-server-v2.34.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `9dff84a007f05348c93228f092c8827801afd872acc8768507136d6329d7a0c5`</small>
- [static-web-server-v2.34.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `06d75bd2e109a4787bdda5cfff99705e39301f790961dc9a62ee7dc80aa49677`</small>
- [static-web-server-v2.34.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `5f7f877a0385844192e1d55524cf95299962328ae52a34f242b17927c63b3734`</small>
- [static-web-server-v2.34.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `46b6ef7f3da3a88f10d3d54f8d919bdc703abc34748609483f9dc741a63cf1ee`</small>

### ARM

- [static-web-server-v2.34.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `0817ee4c08810036f3418c4ee05f9ee642121917fdf7a87c4f0acbd57feaaee9`</small>
- [static-web-server-v2.34.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `0d0e08a3b0e5a51ce0eb77e2c6456554add19de86f23614597d558aa2f45bcfa`</small>
- [static-web-server-v2.34.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `579bd2a9ca795237f91ff6f17bbca2356c0203f320ed777e9a20d4413e5dd709`</small>

### PowerPC

- [static-web-server-v2.34.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `228bae47bfedfb19653f3bfa370091e5ba0887a88909401cf7949cfdcdf5f486`</small>

### S390X

- [static-web-server-v2.34.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.34.0/static-web-server-v2.34.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `b59dbf00d99097d04820f24f3c3f060e070454b9044ec2e28f64848b537bb3dd`</small>

## Source files

- [static-web-server-2.34.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.34.0.tar.gz)<br>
<small>**SHA256SUM:** `f0b6ef64f68445c98f1ffd22265d5675e64157e572431fa4fd362970199d0b5e`</small>
- [static-web-server-2.34.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.34.0.zip)<br>
<small>**SHA256SUM:** `7f6ac3d0af0ee10a119b802f3403c38e8b34ffaf3ebcdd8f0dcc932be26fd9da`</small>
