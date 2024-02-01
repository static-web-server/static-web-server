# Download and Install

Latest **v2.25.0** release `2024-01-23` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.25.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.25.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `0afcdc3d3389acfb9e4632d8f2bddae5ed89200a4c3a338dd9d4f6dda1d05b59`</small>
- [static-web-server-v2.25.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `9fa1e7ecdeaa0f15a4e3c09d57ca518800bb6e1e8df668015d28cf4ce94ca8ea`</small>
- [static-web-server-v2.25.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `9967999b55810fec7a9b6d946caf78175475270bebaf364ad20ae1291236dc1c`</small>
- [static-web-server-v2.25.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `4d0e3315a743f04e8741a0b4b6ed9170f295e6a26edeb2309af48700a32d41b0`</small>
- [static-web-server-v2.25.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `772653e8cf111e13015f128334bb6f3959da6662a0d6b6ebb3a4e015076656a8`</small>
- [static-web-server-v2.25.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `d08171d1e6d5810c987f3f6151951b9f407281b4f62bd1e0faefa286ba646232`</small>
- [static-web-server-v2.25.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `647eeb776afa877066a144405562bfce4b31f48cac84e23e42837f00102e0aa0`</small>
- [static-web-server-v2.25.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `a1ab3a1fbf84b9ce5b825d9a83d468b8c447d41ed5a2a192cdf32e009f5ffcf9`</small>

### ARM64

- [static-web-server-v2.25.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `1267b7d565852706abd0b18eb24abd391fbba1bce71dd2fbd404b804c46d39aa`</small>
- [static-web-server-v2.25.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `5593bf34263322024fd06c166208ec0e3b15b5c43b9fcd1324692c4284469a89`</small>
- [static-web-server-v2.25.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `8162e625bd2d3ec61291a3fbbafe6103324d6d7494e2bcd86e9742fde6377cae`</small>
- [static-web-server-v2.25.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `e351629fd7abcbe9bff2fd616356f83a1d360a030049f40e6d2c18fde901a20b`</small>
- [static-web-server-v2.25.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `fa0e2b5dc846789b33197ff222c3a3dd75f9466b9af15c93cf7eb5fac1672c35`</small>

### x86

- [static-web-server-v2.25.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5ca194c5564230b56bb871f7a2ab32fc9614d1b5b7520e7fcf86adb3c82c49f6`</small>
- [static-web-server-v2.25.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `91e14b4a204d41efbc18403461e801062181adc5d5db40a7b78bf82ade3213b1`</small>
- [static-web-server-v2.25.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `cd8a22aad953d84979b4b72643ec1df0f5b5118a397285f4c7d31b8e19cf2e07`</small>
- [static-web-server-v2.25.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `a1b29c59d6910117e68ed197621b8616f9c98bd8a36fb741260f0fe5039f0f35`</small>

### ARM

- [static-web-server-v2.25.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `f85f4cff84d304fb489ca0262c2dfd9545a3b8d2da10e8456bd1866b4c101024`</small>
- [static-web-server-v2.25.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `0b1bc161757b29ea0bb3a14eecc2f06f5433570fb007907b7ff9bdbf2d072750`</small>
- [static-web-server-v2.25.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `bd2e9503ffa9855e02092d2c61613726c9c053cf6246e46cbf05fee46837a1da`</small>

### PowerPC

- [static-web-server-v2.25.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `ef3f680dfbde8c904f3a2cb00240ee41069e506fc740e07c3f31ecba5b772dbe`</small>

### S390X

- [static-web-server-v2.25.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.25.0/static-web-server-v2.25.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `4af3ad160ec20147280f05fe755052452e4af0ffc8bca7412778c8a2eb609363`</small>

## Source files

- [static-web-server-2.25.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.25.0.tar.gz)<br>
<small>**SHA256SUM:** `783ff440489d4ee3bde4ee05c9b952e9a2f79bd979201df554d193925b668ff8`</small>
- [static-web-server-2.25.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.25.0.zip)<br>
<small>**SHA256SUM:** `bb66ff9393744a99760439a74cd472570fcb4e91ae5777ab2c73a209681e9896`</small>
