#!/bin/bash
set -ex

# This script is intended to be run from the root of the WPT repository.
# It handles syncing tc39/test262 into third_party/test262.

# Ensure we are in the WPT root
SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

SPEC_DIR="${1:-../test262-spec}"

if [ ! -d "$SPEC_DIR" ]; then
    echo "Error: Test262 spec directory not found at $SPEC_DIR"
    exit 1
fi

LATEST_SHA=$(git -C "$SPEC_DIR" rev-parse HEAD)
echo "Latest remote Test262 SHA: $LATEST_SHA"

# Base destination directory
TEST262_DEST="$WPT_ROOT/third_party/test262"
TEST_DEST="$TEST262_DEST/test"
HARNESS_DEST="$TEST262_DEST/harness"

# Ensure directories exist
mkdir -p "$TEST_DEST"
mkdir -p "$HARNESS_DEST"

# Use the persistent exclude list from the destination directory to protect WPT-specific metadata
EXCLUDE_FILE="$TEST262_DEST/.rsync-exclude"

RSYNC_OPTS=(-a --delete --exclude-from="$EXCLUDE_FILE")

# Sync the harness files
rsync "${RSYNC_OPTS[@]}" "$SPEC_DIR/harness/" "$HARNESS_DEST/"

# Sync all tests
rsync "${RSYNC_OPTS[@]}" "$SPEC_DIR/test/" "$TEST_DEST/"

# Write the version info
printf "[test262]\nsource = \"https://github.com/tc39/test262\"\nrev = \"${LATEST_SHA}\"\n" > "$TEST262_DEST/vendored.toml"
