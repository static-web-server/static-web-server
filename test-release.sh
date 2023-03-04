#!/bin/bash -e
cargo wasix build

PWD=$(pwd)
cd /prog/wasmer/lib/cli
cargo run --release --features compiler,cranelift \
  -- run --net --mapdir /public:/prog/deploy/wasmer-web/wapm/public /prog/static-web-server/target/wasm32-wasmer-wasi/debug/static-web-server.rustc.wasm \
  -- -p 9080 --log-level error
cd $PWD

