# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from collections import OrderedDict, defaultdict

from ..reader import LogHandler


class SummaryHandler(LogHandler):
    """Handler class for storing suite summary information.

    Can handle multiple suites in a single run. Summary
    information is stored on the self.summary instance variable.

    Per suite summary information can be obtained by calling 'get'
    or iterating over this class.
    """

    def __init__(self, **kwargs):
        super(SummaryHandler, self).__init__(**kwargs)

        self.summary = OrderedDict()
        self.current_suite = None

    @property
    def current(self):
        return self.summary.get(self.current_suite)

    def __getitem__(self, suite):
        """Return summary information for the given suite.

        The summary is of the form:

            {
              'counts': {
                '<check>': {
                  'count': int,
                  'expected': {
                    '<status>': int,
                  },
                  'unexpected': {
                    '<status>': int,
                  },
                  'known_intermittent': {
                    '<status>': int,
                  },
                },
              },
              'unexpected_logs': {
                '<test>': [<data>]
              },
              'intermittent_logs': {
                '<test>': [<data>]
              }
            }

        Valid values for <check> are `test`, `subtest` and `assert`. Valid
        <status> keys are defined in the :py:mod:`mozlog.logtypes` module.  The
        <test> key is the id as logged by `test_start`. Finally the <data>
        field is the log data from any `test_end` or `test_status` log messages
        that have an unexpected result.

        Mozlog's structuredlog has a `known_intermittent` field indicating if a
        `test` and `subtest` <status> are expected to arise intermittently.
        Known intermittent results are logged as both as `expected` and
        `known_intermittent`.
        """
        return self.summary[suite]

    def __iter__(self):
        """Iterate over summaries.

        Yields a tuple of (suite, summary). The summary returned is
        the same format as returned by 'get'.
        """
        for suite, data in self.summary.items():
            yield suite, data

    @classmethod
    def aggregate(cls, key, counts, include_skip=True):
        """Helper method for aggregating count data by 'key' instead of by 'check'."""
        assert key in ("count", "expected", "unexpected", "known_intermittent")

        res = defaultdict(int)
        for check, val in counts.items():
            if key == "count":
                res[check] += val[key]
                continue

            for status, num in val[key].items():
                if not include_skip and status == "skip":
                    continue
                res[check] += num
        return res

    def suite_start(self, data):
        self.current_suite = data.get("name", "suite {}".format(len(self.summary) + 1))
        if self.current_suite not in self.summary:
            self.summary[self.current_suite] = {
                "counts": {
                    "test": {
                        "count": 0,
                        "expected": defaultdict(int),
                        "unexpected": defaultdict(int),
                        "known_intermittent": defaultdict(int),
                    },
                    "subtest": {
                        "count": 0,
                        "expected": defaultdict(int),
                        "unexpected": defaultdict(int),
                        "known_intermittent": defaultdict(int),
                    },
                    "assert": {
                        "count": 0,
                        "expected": defaultdict(int),
                        "unexpected": defaultdict(int),
                        "known_intermittent": defaultdict(int),
                    },
                },
                "unexpected_logs": OrderedDict(),
                "intermittent_logs": OrderedDict(),
                "harness_errors": [],
            }

    def test_start(self, data):
        self.current["counts"]["test"]["count"] += 1

    def test_status(self, data):
        logs = self.current["unexpected_logs"]
        intermittent_logs = self.current["intermittent_logs"]
        count = self.current["counts"]
        count["subtest"]["count"] += 1

        if "expected" in data:
            if data["status"] not in data.get("known_intermittent", []):
                count["subtest"]["unexpected"][data["status"].lower()] += 1
                if data["test"] not in logs:
                    logs[data["test"]] = []
                logs[data["test"]].append(data)
            else:
                count["subtest"]["expected"][data["status"].lower()] += 1
                count["subtest"]["known_intermittent"][data["status"].lower()] += 1
                if data["test"] not in intermittent_logs:
                    intermittent_logs[data["test"]] = []
                intermittent_logs[data["test"]].append(data)
        else:
            count["subtest"]["expected"][data["status"].lower()] += 1

    def test_end(self, data):
        logs = self.current["unexpected_logs"]
        intermittent_logs = self.current["intermittent_logs"]
        count = self.current["counts"]
        if "expected" in data:
            if data["status"] not in data.get("known_intermittent", []):
                count["test"]["unexpected"][data["status"].lower()] += 1
                if data["test"] not in logs:
                    logs[data["test"]] = []
                logs[data["test"]].append(data)
            else:
                count["test"]["expected"][data["status"].lower()] += 1
                count["test"]["known_intermittent"][data["status"].lower()] += 1
                if data["test"] not in intermittent_logs:
                    intermittent_logs[data["test"]] = []
                intermittent_logs[data["test"]].append(data)
        else:
            count["test"]["expected"][data["status"].lower()] += 1

    def assertion_count(self, data):
        count = self.current["counts"]
        count["assert"]["count"] += 1

        if data["min_expected"] <= data["count"] <= data["max_expected"]:
            if data["count"] > 0:
                count["assert"]["expected"]["fail"] += 1
            else:
                count["assert"]["expected"]["pass"] += 1
        elif data["max_expected"] < data["count"]:
            count["assert"]["unexpected"]["fail"] += 1
        else:
            count["assert"]["unexpected"]["pass"] += 1

    def log(self, data):
        if not self.current_suite:
            return

        logs = self.current["harness_errors"]
        level = data.get("level").upper()

        if level in ("CRITICAL", "ERROR"):
            logs.append(data)
