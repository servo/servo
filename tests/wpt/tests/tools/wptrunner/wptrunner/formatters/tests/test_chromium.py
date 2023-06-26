# mypy: ignore-errors

import json
import sys
from os.path import dirname, join
from io import StringIO

from mozlog import handlers, structuredlog
import pytest

sys.path.insert(0, join(dirname(__file__), "..", ".."))
from formatters.chromium import ChromiumFormatter


@pytest.fixture
def logger():
    test_logger = structuredlog.StructuredLogger("test_a")
    try:
        yield test_logger
    finally:
        # Loggers of the same name share state globally:
        #   https://searchfox.org/mozilla-central/rev/1c54648c082efdeb08cf6a5e3a8187e83f7549b9/testing/mozbase/mozlog/mozlog/structuredlog.py#195-196
        #
        # Resetting the state here ensures the logger will not be shut down in
        # the next test.
        test_logger.reset_state()


def test_chromium_required_fields(logger, capfd):
    # Test that the test results contain a handful of required fields.

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # output a bunch of stuff
    logger.suite_start(["test-id-1"], run_info={}, time=123)
    logger.test_start("test-id-1")
    logger.test_end("test-id-1", status="PASS", expected="PASS")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_obj = json.load(output)

    # Check for existence of required fields
    assert "interrupted" in output_obj
    assert "path_delimiter" in output_obj
    assert "version" in output_obj
    assert "num_failures_by_type" in output_obj
    assert "tests" in output_obj

    test_obj = output_obj["tests"]["test-id-1"]
    assert "actual" in test_obj
    assert "expected" in test_obj


def test_time_per_test(logger, capfd):
    # Test that the formatter measures time per test correctly.

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    logger.suite_start(["test-id-1", "test-id-2"], run_info={}, time=50)
    logger.test_start("test-id-1", time=100)
    logger.test_start("test-id-2", time=200)
    logger.test_end("test-id-1", status="PASS", expected="PASS", time=300)
    logger.test_end("test-id-2", status="PASS", expected="PASS", time=199)
    logger.suite_end()

    logger.suite_start(["test-id-1"], run_info={}, time=400)
    logger.test_start("test-id-1", time=500)
    logger.test_end("test-id-1", status="PASS", expected="PASS", time=600)
    logger.suite_end()

    # Write the final results.
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_obj = json.load(output)

    test1_obj = output_obj["tests"]["test-id-1"]
    test2_obj = output_obj["tests"]["test-id-2"]
    # Test 1 run 1: 300ms - 100ms = 0.2s
    # Test 1 run 2: 600ms - 500ms = 0.1s
    assert test1_obj["time"] == pytest.approx(0.2)
    assert len(test1_obj["times"]) == 2
    assert test1_obj["times"][0] == pytest.approx(0.2)
    assert test1_obj["times"][1] == pytest.approx(0.1)
    assert "time" not in test2_obj
    assert "times" not in test2_obj


def test_chromium_test_name_trie(logger, capfd):
    # Ensure test names are broken into directories and stored in a trie with
    # test results at the leaves.

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # output a bunch of stuff
    logger.suite_start(["/foo/bar/test-id-1", "/foo/test-id-2"], run_info={},
                       time=123)
    logger.test_start("/foo/bar/test-id-1")
    logger.test_end("/foo/bar/test-id-1", status="TIMEOUT", expected="FAIL")
    logger.test_start("/foo/test-id-2")
    logger.test_end("/foo/test-id-2", status="ERROR", expected="TIMEOUT")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_obj = json.load(output)

    # Ensure that the test names are broken up by directory name and that the
    # results are stored at the leaves.
    test_obj = output_obj["tests"]["foo"]["bar"]["test-id-1"]
    assert test_obj["actual"] == "TIMEOUT"
    assert test_obj["expected"] == "FAIL"

    test_obj = output_obj["tests"]["foo"]["test-id-2"]
    # The ERROR status is mapped to FAIL for Chromium
    assert test_obj["actual"] == "FAIL"
    assert test_obj["expected"] == "TIMEOUT"


