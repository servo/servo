#!/bin/sh

set -e

if ! command -v gen-bundle > /dev/null 2>&1; then
    echo "gen-bundle is not installed. Please run:"
    echo "  go install github.com/WICG/webpackage/go/bundle/cmd/...@latest"
    echo '  export PATH=$PATH:$(go env GOPATH)/bin'
    exit 1
fi

# TODO: Stop hard-coding "web-platform.test" when generating Web Bundles on the
# fly.
wpt_test_origin=https://web-platform.test:8444
wpt_test_remote_origin=https://www1.web-platform.test:8444

gen-bundle \
  -version b2 \
  -baseURL $wpt_test_origin/web-bundle/resources/wbn/static-element/ \
  -primaryURL $wpt_test_origin/web-bundle/resources/wbn/static-element/resources/style.css \
  -dir static-element/ \
  -o wbn/static-element.wbn

# Create a bundle, nested-main.wbn, which includes nested-sub.wbn.
cp -a wbn/subresource.wbn nested/nested-sub.wbn
gen-bundle \
  -version b2 \
  -baseURL $wpt_test_origin/web-bundle/resources/wbn/ \
  -primaryURL $wpt_test_origin/web-bundle/resources/wbn/resource.js \
  -dir nested/ \
  -o wbn/nested-main.wbn

gen-bundle \
  -version b2 \
  -har non-utf8-query-encoding.har \
  -primaryURL $wpt_test_origin/web-bundle/resources/wbn/static-element/resources/script.js?x=%A4%A2 \
  -o wbn/non-utf8-query-encoding.wbn

gen-bundle \
  -version b2 \
  -har corp.har \
  -primaryURL $wpt_test_remote_origin/web-bundle/resources/wbn/cors/no-corp.js \
  -o wbn/cors/corp.wbn

gen-bundle \
  -version b2 \
  -baseURL $wpt_test_origin/web-bundle/resources/wbn/ \
  -primaryURL $wpt_test_origin/web-bundle/resources/wbn/location.html \
  -dir location/ \
  -o wbn/location.wbn

gen-bundle \
  -version b2 \
  -har relative-url.har \
  -o wbn/relative-url.wbn

gen-bundle \
  -version b2 \
  -baseURL $wpt_test_origin/web-bundle/resources/wbn/ \
  -dir subresource/ \
  -o wbn/subresource.wbn

gen-bundle \
  -version b2 \
  -baseURL $wpt_test_origin/web-bundle/resources/wbn/dynamic/ \
  -dir dynamic1/ \
  -o wbn/dynamic1.wbn

gen-bundle \
  -version b2 \
  -baseURL $wpt_test_origin/web-bundle/resources/wbn/dynamic/ \
  -dir dynamic2/ \
  -o wbn/dynamic2.wbn

gen-bundle \
  -version b2 \
  -baseURL $wpt_test_remote_origin/web-bundle/resources/wbn/dynamic/ \
  -dir dynamic1/ \
  -o wbn/dynamic1-crossorigin.wbn

gen-bundle \
  -version b2 \
  -baseURL $wpt_test_origin/web-bundle/resources/ \
  -dir path-restriction/ \
  -o wbn/path-restriction.wbn

gen-bundle \
  -version b2 \
  -har cross-origin.har \
  -o wbn/cors/cross-origin.wbn

gen-bundle \
  -version b2 \
  -har cross-origin-no-cors.har \
  -o wbn/no-cors/cross-origin.wbn

gen-bundle \
  -version b2 \
  -har uuid-in-package.har \
  -o wbn/uuid-in-package.wbn

gen-bundle \
  -version b2 \
  -har simple-cross-origin.har \
  -o wbn/simple-cross-origin.wbn
