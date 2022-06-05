# mypy: allow-untyped-defs

import json
import time

from collections import defaultdict
from mozlog.formatters import base


class ChromiumFormatter(base.BaseFormatter):  # type: ignore
    """Formatter to produce results matching the Chromium JSON Test Results format.
    https://chromium.googlesource.com/chromium/src/+/master/docs/testing/json_test_results_format.md

    Notably, each test has an "artifacts" field that is a dict consisting of
        "log": a list of strings (one per subtest + one for harness status, see
            _append_test_message for the format)
        "screenshots": a list of strings in the format of "url: base64"

    """

    def __init__(self):
        # Whether the run was interrupted, either by the test runner or user.
        self.interrupted = False

        # A map of test status to the number of tests that had that status.
        self.num_failures_by_status = defaultdict(int)

        # Start time, expressed as offset since UNIX epoch in seconds. Measured
        # from the first `suite_start` event.
        self.start_timestamp_seconds = None

        # A map of test names to test start timestamps, expressed in seconds
        # since UNIX epoch. Only contains tests that are currently running
        # (i.e., have not received the `test_end` event).
        self.test_starts = {}

        # Trie of test results. Each directory in the test name is a node in
        # the trie and the leaf contains the dict of per-test data.
        self.tests = {}

        # Two dictionaries keyed by test name. Values are lists of strings:
        # actual metadata content and other messages, respectively.
        # See _append_test_message for examples.
        self.actual_metadata = defaultdict(list)
        self.messages = defaultdict(list)

        # List of tests that have failing subtests.
        self.tests_with_subtest_fails = set()

        # Browser log for the current test under execution.
        # These logs are from ChromeDriver's stdout/err, so we cannot say for
        # sure which test a message is from, but instead we correlate them based
        # on timing.
        self.browser_log = []

    def _append_test_message(self, test, subtest, wpt_actual_status, message):
        r"""
        Appends the message data for a test or subtest.

        :param str test: the name of the test
        :param str subtest: the name of the subtest with the message. Will be
                            None if this is called for a test.
        :param str wpt_actual_status: the test status as reported by WPT
        :param str message: the string to append to the message for this test

        Example actual_metadata of a test with a subtest:
            "[test_name]\n  expected: OK\n"
            "  [subtest_name]\n    expected: FAIL\n"

        NOTE: throughout this function we output a key called "expected" but
        fill it in with the actual status. This is by design. The goal of this
        output is to look exactly like WPT's expectation metadata so that it
        can be easily diff-ed.

        Messages are appended verbatim to self.messages[test].
        """
        if subtest:
            result = "  [%s]\n    expected: %s\n" % (subtest, wpt_actual_status)
            self.actual_metadata[test].append(result)
            if message:
                self.messages[test].append("%s: %s\n" % (subtest, message))
        else:
            # No subtest, so this is the top-level test. The result must be
            # prepended to the list, so that it comes before any subtest.
            test_name_last_part = test.split("/")[-1]
            result = "[%s]\n  expected: %s\n" % (test_name_last_part, wpt_actual_status)
            self.actual_metadata[test].insert(0, result)
            if message:
                self.messages[test].insert(0, "Harness: %s\n" % message)

    def _append_artifact(self, cur_dict, artifact_name, artifact_value):
        """
        Appends artifacts to the specified dictionary.
        :param dict cur_dict: the test leaf dictionary to append to
        :param str artifact_name: the name of the artifact
        :param str artifact_value: the value of the artifact
        """
        assert isinstance(artifact_value, str), "artifact_value must be a str"
        if "artifacts" not in cur_dict.keys():
            cur_dict["artifacts"] = defaultdict(list)
        cur_dict["artifacts"][artifact_name].append(artifact_value)

    def _store_test_result(self, name, actual, expected, actual_metadata,
                           messages, wpt_actual, subtest_failure,
                           duration=None, reftest_screenshots=None):
        """
        Stores the result of a single test in |self.tests|

        :param str name: name of the test.
        :param str actual: actual status of the test.
        :param str expected: expected statuses of the test.
        :param list actual_metadata: a list of metadata items.
        :param list messages: a list of test messages.
        :param str wpt_actual: actual status reported by wpt, may differ from |actual|.
        :param bool subtest_failure: whether this test failed because of subtests.
        :param Optional[float] duration: time it took in seconds to run this test.
        :param Optional[list] reftest_screenshots: see executors/base.py for definition.
        """
        # The test name can contain a leading / which will produce an empty
        # string in the first position of the list returned by split. We use
        # filter(None) to remove such entries.
        name_parts = filter(None, name.split("/"))
        cur_dict = self.tests
        for name_part in name_parts:
            cur_dict = cur_dict.setdefault(name_part, {})
        # Splitting and joining the list of statuses here avoids the need for
        # recursively postprocessing the |tests| trie at shutdown. We assume the
        # number of repetitions is typically small enough for the quadratic
        # runtime to not matter.
        statuses = cur_dict.get("actual", "").split()
        statuses.append(actual)
        cur_dict["actual"] = " ".join(statuses)
        cur_dict["expected"] = expected
        if duration is not None:
            # Record the time to run the first invocation only.
            cur_dict.setdefault("time", duration)
            durations = cur_dict.setdefault("times", [])
            durations.append(duration)
        if subtest_failure:
            self._append_artifact(cur_dict, "wpt_subtest_failure", "true")
        if wpt_actual != actual:
            self._append_artifact(cur_dict, "wpt_actual_status", wpt_actual)
        if wpt_actual == 'CRASH':
            for line in self.browser_log:
                self._append_artifact(cur_dict, "wpt_crash_log", line)
        for metadata in actual_metadata:
            self._append_artifact(cur_dict, "wpt_actual_metadata", metadata)
        for message in messages:
            self._append_artifact(cur_dict, "wpt_log", message)

        # Store screenshots (if any).
        for item in reftest_screenshots or []:
            if not isinstance(item, dict):
                # Skip the relation string.
                continue
            data = "%s: %s" % (item["url"], item["screenshot"])
            self._append_artifact(cur_dict, "screenshots", data)

        # Figure out if there was a regression, unexpected status, or flake.
        # This only happens for tests that were run
        if actual != "SKIP":
            if actual not in expected:
                cur_dict["is_unexpected"] = True
                if actual != "PASS":
                    cur_dict["is_regression"] = True
            if len(set(statuses)) > 1:
                cur_dict["is_flaky"] = True

        # Update the count of how many tests ran with each status. Only includes
        # the first invocation's result in the totals.
        if len(statuses) == 1:
            self.num_failures_by_status[actual] += 1

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
        if status in ("ERROR", "PRECONDITION_FAILED"):
            return "FAIL"
        if status == "INTERNAL-ERROR":
            return "CRASH"
        # Any other status just gets returned as-is.
        return status

    def _get_expected_status_from_data(self, actual_status, data):
        """
        Gets the expected statuses from a |data| dictionary.

        If there is no expected status in data, the actual status is returned.
        This is because mozlog will delete "expected" from |data| if it is the
        same as "status". So the presence of "expected" implies that "status" is
        unexpected. Conversely, the absence of "expected" implies the "status"
        is expected. So we use the "expected" status if it's there or fall back
        to the actual status if it's not.

        If the test has multiple statuses, it will have other statuses listed as
        "known_intermittent" in |data|. If these exist, they will be added to
        the returned status with spaced in between.

        :param str actual_status: the actual status of the test
        :param data: a data dictionary to extract expected status from
        :return str: the expected statuses as a string
        """
        expected_statuses = self._map_status_name(data["expected"]) if "expected" in data else actual_status
        if data.get("known_intermittent"):
            all_statsues = {self._map_status_name(other_status) for other_status in data["known_intermittent"]}
            all_statsues.add(expected_statuses)
            expected_statuses = " ".join(sorted(all_statsues))
        return expected_statuses

    def _get_time(self, data):
        """Get the timestamp of a message in seconds since the UNIX epoch."""
        maybe_timestamp_millis = data.get("time")
        if maybe_timestamp_millis is not None:
            return float(maybe_timestamp_millis) / 1000
        return time.time()

    def _time_test(self, test_name, data):
        """Time how long a test took to run.

        :param str test_name: the name of the test to time
        :param data: a data dictionary to extract the test end timestamp from
        :return Optional[float]: a nonnegative duration in seconds or None if
                                 the measurement is unavailable or invalid
        """
        test_start = self.test_starts.pop(test_name, None)
        if test_start is not None:
            # The |data| dictionary only provides millisecond resolution
            # anyway, so further nonzero digits are unlikely to be meaningful.
            duration = round(self._get_time(data) - test_start, 3)
            if duration >= 0:
                return duration
        return None

    def suite_start(self, data):
        if self.start_timestamp_seconds is None:
            self.start_timestamp_seconds = self._get_time(data)

    def test_start(self, data):
        test_name = data["test"]
        self.test_starts[test_name] = self._get_time(data)

    def test_status(self, data):
        test_name = data["test"]
        wpt_actual_status = data["status"]
        actual_status = self._map_status_name(wpt_actual_status)
        expected_statuses = self._get_expected_status_from_data(actual_status, data)

        is_unexpected = actual_status not in expected_statuses
        if is_unexpected and test_name not in self.tests_with_subtest_fails:
            self.tests_with_subtest_fails.add(test_name)
        # We should always get a subtest in the data dict, but it's technically
        # possible that it's missing. Be resilient here.
        subtest_name = data.get("subtest", "UNKNOWN SUBTEST")
        self._append_test_message(test_name, subtest_name,
                                  wpt_actual_status, data.get("message", ""))

    def test_end(self, data):
        test_name = data["test"]
        # Save the status reported by WPT since we might change it when
        # reporting to Chromium.
        wpt_actual_status = data["status"]
        actual_status = self._map_status_name(wpt_actual_status)
        expected_statuses = self._get_expected_status_from_data(actual_status, data)
        duration = self._time_test(test_name, data)
        subtest_failure = False
        if test_name in self.tests_with_subtest_fails:
            subtest_failure = True
            # Clean up the test list to avoid accumulating too many.
            self.tests_with_subtest_fails.remove(test_name)
            # This test passed but it has failing subtests. Since we can only
            # report a single status to Chromium, we choose FAIL to indicate
            # that something about this test did not run correctly.
            if actual_status == "PASS":
                actual_status = "FAIL"

        self._append_test_message(test_name, None, wpt_actual_status,
                                  data.get("message", ""))
        self._store_test_result(test_name,
                                actual_status,
                                expected_statuses,
                                self.actual_metadata[test_name],
                                self.messages[test_name],
                                wpt_actual_status,
                                subtest_failure,
                                duration,
                                data.get("extra", {}).get("reftest_screenshots"))

        # Remove the test from dicts to avoid accumulating too many.
        self.actual_metadata.pop(test_name)
        self.messages.pop(test_name)

        # New test, new browser logs.
        self.browser_log = []

    def shutdown(self, data):
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

    def process_output(self, data):
        cmd = data.get("command", "")
        if any(c in cmd for c in ["chromedriver", "logcat"]):
            self.browser_log.append(data['data'])
