<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.35.0** release `2025-01-10` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.35.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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
export SWS_INSTALL_VERSION="2.35.0" # full list at https://github.com/static-web-server/static-web-server/tags
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

- [static-web-server-v2.35.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `bebed68f1bf2c044a5bf6ffa30afae34694ca2009c6df2ff9968f5a0221ecb0e`</small>
- [static-web-server-v2.35.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `5161067b9f417eda2e1c184b1f78808e27ca759c0ed327630c1b42d5323a90d5`</small>
- [static-web-server-v2.35.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `e3b8811f380534ccee2cc68b563dad90e3699fe0ef2721c8142ac7a51a1f18e3`</small>
- [static-web-server-v2.35.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `81059421d64e9d0b6d5864321d827751e7641b8a9a015bbbfca734e04e588080`</small>
- [static-web-server-v2.35.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f4c65b17c1f5ba02e1954861f8a164520f52fb3dadd16eea4edf51b738a8e9bd`</small>
- [static-web-server-v2.35.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `8b4e3a453ff206366571e9e0bbc337bc56a58a890dd74fc8cb58434010935b4c`</small>
- [static-web-server-v2.35.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `95a3401d3295ce31c2b3214b9c19a9abe756ae7fd5119e144a0093ccd10dc1be`</small>
- [static-web-server-v2.35.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `456ca76fea76a949540513fda76d4cd0a6c17366de8eece3046fa6e1902f8df0`</small>

### ARM64

- [static-web-server-v2.35.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `92e34f7ceaef3d5eef0601e8b2b5fb6bb95fb7a004bf5147495cb3dfc6e8934d`</small>
- [static-web-server-v2.35.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `2aa25aba0bef4d6a3b3bbe16e315cf7caf323895b970ce3ae723cf33a3277022`</small>
- [static-web-server-v2.35.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `4a4a9536c9b420ac2db6ac6018416785aeae4bc9b6e884edeb295493ce313deb`</small>
- [static-web-server-v2.35.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `a5c02ecf4f6e47bea1f6d91d6f67e7bc1e5b1dc805d147990712b60119f96fb6`</small>
- [static-web-server-v2.35.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `84ee0402a1cd6eff503733d21ca9998dcfe84f0980cd1830e2476a70cea6a271`</small>

### x86

- [static-web-server-v2.35.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `80309dcdfd900933747b03dbbb85aed285a379d60a62d9cb80ac91638269b323`</small>
- [static-web-server-v2.35.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `b232e7c6295b74b98aef4b2f3575248403e3b71b287ba710b2042fcad991e14b`</small>
- [static-web-server-v2.35.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `928a07a91dba97375f278d2169abb2f1485b49f48977c8473fb67a77f8aa06af`</small>
- [static-web-server-v2.35.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `78317385b0de4ed44eb6a1d4854bb311668845fd3a782774e5667541dc53e11f`</small>

### ARM

- [static-web-server-v2.35.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `899254ab51d1a50962410d25fce8770dc72370bb5b89acb8b7ee6f33a6028d26`</small>
- [static-web-server-v2.35.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `945e33358074d13412e9ea0c332e6b00baa127f99db88273f1de36aa19176e2b`</small>
- [static-web-server-v2.35.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `067638c309b3a25b0df069dde66f7034cb70e39e145bbb79cf58350075b870ec`</small>

### PowerPC

- [static-web-server-v2.35.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `618f5c29c96cba22cf045637658c78257486e640c57d14c9b656e5ba868a01a7`</small>

### S390X

- [static-web-server-v2.35.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.35.0/static-web-server-v2.35.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `bff30e26cc618281c8b990863408036104eb19dbc5143a9666b9b9b4ff1a849f`</small>

## Source files

- [static-web-server-2.35.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.35.0.tar.gz)<br>
<small>**SHA256SUM:** `adf260f0aa3ccc18955f9f68ce11356c8d0e3fbb0d9b9446b137430427dafb3b`</small>
- [static-web-server-2.35.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.35.0.zip)<br>
<small>**SHA256SUM:** `964b4eb3830d3f161a02f334955006da884de433badb801f11280e095d3b3df8`</small>
