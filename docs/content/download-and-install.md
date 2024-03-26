# Download and Install

Latest **v2.28.0** release `2024-03-09` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.28.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.28.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `e585c82742e2e08c11863f44116eef21c0cddb13cb8f8e3aa866f22afd6296d5`</small>
- [static-web-server-v2.28.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `14a29b6bf98c947834c3c316b453ef5edbf8ed7c7c31a69314b686d9e6a7f57b`</small>
- [static-web-server-v2.28.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `bfededa88e9e491d899a6d1f7ee5d5071f3f65546d9ec054fb2456c440d45b88`</small>
- [static-web-server-v2.28.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `49e1011c46b13edda5606de5f1e8f15071c74ff095ef3dd6ff1966146f6b3ee9`</small>
- [static-web-server-v2.28.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `169146281bcd085887ec1f29b670e264bd80d0e0526b89a82e063f2b8616792c`</small>
- [static-web-server-v2.28.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `4683847c4fa22e87594d7a20055f700d10851cd8253c7c1b8ed2c4691a7e3702`</small>
- [static-web-server-v2.28.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `3d45fe59ca16b725420b38d0cb8c395091e0a0ebaec39796ba1267a475a3894a`</small>
- [static-web-server-v2.28.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `4ba668ab3c7f557077c2718d402e2b2170d692cef90cc8e0610bf649cb599f3d`</small>

### ARM64

- [static-web-server-v2.28.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `df31720608caccd651dea95743c5db166c7986e6f71fec375c12d09c13a82ada`</small>
- [static-web-server-v2.28.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `587aec97f588588bfaddad3ad450ea628c87b92fffdeb3f708426f7e01c23f5f`</small>
- [static-web-server-v2.28.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `2d6714cbc695439689d4918d6a6af546fef7cbc7cab8e9cddcc70d6b1bce7835`</small>
- [static-web-server-v2.28.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `4ff7494557a14c8deade3a59197289588e9219b976de55c4487e34c29608f2c8`</small>
- [static-web-server-v2.28.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5e8d05afc5207cf9003e1baf164cc297f5d0390f1464d939162f2013c6760c0f`</small>

### x86

- [static-web-server-v2.28.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `c588c4bbc61c06ce90274f32ab84fa93e9fa6b51de2f716f3572688fb14280ca`</small>
- [static-web-server-v2.28.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `e77165d8a8ce9701c04c7e09d6d6d7a6a37fea19204cf6d7e195a025f5420130`</small>
- [static-web-server-v2.28.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `ea173dca989fecb994f9d9fb3342ba9ec79495a8e8f39b5e93f4c22002255280`</small>
- [static-web-server-v2.28.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `3dce8547e78cfda596485549d0cbe3e0a7ac93ed05387e57e9d7080ebbd5d0f8`</small>

### ARM

- [static-web-server-v2.28.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `b02f81e44d94d0bd4f71ea24f3ed4d104bf32499ea1f2f29a47300375cc79d03`</small>
- [static-web-server-v2.28.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `1aecc2dab234f8aa7cb647cc11c8b2721a7822c25c668a9b9bc58792d944cb7a`</small>
- [static-web-server-v2.28.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `492dda3749af5083e5387d47573b43278083ce62de09b2699902e1ba40bf1e45`</small>

### PowerPC

- [static-web-server-v2.28.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `b7b2975b6ad27521b9e758f8a26b4371a5153abee4d219ef34d6634bc0aeb4cb`</small>

### S390X

- [static-web-server-v2.28.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.28.0/static-web-server-v2.28.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `8813fb23a96c2dd5878020cb7ff3e46924dced4894f7b79a6a6caa0252f8f054`</small>

## Source files

- [static-web-server-2.28.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.28.0.tar.gz)<br>
<small>**SHA256SUM:** `f06516b0c90061921ec7f5f42cf08f1d35eed62f80d00ccbfad5d7d776860d45`</small>
- [static-web-server-2.28.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.28.0.zip)<br>
<small>**SHA256SUM:** `8dda1370fd187d8c08cd263dcdd3e10e56e8dbf828a10247ae73246caa75f9a1`</small>
