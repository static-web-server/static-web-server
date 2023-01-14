#!/bin/bash -e
cargo wasix build --release
cp -f target/wasm32-wasmer-wasi/release/static-web-server.rustc.wasm /prog/deploy/wasmer-web/public/bin/web-server.wasm
cp -f target/wasm32-wasmer-wasi/release/static-web-server.rustc.wasm /prog/deploy/wasmer-web/wapm/web-server.wasm
chmod +x /prog/deploy/wasmer-web/public/bin/web-server.wasm
cp -f target/wasm32-wasmer-wasi/release/static-web-server.rustc.wasm /prog/packages/static-web-server/web-server.wasm
chmod +x /prog/packages/static-web-server/web-server.wasm