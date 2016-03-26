#!/bin/bash
#
# Helper script to upload docs to doc.servo.org.
# Requires ghp-import (from pip)
# GitHub API token must be passed in environment var TOKEN

set -e

cd "$(dirname $0)/../.."

./mach doc
# etc/doc.servo.org/index.html overwrites $(mach rust-root)/doc/index.html
cp etc/doc.servo.org/* target/doc/

python components/style/list_properties.py

ghp-import -n target/doc
git push -qf https://${TOKEN}@github.com/servo/doc.servo.org.git gh-pages
