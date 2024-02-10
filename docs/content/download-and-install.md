# Download and Install

Latest **v2.26.0** release `2024-02-10` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.26.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.26.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `305c0b537886b51f53f3dfe7458848b9083807c17626eaaf1925ed09e1bab247`</small>
- [static-web-server-v2.26.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `fdea3921212373139f4a312ff03388e8163c412b89d5426a5c85dae2f2767643`</small>
- [static-web-server-v2.26.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `edbb5c6fb72d97c8f498d644ae7005ac25133f260b4ee433dfd5851c1345ffe3`</small>
- [static-web-server-v2.26.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `dfa4c8c3e363379b881043ecff642375beb3b0a0fa0397b6adc80c1a7c35ce3d`</small>
- [static-web-server-v2.26.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `e2be439376d925c5069dc2d45d4b5d2cdb58caa13f48a04c203357e0e7ea5b28`</small>
- [static-web-server-v2.26.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `c7a03eb6fab17d0ae4828d3c65bb8e48ecba411aca275fd3a8b7b25fb93b6c18`</small>
- [static-web-server-v2.26.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `d6753d359a68d40ed9affb4f42cd7bf4d8a2328ded76aa6c455e61a00fd1b73a`</small>
- [static-web-server-v2.26.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `cc68f311a884f3a0c799f5a2892efdb4ded60dacf02d65f904dd3979b6548630`</small>

### ARM64

- [static-web-server-v2.26.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `6b7fb9c0036300b01c09a21230d786ceb81e5ca08f6b0f63db063cd507c13474`</small>
- [static-web-server-v2.26.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `a03ad8636f744128e87e692d949a6514bb3a073c8e5a097f02b117099d839892`</small>
- [static-web-server-v2.26.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `44ed652845e41c88091166b89899a888569fcff383c6826ebbc8da3645f05061`</small>
- [static-web-server-v2.26.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `fbf4760b5747cab106152767cb3b5ba80293c7fbc4b7f934fe9afbdc5edc147f`</small>
- [static-web-server-v2.26.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `4633ba13cb1bbb1a57c1a654ad1b2ea657f05cd405140a5cb18da20d172d4c6c`</small>

### x86

- [static-web-server-v2.26.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `52c2e89740f1ffb546ee164efa95e507ad6912c6cc8db350922f6f702ec06911`</small>
- [static-web-server-v2.26.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `80e88cb5815a94b48a56d27fd4a9523df4052941eb20470357213658f0f7cb23`</small>
- [static-web-server-v2.26.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `0ea8c6394f5f9c893463da9ee87f3002edc6ff4f0435ca26e66adab1f6407688`</small>
- [static-web-server-v2.26.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `e272687a2265d90d66a00c71693ba99c1bbb73062d729987873561b3b1511f99`</small>

### ARM

- [static-web-server-v2.26.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `a653497f7fcb95ede3fc91c0b2895810781acf21b99982439dd76cd570eb63ab`</small>
- [static-web-server-v2.26.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `f42133fc08524a533f88860b0caf5c2ea1a94994f5625337f68f2d9b6d90d4f8`</small>
- [static-web-server-v2.26.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `8f5fa75880bf71079736c3eb7ab25574057e710c623d014892ff3f2c9e101d16`</small>

### PowerPC

- [static-web-server-v2.26.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `fd1a6eca4438bca740ed5ed16385d7fd0e6e0246d97d971c1bf5ad0197881d80`</small>

### S390X

- [static-web-server-v2.26.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.26.0/static-web-server-v2.26.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `4d0d574ad98586c4d8f4a7daaff55e6f505e40d26dc7ee7e9e5002ccedeadb8f`</small>

## Source files

- [static-web-server-2.26.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.26.0.tar.gz)<br>
<small>**SHA256SUM:** `e51ba23befe4c5d6b35b2cdb7b65819c396cb8ebd1edf5d7c27a8b3ad339f6a1`</small>
- [static-web-server-2.26.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.26.0.zip)<br>
<small>**SHA256SUM:** `11c7a53cd0be3a8ac5632e50fe29b565d136a1009f23d4499f3b1e5db263b8ab`</small>
