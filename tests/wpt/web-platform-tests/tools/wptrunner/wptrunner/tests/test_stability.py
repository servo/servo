# mypy: allow-untyped-defs

import sys
from collections import OrderedDict, defaultdict
from unittest import mock

from mozlog.structuredlog import StructuredLogger
from mozlog.formatters import TbplFormatter
from mozlog.handlers import StreamHandler

from .. import stability, wptrunner

def test_is_inconsistent():
    assert stability.is_inconsistent({"PASS": 10}, 10) is False
    assert stability.is_inconsistent({"PASS": 9}, 10) is True
    assert stability.is_inconsistent({"PASS": 9, "FAIL": 1}, 10) is True
    assert stability.is_inconsistent({"PASS": 8, "FAIL": 1}, 10) is True


def test_find_slow_status():
    assert stability.find_slow_status({
        "longest_duration": {"TIMEOUT": 10},
        "timeout": 10}) is None
    assert stability.find_slow_status({
        "longest_duration": {"CRASH": 10},
        "timeout": 10}) is None
    assert stability.find_slow_status({
        "longest_duration": {"ERROR": 10},
        "timeout": 10}) is None
    assert stability.find_slow_status({
        "longest_duration": {"PASS": 1},
        "timeout": 10}) is None
    assert stability.find_slow_status({
        "longest_duration": {"PASS": 81},
        "timeout": 100}) == "PASS"
    assert stability.find_slow_status({
        "longest_duration": {"TIMEOUT": 10, "FAIL": 81},
        "timeout": 100}) == "FAIL"
    assert stability.find_slow_status({
        "longest_duration": {"SKIP": 0}}) is None


def test_get_steps():
    logger = None

    steps = stability.get_steps(logger, 0, 0, [])
    assert len(steps) == 0

    steps = stability.get_steps(logger, 0, 0, [{}])
    assert len(steps) == 0

    repeat_loop = 1
    flag_name = 'flag'
    flag_value = 'y'
    steps = stability.get_steps(logger, repeat_loop, 0, [
                                {flag_name: flag_value}])
    assert len(steps) == 1
    assert steps[0][0] == "Running tests in a loop %d times with flags %s=%s" % (
        repeat_loop, flag_name, flag_value)

    repeat_loop = 0
    repeat_restart = 1
    flag_name = 'flag'
    flag_value = 'n'
    steps = stability.get_steps(logger, repeat_loop, repeat_restart, [
                                {flag_name: flag_value}])
    assert len(steps) == 1
    assert steps[0][0] == "Running tests in a loop with restarts %d times with flags %s=%s" % (
        repeat_restart, flag_name, flag_value)

    repeat_loop = 10
    repeat_restart = 5
    steps = stability.get_steps(logger, repeat_loop, repeat_restart, [{}])
    assert len(steps) == 2
    assert steps[0][0] == "Running tests in a loop %d times" % repeat_loop
    assert steps[1][0] == (
        "Running tests in a loop with restarts %d times" % repeat_restart)


def test_log_handler():
    handler = stability.LogHandler()
    data = OrderedDict()
    data["test"] = "test_name"
    test = handler.find_or_create_test(data)
    assert test["subtests"] == OrderedDict()
    assert test["status"] == defaultdict(int)
    assert test["longest_duration"] == defaultdict(float)
    assert test == handler.find_or_create_test(data)

    start_time = 100
    data["time"] = start_time
    handler.test_start(data)
    assert test["start_time"] == start_time

    data["subtest"] = "subtest_name"
    subtest = handler.find_or_create_subtest(data)
    assert subtest["status"] == defaultdict(int)
    assert subtest["messages"] == set()
    assert subtest == handler.find_or_create_subtest(data)

    data["status"] = 0
    assert subtest["status"][data["status"]] == 0
    handler.test_status(data)
    assert subtest["status"][data["status"]] == 1
    handler.test_status(data)
    assert subtest["status"][data["status"]] == 2
    data["status"] = 1
    assert subtest["status"][data["status"]] == 0
    message = "test message"
    data["message"] = message
    handler.test_status(data)
    assert subtest["status"][data["status"]] == 1
    assert len(subtest["messages"]) == 1
    assert message in subtest["messages"]

    test_duration = 10
    data["time"] = data["time"] + test_duration
    handler.test_end(data)
    assert test["longest_duration"][data["status"]] == test_duration
    assert "timeout" not in test

    data["test2"] = "test_name_2"
    timeout = 5
    data["extra"] = {}
    data["extra"]["test_timeout"] = timeout
    handler.test_start(data)
    handler.test_end(data)
    assert test["timeout"] == timeout * 1000


def test_err_string():
    assert stability.err_string(
        {'OK': 1, 'FAIL': 1}, 1) == "**Duplicate subtest name**"
    assert stability.err_string(
        {'OK': 2, 'FAIL': 1}, 2) == "**Duplicate subtest name**"
    assert stability.err_string({'SKIP': 1}, 0) == "Duplicate subtest name"
    assert stability.err_string(
        {'SKIP': 1, 'OK': 1}, 1) == "Duplicate subtest name"

    assert stability.err_string(
        {'FAIL': 1}, 2) == "**FAIL: 1/2, MISSING: 1/2**"
    assert stability.err_string(
        {'FAIL': 1, 'OK': 1}, 3) == "**FAIL: 1/3, OK: 1/3, MISSING: 1/3**"

    assert stability.err_string(
        {'OK': 1, 'FAIL': 1}, 2) == "**FAIL: 1/2, OK: 1/2**"

    assert stability.err_string(
        {'OK': 2, 'FAIL': 1, 'SKIP': 1}, 4) == "FAIL: 1/4, OK: 2/4, SKIP: 1/4"
    assert stability.err_string(

        {'FAIL': 1, 'SKIP': 1, 'OK': 2}, 4) == "FAIL: 1/4, OK: 2/4, SKIP: 1/4"


def test_check_stability_iterations():
    logger = StructuredLogger("test-stability")
    logger.add_handler(StreamHandler(sys.stdout, TbplFormatter()))

    kwargs = {"verify_log_full": False}

    def mock_run_tests(**kwargs):
        repeats = kwargs.get("repeat", 1)
        for _ in range(repeats):
            logger.suite_start(tests=[], name="test")
            for _ in range(kwargs.get("rerun", 1)):
                logger.test_start("/example/test.html")
                logger.test_status("/example/test.html", subtest="test1", status="PASS")
                logger.test_end("/example/test.html", status="OK")
            logger.suite_end()

        status = wptrunner.TestStatus()
        status.total_tests = 1
        status.repeated_runs = repeats
        status.expected_repeated_runs = repeats

        return (None, status)

    # Don't actually load wptrunner, because that will end up starting a browser
    # which we don't want to do in this test.
    with mock.patch("wptrunner.stability.wptrunner.run_tests") as mock_run:
        mock_run.side_effect = mock_run_tests
        assert stability.check_stability(logger,
                                         repeat_loop=10,
                                         repeat_restart=5,
                                         chaos_mode=False,
                                         output_results=False,
                                         **kwargs) is None
