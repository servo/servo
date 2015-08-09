#!/usr/bin/env python
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import base

class UnittestFormatter(base.BaseFormatter):
    """Formatter designed to produce output in a format like that used by
    the ``unittest`` module in the standard library."""
    def __init__(self):
        self.fails = []
        self.errors = []
        self.tests_run = 0
        self.start_time = None
        self.end_time = None

    def suite_start(self, data):
        self.start_time = data["time"]

    def test_start(self, data):
        self.tests_run += 1

    def test_end(self, data):
        char = "."
        if "expected" in data:
            status = data["status"]
            char = {"FAIL": "F",
                    "ERROR": "E",
                    "PASS": "X"}[status]

            if status == "FAIL":
                self.fails.append(data)
            elif status == "ERROR":
                self.errors.append(data)

        elif data["status"] == "SKIP":
            char = "S"
        return char

    def suite_end(self, data):
        self.end_time = data["time"]
        summary = "\n".join([self.output_fails(),
                             self.output_errors(),
                             self.output_summary()])
        return "\n%s\n" % summary

    def output_fails(self):
        return "\n".join("FAIL %(test)s\n%(message)s\n" % data
                         for data in self.fails)

    def output_errors(self):
        return "\n".join("ERROR %(test)s\n%(message)s" % data
                         for data in self.errors)

    def output_summary(self):
        return ("Ran %i tests in %.1fs" % (self.tests_run,
                                           (self.end_time - self.start_time) / 1000))
