# Download and Install

Latest **v2.22.1** release `2023-09-19` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.22.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

### TrueNAS SCALE

If you use [TrueNAS SCALE](https://www.truenas.com/truenas-scale/) then visit [TrueCharts Community Website](https://truecharts.org/charts/stable/static-web-server/) and its [Introduction to SCALE](https://truecharts.org/manual/SCALE/guides/scale-intro) page to install SWS application in your instance.  

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.22.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `b8fbe4ffddc118819743dce6c4a18b6ceb53685959b96559f963db50921352d3`</small>
- [static-web-server-v2.22.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `816a97b6d5a845682c145a9d773146ea5398e3ebc5ca8c2cffa2b2725c1aef75`</small>
- [static-web-server-v2.22.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `44a84f71caada050a77a756c1e403aead1adfb33d26ba8bddfa28014c631f6f6`</small>
- [static-web-server-v2.22.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `a0f83990a26d89915c5eff7ef8c7751a39024de1725b78330d33b5ec19a8aa2c`</small>
- [static-web-server-v2.22.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `8ed958e2100efc0ae5ef24399b3eed6ad74f0c2391a8ba6ba490e18192569a44`</small>
- [static-web-server-v2.22.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `067cb6672ed417a2b43f983fabe9bdb06d2868f6edacfbf19ab970a81a3b83fd`</small>
- [static-web-server-v2.22.1-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `e65b4702151273845986c95e0200ab4ea3d1930a0b0792a1ddadd251bffde6a5`</small>
- [static-web-server-v2.22.1-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `a50f0e042344da82a7321048303130731025d531eda2ad03e1e7f4a72bba4c8c`</small>

### ARM64

- [static-web-server-v2.22.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `9454dd628bf5c8e760537663a9a33904b4f3b3f0e9d5e683d35a35d84e37032f`</small>
- [static-web-server-v2.22.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `b9575e8310529f3659cea3302ed3cc5c23beb6a255ef784c151b316b823bbd50`</small>
- [static-web-server-v2.22.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `bdb43077d68f4e5bcbeadbacfb095ab9bb29ae72bd70118c6248bab86be04ece`</small>
- [static-web-server-v2.22.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `b5b342a4834415992c9333825c45db77fef17d69d885ca8a149d855d0253e064`</small>

### x86

- [static-web-server-v2.22.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `b5c52eb7064e7e36aa67ebb257cf79fe64a34abefdea45141143a424b70ad6d5`</small>
- [static-web-server-v2.22.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `e31885364b8c3e35c3122c4a812e3f82c4aacae7ef740f2cd4a406393a8e8e62`</small>
- [static-web-server-v2.22.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `914f851692a0d5faa59c0362cf018a97c508c85ad07b52579fbb079fc71237e7`</small>
- [static-web-server-v2.22.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `2b57db1bdf47579856bd9d81ba2dd287befd6588db5b7d644bd656cd2779330a`</small>

### ARM

- [static-web-server-v2.22.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `53b567dddccc52ff820df056a16fe21f78afadfca44ac789e62d9b77e79058e1`</small>
- [static-web-server-v2.22.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `b80eef0219bd6e7cb17a8e127aa6a9af856a4251e6211078806356f2cdb289ee`</small>
- [static-web-server-v2.22.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.22.1/static-web-server-v2.22.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `1bb03e4d30697a4e82df0f33df175ee16ab38872100d88f46384a7cb7f5b4179`</small>

## Source files

- [static-web-server-2.22.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.22.1.tar.gz)<br>
<small>**SHA256SUM:** `d618e83732c73e7466b4f398532d5acae45565bb73d7a66fdd5c3e7480d242d9`</small>
- [static-web-server-2.22.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.22.1.zip)<br>
<small>**SHA256SUM:** `a60a17df5187f52d338255b038b3ec501d2324d9872728c6517f206d795a5006`</small>
