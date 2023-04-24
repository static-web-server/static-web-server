# Download and Install

Latest **v2.16.0** release `2023-04-25` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.16.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

</div>

See also [the release history](https://github.com/static-web-server/static-web-server/releases) on GitHub.

!!! info "Docker"
    If you are working with Docker containers then check out [the Docker page](https://static-web-server.net/features/docker/).

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
nix-env -iA nixpkgs.static-web-server
```

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

- [static-web-server-v2.16.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `3879ad279fa27924d04973ae8a41ac4374d1e57100bb3bb92ab94dcb4b0caaee`</small>
- [static-web-server-v2.16.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `f37e3c6117116c39df7c09e29e8ccbaf26ee943ecba2a4e25f9fcaaeb87615ac`</small>
- [static-web-server-v2.16.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `a2b65097fd1c2cfa60b53936e41019aea093b969466d1b389cac25549f9525e2`</small>
- [static-web-server-v2.16.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `0dfd61fdb99ab9d6ad95e007618847b0366bfff98b6d46919a9787fda8ce6e50`</small>
- [static-web-server-v2.16.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `963dafa87b703e4d493da81dc55a895ca36e12dbf9ed1e81673d34fa519e1f56`</small>
- [static-web-server-v2.16.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `ecf5a2cee8526f199ff665169bbd2ae191b021a6565cd0c72be784d2c413e127`</small>

### ARM64

- [static-web-server-v2.16.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `201c841f80b7c0933b0f4a575f536b8e3c25c905bea98107c49e1514ceb1aead`</small>
- [static-web-server-v2.16.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `2afdff091fe334481750d858f5e4b0f1ec89bbfd8d517729af995e6c6c2bb757`</small>
- [static-web-server-v2.16.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `d3ae3106aed1cef02b528ce98b5e4fdcc71681b4da2334b484b4f62d10fbb873`</small>
- [static-web-server-v2.16.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `af98780a3d5632c6f4a59b5ee88b7286df59d00032507839bc482969583e3575`</small>

### x86

- [static-web-server-v2.16.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `1536e0b877b5514d3f76fcadfd7fa6daee5ed11c2d69d2a58b771f6e8fbc094f`</small>
- [static-web-server-v2.16.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `bc804914c6b559c29980b1820ad598dbe47262f177b158304a3d811c81df5d08`</small>
- [static-web-server-v2.16.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `b76c0ec0f808e79636cb2d298e3ef0770d4180d9ea8ec10a8e6375f2efaacb14`</small>
- [static-web-server-v2.16.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `c8c929cd1b4fa0cf4c736d37f7843591ffa5dd7d260d564400fb90299cd237fc`</small>

### ARM

- [static-web-server-v2.16.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `ffaea591dd1228e9255dd2bd10d2ceb5a3c863447d88577ad963012a9ed47af2`</small>
- [static-web-server-v2.16.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `2e33a63b741570c56eddaec273bbd6152890c430a8801034bee0aa761e5260a0`</small>
- [static-web-server-v2.16.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.16.0/static-web-server-v2.16.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `497ef2dfc2752cd0f929ea947b1e3848e58078348b4e264981498935c8fa339e`</small>

## Source files

- [static-web-server-2.16.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.16.0.tar.gz)<br>
<small>**SHA256SUM:** `51f994111a10c3e86ab48a3c1bf5024901386d11962aca5f3c759eeb0c853844`</small>
- [static-web-server-2.16.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.16.0.zip)<br>
<small>**SHA256SUM:** `d4287732623117d8ae04a448f54cb6e56090f5110dadb9adc91e7f4c558f639c`</small>
