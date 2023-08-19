# Download and Install

Latest **v2.21.0** release `2023-08-19` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.21.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.21.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `cf81a168cf60131e347ad583d64441e6ca0b218e93c8753e5bda5b172d16faa4`</small>
- [static-web-server-v2.21.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `1f2f81dab6ff88e47c4afa9c4f89b2ebc76dfc938075a2f4eb1242ba149ff071`</small>
- [static-web-server-v2.21.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `6b231fa2886d926fee186353b6c2b00dd60b535a19196e764899fa473655b383`</small>
- [static-web-server-v2.21.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `e7363d00608f17726784a6d729a1fb600b249b1c90e3c37771e4a0bd64dea5ef`</small>
- [static-web-server-v2.21.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `d6a7ac80a67502598031b2e0d6467be7dc5d1da16cbd6f082105f158c82e75c3`</small>
- [static-web-server-v2.21.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `15f44d0a0f82fb1a21aa15c0788a06a61aaf8b7ebd3a7587e786d39881a25b56`</small>
- [static-web-server-v2.21.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `872c7414e90b1df0ed81bb8cf84ef2178551639406c7110e66e4868f5ddb7eab`</small>

### ARM64

- [static-web-server-v2.21.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `aa770b3c1ae686c39ce390f2da09d8825e708e0beb8b10013bdfdd410a410908`</small>
- [static-web-server-v2.21.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `a89e0dad0e347a950499239fcd6ff913380d02c7fb9bb68f32ec4ece384ece8b`</small>
- [static-web-server-v2.21.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `96044177b461d187c638a1b7bca5c0502b3cfba4116271c2ddd433af87764b98`</small>
- [static-web-server-v2.21.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `a1f770d620f546331c34bbd30b6ab103796ae32663ddd219366a3f6c47b8ba1b`</small>

### x86

- [static-web-server-v2.21.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5fc87847c41d531b6271ee1373b42882488a23968f43ac5aee65f337b331fae9`</small>
- [static-web-server-v2.21.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `f3b5c961238540d298b2efc788e2fdb033f668dcd8b7fc0bb23b7e5dc8a23ee3`</small>
- [static-web-server-v2.21.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `3651dc2e042346bc3b393fdde99839687c19614c7e8797966a38380d7c208a2b`</small>
- [static-web-server-v2.21.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `930a6bfb78d22a66fae81d1d6fab35c4e15060f51a3ac08cba84695226902134`</small>

### ARM

- [static-web-server-v2.21.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `2de4dbc61806850727f9eb47c73631d82bac1c5f3176c1bf04f4c0dd2ad426d9`</small>
- [static-web-server-v2.21.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `399cc66a26c341aee30eebcae8fc15898cf398c9b59e2ff27ab8053bdd5fe496`</small>
- [static-web-server-v2.21.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.0/static-web-server-v2.21.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `b57cb6c6a6d381ac1fac2c198bf3f3a54b2bf191e98a01af18a8b4c8370176d2`</small>

## Source files

- [static-web-server-2.21.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.21.0.tar.gz)<br>
<small>**SHA256SUM:** `ac78e9adb9bda0619db5f0bb7416d7931c16c03b0b2e14c07c574d2babd24d7c`</small>
- [static-web-server-2.21.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.21.0.zip)<br>
<small>**SHA256SUM:** `10d612865edd9b56282893bc1aaebadbcb5e87d2d6b68dffc818209a4984c31f`</small>
