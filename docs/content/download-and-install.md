<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.42.0** release `2026-03-27` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.42.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.42.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.42.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `11a8aea2e8b0affd16bc662418133fe36bdc2238aa7e2ce03f2e121b0be3d9a8`</small>
- [static-web-server-v2.42.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `e35c446f8ebee131301e4b91bb3723dac4ab74e3d4b17df26b9981f5b7b13f6a`</small>
- [static-web-server-v2.42.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `b7a41426c6649c6d66ad16571af3d4414a037b2f978d235df8831355bde83bbb`</small>
- [static-web-server-v2.42.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `404eb1cf857528a16067ddb2d9f824e838f3840d5629465df86886dd76d6a114`</small>
- [static-web-server-v2.42.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `1f1f4000edf7ae5ed2995247cd5d910417867ff0f2e2f848239a53d9a1274b7f`</small>
- [static-web-server-v2.42.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `ed32ff29cdcafb37b5cc4efa89147918bc837fbe308a2a6c438c8b9047b71956`</small>
- [static-web-server-v2.42.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `c7d6201e350a33f688f574ea040602175b90a908378c6b196c417c75187eda1f`</small>
- [static-web-server-v2.42.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `cf646470236fb878a927e469d90692c78142aaa936b0595e7b96f2130767e43f`</small>

### ARM64

- [static-web-server-v2.42.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `584c2b03e216fe10615b0ea0a530083a61014714805653650aa833489b679bbe`</small>
- [static-web-server-v2.42.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `f269f706042991f50d6478ea7743a4f2c38a23c1de10654208eec6832adebfd3`</small>
- [static-web-server-v2.42.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `628562f5782e9af3ea308701739b70d7d339738eb1299a018c4b65e4c7bbf1a6`</small>
- [static-web-server-v2.42.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `5791d7aebea60eabfccd4c7529cce1084e3ef524308f194ac1230e7c5ff74e29`</small>
- [static-web-server-v2.42.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `921980444a20579b19024bea39f862354e2de98d5832200a2586ff533cf92f9a`</small>

### x86

- [static-web-server-v2.42.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `6fe968416090befdaea331336447a7a65a9393407645631fa9d1e93bc8fc7c88`</small>
- [static-web-server-v2.42.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `774a7d6473a4f3e264357c6b3a5452c4bf4fde57460b9a3728a0cf8511160719`</small>
- [static-web-server-v2.42.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `6398dfe2ce8b554d342c47cfc021776af46a5be9a58a84a686aa1015e0f26a1d`</small>
- [static-web-server-v2.42.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `b6c66fad817e9d4e053f4a634f81de499ba425f40be8e5c764e5f79d7dee5956`</small>

### ARM

- [static-web-server-v2.42.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `bfa88045ad011c320df99d89ac9134aa4158b1cc3d3835850d5d4806ae43e2ed`</small>
- [static-web-server-v2.42.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `b679a40e94a0594e8a0be21a5e0a00a6b196d0b967b7950f791c13adb4ce6d28`</small>
- [static-web-server-v2.42.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `2faadea4c5243eeac94915d5dff2374ef4acc3e80682e69b4b346c2e6adfbaaa`</small>
- [static-web-server-v2.42.0-armv7-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-armv7-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `82a1c8a129f96b76f3387285eab40973e6d6c655bb1f9500de5397492c2edadc`</small>

### PowerPC

- [static-web-server-v2.42.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f80dd8eae4edf55e4918b993497ec63bf58498861d09dbd2a24f5745840f9ee2`</small>

### S390X

- [static-web-server-v2.42.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.42.0/static-web-server-v2.42.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `3b5e3ca8b244b7910f45028729442dc64238c4ad922082482fbd35fe671e1356`</small>

## Source files

- [static-web-server-2.42.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.42.0.tar.gz)<br>
<small>**SHA256SUM:** `7ef8ad8f22c4655979771d0e269aaf8232617b815fd5528342ecfc7061ecacb8`</small>
- [static-web-server-2.42.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.42.0.zip)<br>
<small>**SHA256SUM:** `29295efccd0e4b7f4c528487c0d84ca90d347f76cfe468e9d599aa2034e196cf`</small>
