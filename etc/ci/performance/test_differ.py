#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import argparse
import json


parser = argparse.ArgumentParser(description="Diff between two runs of performance tests.")
parser.add_argument("file1", help="the first output json from runner")
parser.add_argument("file2", help="the second output json from runner")

args = parser.parse_args()


def load_data(filename):
    with open(filename, 'r') as f:
        results = {}
        totals = {}
        counts = {}
        records = json.load(f)
        for record in records:
            key = record.get('testcase')
            value = record.get('domComplete') - record.get('domLoading')
            totals[key] = totals.get('key', 0) + value
            counts[key] = counts.get('key', 0) + 1
            results[key] = round(totals[key] / counts[key])
        return results


data1 = load_data(args.file1)
data2 = load_data(args.file2)
keys = set(data1.keys()).union(data2.keys())

BLUE = '\033[94m'
GREEN = '\033[92m'
WARNING = '\033[93m'
END = '\033[0m'


total1 = 0
total2 = 0


def print_line(value1, value2, key):
    diff = value2 - value1
    change = diff / value1
    color = BLUE if value1 <= value2 else GREEN
    print("{}{:6} {:6} {:+6} {:+8.2%}   {}.{}".format(color, value1, value2, diff, change, key, END))


for key in keys:
    value1 = data1.get(key)
    value2 = data2.get(key)
    if value1 and not (value2):
        print("{}Test {}: missing from {}.{}".format(WARNING, key, args.file2, END))
    elif value2 and not (value1):
        print("{}Test {}: missing from {}.{}".format(WARNING, key, args.file1, END))
    elif value1 and value2:
        total1 += value1
        total2 += value2
        print_line(value1, value2, key)


print("")
print_line(total1, total2, "TOTAL")
