#!/usr/bin/env bash

set -e
set -u

PKG_PLATFORM="x86_64-unknown-linux-musl"
PKG_NAME=$(cat Cargo.toml | sed -n 's/name = "\([^}]*\)"/\1/p')
PKG_VERSION=$(cat Cargo.toml | sed -n 's/version = "\([^}]*\)"/\1/p')
PKG_BIN_DIR="./bin"
PKG_BIN_PATH="${PKG_BIN_DIR}/${PKG_NAME}"

# Build binary release
cargo clean
rm -rf bin
env CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc \
    cargo build --release --target ${PKG_PLATFORM}
mkdir -p ${PKG_BIN_DIR}
cp -rf target/${PKG_PLATFORM}/release/${PKG_NAME} ${PKG_BIN_DIR}
strip ${PKG_BIN_PATH}
du -sh ${PKG_BIN_PATH}

# Build Docker image
docker build -t ${PKG_NAME} -f ./docker/sws.dockerfile .
