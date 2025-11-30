<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.40.0** release `2025-11-30` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.40.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.40.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.40.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `10ba00ac903419eb566f32394659f82dc3f05e3a8275eb256ca324ebcb7ffe67`</small>
- [static-web-server-v2.40.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `081b17890e3f489619e0a98f73bcb26ae8653c9f43fc88bb72c00ec412452302`</small>
- [static-web-server-v2.40.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `4bc31a2f2d43ea31bef6abe516671f2dfacdfbf8fe5844c5d5c82f0fd6bdb9a8`</small>
- [static-web-server-v2.40.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `a2ba25fb17e2728169d818980e411bd0792e6460f14f0eb340141736e6368fdb`</small>
- [static-web-server-v2.40.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `bb7adb712c2980b541e95db93ab5834f7b839c05773175dea372c9dbbb27e37d`</small>
- [static-web-server-v2.40.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `ec54fa4addecf437f47121fb58604cc6ab448e070530466de2b33ce708164b83`</small>
- [static-web-server-v2.40.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `89f07ad42bbbdfd5b14346a8903545d4aa02ce4800ab2ab7e4e488f1885f3e5f`</small>
- [static-web-server-v2.40.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `caa2e6542fcebcb35ef92976d2f6ea3a190da966a0d7c62e39acf71ee28cfb9b`</small>

### ARM64

- [static-web-server-v2.40.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `4b680d9a491b85d02d0866f3cce0cc722fc79c6630983c7ce0303ce4c35d1849`</small>
- [static-web-server-v2.40.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `40e0e2c9328eac0ba0d8f985b81e7a29b77c7e6307f6333175b46f70f87a6922`</small>
- [static-web-server-v2.40.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `739941c316c699c836f120d5654b41a2afdabb936d4e54ade985c13146c3aee9`</small>
- [static-web-server-v2.40.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `71531c439ebeb3af94b34954053868295b234d9e76ea06f7f60a6a236ff5ab0c`</small>
- [static-web-server-v2.40.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `f7f3152ff9b2e876c9afb258563f5c1442438fbc6a317a5cb6f109750410052f`</small>

### x86

- [static-web-server-v2.40.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `08a5055ab9081ecabf088364d5c57fa5ec0dac658ab51171f589435c667cee39`</small>
- [static-web-server-v2.40.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `05f0e3d639afe124750f95079fe7ad3217ef1c48e661eed5362a4e3dc83815f6`</small>
- [static-web-server-v2.40.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `9cf208559a0020a653a1d76210dcf307b20af53e0b0d9f0e6176ccf686ce80ae`</small>
- [static-web-server-v2.40.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `ae3f3b39637dafc7ac0f6b61edb5a8f5a4dbbca6e1642e41979a1d24a3799fe0`</small>

### ARM

- [static-web-server-v2.40.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `21f01ef4db1907b2526f5467ee31181a03ff8b724d332c93dd5c9fbb1e8e8d48`</small>
- [static-web-server-v2.40.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `e5c396df9576c7b28cb438c8d7435dae5bf19b4672c5ab17074b0d72d9d243ed`</small>
- [static-web-server-v2.40.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `20aace18796a10505dbeb953625b217c55b9387e02371352c7f3fad5b1aa6937`</small>
- [static-web-server-v2.40.0-armv7-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-armv7-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `6f613c09c252a2f08347027051bc4ddf8f6361da58cde94c851070934c44f77e`</small>

### PowerPC

- [static-web-server-v2.40.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f5d5cee11fdd4e5a9edc4cde69ed28aaf3722d2dc6ea40f0e446229ef3f829c3`</small>

### S390X

- [static-web-server-v2.40.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.0/static-web-server-v2.40.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f0c1756447ec77fd64bf385a75e7c9e361c8125eca402ec477124bb395b6933d`</small>

## Source files

- [static-web-server-2.40.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.40.0.tar.gz)<br>
<small>**SHA256SUM:** `47e6ec28b23429cbd8d9f378895e9d38b9b71a3f3fa243cd6023b1b76466e186`</small>
- [static-web-server-2.40.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.40.0.zip)<br>
<small>**SHA256SUM:** `7a18f83e635636f311552fadf349bf5c54291ec573c8a81fe2e88f749d0f7ac0`</small>
