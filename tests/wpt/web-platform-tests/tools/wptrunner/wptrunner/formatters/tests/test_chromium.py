import json
import sys
from os.path import dirname, join
from six.moves import cStringIO as StringIO

from mozlog import handlers, structuredlog

sys.path.insert(0, join(dirname(__file__), "..", ".."))
from formatters import chromium


def test_chromium_required_fields(capfd):
    # Test that the test results contain a handful of required fields.

    # Set up the handler.
    output = StringIO()
    logger = structuredlog.StructuredLogger("test_a")
    logger.add_handler(handlers.StreamHandler(output, chromium.ChromiumFormatter()))

    # output a bunch of stuff
    logger.suite_start(["test-id-1"], run_info={}, time=123)
    logger.test_start("test-id-1")
    logger.test_end("test-id-1", status="PASS", expected="PASS")
    logger.suite_end()

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

def test_chromium_test_name_trie(capfd):
    # Ensure test names are broken into directories and stored in a trie with
    # test results at the leaves.

    # Set up the handler.
    output = StringIO()
    logger = structuredlog.StructuredLogger("test_a")
    logger.add_handler(handlers.StreamHandler(output, chromium.ChromiumFormatter()))

    # output a bunch of stuff
    logger.suite_start(["/foo/bar/test-id-1", "/foo/test-id-2"], run_info={}, time=123)
    logger.test_start("/foo/bar/test-id-1")
    logger.test_end("/foo/bar/test-id-1", status="TIMEOUT", expected="FAIL")
    logger.test_start("/foo/test-id-2")
    logger.test_end("/foo/test-id-2", status="ERROR", expected="TIMEOUT")
    logger.suite_end()

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

def test_num_failures_by_type(capfd):
    # Test that the number of failures by status type is correctly calculated.

    # Set up the handler.
    output = StringIO()
    logger = structuredlog.StructuredLogger("test_a")
    logger.add_handler(handlers.StreamHandler(output, chromium.ChromiumFormatter()))

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
