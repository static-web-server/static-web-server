#!/bin/bash -e
cargo wasix build --release
cp -f target/wasm32-wasmer-wasi/release/static-web-server.wasm \
  /prog/deploy/wasmer-web/wapm/web-server.wasm
cp -f target/wasm32-wasmer-wasi/release/static-web-server.wasm \
  /prog/packages/static-web-server/web-server.wasm
cp -f target/wasm32-wasmer-wasi/release/static-web-server.wasm \
  /prog/quantochat/wapm/quanto-chat-web/web-server.wasm
