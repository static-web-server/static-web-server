<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.32.0** release `2024-06-19` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.32.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.31.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

### TrueNAS SCALE

If you use [TrueNAS SCALE](https://www.truenas.com/truenas-scale/) then visit [TrueCharts Community Website](https://truecharts.org/charts/stable/static-web-server/) and its [Introduction to SCALE](https://truecharts.org/manual/SCALE/guides/scale-intro) page to install SWS application in your instance.  

## Binaries

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.32.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `2c0112758188207af54f2b160e6be6b1318098ae55ab1f9145e1988ea74d3ed7`</small>
- [static-web-server-v2.32.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `7f4c72515a370f5776c2da982417d9a9ccf21773845b1af8e08c72dfe7f59f62`</small>
- [static-web-server-v2.32.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `fbff273a0040b062e96149d811578d10efd547a386c7aa0a2f19e5462849a874`</small>
- [static-web-server-v2.32.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `25ce34d5b48d4e69afd832ddd24823eb058d1dabb7e797dd85c03b6f54fd264d`</small>
- [static-web-server-v2.32.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `7a24eba311d90281d8e2f441918814cf3d4455c943fd3f2c23d494fc3444aced`</small>
- [static-web-server-v2.32.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `8d6346fb74808e3ec4dee6e80df0ac3d1dd69fc61f46214af4449834b8234a07`</small>
- [static-web-server-v2.32.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `338b2c685a49858229f1acc97fd078d37e63d808428f031c1b82084c3f229429`</small>
- [static-web-server-v2.32.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `d830b506c6cb7313c9a1e382be3cd21d8738719c74df093754a439b69844ff10`</small>

### ARM64

- [static-web-server-v2.32.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `1544651fa9bb2c04c8c571ea8debf7698df917336b5e703fcc9a779e17f33fdc`</small>
- [static-web-server-v2.32.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `eaa8f79c088707bbfb999de96e4306d3b22d09c6f57a08a279e5aeafaa9b849f`</small>
- [static-web-server-v2.32.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `da6a4779ed2a41a7ebefa29c0eb02370d2f52a23cce7b9eaf37ab4be1b97abe4`</small>
- [static-web-server-v2.32.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `abde6c8977498b893d7671b6ed428654979297cfc59d4e088dfb27e3d2654227`</small>
- [static-web-server-v2.32.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `cd06b70f0628457e17a8a3616e174b490023ed1066a105ae135ee65545809e38`</small>

### x86

- [static-web-server-v2.32.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `ffb2b7927cfab6bc7a2022c4a0ea7d82c73d5b56bfe833305955ee46d2ed9608`</small>
- [static-web-server-v2.32.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `0c344b167d2e8024142c13c8689075b4c86852479d5359a0f9fb099efe1fac29`</small>
- [static-web-server-v2.32.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `33e9f2481ce6f70ef2d80b1f2c3f712e05a9036a3720a920217ff77ddf067ab3`</small>
- [static-web-server-v2.32.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `e535ec5baf6d459e05abfd2e18b655f95866e5b05c00c3fa816e4f191fd6b3dc`</small>

### ARM

- [static-web-server-v2.32.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `754ba57444461e0f165a234cf0023196ce01bd7d1e9f19446e5f3e9d99746891`</small>
- [static-web-server-v2.32.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `997b82085007a3e86ec002f70420784e4271750be582b8f819279bf9e77c98a8`</small>
- [static-web-server-v2.32.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `2026cc3dc24c19c0c2915312c8df1c8899f128197ee19a92a93c2fdd5b888efd`</small>

### PowerPC

- [static-web-server-v2.32.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `bcfdf97a7e0d0b23e4f592af387dda06b1c08a2234a9c6d17ed673d6f2c5b13c`</small>

### S390X

- [static-web-server-v2.32.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.32.0/static-web-server-v2.32.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `3945675abb6f3023a7ed3b01d50ba71d6c9e3b648e2a1db9715353a30b8eb7bf`</small>

## Source files

- [static-web-server-2.32.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.32.0.tar.gz)<br>
<small>**SHA256SUM:** `0ba853ded8ee63f4714be0ac3aee4ca5c7f97596a18a4f2c3ea0ede373f0e76e`</small>
- [static-web-server-2.32.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.32.0.zip)<br>
<small>**SHA256SUM:** `876597930a20ff296fec15e3be75469d1be16afdebc6825a395f1fa54e69f224`</small>
