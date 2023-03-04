#!/bin/bash -e
cargo wasix build --release --no-default-features

PWD=$(pwd)
cd /prog/wasmer/lib/cli
cargo run --release --features compiler,cranelift \
  -- --mapdir /public:/prog/deploy/wasmer-web/wapm/public /prog/static-web-server/target/wasm32-wasmer-wasi/release/static-web-server.rustc.wasm \
  -- -p 9080 --log-level info
cd $PWD
