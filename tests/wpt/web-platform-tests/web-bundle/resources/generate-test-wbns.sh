#!/bin/sh

set -e

if ! command -v gen-bundle > /dev/null 2>&1; then
    echo "gen-bundle is not installed. Please run:"
    echo "  go get -u github.com/WICG/webpackage/go/bundle/cmd/..."
    exit 1
fi

# TODO: Stop hard-coding "web-platform.test" when generating Web Bundles on the
# fly.
wpt_test_origin=https://web-platform.test:8444

gen-bundle \
  -version b1 \
  -baseURL $wpt_test_origin/web-bundle/resources/wbn/ \
  -primaryURL $wpt_test_origin/web-bundle/resources/wbn/location.html \
  -dir location/ \
  -o wbn/location.wbn
