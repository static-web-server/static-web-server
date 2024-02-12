# Download and Install

Latest **v2.27.0** release `2024-02-13` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.27.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.27.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `3f6efe92cb0e7b7a3def1d6fdb622b61b953449152a4fbd396205df5527529b3`</small>
- [static-web-server-v2.27.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `213456c21d26a797421e0fba5fc19a54b684a527d8090e4dd815be045bc5ef0a`</small>
- [static-web-server-v2.27.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `1ec2c97050a9f0df4d92d5dee72b1609feca1c03e9f96006542498afaef7e731`</small>
- [static-web-server-v2.27.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `0869bbe73e14302670315189b2d42e2306ac6746599256c91e8a274c3abcbdc3`</small>
- [static-web-server-v2.27.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `962e0baaeb8ec64c3fbdd7f46d0c60cc28d0a8ce1de0b69119d0e7f52a05fa66`</small>
- [static-web-server-v2.27.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `5e70f7298870d70acad6278dbd4bfceb03775cca98c52452483ede64ecb5fa08`</small>
- [static-web-server-v2.27.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `18285b7e52556aaa34a8a489594ac36b94ab1fbda7a5105c29a886d34f0a3511`</small>
- [static-web-server-v2.27.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `244361282d7e5d9a3de8d19bd278d0671f477343f946672cc0bec95bd4d00f94`</small>

### ARM64

- [static-web-server-v2.27.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `c3b6b2ebb177ada3264bd30bf13e77176e3e40e2c5c09067f9735dd0b2a5d38c`</small>
- [static-web-server-v2.27.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `45f9704d91b998d61ebbb7053c149b48c0d42a59e3ba74ccedd6fa88fa1076cb`</small>
- [static-web-server-v2.27.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `b06f37c86fd9a0a366b110cfe325504e23bb1260fd7036a80ecb12ec76fa8fe9`</small>
- [static-web-server-v2.27.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `5ac9056e5c54421c7c7e8db4e1fb818f22c423d049155bb4906058f52aeda45d`</small>
- [static-web-server-v2.27.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `a7b9205f8aa1dbefb374bc89c078c1657ab6b321c3a7d47a0ea9ba95e6d75dee`</small>

### x86

- [static-web-server-v2.27.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `a29ee613774b641c6bcc917dc05143dfa50619f465cb42c2fb605532e91122a5`</small>
- [static-web-server-v2.27.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `e268500a832accb616665145f9317a29add473273564e45e3761410a9ad9937f`</small>
- [static-web-server-v2.27.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `7099ccfc4e563ff2242c1a2231159321a24d40060febe9a2c2cec8a3e7e813d5`</small>
- [static-web-server-v2.27.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `4eac72387a7a3f0a0cda9ee171de82e88c84fae98ef4c24769f253608655b22a`</small>

### ARM

- [static-web-server-v2.27.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `2dda978b55fb2d7e0ae112646b1d973075632c057c6a8ea7d87203a124c0f468`</small>
- [static-web-server-v2.27.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `a245916a885aef222f78926c15a7e57f1150b56e7676b8a14c833c00333338ba`</small>
- [static-web-server-v2.27.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `d1c411bb6a47a4a57f273c6eee98754b20aa3cbbeebf28138a0b5e1a7ad8f21c`</small>

### PowerPC

- [static-web-server-v2.27.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `d699da6ccba658941e7526702bd9ed17b94575ca982d5531cd2bdb3b8869098a`</small>

### S390X

- [static-web-server-v2.27.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.27.0/static-web-server-v2.27.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `35e65fc2edef35f00010bae51e130924e98cee5959c35ece84609d29833d844a`</small>

## Source files

- [static-web-server-2.27.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.27.0.tar.gz)<br>
<small>**SHA256SUM:** `3e7597f91cd9e1566cf84371186855d6df0aed719b2370b10c827f38bbe1b45d`</small>
- [static-web-server-2.27.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.27.0.zip)<br>
<small>**SHA256SUM:** `daf31740a9e88c320e412ec87eb3d0969cec88c58dd64891e8231c50920dc27d`</small>
