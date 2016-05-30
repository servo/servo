#!/bin/bash -e
export DISPLAY=:0 # In servo-linux1

source venv/bin/activate

echo "Staring the local server"
python3 -m http.server > /dev/null 2>&1 &

#MANIFEST="page_load_test/test.manifest"
MANIFEST="page_load_test/20160509.manifest" # A manifest that excludes timeout test cases
PERF_FILE="output/perf-$(date +"%s").json"

echo "Running tests"
python3 runner.py --runs 3 $MANIFEST $PERF_FILE 
sudo ntpdate tw.pool.ntp.org
echo "Submitting to Perfherder"
python3 submit_to_perfherder.py $PERF_FILE servo/revision.json

# Kill the http server
trap 'kill $(jobs -pr)' SIGINT SIGTERM EXIT
