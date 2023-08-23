# Download and Install

Latest **v2.21.1** release `2023-08-23` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.21.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.21.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `0e9ca5c8c60f374d3f4cdb6b6778d3a44b657f9ec86e30187b5803c788f4be16`</small>
- [static-web-server-v2.21.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `2aef4e5ec1b45ba18c1d4aeac84c25f99c628fffd7a8821d6351a983bbe1cce5`</small>
- [static-web-server-v2.21.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `0409d60dec536b0e7a24722bb96f9b59ae80ee4b6ba6fd3ad587a6fb93b4dc42`</small>
- [static-web-server-v2.21.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `51594ce6da5499fc22b46b4fc08a80191d369fd2a534a4986a4c44be52568510`</small>
- [static-web-server-v2.21.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `6b356e69d9393e0451d9782937b17216b4f438afd4ee383441474a68827213f2`</small>
- [static-web-server-v2.21.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `e2cf00399512c42995ee11693981c529bebd42c34f30f6ea0ad30a852cbb5835`</small>
- [static-web-server-v2.21.1-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `40dd0fa3b1b07035c92d13bbce1cee980a5a60272bb41a723b252f054344a750`</small>

### ARM64

- [static-web-server-v2.21.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `874e23b9da1757da81be4dda192e2961f77d738e78d4a6d5958ef07065738b01`</small>
- [static-web-server-v2.21.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `d1f1ec8cd6f9a6199bf5cfa07dcd7ff16672a723ee03227ceebabbfe0c957552`</small>
- [static-web-server-v2.21.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `7e8a46283b2a31c771513203cebc1c1d9b642cf3365ff14d3169d3dbb39ba3aa`</small>
- [static-web-server-v2.21.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `a93bade03afe54be6da74b00e847592757f30a37fbabc49a1beef06ba3285202`</small>

### x86

- [static-web-server-v2.21.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `4f85196e0170c2159b7939e3dbc197ab0dbed8964db4e684846e82084a81f883`</small>
- [static-web-server-v2.21.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `77c1bc90c49c4eeb5dff2cc316cd1b13305bb32a677a5bdc823b6d7ca16d4c0d`</small>
- [static-web-server-v2.21.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `facaf610c564db28ebc82c92d1d058c75665c01216c47a15478e7f97d6f1a01f`</small>
- [static-web-server-v2.21.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `6afa871428d6ff86b6eef942094e2d8f3aaec05481900f4b7d191672f1cd592f`</small>

### ARM

- [static-web-server-v2.21.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `fe170cde6bed13c410de4acd71f9d92ad6b7b4ab95ffe5d3af94ca9a44514dde`</small>
- [static-web-server-v2.21.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `88f66dfc6ef7b4e800033cf74656b4c96276c527e953eaa5e4ff0cf3645b9a3d`</small>
- [static-web-server-v2.21.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.21.1/static-web-server-v2.21.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `9be4ffad864da1fd96afd1dad7d997cf5e231a8ed0096c372e33b2a497f4d4dd`</small>

## Source files

- [static-web-server-2.21.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.21.1.tar.gz)<br>
<small>**SHA256SUM:** `fd704637d97602c8112064d139f10c6abc1cb604018ace9828e5105ab8987dc5`</small>
- [static-web-server-2.21.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.21.1.zip)<br>
<small>**SHA256SUM:** `8c0e4b5699694276a65cab296a992b332dfdade4fee18b3b30c1b1a45c5b8414`</small>
