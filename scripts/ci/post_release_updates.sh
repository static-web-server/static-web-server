#!/bin/bash

set -eux -o pipefail

echo "Generating checksums for $SERVER_VERSION..."

release_json="/tmp/release-$SERVER_VERSION.json"
curl \
    -Lo $release_json \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: token $GITHUB_TOKEN" \
    "https://api.github.com/repos/static-web-server/static-web-server/releases/tags/$SERVER_VERSION"

release_dir=/tmp/release-$SERVER_VERSION

cwd=$(pwd)
rm -rf $release_dir
mkdir -p $release_dir
cd $release_dir

server_version_num=$(echo $SERVER_VERSION | sed "s/v//")

# Download precompiled binary assets
while read -r file_url; do
    curl -LO --progress-bar $file_url
done < <(cat $release_json | jq -r ".assets[] | .browser_download_url")

echo "Downloading source code assets..."
curl -Lo static-web-server-$server_version_num.zip --progress-bar \
    https://github.com/static-web-server/static-web-server/archive/refs/tags/$SERVER_VERSION.zip
curl -Lo static-web-server-$server_version_num.tar.gz --progress-bar \
    https://github.com/static-web-server/static-web-server/archive/refs/tags/$SERVER_VERSION.tar.gz

# Compose checksum file name
checksum_file_name="static-web-server-$SERVER_VERSION-SHA256SUM"
rm -rf $checksum_file_name

echo "Calculating and verifying checksum file..."
sha256sum static-web-server-* > $checksum_file_name
sha256sum -c $checksum_file_name

echo "Uploading checksum file to $SERVER_VERSION GitHub release..."
release_id=$(cat $release_json | jq -r ".id")
curl -LX POST \
    --data-binary @$checksum_file_name \
    --header "Authorization: token $GITHUB_TOKEN" \
    --header "Content-Type: application/octet-stream" \
    https://uploads.github.com/repos/static-web-server/static-web-server/releases/$release_id/assets?name=$checksum_file_name

echo "Checksum file uploaded successfully!"
echo

echo "Updating $SERVER_VERSION checksums for 'Download and Install' page..."

cd $cwd
release_date=$(date +%Y-%m-%d)
filename_version="static-web-server-$SERVER_VERSION"
filename_version_num="static-web-server-$server_version_num"

sed_fname=""
if [[ $(uname -s) = "Darwin" ]]; then
    sed_fname=" ''"
fi

# Replace placeholder occurrences
sed "s/{{RELEASE_DATE}}/$release_date/g" docs/content/download-and-install.tmpl.md > docs/content/download-and-install.md

while read -r line; do
    checksum=$(echo $line | awk -F ' ' '{print $1}')
    filename=$(echo $line | awk -F ' ' '{print $2}')
    placeholder_checksum=$(echo $filename | sed "s/^$filename_version-//")

    if [[ "$placeholder_checksum" = "$filename_version_num.tar.gz" ]]; then
        sed -i$sed_fname -e "s/{{SRC_TAR}}/$checksum/" docs/content/download-and-install.md
        continue
    fi

    if [[ "$placeholder_checksum" = "$filename_version_num.zip" ]]; then
        sed -i$sed_fname -e "s/{{SRC_ZIP}}/$checksum/" docs/content/download-and-install.md
        continue
    fi

    sed -i$sed_fname -e "s/{{$placeholder_checksum}}/$checksum/" docs/content/download-and-install.md
done < <(cat "$release_dir/$checksum_file_name")

sed -i$sed_fname -e "s/{{RELEASE_VERSION}}/$SERVER_VERSION/g" docs/content/download-and-install.md
sed -i$sed_fname -e "s/{{RELEASE_VERSION_NUM}}/$server_version_num/g" docs/content/download-and-install.md
echo "Download and install page updated!"

# Update current installer script version
sed -i$sed_fname -e "s/version=\".*\"/version=\"$server_version_num\"/g" $cwd/scripts/installer.sh
echo "Installer script $server_version_num version updated!"

echo
echo "Committing post release page updates..."
git config user.name "github-actions"
git config user.email "actions@users.noreply.github.com"
git checkout master
git pull origin master
git add docs/content/download-and-install.md
git add scripts/installer.sh
git commit -m "chore: release $SERVER_VERSION updates [skip ci]" || exit 0
git push origin master

echo "All changes after release were done successfully!"
