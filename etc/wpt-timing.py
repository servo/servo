#!/usr/bin/env python

# Copyright 2019 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# Usage: python wpt-timing.py [path/to/wpt.log] ...
#
# Given a series of WPT log files as arguments, this script
# extracts the status of each test file (ok; error; timeout; etc.)
# and how long it took to ran, then creates three CSV files, each
# sorted by runtime:
#
# - longest_ok.csv: all tests that passed
# - longest_err.csv: all tests that failed or had an error
# - timeouts.csv: all tests that timed out
#
# This information can be used to quickly determine the longest-running
# tests in the WPT testsuite in order to improve the overall testsuite
# runtime on CI.

import sys
import json
import collections
import csv


def process_log(data):
    tests = {}
    test_results = collections.defaultdict(list)

    for entry in data:
        entry = json.loads(entry)
        if "action" in entry:
            if entry["action"] == "test_start":
                tests[entry["test"]] = {
                    "start": int(entry["time"]),
                    "end": 0,
                }
            elif entry["action"] == "test_end":
                test = tests[entry["test"]]
                test["end"] = int(entry["time"])
                test_results[entry["status"]] += [
                    (entry["test"], test["end"] - test["start"])
                ]

    return test_results


test_results = {
    "SKIP": [],
    "OK": [],
    "PASS": [],
    "ERROR": [],
    "FAIL": [],
    "CRASH": [],
    "TIMEOUT": [],
}
for log_path in sys.argv[1:]:
    with open(log_path) as f:
        data = f.readlines()
        for k, v in process_log(data).items():
            test_results[k] += v

print("Skipped %d tests." % len(test_results["SKIP"]))
print("%d tests timed out." % len(test_results["TIMEOUT"]))

longest_crash = sorted(test_results["CRASH"], key=lambda x: x[1], reverse=True)
print("Longest CRASH test took %dms (%s)" % (longest_crash[0][1], longest_crash[0][0]))

longest_ok = sorted(
    test_results["PASS"] + test_results["OK"],
    key=lambda x: x[1], reverse=True
)
csv_data = [['Test path', 'Milliseconds']]
with open('longest_ok.csv', 'w') as csv_file:
    writer = csv.writer(csv_file)
    writer.writerows(csv_data + longest_ok)

longest_fail = sorted(
    test_results["ERROR"] + test_results["FAIL"],
    key=lambda x: x[1], reverse=True
)
with open('longest_err.csv', 'w') as csv_file:
    writer = csv.writer(csv_file)
    writer.writerows(csv_data + longest_fail)

longest_timeout = sorted(test_results["TIMEOUT"], key=lambda x: x[1], reverse=True)
with open('timeouts.csv', 'w') as csv_file:
    writer = csv.writer(csv_file)
    writer.writerows(csv_data + longest_timeout)
