# Download and Install

Latest **v2.22.0** release `2023-09-18` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.22.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

</div>

See also [the release history](https://github.com/static-web-server/static-web-server/releases) on GitHub.

!!! info "Docker"
    If you are working with Docker containers then check out [the Docker feature page](https://static-web-server.net/features/docker/).

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

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.22.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `f0902d9416e924d89710ea41f83d4d642b2d0828db3950bc9cd2d59071b2943d`</small>
- [static-web-server-v2.22.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `e4db700aeed55d32ae3ef072ce0c75334538020094eb6cadec1a223ef95fc388`</small>
- [static-web-server-v2.22.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `dc919429acc439a8a7f1b130497d843b3b3688a7ad063e893652fd93262ae1e4`</small>
- [static-web-server-v2.22.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `c15733a762cf547aa9416de232694d4ae41467e53dbbb43cab80fa63ebdce436`</small>
- [static-web-server-v2.22.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `aa1a06b1f2ea8a8afcd3c1e2b23304b5d26af1855022b3a5d7d2a90c43159669`</small>
- [static-web-server-v2.22.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `024445aed86bbf4c5c1550dcf8be7d45811e0de2596837f4a436d4054c58626e`</small>
- [static-web-server-v2.22.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `2e3afd251504189278d8e3f9b8c23697779eeb39adde4882f273cd13dc7c0b52`</small>
- [static-web-server-v2.22.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `b774580d834f456bc1085f0654a33119b65f970a3873b9ee4538833f6e93577a`</small>

### ARM64

- [static-web-server-v2.22.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f0fc2617eb31ffd4040fabd26312fd12e93b05f7d8e59ecddaeff41a922d7f6e`</small>
- [static-web-server-v2.22.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `c30e20cfc36f92acb6afb93a54585a5981e903aa815eecc9140f1aebc9752165`</small>
- [static-web-server-v2.22.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `42f058700d7a8e1cecb53e597251e49876644f254c371a06eda05edd107fc54d`</small>
- [static-web-server-v2.22.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `2a09c773d1884bd630a49ba06a160e83be44cb3e4e33b841b9516faa011a98a3`</small>

### x86

- [static-web-server-v2.22.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `607249a9245a67e8bd9f709b33862f774768664f76a9e381acb67d2e276d0430`</small>
- [static-web-server-v2.22.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `4dc6ada52e06167fda7d7021f3727b4b96551b85cfbf7a7bad81d64aafd57092`</small>
- [static-web-server-v2.22.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `e22cfdf35df2043b69b2f818e0979776b30ef3439fea176e98260c8aefd74b81`</small>
- [static-web-server-v2.22.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `323037d60c6cf380a11c4f7f117340292267ff21e047d1c0be71bd980c013b86`</small>

### ARM

- [static-web-server-v2.22.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `92327c6d82fb4e356af03f5c89a37aa0b5cbffcfcf212d441abea23b4dc97dfe`</small>
- [static-web-server-v2.22.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `496e99e68dbd72f427bb98480570c957d25a5beb4c7000a2ee6bf2b29599d1f5`</small>
- [static-web-server-v2.22.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.0/static-web-server-v2.22.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `b53d9f765dfb1ce01c48a36ca1b9744909806fd30d323cadb2ec6993a7c6e657`</small>

## Source files

- [static-web-server-2.22.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.22.0.tar.gz)<br>
<small>**SHA256SUM:** `427c4bece10ee27401e1bdbb64e5296da48d9c5ed52277de48cb0cd6679db1d9`</small>
- [static-web-server-2.22.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.22.0.zip)<br>
<small>**SHA256SUM:** `391631b17d87ebf0281708c7327946544d3da08ab3f8a4e0b1665af3052abc50`</small>
