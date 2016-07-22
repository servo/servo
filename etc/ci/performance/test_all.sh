#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail

case "$1" in
  --servo)
    engine=""
    ;;
  --gecko)
    engine="--engine gecko"
    ;;
  *)
    echo "Can't understand the argument, using the default servo engine"
    echo "Usage: test_all.sh [--gecko]"
    engine=""
    ;;
esac

source venv/bin/activate

echo "Starting the local server"
python3 -m http.server > /dev/null 2>&1 &

#MANIFEST="page_load_test/test.manifest"
MANIFEST="page_load_test/20160509.manifest" # A manifest that excludes timeout test cases
PERF_FILE="output/perf-$(date +"%s").json"

echo "Running tests"
python3 runner.py --runs 3 $MANIFEST $PERF_FILE 
python3 runner.py $engine --runs 3 $MANIFEST $PERF_FILE
sudo ntpdate tw.pool.ntp.org
echo "Submitting to Perfherder"
# XXX: we are using servo's revision to make the graph pretty
python3 submit_to_perfherder.py $engine $PERF_FILE servo/revision.json

# Kill the http server
trap 'kill $(jobs -pr)' SIGINT SIGTERM EXIT