def test_num_failures_by_type(logger, capfd):
    # Test that the number of failures by status type is correctly calculated.

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run some tests with different statuses: 3 passes, 1 timeout
    logger.suite_start(["t1", "t2", "t3", "t4"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="PASS", expected="PASS")
    logger.test_start("t2")
    logger.test_end("t2", status="PASS", expected="PASS")
    logger.test_start("t3")
    logger.test_end("t3", status="PASS", expected="FAIL")
    logger.test_start("t4")
    logger.test_end("t4", status="TIMEOUT", expected="CRASH")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    num_failures_by_type = json.load(output)["num_failures_by_type"]

    # We expect 3 passes and 1 timeout, nothing else.
    assert sorted(num_failures_by_type.keys()) == ["PASS", "TIMEOUT"]
    assert num_failures_by_type["PASS"] == 3
    assert num_failures_by_type["TIMEOUT"] == 1


def test_subtest_messages(logger, capfd):
    # Tests accumulation of test output

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run two tests with subtest messages. The subtest name should be included
    # in the output. We should also tolerate missing messages and subtest names
    # with unusual characters.
    logger.suite_start(["t1", "t2"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_status("t1", status="FAIL", subtest="t1_a",
                       message="t1_a_message")
    # Subtest name includes a backslash and two closing square brackets.
    logger.test_status("t1", status="PASS", subtest=r"t1_\[]]b",
                       message="t1_b_message")
    logger.test_end("t1", status="PASS", expected="PASS")
    logger.test_start("t2")
    # Subtests with empty messages should not be ignored.
    logger.test_status("t2", status="PASS", subtest="t2_a")
    # A test-level message will also be appended
    logger.test_end("t2", status="TIMEOUT", expected="PASS",
                    message="t2_message")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    t1_artifacts = output_json["tests"]["t1"]["artifacts"]
    assert t1_artifacts["wpt_actual_metadata"] == [
        "[t1]\n  expected: PASS\n",
        "  [t1_a]\n    expected: FAIL\n",
        "  [t1_\\\\[\\]\\]b]\n    expected: PASS\n",
    ]
    assert t1_artifacts["wpt_log"] == [
        "t1_a: t1_a_message\n",
        # Only humans will read the log, so there's no need to escape
        # characters here.
        "t1_\\[]]b: t1_b_message\n",
    ]
    assert t1_artifacts["wpt_subtest_failure"] == ["true"]
    t2_artifacts = output_json["tests"]["t2"]["artifacts"]
    assert t2_artifacts["wpt_actual_metadata"] == [
        "[t2]\n  expected: TIMEOUT\n",
        "  [t2_a]\n    expected: PASS\n",
    ]
    assert t2_artifacts["wpt_log"] == [
        "Harness: t2_message\n"
    ]
    assert "wpt_subtest_failure" not in t2_artifacts.keys()


def test_subtest_failure(logger, capfd):
    # Tests that a test fails if a subtest fails

    # Set up the handler.
    output = StringIO()
    formatter = ChromiumFormatter()
    logger.add_handler(handlers.StreamHandler(output, formatter))

    # Run a test with some subtest failures.
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_status("t1", status="FAIL", subtest="t1_a",
                       message="t1_a_message")
    logger.test_status("t1", status="PASS", subtest="t1_b",
                       message="t1_b_message")
    logger.test_status("t1", status="TIMEOUT", subtest="t1_c",
                       message="t1_c_message")

    # Make sure the test name was added to the set of tests with subtest fails
    assert "t1" in formatter.tests_with_subtest_fails

    # The test status is reported as a pass here because the harness was able to
    # run the test to completion.
    logger.test_end("t1", status="PASS", expected="PASS", message="top_message")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    t1_artifacts = test_obj["artifacts"]
    assert t1_artifacts["wpt_actual_metadata"] == [
        "[t1]\n  expected: PASS\n",
        "  [t1_a]\n    expected: FAIL\n",
        "  [t1_b]\n    expected: PASS\n",
        "  [t1_c]\n    expected: TIMEOUT\n",
    ]
    assert t1_artifacts["wpt_log"] == [
        "Harness: top_message\n",
        "t1_a: t1_a_message\n",
        "t1_b: t1_b_message\n",
        "t1_c: t1_c_message\n",
    ]
    assert t1_artifacts["wpt_subtest_failure"] == ["true"]
    # The status of the test in the output is a failure because subtests failed,
    # despite the harness reporting that the test passed. But the harness status
    # is logged as an artifact.
    assert t1_artifacts["wpt_actual_status"] == ["PASS"]
    assert test_obj["actual"] == "FAIL"
    assert test_obj["expected"] == "PASS"
    # Also ensure that the formatter cleaned up its internal state
    assert "t1" not in formatter.tests_with_subtest_fails


def test_expected_subtest_failure(logger, capfd):
    # Tests that an expected subtest failure does not cause the test to fail

    # Set up the handler.
    output = StringIO()
    formatter = ChromiumFormatter()
    logger.add_handler(handlers.StreamHandler(output, formatter))

    # Run a test with some expected subtest failures.
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_status("t1", status="FAIL", expected="FAIL", subtest="t1_a",
                       message="t1_a_message")
    logger.test_status("t1", status="PASS", subtest="t1_b",
                       message="t1_b_message")
    logger.test_status("t1", status="TIMEOUT", expected="TIMEOUT", subtest="t1_c",
                       message="t1_c_message")

    # The subtest failures are all expected so this test should not be added to
    # the set of tests with subtest failures.
    assert "t1" not in formatter.tests_with_subtest_fails

    # The test status is reported as a pass here because the harness was able to
    # run the test to completion.
    logger.test_end("t1", status="OK", expected="OK")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    assert test_obj["artifacts"]["wpt_actual_metadata"] == [
        "[t1]\n  expected: OK\n",
        "  [t1_a]\n    expected: FAIL\n",
        "  [t1_b]\n    expected: PASS\n",
        "  [t1_c]\n    expected: TIMEOUT\n",
    ]
    assert test_obj["artifacts"]["wpt_log"] == [
        "t1_a: t1_a_message\n",
        "t1_b: t1_b_message\n",
        "t1_c: t1_c_message\n",
    ]
    # The status of the test in the output is a pass because the subtest
    # failures were all expected.
    assert test_obj["actual"] == "PASS"
    assert test_obj["expected"] == "PASS"


def test_unexpected_subtest_pass(logger, capfd):
    # A subtest that unexpectedly passes is considered a failure condition.

    # Set up the handler.
    output = StringIO()
    formatter = ChromiumFormatter()
    logger.add_handler(handlers.StreamHandler(output, formatter))

    # Run a test with a subtest that is expected to fail but passes.
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_status("t1", status="PASS", expected="FAIL", subtest="t1_a",
                       message="t1_a_message")

    # Since the subtest behaviour is unexpected, it's considered a failure, so
    # the test should be added to the set of tests with subtest failures.
    assert "t1" in formatter.tests_with_subtest_fails

    # The test status is reported as a pass here because the harness was able to
    # run the test to completion.
    logger.test_end("t1", status="PASS", expected="PASS")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    t1_artifacts = test_obj["artifacts"]
    assert t1_artifacts["wpt_actual_metadata"] == [
        "[t1]\n  expected: PASS\n",
        "  [t1_a]\n    expected: PASS\n",
    ]
    assert t1_artifacts["wpt_log"] == [
        "t1_a: t1_a_message\n",
    ]
    assert t1_artifacts["wpt_subtest_failure"] == ["true"]
    # Since the subtest status is unexpected, we fail the test. But we report
    # wpt_actual_status as an artifact
    assert t1_artifacts["wpt_actual_status"] == ["PASS"]
    assert test_obj["actual"] == "FAIL"
    assert test_obj["expected"] == "PASS"
    # Also ensure that the formatter cleaned up its internal state
    assert "t1" not in formatter.tests_with_subtest_fails


def test_expected_test_fail(logger, capfd):
    # Check that an expected test-level failure is treated as a Pass

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run some tests with different statuses: 3 passes, 1 timeout
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="ERROR", expected="ERROR")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    # The test's actual and expected status should map from "ERROR" to "FAIL"
    assert test_obj["actual"] == "FAIL"
    assert test_obj["expected"] == "FAIL"
    # ..and this test should not be a regression nor unexpected
    assert "is_regression" not in test_obj
    assert "is_unexpected" not in test_obj


def test_unexpected_test_fail(logger, capfd):
    # Check that an unexpected test-level failure is marked as unexpected and
    # as a regression.

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run some tests with different statuses: 3 passes, 1 timeout
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="ERROR", expected="OK")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    # The test's actual and expected status should be mapped, ERROR->FAIL and
    # OK->PASS
    assert test_obj["actual"] == "FAIL"
    assert test_obj["expected"] == "PASS"
    # ..and this test should be a regression and unexpected
    assert test_obj["is_regression"] is True
    assert test_obj["is_unexpected"] is True


def test_flaky_test_expected(logger, capfd):
    # Check that a flaky test with multiple possible statuses is seen as
    # expected if its actual status is one of the possible ones.

    # set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run a test that is known to be flaky
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="ERROR", expected="OK", known_intermittent=["ERROR", "TIMEOUT"])
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    # The test's statuses are all mapped, changing ERROR->FAIL and OK->PASS
    assert test_obj["actual"] == "FAIL"
    # All the possible statuses are merged and sorted together into expected.
    assert test_obj["expected"] == "FAIL PASS TIMEOUT"
    # ...this is not a regression or unexpected because the actual status is one
    # of the expected ones
    assert "is_regression" not in test_obj
    assert "is_unexpected" not in test_obj


