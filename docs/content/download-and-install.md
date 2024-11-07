<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.33.1** release `2024-11-02` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.33.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.33.1" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.33.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `ca8499065c6b5fbe3c7bbe69dfb02eed27d2779a38c43373f916f41e2188240b`</small>
- [static-web-server-v2.33.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `bdcd36e1075850feef581d32f113a4e36aa4c74772fb47141b0f437cf17a48ab`</small>
- [static-web-server-v2.33.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `dd16fe2f1cd18b05c290ef46477f58757d03ccd2d1fa2f1b34d2196e3b8a639a`</small>
- [static-web-server-v2.33.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `abb81d3fce9232abc48cee03a69e56b40feeffa93ba1225d93c433dcfb1f36aa`</small>
- [static-web-server-v2.33.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `9a1cbad49f7cb5499e7411d764e5b014e7cdc5f6abad29b11bd8a22b808dce5b`</small>
- [static-web-server-v2.33.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `9ada6578e8a3992925c6c5cac729c11c6a10be550599167f40c57d9953db2712`</small>
- [static-web-server-v2.33.1-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `dff0426f2af17a9455116345fcd5073f8448fceec81e3a301812db8e892f7836`</small>
- [static-web-server-v2.33.1-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `ba5f2096d71cdc062a814251728d16b6a97643cc79e2273fe3a1c32a9d048140`</small>

### ARM64

- [static-web-server-v2.33.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `6b7b1fdcd521f9e7a85c3aa4e9fc6cee0f54e8055e7e4dc8e148fe15fc9843ee`</small>
- [static-web-server-v2.33.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `7ce44c4c0875a831def185c44973bbfed9cb92adc7b2eacf717687fa1cb696c2`</small>
- [static-web-server-v2.33.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `33c0fb54a6d88abc496a441c2822d60a498cdf1f3da0ce36beefccae071e4ba0`</small>
- [static-web-server-v2.33.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `26e47cb2df989f9252743128b40a62191d7ffb1207ebda204d51bb9384e18b0f`</small>
- [static-web-server-v2.33.1-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `26622a404447706308fcaf63f7342a30b861cf9d4089f9c3816c7b820c8160ba`</small>

### x86

- [static-web-server-v2.33.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `135cf2315c4dc3700be66f3b7f8c15e0dff7b1344d403e42bff6e6630d9c443e`</small>
- [static-web-server-v2.33.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `b14e3c30ee7b665e2766e0352533ae5758949737449d52b1ed8815e4b5dcac9d`</small>
- [static-web-server-v2.33.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `00775b48c71e9953de814681256cef7fe413d9960d613aefafff090fe78b08fc`</small>
- [static-web-server-v2.33.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `83b8ddf0c8bc562b06df71537aa4fbf01a3258c45bf6985b0e6fcc0aa873de28`</small>

### ARM

- [static-web-server-v2.33.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `032d79f06dfcb549ce1d9a823e8072c622862c44114eb77236dd44fce1efa7e8`</small>
- [static-web-server-v2.33.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `75dcefb85ee98f08e2297a9b014e900c12e67cffccb21ac1b792969a4f3e734c`</small>
- [static-web-server-v2.33.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `b9a75022c2559132d3c50ae0ee5b9b52401f5c4e9870195c411a342b1900f893`</small>

### PowerPC

- [static-web-server-v2.33.1-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `d005c2762c0c7f7f22a80a2d0577755157063232e920f05021edf8bd393a8b1c`</small>

### S390X

- [static-web-server-v2.33.1-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.33.1/static-web-server-v2.33.1-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `41e8be6e69da11111875e8fb948ced513a926712740e4e1a299784b1e87cc679`</small>

## Source files

- [static-web-server-2.33.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.33.1.tar.gz)<br>
<small>**SHA256SUM:** `f23865f4245070f7ec3a549c0dc527f2d73ef23f6fc518f3b568772eaa021b6c`</small>
- [static-web-server-2.33.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.33.1.zip)<br>
<small>**SHA256SUM:** `4f3844fb8d203a305eb2be20ae86029a06769c74856d70f615a35dbed5009c17`</small>
