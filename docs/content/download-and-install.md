<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.37.0** release `2025-06-03` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.37.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.37.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.37.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `949941de6157ea88cc01de0346255379ed398ea5a106e19c6c73e2c8e4efecfb`</small>
- [static-web-server-v2.37.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `938f24f06c36fe476b21687fa4ba174e0625f54494ee2d7eb1c2f1a5ad908820`</small>
- [static-web-server-v2.37.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `e2d96344eea55e6223239b5eb3c5d5adee0e3830c845a2bdeda30e06b493bb24`</small>
- [static-web-server-v2.37.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `fc932363a7f8260bb059294729919e896069626468649b9053e7d223a76e1534`</small>
- [static-web-server-v2.37.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0d41d3d4ca0a99589836188c7942c66a511281dadcd85d2b03201d282919a51f`</small>
- [static-web-server-v2.37.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `7f29b190113277791e005680868701ed872a372f01dd4592dae5e09a0dd5a103`</small>
- [static-web-server-v2.37.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `f520f521dbfb3bb6899b16e82c0882f0351144259bcf5d4962254152cded5f12`</small>
- [static-web-server-v2.37.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `4337dc5f6c118a0b3b4d788237c853ae542c0c0677fd075fabfde1f0fa739e2e`</small>

### ARM64

- [static-web-server-v2.37.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `d0a0adaaf2532c9e66da9be15b2914d13ae7ce8435dd532c7df4bc5a5cbb3aa1`</small>
- [static-web-server-v2.37.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `900f64758807cb30a665597793828f578648ae24aa7b86b60e7cea769cc6ef03`</small>
- [static-web-server-v2.37.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `70b9213806e8210d2fd49485a06f16c7f9e6aeeacb2ea12d0b31154ef7cc378f`</small>
- [static-web-server-v2.37.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `6669568b2bddadda1037347d8f391d0ca2ffd9dbd6b219ee68ea7a2f684b6ab5`</small>
- [static-web-server-v2.37.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `8a4fafb301e38a95f9c83935571218d5980976928650fb33d92892e75c335ae2`</small>

### x86

- [static-web-server-v2.37.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `0c27cd37c4facbccf5b004d1bc30788ab7493b813cefe17ef503f3d76b0337d0`</small>
- [static-web-server-v2.37.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `932f0b91761fdac9635a61a1ae13bd070aeb33be4633736b4f48f8794cec290e`</small>
- [static-web-server-v2.37.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `c4fb4ece12c39513e26aa55d4c34f7eec9e6eb60df2c9fd5b712d1329f57446d`</small>
- [static-web-server-v2.37.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `cd753e4d833dee695c27d784b0bb69b8737a4441b55d22c19e66d75354b6c247`</small>

### ARM

- [static-web-server-v2.37.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `953209c46d3459ac419b8b2e75d975e60763b59dff72f38acf4849a5ed350e45`</small>
- [static-web-server-v2.37.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `85a4ce98b9e1bdd5d4a5d84df981b86fedbc67b6010ffdb6b0fb20ffce86cb76`</small>
- [static-web-server-v2.37.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `fa60ba4658fe7ec5c02c993f9ccc40149a125821b12d0359caaf8dccc172478b`</small>

### PowerPC

- [static-web-server-v2.37.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `d121a2ed9ab786366b2ea3358c7b50b5882e65cb9d2fdbc79340d8ee2e0e88cd`</small>

### S390X

- [static-web-server-v2.37.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.37.0/static-web-server-v2.37.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `4d89786f651673680b029008be0cc6784a731af830d36b57ca223b8ceb1b4efb`</small>

## Source files

- [static-web-server-2.37.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.37.0.tar.gz)<br>
<small>**SHA256SUM:** `596444e276dc912b5ae0223cad15fc9d700b66a6e466b8904175f3f7f5546b64`</small>
- [static-web-server-2.37.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.37.0.zip)<br>
<small>**SHA256SUM:** `8ff1ba516a5f30ed725c8dcfd2264eef7314de5b4231db3782a5b9f581ce8767`</small>
