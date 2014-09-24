#!/bin/bash
#
# Helper script to upload docs to doc.servo.org.
# Requires ghp-import (from pip)
# GitHub API token must be passed in environment var TOKEN

set -e

mkdir -p target/doc
cp -R rust/doc/* target/doc/
cp etc/doc.servo.org/* target/doc/
./mach doc # After copying rust/doc, so that the crate index is correct
ghp-import -n target/doc
git push -qf https://${TOKEN}@github.com/servo/doc.servo.org.git gh-pages

