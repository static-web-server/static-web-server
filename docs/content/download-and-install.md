<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.38.0** release `2025-07-21` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.38.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.38.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.38.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `1e4759067d4297ec28f9bf1e3c7886786a165502b00ec9cf522786444489e178`</small>
- [static-web-server-v2.38.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `e8997d0783658f5ed5a9a1d4593f058f8a2f3eda877b8e0e91f39629064f8af7`</small>
- [static-web-server-v2.38.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `fb77c62842fa0e48a7f81d8872183342b3ce099f168c6f3b257a65dbeb882b80`</small>
- [static-web-server-v2.38.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `6744985ae6cc088eb8ea78f3242c8254d9a619b5f97f6011308845cd38023c34`</small>
- [static-web-server-v2.38.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `1e0dbce37f06eb135e3b37da29ab830dcbaa4eaedea685a8f4fd9a2b0c7a6280`</small>
- [static-web-server-v2.38.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `c63d4c28b226db3b1350cef24dc0bb686e5eb8b9c7f7f0f74b75935cb41455ee`</small>
- [static-web-server-v2.38.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `d51ced253a60126865115012f34849a82f965e23717360ff38fc0f043e9fb6fc`</small>
- [static-web-server-v2.38.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `1eff1cccf762746436c837bb182e9713031de8d4313ddbe57abcd05d089e0f72`</small>

### ARM64

- [static-web-server-v2.38.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0707b2e9ced19d890ffecfb2e224c7e78ec1d0f2d739305936583895a3f838a2`</small>
- [static-web-server-v2.38.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `f324fd5bbd8e6379d76849a8256d40edcfc324209909bd513367c7dbe8a9c5d5`</small>
- [static-web-server-v2.38.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `ee9e3ef5e2bce7eabeed95c46bc250f480b8909873265234a22ed64f66d0c063`</small>
- [static-web-server-v2.38.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `358f654294d0fc951cadc402d1729ae3495379ab9a84da2897a3d39ed6f085f2`</small>
- [static-web-server-v2.38.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `ab3c7a8ab2f42ae1e1bf07083ca49764bb144c446840acfdffbfcff0003aad77`</small>

### x86

- [static-web-server-v2.38.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `2c6dd01cb551110cdd86f33c3ddcc19b3d015aad013eadd106aab9c627683c91`</small>
- [static-web-server-v2.38.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `4dc5f4a59186ca93ace5fd733e421e82b75c1fb828faec951d1275ad0b919785`</small>
- [static-web-server-v2.38.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `3edfbebbaaf24202b089404e085b03ea63a2488a29c8a17fcabea3bb968edfec`</small>
- [static-web-server-v2.38.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `7d7c0300f3ecdc322e7c7b1449bc3b5fdd01394d4e950deed5de80586342867c`</small>

### ARM

- [static-web-server-v2.38.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `b4fd49fe1f5e28525efa7dc4467cbc3bc108fb5e9df883553bb10805b3367aae`</small>
- [static-web-server-v2.38.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `942704792485929c197523d4ef31b4c3e9db7330db38841c70c3ab2acccc1fd1`</small>
- [static-web-server-v2.38.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `41710958aba353d390ecee888573272d67c3d15777191c635e48c5a5c1acb1e8`</small>

### PowerPC

- [static-web-server-v2.38.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `58b8b314c9e08c7cca4f6704b041a0aa661db504511b66a67962460ae7227277`</small>

### S390X

- [static-web-server-v2.38.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.0/static-web-server-v2.38.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `9dfdc0a20acee940f1b336dbe07d639b4470abeebe3c38f551f5cea5bc4e00f5`</small>

## Source files

- [static-web-server-2.38.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.38.0.tar.gz)<br>
<small>**SHA256SUM:** `8f806542cd67f192610b2187cf6eb2fd0f0736309bf639af9fb6cb6a13d03d85`</small>
- [static-web-server-2.38.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.38.0.zip)<br>
<small>**SHA256SUM:** `35d5153ac6be0f9835a8bd58673dfe2110b9b0599d49b1de403c8d93e5de7ddf`</small>
