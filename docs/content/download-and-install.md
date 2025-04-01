<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.36.1** release `2025-04-02` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.36.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.36.1" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.36.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `7d67d6dc1931805c624280595836843b6b09ad8663690848d1c8779ae5db1669`</small>
- [static-web-server-v2.36.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `9278bb9d89a243aeef27dfd80cc4dfc9745717ca2f9a297d0dbc6999ca10eca8`</small>
- [static-web-server-v2.36.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `dcf1105a016d1e185edfb0830065e12eea23bb35af5eae64f661117c10970593`</small>
- [static-web-server-v2.36.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `bf609d43ff593fdb50e3f7b51d012a681c9dcb3742157ee9e75baa57a2df0bcd`</small>
- [static-web-server-v2.36.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `81cac7e086ddce2528bcd3a99ced7b25c92d58e1be24d19717157e551e0f967c`</small>
- [static-web-server-v2.36.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `28d353599c3a2407d1cceab6f58cf6aab40886835c226563969629af7e1c7e63`</small>
- [static-web-server-v2.36.1-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `f2ffb2bd10a98a80d28554f77ca800bddad7833bc9447f84418db173015e550e`</small>
- [static-web-server-v2.36.1-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `2f0a32a7fbc16a54644e045760ec952b79f3a98b7771d21c571e64b400074759`</small>

### ARM64

- [static-web-server-v2.36.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `c201cf1979ec29abfc093202613f6daa2c88b36b041e4804bd38b8e0a1318f6f`</small>
- [static-web-server-v2.36.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `21108f2b359086e56faea8a4b3b9170207ad3361ee35b02def45e1b5b71a9880`</small>
- [static-web-server-v2.36.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `25e98b9b3b0f71d7055a3a40c3fca4adf00608eb1685a0adf99beb4f02cd63b9`</small>
- [static-web-server-v2.36.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `47fdd8061a59dd3cef5fc136eb896d01406fc1b6c8cca098e87ef5330497dcba`</small>
- [static-web-server-v2.36.1-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `d6bbddb5ca5b55cb4142e6e30ce1a9f06525595ba5cec9f4bf936ce109f09aae`</small>

### x86

- [static-web-server-v2.36.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `e7b5a98df19f08f59fd710c6adca3cfe9feb839032f30624f7b8461afabc243d`</small>
- [static-web-server-v2.36.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `c6c0050366401bde4882afb85cea1962f02b1c42b06c66cfc8e346f47323e0b5`</small>
- [static-web-server-v2.36.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f494b9ba0a57fe53390fccc6dc2771f0960e79ce14c37ac37d261ee9228edefa`</small>
- [static-web-server-v2.36.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `a1ad328bf942598f967d823bae501465f6feb18da062118d1c60b28d9f1474ae`</small>

### ARM

- [static-web-server-v2.36.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `81c34b3819ce2a139a664501589323d3fb1d2565c4454bc16d5c2a7f66a0e6da`</small>
- [static-web-server-v2.36.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `429567c305444bff1f38869fc31f8be3eb688e7c2f76c3f05b5f883fc0d0254d`</small>
- [static-web-server-v2.36.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `1c640ce9c380d3c82e9ed9304255af721d1405ffeb2a5fe76a20bdb68610e1dc`</small>

### PowerPC

- [static-web-server-v2.36.1-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `a179b59dc57e756ba419ee2c6a5b84dc0ab327fe6e8cba013de861114bd3d6f6`</small>

### S390X

- [static-web-server-v2.36.1-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.36.1/static-web-server-v2.36.1-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0fb67cdcfb77f1fd0c0031795e00579a164289584526a395a3ba8e4deef30499`</small>

## Source files

- [static-web-server-2.36.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.36.1.tar.gz)<br>
<small>**SHA256SUM:** `e242e21b3e4b46395bda21b351438df6b5c54b1319a41a86b52eb49ed5567a40`</small>
- [static-web-server-2.36.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.36.1.zip)<br>
<small>**SHA256SUM:** `6b81abd065f9dfe328f3af365a4b13f7df0a7c3d0fc266abd2b472931e0c833c`</small>
