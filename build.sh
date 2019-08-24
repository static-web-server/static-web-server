#!/usr/bin/env bash

set -e
set -u

PKG_PLATFORM="x86_64-unknown-linux-musl"
PKG_NAME=$(cat Cargo.toml | awk "match(\$0, /name = \"(.\*)\"/, v) {print v[1]}")
PKG_VERSION=$(cat Cargo.toml | awk "match(\$0, /version = \"([0-9.].+)\"/, v) {print v[1]}")
PKG_BIN_DIR="./bin"
PKG_BIN_PATH="${PKG_BIN_DIR}/${PKG_NAME}"

# Build binary release
cargo clean
rm -rf bin
cargo build --release --target ${PKG_PLATFORM}
mkdir -p ${PKG_BIN_DIR}
cp -rf target/${PKG_PLATFORM}/release/${PKG_NAME} ${PKG_BIN_DIR}
strip ${PKG_BIN_PATH}
du -sh ${PKG_BIN_PATH}

# Build Docker image
docker build -t ${PKG_NAME} -f ./docker/sws.dockerfile .
