#!/bin/bash -e
cargo wasix build --no-default-features

CUR=$(pwd)
cd /prog/wasmer/lib/cli
cargo run --features compiler,cranelift,debug \
  -- run --enable-async-threads --net --mapdir /public:$CUR/root/public --mapdir /cfg:$CUR/root/cfg $CUR/target/wasm32-wasmer-wasi/debug/static-web-server.rustc.wasm \
  -- --log-level info
cd $CUR
