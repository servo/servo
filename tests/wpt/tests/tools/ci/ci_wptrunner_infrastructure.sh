#!/bin/bash
set -ex

REL_DIR_NAME=$(dirname "$0")
SCRIPT_DIR=$(cd "$REL_DIR_NAME" && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd "$WPT_ROOT"

BROWSER="$1"
CHANNEL="$2"

run_infra_test() {
    echo "### Running Infrastructure Tests for $1 ###"
    ./tools/ci/taskcluster-run.py "$1" "$2" -- --log-tbpl=- --log-wptreport="../artifacts/wptreport-$1.json" --logcat-dir="../artifacts/" --metadata=infrastructure/metadata/ --include=infrastructure/
}

main() {
  run_infra_test "$BROWSER" "$CHANNEL"
}

main
