#!/bin/bash -e
cargo wasix build

PWD=$(pwd)
cd /prog/wasmer/lib/cli
cargo run --features compiler,cranelift,debug \
  -- run --net --mapdir /public:/prog/deploy/wasmer-web/wapm/public /prog/static-web-server/target/wasm32-wasmer-wasi/debug/static-web-server.rustc.wasm \
  -- -p 9080 --log-level trace
cd $PWD

