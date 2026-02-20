<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.41.0** release `2026-02-20` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.41.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

</div>

See also [the release history](https://github.com/static-web-server/static-web-server/releases) on GitHub.

!!! info "Docker"

    If you are working with Docker containers then check out [the Docker feature page](https://static-web-server.net/features/docker/).

## Installation methods

### Binary installer (Linux/BSDs)

Use the binary installer if your package manager is not supported.

With [curl](https://curl.se/).

```sh
curl --proto '=https' --tlsv1.2 -sSfL https://get.static-web-server.net | sh
```

Or with [GNU wget](https://www.gnu.org/software/wget/) (Busybox `wget` is not supported).

```sh
wget --https-only --secure-protocol=TLSv1_2 -qO- https://get.static-web-server.net | sh
```

`static-web-server` will be installed by default under the `/usr/local/bin` directory.

Alternatively, you can install a specific version of SWS to a custom location by setting environment variables.

```sh
export SWS_INSTALL_VERSION="2.41.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

```sh
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

Using [Homebrew Formulae](https://formulae.brew.sh/formula/static-web-server) (also Linux)

```sh
# Build from source
brew install static-web-server
```

Or using the [SWS Homebrew Tap](https://github.com/static-web-server/homebrew-tap) (also Linux)

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

- [static-web-server-v2.41.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `c1c5472f6a188e4f6ba5d6b7869e0fef1b1c5f21354832af95537aaa4530c04c`</small>
- [static-web-server-v2.41.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `ae4222a1637edee7e62048a41eb0f5f0c5bde3a22c6b17e917b50fdeafd9bccb`</small>
- [static-web-server-v2.41.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `8e69be591f7d36de18f6046a3cc56976fda08be51b4d35ab9f04b9e84a30942d`</small>
- [static-web-server-v2.41.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `fd8f3d6960db1a8306a9d3a2504cfe5e15f3acf8778d9d6b7234683cf71e63a3`</small>
- [static-web-server-v2.41.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `57f9784e6b07da6aa563772c39edeac24e5f7bb8b5b610ad8a28d422def33702`</small>
- [static-web-server-v2.41.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `117061501464bb5230ce786b9885731f9df35ede385453c0aeaf0b0d401a6a27`</small>
- [static-web-server-v2.41.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `d2311d04c552e27549f568fb206a1eabca00588ce1d061c994c9b357ec5b0068`</small>
- [static-web-server-v2.41.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `7f16b0865a2177fa0a39b997257c6225e6bb2f7bbdeccf08119559e3e7d0b506`</small>

### ARM64

- [static-web-server-v2.41.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `2840501b8f6257869dc80bb4d42306481bb0479ef062fea98ea1e6c90e4f3455`</small>
- [static-web-server-v2.41.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `ea40df7ab4158b114acafec01dd1bdc1a5f89d093fd8ce59a52826930ca9bfb8`</small>
- [static-web-server-v2.41.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `33551713e7a3d6906fe54bd27e8a47761e198c35238dc15850630d83a44c2ac0`</small>
- [static-web-server-v2.41.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `29288cf33c927a32367632b5e788892c1205217abd7048578f4374d5db947785`</small>
- [static-web-server-v2.41.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `910f8f5ef5376b9d2d4927cc11cf0a2f5500201e5e01e5b26ab7a8ff52f13ef4`</small>

### x86

- [static-web-server-v2.41.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `b4802eb9471d1c39ab99383da762f6ed2a4877e8ac2ede002ae668fe65a24e47`</small>
- [static-web-server-v2.41.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `45a75c7fe17a7fdb82f4aa3a7d27dad26e7c58ff63dbec076581c93e9160c7d0`</small>
- [static-web-server-v2.41.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `54606a38f88c7b4f8d33fb48eb68659e5f6bf7b5e1244a2affe1ae81b9422c3f`</small>
- [static-web-server-v2.41.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `61574370efbacf1e4b4ab87fc803f11371d4a5fab249231d64c8d659c787bfb5`</small>

### ARM

- [static-web-server-v2.41.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `e6e9bd6463a5dd693867f23e81c80353f5dbb13be3471ae622621096c84074a9`</small>
- [static-web-server-v2.41.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `069eb070829046c2935d0f7f01418a45513d313673a6483460ae6975510fa4a8`</small>
- [static-web-server-v2.41.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `e3a61df56c3ceebef0f1b2eae5a8a6bfa4c639aab2bf664ad269fae92cf8700e`</small>
- [static-web-server-v2.41.0-armv7-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-armv7-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `254b72bbeda933d2842380a2e7236cc7d08eecf0502f354ab69d4e2634a20a60`</small>

### PowerPC

- [static-web-server-v2.41.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `5838fe49fc1276ad09a2d625a0a8fe9258eaec7167c94d02e3af6570a382c88a`</small>

### S390X

- [static-web-server-v2.41.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.41.0/static-web-server-v2.41.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `416374ea07876d03d7c7a5e2aad43a629d642189a768ed7d518c360470730430`</small>

## Source files

- [static-web-server-2.41.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.41.0.tar.gz)<br>
<small>**SHA256SUM:** `d3b3f7144180b74db2a39f8a21df546e1321336b0cb85d97f6769b6235668d2b`</small>
- [static-web-server-2.41.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.41.0.zip)<br>
<small>**SHA256SUM:** `f50ed6a74cf1e4d3a418b280bdd6fd0cf3f69ec7423b13806ffd1a93065f84a4`</small>
