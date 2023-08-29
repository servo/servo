# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from collections import defaultdict, namedtuple

RunSummary = namedtuple(
    "RunSummary",
    (
        "unexpected_statuses",
        "expected_statuses",
        "known_intermittent_statuses",
        "log_level_counts",
        "action_counts",
    ),
)


class StatusHandler(object):
    """A handler used to determine an overall status for a test run according
    to a sequence of log messages."""

    def __init__(self):
        # The count of each type of unexpected result status (includes tests and subtests)
        self.unexpected_statuses = defaultdict(int)
        # The count of each type of expected result status (includes tests and subtests)
        self.expected_statuses = defaultdict(int)
        # The count of known intermittent result statuses (includes tests and subtests)
        self.known_intermittent_statuses = defaultdict(int)
        # The count of actions logged
        self.action_counts = defaultdict(int)
        # The count of messages logged at each log level
        self.log_level_counts = defaultdict(int)
        # The count of "No tests run" error messages seen
        self.no_tests_run_count = 0

    def __call__(self, data):
        action = data["action"]
        known_intermittent = data.get("known_intermittent", [])
        self.action_counts[action] += 1

        if action == "log":
            if data["level"] == "ERROR" and data["message"] == "No tests ran":
                self.no_tests_run_count += 1
            self.log_level_counts[data["level"]] += 1

        if action in ("test_status", "test_end"):
            status = data["status"]
            # Don't count known_intermittent status as unexpected
            if "expected" in data and status not in known_intermittent:
                self.unexpected_statuses[status] += 1
            else:
                self.expected_statuses[status] += 1
                # Count known_intermittent as expected and intermittent.
                if status in known_intermittent:
                    self.known_intermittent_statuses[status] += 1

        if action == "assertion_count":
            if data["count"] < data["min_expected"]:
                self.unexpected_statuses["PASS"] += 1
            elif data["count"] > data["max_expected"]:
                self.unexpected_statuses["FAIL"] += 1
            elif data["count"]:
                self.expected_statuses["FAIL"] += 1
            else:
                self.expected_statuses["PASS"] += 1

        if action == "lsan_leak":
            if not data.get("allowed_match"):
                self.unexpected_statuses["FAIL"] += 1

        if action == "lsan_summary":
            if not data.get("allowed", False):
                self.unexpected_statuses["FAIL"] += 1

        if action == "mozleak_total":
            if data["bytes"] is not None and data["bytes"] > data.get("threshold", 0):
                self.unexpected_statuses["FAIL"] += 1

    def summarize(self):
        return RunSummary(
            dict(self.unexpected_statuses),
            dict(self.expected_statuses),
            dict(self.known_intermittent_statuses),
            dict(self.log_level_counts),
            dict(self.action_counts),
        )
