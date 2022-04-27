import json
import time
from io import StringIO

from mozlog import handlers, structuredlog

from ..formatters.wptscreenshot import WptscreenshotFormatter
from ..formatters.wptreport import WptreportFormatter


def test_wptreport_runtime(capfd):
    # setup the logger
    output = StringIO()
    logger = structuredlog.StructuredLogger("test_a")
    logger.add_handler(handlers.StreamHandler(output, WptreportFormatter()))

    # output a bunch of stuff
    logger.suite_start(["test-id-1"], run_info={})
    logger.test_start("test-id-1")
    time.sleep(0.125)
    logger.test_end("test-id-1", "PASS")
    logger.suite_end()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_obj = json.load(output)
    # be relatively lax in case of low resolution timers
    # 62 is 0.125s = 125ms / 2 = 62ms (assuming int maths)
    # this provides a margin of 62ms, sufficient for even DOS (55ms timer)
    assert output_obj["results"][0]["duration"] >= 62


def test_wptreport_run_info_optional(capfd):
    """per the mozlog docs, run_info is optional; check we work without it"""
    # setup the logger
    output = StringIO()
    logger = structuredlog.StructuredLogger("test_a")
    logger.add_handler(handlers.StreamHandler(output, WptreportFormatter()))

    # output a bunch of stuff
    logger.suite_start(["test-id-1"])  # no run_info arg!
    logger.test_start("test-id-1")
    logger.test_end("test-id-1", "PASS")
    logger.suite_end()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_obj = json.load(output)
    assert "run_info" not in output_obj or output_obj["run_info"] == {}


def test_wptreport_lone_surrogate(capfd):
    output = StringIO()
    logger = structuredlog.StructuredLogger("test_a")
    logger.add_handler(handlers.StreamHandler(output, WptreportFormatter()))

    # output a bunch of stuff
    logger.suite_start(["test-id-1"])  # no run_info arg!
    logger.test_start("test-id-1")
    logger.test_status("test-id-1",
                       subtest="Name with surrogate\uD800",
                       status="FAIL",
                       message="\U0001F601 \uDE0A\uD83D")
    logger.test_end("test-id-1",
                    status="PASS",
                    message="\uDE0A\uD83D \U0001F601")
    logger.suite_end()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_obj = json.load(output)
    test = output_obj["results"][0]
    assert test["message"] == "U+de0aU+d83d \U0001F601"
    subtest = test["subtests"][0]
    assert subtest["name"] == "Name with surrogateU+d800"
    assert subtest["message"] == "\U0001F601 U+de0aU+d83d"


def test_wptreport_known_intermittent(capfd):
    output = StringIO()
    logger = structuredlog.StructuredLogger("test_a")
    logger.add_handler(handlers.StreamHandler(output, WptreportFormatter()))

    # output a bunch of stuff
    logger.suite_start(["test-id-1"])  # no run_info arg!
    logger.test_start("test-id-1")
    logger.test_status("test-id-1",
                       "a-subtest",
                       status="FAIL",
                       expected="PASS",
                       known_intermittent=["FAIL"])
    logger.test_end("test-id-1",
                    status="OK",)
    logger.suite_end()

    # check nothing got output to stdout/stderr
    # (note that mozlog outputs exceptions during handling to stderr!)
    captured = capfd.readouterr()
    assert captured.out == ""
    assert captured.err == ""

    # check the actual output of the formatter
    output.seek(0)
    output_obj = json.load(output)
    test = output_obj["results"][0]
    assert test["status"] == "OK"
    subtest = test["subtests"][0]
    assert subtest["expected"] == "PASS"
    assert subtest["known_intermittent"] == ['FAIL']


def test_wptscreenshot_test_end(capfd):
    formatter = WptscreenshotFormatter()

    # Empty
    data = {}
    assert formatter.test_end(data) is None

    # No items
    data['extra'] = {"reftest_screenshots": []}
    assert formatter.test_end(data) is None

    # Invalid item
    data['extra']['reftest_screenshots'] = ["no dict item"]
    assert formatter.test_end(data) is None

    # Random hash
    data['extra']['reftest_screenshots'] = [{"hash": "HASH", "screenshot": "DATA"}]
    assert 'data:image/png;base64,DATA\n' == formatter.test_end(data)

    # Already cached hash
    assert formatter.test_end(data) is None
