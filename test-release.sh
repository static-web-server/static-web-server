#!/bin/bash -e
cargo wasix build --release --no-default-features

CUR=$(pwd)
cd /prog/wasmer/lib/cli
RUST_BACKTRACE=1 cargo run --release --features compiler,cranelift \
  -- run --net --mapdir /public:/prog/wasmer/lib/wasi-web/dist --mapdir /cfg:$CUR/root/cfg $CUR/target/wasm32-wasmer-wasi/release/static-web-server.wasm -- --log-level error
cd $CUR
