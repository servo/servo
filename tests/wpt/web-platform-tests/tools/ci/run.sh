#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

if [[ $RUN_JOB -eq 1 ]]; then
    $SCRIPT
fi
