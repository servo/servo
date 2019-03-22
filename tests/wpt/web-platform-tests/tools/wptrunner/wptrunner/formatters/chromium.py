import json
import time

from collections import defaultdict
from mozlog.formatters import base


class ChromiumFormatter(base.BaseFormatter):
    """Formatter to produce results matching the Chromium JSON Test Results format.
    https://chromium.googlesource.com/chromium/src/+/master/docs/testing/json_test_results_format.md
    """

    def __init__(self):
        # Whether the run was interrupted, either by the test runner or user.
        self.interrupted = False

        # A map of test status to the number of tests that had that status.
        self.num_failures_by_status = defaultdict(int)

        # Start time, expressed as offset since UNIX epoch in seconds.
        self.start_timestamp_seconds = None

        # Trie of test results. Each directory in the test name is a node in
        # the trie and the leaf contains the dict of per-test data.
        self.tests = {}

    def _store_test_result(self, name, actual, expected):
        """
        Stores the result of a single test in |self.tests|
        :param str name: name of the test.
        :param str actual: actual status of the test.
        :param str expected: expected status of the test.
        """
        # The test name can contain a leading / which will produce an empty
        # string in the first position of the list returned by split. We use
        # filter(None) to remove such entries.
        name_parts = filter(None, name.split("/"))
        cur_dict = self.tests
        for name_part in name_parts:
            cur_dict = cur_dict.setdefault(name_part, {})
        cur_dict["actual"] = actual
        cur_dict["expected"] = expected

    def _map_status_name(self, status):
        """
        Maps a WPT status to a Chromium status.

        Chromium has five main statuses that we have to map to:
        CRASH: the test harness crashed
        FAIL: the test did not run as expected
        PASS: the test ran as expected
        SKIP: the test was not run
        TIMEOUT: the did not finish in time and was aborted

        :param str status: the string status of a test from WPT
        :return: a corresponding string status for Chromium
        """
        if status == "OK":
            return "PASS"
        if status == "NOTRUN":
            return "SKIP"
        if status == "EXTERNAL-TIMEOUT":
            return "TIMEOUT"
        if status in ("ERROR", "CRASH"):
            # CRASH in WPT means a browser crash, which Chromium treats as a
            # test failure.
            return "FAIL"
        if status == "INTERNAL-ERROR":
            # CRASH in Chromium refers to an error in the test runner not the
            # browser.
            return "CRASH"
        # Any other status just gets returned as-is.
        return status

    def suite_start(self, data):
        self.start_timestamp_seconds = data["time"] if "time" in data else time.time()

    def test_end(self, data):
        actual_status = self._map_status_name(data["status"])
        expected_status = self._map_status_name(data["expected"]) if "expected" in data else "PASS"
        self._store_test_result(data["test"], actual_status, expected_status)

        # Update the count of how many tests ran with each status.
        self.num_failures_by_status[actual_status] += 1

    def suite_end(self, data):
        # Create the final result dictionary
        final_result = {
            # There are some required fields that we just hard-code.
            "interrupted": False,
            "path_delimiter": "/",
            "version": 3,
            "seconds_since_epoch": self.start_timestamp_seconds,
            "num_failures_by_type": self.num_failures_by_status,
            "tests": self.tests
        }
        return json.dumps(final_result)
