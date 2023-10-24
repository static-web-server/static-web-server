#!/bin/bash

set -e

echo "Getting latest release number..."

curl \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: token $GITHUB_TOKEN" \
    "https://api.github.com/repos/static-web-server/static-web-server/releases?per_page=5&page=1" | \
jq -c "[ .[] | select( .tag_name | contains(\"v2.\")) ] | .[0]" | \
jq -r ".tag_name" > /tmp/version

version=$(cat /tmp/version)

echo "SERVER_VERSION=${version#*v}" > /tmp/version

echo "Version saved on '/tmp/version'"
