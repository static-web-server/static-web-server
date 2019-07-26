#!/usr/bin/env bash

set -e
set -u

BINARY_NAME="static-web-server"
TARGET="x86_64-unknown-linux-musl"
USERNAME=$(git config --global user.github)

cargo release --skip-push --skip-publish $1

TAG_LATEST=$(git describe --tags --abbrev=0)

github-release release \
    --user $USERNAME \
    --repo $BINARY_NAME \
    --tag $TAG_LATEST \
    --name $TAG_LATEST $2

TARBALL_NAME="${BINARY_NAME}-${TAG_LATEST}-${TARGET}.tar.gz"
TMP_DIR="/tmp"
TARBALL_FILE="${TMP_DIR}/${TARBALL_NAME}"

cp bin/${BINARY_NAME} /tmp
cd $TMP_DIR
tar -zcvf ${TARBALL_NAME} ${BINARY_NAME}

github-release upload \
    --user $USERNAME \
    --repo $BINARY_NAME \
    --tag $TAG_LATEST \
    --name $TARBALL_NAME \
    --file $TARBALL_FILE
