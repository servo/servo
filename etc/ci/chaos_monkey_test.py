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


TEST_CMD = [
    "./mach",
    "test-wpt",
    "--release",
    "--processes=24",
    "--binary-arg=--random-pipeline-closure-probability=0.1",
    "--binary-arg=--random-pipeline-closure-seed=123",
    "--binary-arg=--multiprocess",
    "--binary-arg=--soft-fail",
    "--log-raw=-",
    # We run the content-security-policy test because it creates
    # cross-origin iframes, which are good for stress-testing pipelines
    "content-security-policy"
]

# Note that there will probably be test failures caused
# by random pipeline closure, so we ignore the status code
# returned by the test command (which is why we can't use check_output).

test_results = Popen(TEST_CMD, stdout=PIPE)
any_crashes = False

for line in test_results.stdout:
    report = json.loads(line.decode('utf-8'))
    if report.get("action") == "process_output":
        print("{} - {}".format(report.get("thread"), report.get("data")))
    status = report.get("status")
    if status:
        print("{} - {} - {}".format(report.get("thread"), status, report.get("test")))
        if status == "CRASH":
            any_crashes = True

if any_crashes:
    sys.exit(1)
