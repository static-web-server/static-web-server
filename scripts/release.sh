#!/usr/bin/env bash

echo "Creating a Github release..."

if [[ "$CIRRUS_RELEASE" == "" ]]; then
  echo "Not a Github release. Nothing to deploy!"
  exit 0
fi

if [[ "$GITHUB_TOKEN" == "" ]]; then
  echo "Please provide GitHub access token via GITHUB_TOKEN environment variable!"
  exit 1
fi

file_content_type="application/octet-stream"
files_to_upload=(
  static-web-server-$CIRRUS_TAG-i686-unknown-freebsd.tar.gz
  static-web-server-$CIRRUS_TAG-x86_64-unknown-freebsd.tar.gz
)

for fpath in $files_to_upload
do
  echo "Uploading Github release asset $fpath..."
  name=$(basename "$fpath")
  url_to_upload="https://uploads.github.com/repos/$CIRRUS_REPO_FULL_NAME/releases/$CIRRUS_RELEASE/assets?name=$name"
  curl -X POST \
    --data-binary @$fpath \
    --header "Authorization: token $GITHUB_TOKEN" \
    --header "Content-Type: $file_content_type" \
    $url_to_upload
done

echo
echo "Releases published successfully."
