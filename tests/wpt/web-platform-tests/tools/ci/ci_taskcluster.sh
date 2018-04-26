#!/bin/bash

if [ $1 == "firefox" ]; then
    ./wpt run firefox --log-tbpl=- --log-tbpl-level=debug --log-wptreport=../artifacts/wpt_report.json --this-chunk=$3 --total-chunks=$4 --test-type=$2 -y --install-browser --no-pause --no-restart-on-unexpected
elif [ $1 == "chrome" ]; then
    ./wpt run chrome --log-tbpl=- --log-tbpl-level=debug --log-wptreport=../artifacts/wpt_report.json --this-chunk=$3 --total-chunks=$4 --test-type=$2 -y  --no-pause --no-restart-on-unexpected
fi
