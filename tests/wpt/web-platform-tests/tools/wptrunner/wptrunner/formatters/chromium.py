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

        # Message dictionary, keyed by test name. Value is a concatenation of
        # the subtest messages for this test.
        self.messages = defaultdict(str)

        # List of tests that have failing subtests.
        self.tests_with_subtest_fails = set()

    def _append_test_message(self, test, subtest, status, expected, message):
        """
        Appends the message data for a test.
        :param str test: the name of the test
        :param str subtest: the name of the subtest with the message
        :param str status: the subtest status
        :param str expected: the expected subtest statuses
        :param str message: the string to append to the message for this test

        Here's an example of a message:
        [TIMEOUT expected FAIL] Test Name foo: assert_equals: expected 1 but got 2
        """
        if not message:
            return
        # Add the prefix, with the test status and subtest name (if available)
        prefix = "[%s" % status
        if expected and status not in expected:
            prefix += " expected %s] " % expected
        else:
            prefix += "] "
        if subtest:
            prefix += "%s: " % subtest
        self.messages[test] += prefix + message + "\n"

    def _append_artifact(self, cur_dict, artifact_name, artifact_value):
        """
        Appends artifacts to the specified dictionary.
        :param dict cur_dict: the test leaf dictionary to append to
        :param str artifact_name: the name of the artifact
        :param str artifact_value: the value of the artifact
        """
        if "artifacts" not in cur_dict.keys():
            cur_dict["artifacts"] = {}
        # Artifacts are all expected to be lists, so even though we only have a
        # single |artifact_value| we still put it in a list.
        cur_dict["artifacts"][artifact_name] = [artifact_value]

    def _store_test_result(self, name, actual, expected, message, wpt_actual, subtest_failure):
        """
        Stores the result of a single test in |self.tests|
        :param str name: name of the test.
        :param str actual: actual status of the test.
        :param str expected: expected statuses of the test.
        :param str message: test output, such as status, subtest, errors etc.
        :param str wpt_actual: actual status reported by wpt, may differ from |actual|.
        :param bool subtest_failure: whether this test failed because of subtests
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
        if subtest_failure:
            self._append_artifact(cur_dict, "wpt_subtest_failure", "true")
        if wpt_actual != actual:
            self._append_artifact(cur_dict, "wpt_actual_status", wpt_actual)
        if message != "":
            self._append_artifact(cur_dict, "log", message)

        # Figure out if there was a regression or unexpected status. This only
        # happens for tests that were run
        if actual != "SKIP":
            if actual not in expected:
                cur_dict["is_unexpected"] = True
                if actual != "PASS":
                    cur_dict["is_regression"] = True

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
        if status in ("ERROR", "CRASH", "PRECONDITION_FAILED"):
            # CRASH in WPT means a browser crash, which Chromium treats as a
            # test failure.
            return "FAIL"
        if status == "INTERNAL-ERROR":
            # CRASH in Chromium refers to an error in the test runner not the
            # browser.
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
        "known_intermittent" in |data|. If these exist, they will be appended to
        the returned status with spaced in between.

        :param str actual_status: the actual status of the test
        :param data: a data dictionary to extract expected status from
        :return str: the expected statuses as a string
        """
        expected_statuses = self._map_status_name(data["expected"]) if "expected" in data else actual_status
        if data.get("known_intermittent"):
            expected_statuses += " " + " ".join(
                [self._map_status_name(other_status) for other_status in data["known_intermittent"]])
        return expected_statuses

    def suite_start(self, data):
        # |data| contains a timestamp in microseconds, while time.time() gives
        # it in seconds.
        self.start_timestamp_seconds = (float(data["time"]) / 1000 if "time" in data
                                        else time.time())

    def test_status(self, data):
        test_name = data["test"]
        actual_status = self._map_status_name(data["status"])
        expected_statuses = self._get_expected_status_from_data(actual_status, data)

        is_unexpected = actual_status not in expected_statuses
        if is_unexpected and test_name not in self.tests_with_subtest_fails:
            self.tests_with_subtest_fails.add(test_name)
        if "message" in data:
            self._append_test_message(test_name, data["subtest"], actual_status, expected_statuses, data["message"])

    def test_end(self, data):
        test_name = data["test"]
        # Save the status reported by WPT since we might change it when reporting
        # to Chromium.
        wpt_actual_status = data["status"]
        actual_status = self._map_status_name(wpt_actual_status)
        expected_statuses = self._get_expected_status_from_data(actual_status, data)
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

        if "message" in data:
            self._append_test_message(test_name, None, actual_status,
                                      expected_statuses, data["message"])
        self._store_test_result(test_name, actual_status, expected_statuses,
                                self.messages[test_name], wpt_actual_status,
                                subtest_failure)

        # Remove the test from messages dict to avoid accumulating too many.
        self.messages.pop(test_name)

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
