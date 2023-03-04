#!/bin/bash -e
cargo wasix build --release

if [[ -d /prog/deploy/wasmer-web/wapm ]]; then
  cp -f target/wasm32-wasmer-wasi/release/static-web-server.rustc.wasm \
    /prog/deploy/wasmer-web/wapm/web-server.wasm
fi
if [[ -d /prog/packages/static-web-server ]]; then
  cp -f target/wasm32-wasmer-wasi/release/static-web-server.rustc.wasm \
    /prog/packages/static-web-server/web-server.wasm
fi
if [[ -d /prog/quantochat ]]; then
  cp -f target/wasm32-wasmer-wasi/release/static-web-server.rustc.wasm \
    /prog/quantochat/wapm/quanto-chat-web/web-server.wasm
fi
