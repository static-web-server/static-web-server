# Download and Install

Latest **v2.31.0** release `2024-05-19` ([changelog](https://github.com/static-web-server/static-web-server/releases/tag/v2.31.0), [sha256sum](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-SHA256SUM))

<div class="featured-downloads">

<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-unknown-linux-gnu.tar.gz">Linux 64-bit</a> <a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-apple-darwin.tar.gz">macOS 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-pc-windows-msvc.zip">Windows 64-bit</a>
<a class="md-button md-button-sm" href="https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-unknown-freebsd.tar.gz">FreeBSD 64-bit</a>

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

- [static-web-server-v2.31.0-x86_64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `91d6a35a4774d2b5d2c199bf37481961b89c320d95e43214118015f2eff80926`</small>
- [static-web-server-v2.31.0-x86_64-pc-windows-gnu.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-pc-windows-gnu.zip)<br>
<small>**SHA256SUM:** `ca4bea0592ca976257674fcbc2577dcc98e1be0f643508d96bef392e2ab6dcb0`</small>
- [static-web-server-v2.31.0-x86_64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `2c9d2f94f8924283a662d0acab199593a3324794333c57841dc43481a8542e59`</small>
- [static-web-server-v2.31.0-x86_64-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `827f4b2684cf4d1277eab21d5429597e60cf29456d786e5e3fbe18b785b3712c`</small>
- [static-web-server-v2.31.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `c0da859bdcc9dab3f0b95dca78af7f29cd1404ed136c118f379ef398cce25936`</small>
- [static-web-server-v2.31.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `c54a6abb744ff7c51f467e8e0d0084e648487b7e4e0383c9f28bb816c6b4d515`</small>
- [static-web-server-v2.31.0-x86_64-unknown-netbsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-unknown-netbsd.tar.gz)<br>
<small>**SHA256SUM:** `4cdc0bf317255fc27c7a314fd17de506f05435e7340827a77ec5d98a958d48fc`</small>
- [static-web-server-v2.31.0-x86_64-unknown-illumos.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-x86_64-unknown-illumos.tar.gz)<br>
<small>**SHA256SUM:** `be5ac431111a81d303e12763703eed7e58502f1919bde306ef242fef8c70eb88`</small>

### ARM64

- [static-web-server-v2.31.0-aarch64-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-aarch64-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `7a0247e1d27739923cdaa07a6b946dbc5e50762cc9a2d14a474a7579a1aaf038`</small>
- [static-web-server-v2.31.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-aarch64-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `608ed51478d7298d459e512d77781720ce573dfaebef539213209964a995bed4`</small>
- [static-web-server-v2.31.0-aarch64-apple-darwin.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-aarch64-apple-darwin.tar.gz)<br>
<small>**SHA256SUM:** `de9ae1cf34949b6d70116008602b8e98efe1ebb561544a6caead8e3019c8c230`</small>
- [static-web-server-v2.31.0-aarch64-linux-android.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-aarch64-linux-android.tar.gz)<br>
<small>**SHA256SUM:** `307b1c6ad3a2909af0c8aca74cf58780637c1a6936d057218526d91a08fea6f0`</small>
- [static-web-server-v2.31.0-aarch64-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-aarch64-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `5b7e5957980d47658e751c625722d2d1cd9f9af19059afae9868e31f6f303943`</small>

### x86

- [static-web-server-v2.31.0-i686-pc-windows-msvc.zip](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-i686-pc-windows-msvc.zip)<br>
<small>**SHA256SUM:** `4ba5e0beabe4ddd228cbb7d1d2a0b9ef1a8b9a72b546b69b3ac72f47e2e89172`</small>
- [static-web-server-v2.31.0-i686-unknown-freebsd.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-i686-unknown-freebsd.tar.gz)<br>
<small>**SHA256SUM:** `3b1454cb6063c75b7b1edfae65626193899d544282d6d8917153c3be750b8853`</small>
- [static-web-server-v2.31.0-i686-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-i686-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `b7197173efe5c22a62363bdb696de4be7d2a794957bba072dccc2a8885f4f633`</small>
- [static-web-server-v2.31.0-i686-unknown-linux-musl.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-i686-unknown-linux-musl.tar.gz)<br>
<small>**SHA256SUM:** `dfb798c09c7638bdbbc36f1ac96336ea68a3b941793d847353852ec9cd5996d4`</small>

### ARM

- [static-web-server-v2.31.0-arm-unknown-linux-gnueabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-arm-unknown-linux-gnueabihf.tar.gz)<br>
<small>**SHA256SUM:** `5a00004ed31bf03b2e3b0c03725e686e3f0ff537a6b3b80f1f7105150bcb2475`</small>
- [static-web-server-v2.31.0-arm-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-arm-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `5fea54f999dabdba4ac4353f7f4d0170b73bc59917868289935eaf33acd9f46a`</small>
- [static-web-server-v2.31.0-armv7-unknown-linux-musleabihf.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-armv7-unknown-linux-musleabihf.tar.gz)<br>
<small>**SHA256SUM:** `8c12f1078bbfbae92bfa3c89c3db9257a499d1ce8b6564e77c23f1eab9679969`</small>

### PowerPC

- [static-web-server-v2.31.0-powerpc64le-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-powerpc64le-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `df125721f8d8d77ffc545e439656421c577c87cd2d87e1012f4154f7bc32d66e`</small>

### S390X

- [static-web-server-v2.31.0-s390x-unknown-linux-gnu.tar.gz](https://github.com/static-web-server/static-web-server/releases/download/v2.31.0/static-web-server-v2.31.0-s390x-unknown-linux-gnu.tar.gz)<br>
<small>**SHA256SUM:** `55bbdfa58810c2229568350b8ec3f6465262cacf6a16765fe04141ebd95df530`</small>

## Source files

- [static-web-server-2.31.0.tar.gz](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.31.0.tar.gz)<br>
<small>**SHA256SUM:** `5bc6c63018057583887a9fd2b55320c4c0b767829b145387bc1c382733bbe073`</small>
- [static-web-server-2.31.0.zip](https://github.com/static-web-server/static-web-server/archive/refs/tags/v2.31.0.zip)<br>
<small>**SHA256SUM:** `6173c525b3a8e1d8ba858861d89037f0c28ab1e7cc72e74b44f493e276e3bb52`</small>
