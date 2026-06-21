#!/bin/bash

set -eux -o pipefail

echo "Post Release Installer updates for $SERVER_VERSION..."

server_version_num=$(echo $SERVER_VERSION | sed "s/v//")
sed_bk=".bk"

# Update current installer script version
sed -i$sed_bk -e "s/SWS_INSTALL_VERSION\:\-\".*\"/SWS_INSTALL_VERSION\:\-\"$server_version_num\"/g" ./scripts/installer.sh
rm -rf ./*.bk
echo "Installer script's version was updated to $server_version_num!"

echo

echo "All changes after release were done successfully!"
