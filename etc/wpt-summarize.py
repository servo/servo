#!/usr/bin/env python

# Copyright 2019 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# Usage: python wpt-summarize.py /wpt/test/url.html [--full]
#
# Extract all log lines for a particular test file from a WPT
# logs, outputting invidual JSON objects that can be manipulated
# with tools like jq. If a particular URL results in no output,
# the URL is likely used as a reference test's reference file,
# so passing `--full` will find any output from Servo process
# command lines that include the URL.

import sys
import json

full_search = len(sys.argv) > 3 and sys.argv[3] == '--full'

with open(sys.argv[1]) as f:
    data = f.readlines()
    thread = None
    for entry in data:
        entry = json.loads(entry)
        if thread and "thread" in entry:
            if entry["thread"] == thread:
                print(json.dumps(entry))
                if "action" in entry and entry["action"] == "test_end":
                    thread = None
        else:
            if ("action" in entry
                    and entry["action"] == "test_start"
                    and entry["test"] == sys.argv[2]):
                thread = entry["thread"]
                print(json.dumps(entry))
            elif (full_search
                  and "command" in entry
                  and sys.argv[2] in entry["command"]):
                thread = entry["thread"]
                print(json.dumps(entry))