def test_flaky_test_unexpected(logger, capfd):
    # Check that a flaky test with multiple possible statuses is seen as
    # unexpected if its actual status is NOT one of the possible ones.

    # set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run a test that is known to be flaky
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="ERROR", expected="OK", known_intermittent=["TIMEOUT"])
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    # The test's statuses are all mapped, changing ERROR->FAIL and OK->PASS
    assert test_obj["actual"] == "FAIL"
    # All the possible statuses are merged and sorted together into expected.
    assert test_obj["expected"] == "PASS TIMEOUT"
    # ...this is a regression and unexpected because the actual status is not
    # one of the expected ones
    assert test_obj["is_regression"] is True
    assert test_obj["is_unexpected"] is True


def test_precondition_failed(logger, capfd):
    # Check that a failed precondition gets properly handled.

    # set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run a test with a precondition failure
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="PRECONDITION_FAILED", expected="OK")
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    # The precondition failure should map to FAIL status, but we should also
    # have an artifact containing the original PRECONDITION_FAILED status.
    assert test_obj["actual"] == "FAIL"
    assert test_obj["artifacts"]["wpt_actual_status"] == ["PRECONDITION_FAILED"]
    # ...this is an unexpected regression because we expected a pass but failed
    assert test_obj["is_regression"] is True
    assert test_obj["is_unexpected"] is True


