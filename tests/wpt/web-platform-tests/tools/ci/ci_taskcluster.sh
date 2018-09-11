#!/bin/bash
set -ex

# This is allowed to fail
./wpt manifest-download || echo

if [ $1 == "firefox" ]; then
    ./wpt run firefox --log-tbpl=../artifacts/log_tbpl.log --log-tbpl-level=info --log-wptreport=../artifacts/wpt_report.json --log-mach=- --this-chunk=$4 --total-chunks=$5 --test-type=$3 -y --install-browser --channel=$2 --no-pause --no-restart-on-unexpected --reftest-internal --install-fonts --no-fail-on-unexpected
elif [ $1 == "chrome" ]; then
    ./wpt run chrome --log-tbpl=../artifacts/log_tbpl.log --log-tbpl-level=info --log-wptreport=../artifacts/wpt_report.json --log-mach=- --channel=$2 --this-chunk=$4 --total-chunks=$5 --test-type=$3 -y  --no-pause --no-restart-on-unexpected --install-fonts --no-fail-on-unexpected
fi
gzip ../artifacts/wpt_report.json
