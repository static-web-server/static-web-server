#!/bin/bash -e
cargo wasix build --release
cp -f target/wasm32-wasmer-wasi/release/static-web-server.rustc.wasm /prog/ate/wasmer-web/public/bin/web-server.wasm
chmod +x /prog/ate/wasmer-web/public/bin/web-server.wasm