def test_repeated_test_statuses(logger, capfd):
    # Check that the logger outputs all statuses from multiple runs of a test.

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run a test suite for the first time.
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="PASS", expected="PASS", known_intermittent=[])
    logger.suite_end()

    # Run the test suite for the second time.
    logger.suite_start(["t1"], run_info={}, time=456)
    logger.test_start("t1")
    logger.test_end("t1", status="FAIL", expected="PASS", known_intermittent=[])
    logger.suite_end()

    # Write the final results.
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    status_totals = output_json["num_failures_by_type"]
    assert status_totals["PASS"] == 1
    # A missing result type is the same as being present and set to zero (0).
    assert status_totals.get("FAIL", 0) == 0

    # The actual statuses are accumulated in a ordered space-separated list.
    test_obj = output_json["tests"]["t1"]
    assert test_obj["actual"] == "PASS FAIL"
    assert test_obj["expected"] == "PASS"


def test_flaky_test_detection(logger, capfd):
    # Check that the logger detects flakiness for a test run multiple times.

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    logger.suite_start(["t1", "t2"], run_info={})
    logger.test_start("t1")
    logger.test_start("t2")
    logger.test_end("t1", status="FAIL", expected="PASS")
    logger.test_end("t2", status="FAIL", expected="FAIL")
    logger.suite_end()

    logger.suite_start(["t1", "t2"], run_info={})
    logger.test_start("t1")
    logger.test_start("t2")
    logger.test_end("t1", status="PASS", expected="PASS")
    logger.test_end("t2", status="FAIL", expected="FAIL")
    logger.suite_end()

    # Write the final results.
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    # We consider a test flaky if it runs multiple times and produces more than
    # one kind of result.
    test1_obj = output_json["tests"]["t1"]
    test2_obj = output_json["tests"]["t2"]
    assert test1_obj["is_flaky"] is True
    assert "is_flaky" not in test2_obj


