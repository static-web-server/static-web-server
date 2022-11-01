#!/bin/bash -e
cargo wasix build
wasm-opt -O1 --strip-debug target/wasm32-wasmer-wasi/debug/static-web-server.wasi.wasm -o target/web-server-debug.wasm
cp -f target/web-server-debug.wasm /prog/ate/wasmer-web/public/bin/web-server-debug.wasm
chmod +x /prog/ate/wasmer-web/public/bin/web-server-debug.wasm
