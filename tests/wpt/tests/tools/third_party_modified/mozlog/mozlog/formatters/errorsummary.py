# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


import json
from collections import defaultdict

from .base import BaseFormatter


class ErrorSummaryFormatter(BaseFormatter):
    def __init__(self):
        self.test_to_group = {}
        self.groups = defaultdict(
            lambda: {
                "status": None,
                "start": None,
                "end": None,
            }
        )
        self.line_count = 0

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
            "known_intermittent": item.get("known_intermittent", []),
        }
        return self._output("test_result", data)

    def _update_group_result(self, group, item):
        ginfo = self.groups[group]

        if item["status"] == "SKIP":
            if ginfo["status"] is None:
                ginfo["status"] = "SKIP"
        elif (
            ("expected" not in item and item["status"] in ["OK", "PASS"]) or
            ("expected" in item and item["status"] == item["expected"]) or
            item["status"] in item.get("known_intermittent", [])
        ):
            if ginfo["status"] in (None, "SKIP"):
                ginfo["status"] = "OK"
        else:
            ginfo["status"] = "ERROR"

    def suite_start(self, item):
        self.test_to_group = {v: k for k in item["tests"] for v in item["tests"][k]}
        return self._output("test_groups", {"groups": list(item["tests"].keys())})

    def suite_end(self, data):
        output = []
        for group, info in self.groups.items():
            if info["start"] is None or info["end"] is None:
                duration = None
            else:
                duration = info["end"] - info["start"]

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

    def test_start(self, item):
        group = self.test_to_group.get(item["test"], None)
        if group and self.groups[group]["start"] is None:
            self.groups[group]["start"] = item["time"]

    def test_status(self, item):
        group = self.test_to_group.get(item["test"], None)
        if group:
            self._update_group_result(group, item)

        if "expected" not in item:
            return

        return self._output_test(item["test"], item["subtest"], item)

    def test_end(self, item):
        group = self.test_to_group.get(item["test"], None)
        if group:
            self._update_group_result(group, item)
            self.groups[group]["end"] = item["time"]

        if "expected" not in item:
            return

        return self._output_test(item["test"], None, item)

    def log(self, item):
        if item["level"] not in ("ERROR", "CRITICAL"):
            return

        data = {"level": item["level"], "message": item["message"]}
        return self._output("log", data)

    def crash(self, item):
        data = {
            "test": item.get("test"),
            "signature": item["signature"],
            "stackwalk_stdout": item.get("stackwalk_stdout"),
            "stackwalk_stderr": item.get("stackwalk_stderr"),
        }

        if item.get("test"):
            data["group"] = self.test_to_group.get(item["test"], "")
            if data["group"] == "":
                # item['test'] could be the group name, not a test name
                if item["test"] in self.groups:
                    data["group"] = item["test"]

            # unlike test group summary, if we crash expect error unless expected
            if (
                (
                    "expected" in item and
                    "status" in item and
                    item["status"] in item["expected"]
                ) or
                ("expected" in item and "CRASH" == item["expected"]) or
                "status" in item and
                item["status"] in item.get("known_intermittent", [])
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
