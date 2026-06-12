<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.43.0** release `2026-06-11` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.43.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.43.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

Pre-compiled binaries are grouped by CPU architectures or features, depending on the case.

### x86_64

- [static-web-server-v2.43.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `3166370de48ef9f32a1f4919873b0f45a3f1d15b3b82b9a165ea4b7cd9f245cc`</small>
- [static-web-server-v2.43.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `ee28f517b8e3e62e9954f178384e8e231df24a886fe97155a65d96030661b97b`</small>
- [static-web-server-v2.43.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `b048aab9891eeffc4c3675d89c94d454d97e0f7fa7df1c64fe26b92e300f2f26`</small>
- [static-web-server-v2.43.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `b99f5ec372d9302769ded1e91c800e9a6627f3c7c71072c528b6a61b117a48a6`</small>
- [static-web-server-v2.43.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `da3c4e599d6bb2bf6d3aebb2a97c7ebdae842896cadafec97792b96a595bf5aa`</small>
- [static-web-server-v2.43.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `dd5ef20ce9562e689eb2bd84cae50c34114a7627aef745a60a48bd2b3c820823`</small>
- [static-web-server-v2.43.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `b2f15c6737db0b456278a3b629d36c23e38b282415964583f19a4ffe32e59d5b`</small>
- [static-web-server-v2.43.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `f13c671d02587f4a5f4151ac0c4538a77b98ee1988e3ce9013cabc3dda4d3c5f`</small>

#### FIPS

- [static-web-server-v2.43.0-x86_64-unknown-linux-gnu-fips.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-unknown-linux-gnu-fips.tar.gz)<br>
<small>**SHA256SUM:** `232b9136c6c5b72ffaf941dc39531aad6ddad7c2abcdb8b63732ed487f9ec095`</small>
- [static-web-server-v2.43.0-x86_64-unknown-linux-musl-fips.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-x86_64-unknown-linux-musl-fips.tar.gz)<br>
<small>**SHA256SUM:** `8b6332d8d7f014c4b7ebb17cd80929b6da2ce310acf05b55bb94ff0727bef890`</small>

### ARM64

- [static-web-server-v2.43.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `6aeca9e23580a187c043daef9c6bf8c12b80bb8d75ed84a23beead2cd193a20d`</small>
- [static-web-server-v2.43.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `5f6765107b903c5ea64b32818c7d2650623238bc3b6539195a37a7fcf835cab8`</small>
- [static-web-server-v2.43.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `63b1e782eb7c743f2cbbbf8be64c615eba757661e11da833d4987abd597bbc9c`</small>
- [static-web-server-v2.43.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `5606d0dd9903ff3b793bcca3014d12c0e1a1438b1bdd649769b7f2f0877d51da`</small>
- [static-web-server-v2.43.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `492b1a3abf65c7ed57b0954796ee942f72c37a0a34c3e55c6809390815710fc3`</small>

#### FIPS

- [static-web-server-v2.43.0-aarch64-unknown-linux-gnu-fips.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-aarch64-unknown-linux-gnu-fips.tar.gz)<br>
<small>**SHA256SUM:** `88b59e644466285fdeedc5a30fa10a7bdd4778293f52d0b022a5db0134362d8d`</small>
- [static-web-server-v2.43.0-aarch64-unknown-linux-musl-fips.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-aarch64-unknown-linux-musl-fips.tar.gz)<br>
<small>**SHA256SUM:** `f5cc3917af321b54092438f8ff694c191eb1a9cc863d58d981566c5f1e20b2ee`</small>

### x86

- [static-web-server-v2.43.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `555fde4a81fb04d766a4de142d02dcd7fb18847efaf95485103f2d84ae140224`</small>
- [static-web-server-v2.43.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `9caf55de628668c561c7c86837150e625f86465160f9384c210e54eefdd230cd`</small>
- [static-web-server-v2.43.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `15300f78f554f2b245a3a277f7bbbbc0f8e4455e9f8671a16c9d30b3d6f5392f`</small>
- [static-web-server-v2.43.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `76c6f9fbf9166fd4bffef4aa2a475d4d4407e94e3ebebdbeb3f0d5c629518e8b`</small>

### ARM

- [static-web-server-v2.43.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `595f15f44f88bd6f61c7d905e81ac751a0008208e2b2b67f5d3b0fe912905185`</small>
- [static-web-server-v2.43.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `4d217f7e227b30e0f5045f44be0a473125c349aaa5f275d2a68283d24fc46618`</small>
- [static-web-server-v2.43.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `6022b6fc90fd473bef3455f01b908d1b15467f28ed1fa22c2436b059b18f05ce`</small>
- [static-web-server-v2.43.0-armv7-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-armv7-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `afe1f4d694a89ce67594443a9cdc16c8a1c03ef9a9e82b019581a0fa718a3490`</small>

### PowerPC

- [static-web-server-v2.43.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `060d9c825e803c8219c4a804fedfd54ecb082abef4b112bba43700f4da77f245`</small>

### S390X

- [static-web-server-v2.43.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.43.0/static-web-server-v2.43.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `e6e1873dad8b5bffa61a6a93674ffff396eadfb83d7754a5c2b299514366e25b`</small>

## Source files

- [static-web-server-2.43.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.43.0.tar.gz)<br>
<small>**SHA256SUM:** `bc88f3bf22fceab1eb49f8a81277f4d73348849fab7376fb746607e0063f0a73`</small>
- [static-web-server-2.43.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.43.0.zip)<br>
<small>**SHA256SUM:** `38da92fe3cfef41d370815e52275cb6e0e195e4515e2b364e9c1f593d5f85e95`</small>
