import json
import sys
import time
from os.path import dirname, join
from StringIO import StringIO

import mock

from mozlog import handlers, structuredlog

sys.path.insert(0, join(dirname(__file__), "..", ".."))

from wptrunner import formatters
from wptrunner.formatters import WptreportFormatter


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
                       subtest=u"Name with surrogate\uD800",
                       status="FAIL",
                       message=u"\U0001F601 \uDE0A\uD83D")
    logger.test_end("test-id-1",
                    status="PASS",
                    message=u"\uDE0A\uD83D \U0001F601")
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
    assert test["message"] == u"U+de0aU+d83d \U0001F601"
    subtest = test["subtests"][0]
    assert subtest["name"] == u"Name with surrogateU+d800"
    assert subtest["message"] == u"\U0001F601 U+de0aU+d83d"


def test_wptreport_lone_surrogate_ucs2(capfd):
    # Since UCS4 is a superset of UCS2 we can meaningfully test the UCS2 code on a
    # UCS4 build, but not the reverse. However UCS2 is harder to handle and UCS4 is
    # the commonest (and sensible) configuration, so that's OK.
    output = StringIO()
    logger = structuredlog.StructuredLogger("test_a")
    logger.add_handler(handlers.StreamHandler(output, WptreportFormatter()))

    with mock.patch.object(formatters, 'surrogate_replacement', formatters.SurrogateReplacementUcs2()):
        # output a bunch of stuff
        logger.suite_start(["test-id-1"])  # no run_info arg!
        logger.test_start("test-id-1")
        logger.test_status("test-id-1",
                           subtest=u"Name with surrogate\uD800",
                           status="FAIL",
                           message=u"\U0001F601 \uDE0A\uD83D \uD83D\uDE0A")
        logger.test_end("test-id-1",
                        status="PASS",
                        message=u"\uDE0A\uD83D \uD83D\uDE0A \U0001F601")
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
    assert test["message"] == u"U+de0aU+d83d \U0001f60a \U0001F601"
    subtest = test["subtests"][0]
    assert subtest["name"] == u"Name with surrogateU+d800"
    assert subtest["message"] == u"\U0001F601 U+de0aU+d83d \U0001f60a"
