#!/bin/bash
#
# Helper script to upload docs to doc.servo.org.
# Requires ghp-import (from pip)
# GitHub API token must be passed in environment var TOKEN

set -e

cd "$(dirname $0)/../.."

mkdir -p components/servo/target/doc
./mach bootstrap-rust
# Ordered so that:
# * etc/doc.servo.org/index.html overwrites $(mach rust-root)/doc/index.html
# * ./mach doc overwrites $(mach rust-root)/doc/search-index.js
cp -R $(./mach rust-root)/doc/* components/servo/target/doc/
cp etc/doc.servo.org/* components/servo/target/doc/
./mach doc

ghp-import -n components/servo/target/doc
git push -qf https://${TOKEN}@github.com/servo/doc.servo.org.git gh-pages
