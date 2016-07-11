#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail

# Don't include body to prevent multiline and unescaped body string
git log -n 1 --pretty=format:'{%n  "commit": "%H",%n  "subject": "%s",%n  "author": {%n    "name": "%aN",%n    "email": "%aE",%n    "timestamp": "%at"%n  }%n  %n}'

