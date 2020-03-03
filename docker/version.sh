#!/usr/bin/env bash

set -e
set -u

LATEST_TAG=$1
BASE_PATH="$(pwd)/docker"

if [ $# -eq 0 ]; then
    echo "Usage: ./version.sh <tag or branch>"
    exit
fi

export VERSION=$LATEST_TAG
export ALPINE_VERSION=3.10

PLATFORMS=(
    "alpine"
    "scratch"
)

for PLATFORM in "${PLATFORMS[@]}"; do
    PLATFORM_DIR="${BASE_PATH}/${PLATFORM}"
    
    if [ ! -d "$PLATFORM_DIR" ]; then
        echo "Directory no found for \"${PLATFORM_DIR}\""
        exit 1
    fi
    
    echo "Generating Dockerfile for platform \"${PLATFORM}\""
    
    rm -rf "${PLATFORM_DIR}/Dockerfile"
    
    envsubst \$ALPINE_VERSION,\$VERSION <"${PLATFORM_DIR}/Dockerfile.tmpl" >"${PLATFORM_DIR}/Dockerfile"
done

echo "All Dockerfiles were updated!"