def test_known_intermittent_empty(logger, capfd):
    # If the known_intermittent list is empty, we want to ensure we don't append
    # any extraneous characters to the output.

    # set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run a test and include an empty known_intermittent list
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="OK", expected="OK", known_intermittent=[])
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    # Both actual and expected statuses get mapped to Pass. No extra whitespace
    # anywhere.
    assert test_obj["actual"] == "PASS"
    assert test_obj["expected"] == "PASS"


def test_known_intermittent_duplicate(logger, capfd):
    # We don't want to have duplicate statuses in the final "expected" field.

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # There are two duplications in this input:
    # 1. known_intermittent already contains expected;
    # 2. both statuses in known_intermittent map to FAIL in Chromium.
    # In the end, we should only get one FAIL in Chromium "expected".
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="ERROR", expected="ERROR", known_intermittent=["FAIL", "ERROR"])
    logger.suite_end()
    logger.shutdown()

    # Check nothing got output to stdout/stderr.
    # (Note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # Check the actual output of the formatter.
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    assert test_obj["actual"] == "FAIL"
    # No duplicate "FAIL" in "expected".
    assert test_obj["expected"] == "FAIL"


def test_reftest_screenshots(logger, capfd):
    # reftest_screenshots, if present, should be plumbed into artifacts.

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    # Run a reftest with reftest_screenshots.
    logger.suite_start(["t1"], run_info={}, time=123)
    logger.test_start("t1")
    logger.test_end("t1", status="FAIL", expected="PASS", extra={
        "reftest_screenshots": [
            {"url": "foo.html", "hash": "HASH1", "screenshot": "DATA1"},
            "!=",
            {"url": "foo-ref.html", "hash": "HASH2", "screenshot": "DATA2"},
        ]
    })
    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    assert test_obj["artifacts"]["screenshots"] == [
        "foo.html: DATA1",
        "foo-ref.html: DATA2",
    ]


def test_process_output_crashing_test(logger, capfd):
    """Test that chromedriver logs are preserved for crashing tests"""

    # Set up the handler.
    output = StringIO()
    logger.add_handler(handlers.StreamHandler(output, ChromiumFormatter()))

    logger.suite_start(["t1", "t2", "t3"], run_info={}, time=123)

    logger.test_start("t1")
    logger.process_output(100, "This message should be recorded", "/some/path/to/chromedriver --some-flag")
    logger.process_output(101, "This message should not be recorded", "/some/other/process --another-flag")
    logger.process_output(100, "This message should also be recorded", "/some/path/to/chromedriver --some-flag")
    logger.test_end("t1", status="CRASH", expected="CRASH")

    logger.test_start("t2")
    logger.process_output(100, "Another message for the second test", "/some/path/to/chromedriver --some-flag")
    logger.test_end("t2", status="CRASH", expected="PASS")

    logger.test_start("t3")
    logger.process_output(100, "This test fails", "/some/path/to/chromedriver --some-flag")
    logger.process_output(100, "But the output should not be captured", "/some/path/to/chromedriver --some-flag")
    logger.process_output(100, "Because it does not crash", "/some/path/to/chromedriver --some-flag")
    logger.test_end("t3", status="FAIL", expected="PASS")

    logger.suite_end()
    logger.shutdown()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_json = json.load(output)

    test_obj = output_json["tests"]["t1"]
    assert test_obj["artifacts"]["wpt_crash_log"] == [
        "This message should be recorded",
        "This message should also be recorded"
    ]

    test_obj = output_json["tests"]["t2"]
    assert test_obj["artifacts"]["wpt_crash_log"] == [
        "Another message for the second test"
    ]

    test_obj = output_json["tests"]["t3"]
    assert "wpt_crash_log" not in test_obj["artifacts"]
