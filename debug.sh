#!/bin/bash -e
cargo wasix build
wasm-opt -O1 --strip-debug target/wasm32-wasmer-wasi/debug/static-web-server.wasi.wasm -o target/web-server-debug.wasm

if [[ -d /prog/deploy/wasmer-web/wapm ]]; then
  cp -f target/web-server-debug.wasm \
    /prog/deploy/wasmer-web/wapm/web-server-debug.wasm
fi
if [[ -d /prog/packages/static-web-server ]]; then
  cp -f target/web-server-debug.wasm \
    /prog/packages/static-web-server/web-server-debug.wasm
fi
if [[ -d /prog/quantochat ]]; then
  cp -f target/web-server-debug.wasm \
    /prog/quantochat/wapm/quanto-chat-web/web-server-debug.wasm
fi
