# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json
import sys
from subprocess import Popen, PIPE


def is_crash(report):
    status = report.get("status", None)
    if status:
        status.upper() == "CRASH"

TEST_CMD = [
    "./mach",
    "test-wpt",
    "--release",
    "--processes=24",
    "--binary-arg=--random-pipeline-closure-probability=0.02",
    "--binary-arg=--random-pipeline-closure-seed=123",
    "--binary-arg=--multiprocess",
    "--binary-arg=--soft-fail",
    "--log-raw=-",
    "--log-raw=chaos-monkey.log",
    # We run the content-security-policy test because it creates
    # cross-origin iframes, which are good for stress-testing pipelines
    "content-security-policy"
]

# Note that there will probably be test failures caused
# by random pipeline closure, so we ignore the status code
# returned by the test command (which is why we can't use check_output).

TEST_RESULTS = Popen(TEST_CMD, stdout=PIPE)
TEST_CRASHES = False
TEST_STDOUT = {}

for line in TEST_RESULTS.stdout:
    report = json.loads(line.decode('utf-8'))
    if report.get("action") == "process_output":
        print(report.get("thread") + " - " + report.get("data"))
    status = report.get("status")
    if status:
        print(report.get("thread") + " - " + status + " - " + report.get("test"))
        TEST_CRASHES = TEST_CRASHES or (status == "CRASH")

if TEST_CRASHES:
    sys.exit(1)
