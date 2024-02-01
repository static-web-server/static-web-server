# Download and Install

Latest **{{RELEASE_VERSION}}** release `{{RELEASE_DATE}}` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/{{RELEASE_VERSION}}), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-{{RELEASE_VERSION}}-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `{{x86_64-apple-darwin.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `{{x86_64-pc-windows-gnu.zip}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `{{x86_64-pc-windows-msvc.zip}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `{{x86_64-unknown-freebsd.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `{{x86_64-unknown-linux-gnu.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `{{x86_64-unknown-linux-musl.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `{{x86_64-unknown-netbsd.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `{{x86_64-unknown-illumos.tar.gz}}`</small>

### ARM64

- [static-web-server-{{RELEASE_VERSION}}-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `{{aarch64-unknown-linux-gnu.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `{{aarch64-unknown-linux-musl.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `{{aarch64-apple-darwin.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `{{aarch64-linux-android.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `{{aarch64-pc-windows-msvc.zip}}`</small>

### x86

- [static-web-server-{{RELEASE_VERSION}}-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `{{i686-pc-windows-msvc.zip}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `{{i686-unknown-freebsd.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `{{i686-unknown-linux-gnu.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `{{i686-unknown-linux-musl.tar.gz}}`</small>

### ARM

- [static-web-server-{{RELEASE_VERSION}}-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `{{arm-unknown-linux-gnueabihf.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `{{arm-unknown-linux-musleabihf.tar.gz}}`</small>
- [static-web-server-{{RELEASE_VERSION}}-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `{{armv7-unknown-linux-musleabihf.tar.gz}}`</small>

### PowerPC

- [static-web-server-{{RELEASE_VERSION}}-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `{{powerpc64le-unknown-linux-gnu.tar.gz}}`</small>

### S390X

- [static-web-server-{{RELEASE_VERSION}}-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/{{RELEASE_VERSION}}/static-web-server-{{RELEASE_VERSION}}-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `{{s390x-unknown-linux-gnu.tar.gz}}`</small>

## Source files

- [static-web-server-{{RELEASE_VERSION_NUM}}.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/{{RELEASE_VERSION}}.tar.gz)<br>
<small>**SHA256SUM:** `{{SRC_TAR}}`</small>
- [static-web-server-{{RELEASE_VERSION_NUM}}.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/{{RELEASE_VERSION}}.zip)<br>
<small>**SHA256SUM:** `{{SRC_ZIP}}`</small>
