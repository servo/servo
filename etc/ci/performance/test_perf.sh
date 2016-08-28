#!/usr/bin/env bash
wget http://people.mozilla.org/~jmaher/taloszips/zips/tp5n.zip -O tp5n.zip && \
unzip -o tp5n.zip

virtualenv venv --python=/usr/bin/python3
source venv/bin/activate
pip install "treeherder-client>=3.0.0"

mkdir -p servo
mkdir -p output
./git_log_to_json.sh > servo/revision.json && \
cp -r ../../../resources servo && \
cp ../../../target/release/servo servo && \
./test_all.sh --servo
