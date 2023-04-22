#!/bin/bash -e
cargo wasix build --release --no-default-features

CUR=$(pwd)
cd /prog/wasmer/lib/cli
cargo run --release --features compiler,cranelift \
  -- run --net --mapdir /public:$CUR/root/public --mapdir /cfg:$CUR/root/cfg $CUR/target/wasm32-wasmer-wasi/release/static-web-server.wasm -- --log-level info
cd $CUR
