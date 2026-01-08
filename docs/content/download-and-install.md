<!-- Content generated. DO NOT EDIT. -->
# Download and Install

Latest **v2.40.1** release `2025-12-08` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.40.1), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

The latest `static-web-server` version will be installed by default under the `/usr/local/bin` directory.

Alternatively, you can install a specific version of SWS to a custom location by setting environment variables.

```sh
export SWS_INSTALL_VERSION="2.40.1" # full list at https://github.com/static-web-server/static-web-server/tags
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

Pre-compiled binaries grouped by CPU architectures.

### x86_64

- [static-web-server-v2.40.1-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `fbdb678ab2e7886ef20719bde2a929c49ec1b24dc4900f409bd377e49a85dd32`</small>
- [static-web-server-v2.40.1-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `60861fe03723c8cb2aef8627e44dc88638ab0a3b6fc2830da12727cbbdebde4b`</small>
- [static-web-server-v2.40.1-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `7d4ef56415663f1d2333848a94af7aba116807e2efda34df2db0b51ee5d58705`</small>
- [static-web-server-v2.40.1-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `65aaa3a66b12bd881b18c704715684f6474055197cb2b1f8a6649bb454fdce66`</small>
- [static-web-server-v2.40.1-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0330cd4a6265a96fac4be6620abfd2a892e9ff32f3b8e2de0630b09cb3236f67`</small>
- [static-web-server-v2.40.1-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `be4d06d6c71aeab2cb913da176df8a66de5e91a77a250c96bfb3d2d541f9b1bd`</small>
- [static-web-server-v2.40.1-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `25aae44a6be223a5497f2c8444d0744964de581c91161852c88b7a41eccaf860`</small>
- [static-web-server-v2.40.1-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `3be8523a49235360e935a01e98ca850a557dce03d89284f600e0039855894256`</small>

### ARM64

- [static-web-server-v2.40.1-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `f3db363fdb41898a40d9aaba0714e06d0a03513d2cacacf7ee6066222b422f7f`</small>
- [static-web-server-v2.40.1-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `a1dcaf126a2afafbfc526901c40c5f006b0badde79cbdc9e22d82c0886903168`</small>
- [static-web-server-v2.40.1-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `58370cc01799c826fc5d3398762c2e6acb5440ab0ec9a289ab9b709e76cff242`</small>
- [static-web-server-v2.40.1-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `83b9bc7b8911b4a02e069a25a28cf524eeaffd87449fa4108df000cd2f3b3340`</small>
- [static-web-server-v2.40.1-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `d52ccf91fe220becf6cb5e9017ac5391fabb49d3589bb367c42e6a8815a9092f`</small>

### x86

- [static-web-server-v2.40.1-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `4d022579f9f492aa0b537de2acc139753d51d5e346b42dc356996470201a17af`</small>
- [static-web-server-v2.40.1-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `3887ea773e0ecc7f8a023ec6d7776dd3674f002c698dfc15b90a2cf026fe4b19`</small>
- [static-web-server-v2.40.1-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `a17e93cd88c1ff8270156216f51273a90a2e65d2f3f88a4d4cec8d43b6c741cf`</small>
- [static-web-server-v2.40.1-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `fffe32bf5b6bd7ccf2796be5e8abb542e4af9f936f4369774741dce08b5541de`</small>

### ARM

- [static-web-server-v2.40.1-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `aa1d73a1df510684bbbf23a86dcdb4e1b9182901ced93274cdb511ce67dbba7d`</small>
- [static-web-server-v2.40.1-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `aea16c6f946462fddd0743189c830e6f4443a492d808242ce52b10217a94f974`</small>
- [static-web-server-v2.40.1-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `4ee0344d90c1dcfa190d027e91122cf98ac49ea05890a84f9d2f0fcd767ba3d9`</small>
- [static-web-server-v2.40.1-armv7-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-armv7-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `49d2036beadd25bfd06481574337577bd1bc7e23bada28ade115bef4e40ef8a8`</small>

### PowerPC

- [static-web-server-v2.40.1-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `ee36176207417efc62217ebf114b13a0000221f6b6b3c6cacb38ce422457c5ef`</small>

### S390X

- [static-web-server-v2.40.1-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.40.1/static-web-server-v2.40.1-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `d98db1e24c02359fdbe5a707e7bb0a9e6b287cadf643bf397b24d2e3fb9cbfec`</small>

## Source files

- [static-web-server-2.40.1.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.40.1.tar.gz)<br>
<small>**SHA256SUM:** `db6ee202a926452d278c14872083744a67ec31710db5fd71e00e551ee0955eb4`</small>
- [static-web-server-2.40.1.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.40.1.zip)<br>
<small>**SHA256SUM:** `e9ee303745719bfef98876eb6696599d72177c4e357c29491865ecf3b8a75b08`</small>
