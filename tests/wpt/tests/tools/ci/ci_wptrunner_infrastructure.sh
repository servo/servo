#!/bin/bash
set -ex

REL_DIR_NAME=$(dirname "$0")
SCRIPT_DIR=$(cd "$REL_DIR_NAME" && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd "$WPT_ROOT"

run_infra_test() {
    echo "### Running Infrastructure Tests for $1 ###"
    ./tools/ci/taskcluster-run.py "$1" "$2" -- --log-tbpl=- --log-wptreport="../artifacts/wptreport-$1.json" --logcat-dir="../artifacts/" --metadata=infrastructure/metadata/ --include=infrastructure/
}

main() {
  run_infra_test "chrome" "dev"
  run_infra_test "firefox" "nightly"
  run_infra_test "firefox_android" "nightly"
}

main
