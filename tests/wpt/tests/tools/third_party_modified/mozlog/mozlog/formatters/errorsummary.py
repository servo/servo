# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


import json
import math
import os
from collections import defaultdict

from .base import BaseFormatter


class ErrorSummaryFormatter(BaseFormatter):
    def __init__(self):
        self.test_to_group = {}
        self.manifest_groups = set()
        self.groups = defaultdict(
            lambda: {
                "status": None,
                "group_start": None,
                "group_end": None,
                "all_skipped": True,
                "test_starts": {},
                "test_times": [],
            }
        )
        self.test_time_divisor = 1
        self.line_count = 0
        self.dump_passing_tests = False

        if os.environ.get("MOZLOG_DUMP_ALL_TESTS"):
            self.dump_passing_tests = True

    def __call__(self, data):
        rv = BaseFormatter.__call__(self, data)
        self.line_count += 1
        return rv

    def _output(self, data_type, data):
        data["action"] = data_type
        data["line"] = self.line_count
        return "%s\n" % json.dumps(data)

    def _output_test(self, test, subtest, item):
        data = {
            "test": test,
            "subtest": subtest,
            "group": self.test_to_group.get(test, ""),
            "status": item["status"],
            "expected": item["expected"],
            "message": item.get("message"),
            "stack": item.get("stack"),
            "modifiers": item.get("extra", {}).get("modifiers", ""),
            "known_intermittent": item.get("known_intermittent", []),
        }
        return self._output("test_result", data)

    def _get_group_result(self, group, item):
        group_info = self.groups[group]
        result = group_info["status"]

        if result == "ERROR":
            return result

        # If status == expected, we delete item[expected]
        test_status = item["status"]
        test_expected = item.get("expected", test_status)
        known_intermittent = item.get("known_intermittent", [])

        if test_status == "SKIP":
            if result is None:
                result = "SKIP"
        else:
            self.groups[group]["all_skipped"] = False
            if test_status == test_expected or test_status in known_intermittent:
                result = "OK"
            else:
                result = "ERROR"

        return result

    def _clean_test_name(self, test):
        retVal = test
        # remove extra stuff like "(finished)"
        if "(finished)" in test:
            retVal = test.split(" ")[0]
        return retVal

    def suite_start(self, item):
        self.test_to_group = {v: k for k in item["tests"] for v in item["tests"][k]}
        self.manifest_groups = set(item["tests"].keys())
        # Initialize groups with no tests (missing manifests) with SKIP status
        for group, tests in item["tests"].items():
            if not tests:  # Empty test list
                self.groups[group] = {
                    "status": "SKIP",
                    "group_start": None,
                    "group_end": None,
                    "all_skipped": True,
                    "test_starts": {},
                    "test_times": [],
                }
        return self._output("test_groups", {"groups": list(item["tests"].keys())})

    def suite_end(self, data):
        output = []
        for group, info in self.groups.items():
            if info["group_start"] is not None and info["group_end"] is not None:
                duration = info["group_end"] - info["group_start"]
            else:
                duration = sum(info["test_times"])

            output.append(
                self._output(
                    "group_result",
                    {
                        "group": group,
                        "status": info["status"],
                        "duration": duration,
                    },
                )
            )

        return "".join(output)

    def group_start(self, item):
        group = item["name"]
        if group in self.manifest_groups:
            self.groups[group]["group_start"] = item["time"]
        else:
            extra = item.get("extra") or {}
            threads = extra.get("threads")
            if threads and threads > 1:
                self.test_time_divisor = threads

    def group_end(self, item):
        group = item["name"]
        if group in self.manifest_groups and not self.groups[group]["all_skipped"]:
            self.groups[group]["group_end"] = item["time"]
        elif group not in self.manifest_groups:
            self.test_time_divisor = 1

    def test_start(self, item):
        test = self._clean_test_name(item["test"])
        group = item.get("group", self.test_to_group.get(test, None))
        if group:
            self.groups[group]["test_starts"][test] = item["time"]

    def test_status(self, item):
        group = item.get(
            "group", self.test_to_group.get(self._clean_test_name(item["test"]), None)
        )
        if group:
            self.groups[group]["status"] = self._get_group_result(group, item)

        if not self.dump_passing_tests and "expected" not in item:
            return

        if item.get("expected", "") == "":
            item["expected"] = item["status"]

        return self._output_test(
            self._clean_test_name(item["test"]), item["subtest"], item
        )

    def test_end(self, item):
        test = self._clean_test_name(item["test"])
        group = item.get("group", self.test_to_group.get(test, None))
        if group:
            self.groups[group]["status"] = self._get_group_result(group, item)
            start_time = self.groups[group]["test_starts"].pop(test, None)
            if item["status"] != "SKIP" and start_time is not None:
                elapsed = item["time"] - start_time
                self.groups[group]["test_times"].append(
                    math.ceil(elapsed / self.test_time_divisor)
                )

        if not self.dump_passing_tests and "expected" not in item:
            return

        if item.get("expected", "") == "":
            item["expected"] = item["status"]

        return self._output_test(test, None, item)

    def log(self, item):
        if item["level"] not in ("ERROR", "CRITICAL"):
            return

        data = {"level": item["level"], "message": item["message"]}
        return self._output("log", data)

    def shutdown_failure(self, item):
        data = {"status": "FAIL", "test": item["group"], "message": item["message"]}
        data["group"] = [g for g in self.groups if item["group"].endswith(g)]
        if data["group"]:
            data["group"] = data["group"][0]
            self.groups[data["group"]]["status"] = "FAIL"
        else:
            self.log({
                "level": "ERROR",
                "message": "Group '%s' was not found in known groups: %s.  Please look at item: %s"
                % (item["group"], self.groups, item),
            })
        return self._output("log", data)

    def crash(self, item):
        data = {
            "test": item.get("test"),
            "signature": item["signature"],
            "stackwalk_stdout": item.get("stackwalk_stdout"),
            "stackwalk_stderr": item.get("stackwalk_stderr"),
        }

        if item.get("test"):
            data["group"] = self.test_to_group.get(
                self._clean_test_name(item["test"]), ""
            )
            if data["group"] == "":
                # item['test'] could be the group name, not a test name
                if self._clean_test_name(item["test"]) in self.groups:
                    data["group"] = self._clean_test_name(item["test"])

            # unlike test group summary, if we crash expect error unless expected
            if (
                (
                    "expected" in item
                    and "status" in item
                    and item["status"] in item["expected"]
                )
                or ("expected" in item and "CRASH" == item["expected"])
                or "status" in item
                and item["status"] in item.get("known_intermittent", [])
            ):
                self.groups[data["group"]]["status"] = "PASS"
            else:
                self.groups[data["group"]]["status"] = "ERROR"

        return self._output("crash", data)

    def lint(self, item):
        data = {
            "level": item["level"],
            "path": item["path"],
            "message": item["message"],
            "lineno": item["lineno"],
            "column": item.get("column"),
            "rule": item.get("rule"),
            "linter": item.get("linter"),
        }
        self._output("lint", data)
