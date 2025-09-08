<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.38.1** release `2025-09-08` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.38.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.38.1" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.38.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `85293805167d363867ad8a9c64d4a28dfe21c5004e1696887d11a0ed6c7b372f`</small>
- [static-web-server-v2.38.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `d7dd69b74355b0ba9ce48f72612cdd09eb896565172991cc420e56b2385dc443`</small>
- [static-web-server-v2.38.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `8d0a70026ea180e8ff64d0f2ba711c2ef0be225f5d7d7e42d8f0600a96c59635`</small>
- [static-web-server-v2.38.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `f21f15c7ec5d28cdf61653ab25cf2a741a4db74bb951947bd84d157f8588f507`</small>
- [static-web-server-v2.38.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `ba500f832dc9f4c9a6dfc16c2848871f9a7d6e592fffc9c78270459867b8e144`</small>
- [static-web-server-v2.38.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `531c4159b3dce809c2f8547367ac39117c7592ed9f1992407a3c243edf320755`</small>
- [static-web-server-v2.38.1-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `0d8417818833283e150fe925d93e0ce521d400fb9923e71652d7248dbc9bd235`</small>
- [static-web-server-v2.38.1-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `de21ef942a1221890480e1e5a6b833d1d2ee88d43a18cd40dc24979d6f159589`</small>

### ARM64

- [static-web-server-v2.38.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `847eb2b0248f170a9ea9ff0dcd3ecd7a3b4ccf4e2f7125518cab20701993ed12`</small>
- [static-web-server-v2.38.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `0c7a82a64ca07ff75832234c71cec85a08fb3bdadd705d99d8c7558bc351bca3`</small>
- [static-web-server-v2.38.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `dfe0d1e808953d8c29991a73cc64026a87015fc20a8ed881ee6a6ccab76d6f0a`</small>
- [static-web-server-v2.38.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `810fd1b14baef769d7bf02e7f4624ec38b78e37a17854e30d9e2170bf9f25d46`</small>
- [static-web-server-v2.38.1-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5397d227c6bca52dc8895f96ef7c762fbd22bf357aae0515f1d6c8a32f0dd78e`</small>

### x86

- [static-web-server-v2.38.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `a9add943d8bf531299994065339b4b723709358577e41eadbdd8151143f7e5b6`</small>
- [static-web-server-v2.38.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `c253fe83a14b7df041eb921a7dd9028f9763d5a9ea3bd9cd4cd77b8516704813`</small>
- [static-web-server-v2.38.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `1c6ca522e941207233f122e0dec96006ec573158cef262bb149d404a6b37402b`</small>
- [static-web-server-v2.38.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `ce80e1f4768ff3c841abe99c17dee31c7cd297ed354c7db04e460270703d5cd1`</small>

### ARM

- [static-web-server-v2.38.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `f896427de4201408fede11f3895aca0038b6333fdff63d872d8a2aa1e7e8c798`</small>
- [static-web-server-v2.38.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `335a240d5bf00a026ed7d16babd3a3207daef0d3f201450f82643a575730ab8c`</small>
- [static-web-server-v2.38.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `6fe8edf7459d3158008da2a6d40cd7cd34a85187c0c02494853d6633201de268`</small>

### PowerPC

- [static-web-server-v2.38.1-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `2ae790065cb4220dd6940b77dc58e02a67ff65e4ea41706445477967f8b0432f`</small>

### S390X

- [static-web-server-v2.38.1-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.38.1/static-web-server-v2.38.1-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `938c61782e0bc9f5659adfc914035821dc55ec4af1b1d011ad03d526b6badf7d`</small>

## Source files

- [static-web-server-2.38.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.38.1.tar.gz)<br>
<small>**SHA256SUM:** `fce33a832f2ad3f9a96ced59e44a8aeb6c01a804e9cfe8fb9879979c27bbd5f1`</small>
- [static-web-server-2.38.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.38.1.zip)<br>
<small>**SHA256SUM:** `230d38fbaa94f2f27ecd0e03fb7addb420166db9792b1449c240511f26ece481`</small>
