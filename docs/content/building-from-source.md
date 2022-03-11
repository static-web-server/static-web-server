# Building from Source

If you want to build **`SWS`** from source, all what you need is to have a [Rust 2021 Edition](https://blog.rust-lang.org/2021/05/11/edition-2021.html) installed.

Make sure to install Rust [1.56.0](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html) or higher (or nightly) along with [the toolchain(s)](https://rust-lang.github.io/rustup/concepts/toolchains.html) of your preference.

Then clone the repository and use [Cargo](https://doc.rust-lang.org/cargo/) to build the project from source.

```sh
git clone https://github.com/joseluisq/static-web-server.git
cd static-web-server
cargo build --release
```

Finally, the release binary should be available at `target/release/static-web-server`.

!!! info "Note"
    Please don't use the project's `Makefile` since it's only intended for development and some on-demand tasks.

