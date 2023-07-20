# Download and Install

Latest **v2.20.1** release `2023-07-21` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.20.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.20.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `107af7e3286dd0ccf2b283450060d18fbd6b7b5902b96d4623cedd6c0cca8fb3`</small>
- [static-web-server-v2.20.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `14dc76fb702acb9e5a635e1947036a60952654200eab4409c57d08f128c7f321`</small>
- [static-web-server-v2.20.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `7c1a21434780941b5b42c593b60babdda571a701473f618c1b22cb0fb7d73fcc`</small>
- [static-web-server-v2.20.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `440f5c471f84b9da9fb5c8916e6d0aa3fc9b055fcbebbcea9e928068030870c1`</small>
- [static-web-server-v2.20.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `ae802caea701b828aaaef6bc9655e4484d419cf0dcfd3127f5acaf332034c12d`</small>
- [static-web-server-v2.20.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `54b771f9c0b53c7185a3e41af81b7fca18eaf7a6fd4f7123c79cc6e6fca48777`</small>

### ARM64

- [static-web-server-v2.20.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `b49067ec59238ee7f20707b727a0610312854945bb642b93d30fea9f332ee7ca`</small>
- [static-web-server-v2.20.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `ca5b76ac8c712bc3e02d9d252e6b7c829c010aaeab6a2fa145bd276175b96ba6`</small>
- [static-web-server-v2.20.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `2a7ef3c08f9e20541e2036ab60dab4af8063782eaffd208e0ba78c4e8c46ffbf`</small>
- [static-web-server-v2.20.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `93831cbaa954fabdef763eebe58f1efb9d715f95fd966a67200df6cae9cf604e`</small>

### x86

- [static-web-server-v2.20.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `52a84e18b3a0085ccc7bb1e79ccf554e239e9e5183715aefa81d49c8c3ea0795`</small>
- [static-web-server-v2.20.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `0cd0b00338d037917ec1215e23d903f857639bff04273061224e3ec35dd9fd1b`</small>
- [static-web-server-v2.20.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `76f3ba8ff5246b89fcd024bd393c87185054e487e0b586aa98e007420bd60e5f`</small>
- [static-web-server-v2.20.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `f1454ac4432af2a58518639bf5a411cddbf8709062d75b436aaf5104e307d2e0`</small>

### ARM

- [static-web-server-v2.20.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `617c5f789c1bd1aab1b28f28d95fc9821a0585c6a49c0f90ff1c2d46fcb9d683`</small>
- [static-web-server-v2.20.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `257152b0faaf698a72c7d937fa252afbaa225033d2a0d8c4c88685304bfb1bd6`</small>
- [static-web-server-v2.20.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.20.1/static-web-server-v2.20.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `fb881c78accefe6b90856748be602cd40b9680381283c388a2d5e70ec4a8aa37`</small>

## Source files

- [static-web-server-2.20.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.20.1.tar.gz)<br>
<small>**SHA256SUM:** `9017890135fe11139b96e0b62dba2d8fcabf1db9f89daa2e39a0d206b9301ee5`</small>
- [static-web-server-2.20.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.20.1.zip)<br>
<small>**SHA256SUM:** `051841963f11bf4452d99f6bf672c949872caa55b07cc17483589f31f490b23b`</small>
