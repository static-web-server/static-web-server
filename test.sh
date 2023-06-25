#!/bin/bash -e
cargo wasix build --no-default-features

CUR=$(pwd)
cd /prog/wasmer/lib/cli
# $CUR/root/public
RUST_BACKTRACE=1 RUST_LOG=wasmer_wasix=trace cargo run --features compiler,cranelift \
  -- run --net --mapdir /public:/prog/wasmer/lib/wasi-web/dist --mapdir /cfg:$CUR/root/cfg $CUR/target/wasm32-wasmer-wasi/debug/static-web-server.rustc.wasm \
  -- --log-level info
cd $CUR
