# Download and Install

Latest **v2.24.1** release `2023-11-15` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.24.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

### WebAssembly

Via [Wasmer](https://wasmer.io/wasmer/static-web-server/)

```sh
wasmer run wasmer/static-web-server --net --enable-threads --mapdir /public:/my/host/dir -- --port 8787
```

### TrueNAS SCALE

If you use [TrueNAS SCALE](https://www.truenas.com/truenas-scale/) then visit [TrueCharts Community Website](https://truecharts.org/charts/stable/static-web-server/) and its [Introduction to SCALE](https://truecharts.org/manual/SCALE/guides/scale-intro) page to install SWS application in your instance.  

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.24.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `1e04d2db56a0ac98921d3f4cad444d6442a36df71af102bba4b571b672d2463e`</small>
- [static-web-server-v2.24.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `cbbd76464b023be1efe75cb41a3533e50ed88fefbf5c8320242cce01bfe85649`</small>
- [static-web-server-v2.24.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `e740945667772611316a92fb3a0035e369e27a5bb73c104b5c4748a146805037`</small>
- [static-web-server-v2.24.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `fb7b9a4d9b4cefd967700f47575b2f08bddd481d2c4f28131cdecc89bc752982`</small>
- [static-web-server-v2.24.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `b79f24feea19a37eeaa5f64d0e5be5d2e174770b7a858392680d226a1d4db047`</small>
- [static-web-server-v2.24.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `a4610bc2b90e42e97f38a6f4aa1f8a6e32c89924fba45f287e1e9cb8e23670b6`</small>
- [static-web-server-v2.24.1-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `4e66a2a796b5afbcb78910fca84e14b70f0a906e2c461a3bc63bd2ed702789e9`</small>
- [static-web-server-v2.24.1-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `344ddf99552fc6261da3035363efd8561d5cd951ed1818e4aa9896ddcff7c28d`</small>

### ARM64

- [static-web-server-v2.24.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `bf99a4e57a009ee19d523ed4d8b4316ec6b635aa81c48e1644439d9f8f25ab38`</small>
- [static-web-server-v2.24.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `a005d8174219080f2cbac08214319f4566a2040066cc9c9d1c212013b1d2902a`</small>
- [static-web-server-v2.24.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `46a70b022cf91f940962367c13df4013ffc40a645692cf8e6a25cb66c90e8748`</small>
- [static-web-server-v2.24.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `bfd0aa55242d553b0c6757f96c22b554a574231559d0b1b825fd0a46865c1002`</small>
- [static-web-server-v2.24.1-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `ec5bafe5d934a2786010a968e1be75af4166704cdce820504f908776adfc1777`</small>

### x86

- [static-web-server-v2.24.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `53b988adea40c75bb122833ae490051f9bfa06eb1f249dff6e67bc41fb33f796`</small>
- [static-web-server-v2.24.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `5a53a85785a685288bb532f83d5faaaa235c3ba38d16f54b074a1c3cba70d805`</small>
- [static-web-server-v2.24.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `a8e601ff06c739cf565c85fdec331207242dfa35d92437e1e7bef525f6e067fc`</small>
- [static-web-server-v2.24.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `398bbc37e170e52361309194f32d526348f96efdc3b232ecf5053442188b523f`</small>

### ARM

- [static-web-server-v2.24.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `21c60f8e5423c084b69b1589cc918c5ce381bd88884f7f6438c3a456320193f0`</small>
- [static-web-server-v2.24.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `03dffb5af4a8a416ec224b37c077479542ccee37af0b76eefdb6810469bd395c`</small>
- [static-web-server-v2.24.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `9f0b27cf5024022aae2c441c1442ecd2b098e5b7b2b920ea106db19085a19053`</small>

## PowerPC

- [static-web-server-v2.24.1-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `5d07cb861be7ed543dca9958547865b882994192b8d7f21ff52938c50b5b8f9b`</small>

## S390X

- [static-web-server-v2.24.1-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.24.1/static-web-server-v2.24.1-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `fe5bd000fd1c07a978dbb4dbd1e3fe64e13566f4d96944b0ed64e51c9fdf692b`</small>

## Source files

- [static-web-server-2.24.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.24.1.tar.gz)<br>
<small>**SHA256SUM:** `85a21ce040df933de87d2b71057d5f712cda8c1345ba270b015658a8a7f83cc0`</small>
- [static-web-server-2.24.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.24.1.zip)<br>
<small>**SHA256SUM:** `6b30e332c785d249c5b26579c723e7a65f230e84b0d4a1dfa40c54c374f55fb6`</small>
