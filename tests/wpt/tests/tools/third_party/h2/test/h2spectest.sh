#!/usr/bin/env bash
# A test script that runs the example Python Twisted server and then runs
# h2spec against it. Prints the output of h2spec. This script does not expect
# to be run directly, but instead via `tox -e h2spec`.

set -x

# Kill all background jobs on exit.
trap 'kill $(jobs -p)' EXIT

pushd examples/asyncio
python asyncio-server.py  &
popd

# Wait briefly to let the server start up
sleep 2

# Go go go!
h2spec -k -t -v -p 8443 $@
