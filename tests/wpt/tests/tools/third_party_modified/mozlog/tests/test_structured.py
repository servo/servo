# -*- coding: utf-8 -*-

import argparse
import contextlib
import json
import optparse
import os
import sys
import tempfile
import unittest
from io import StringIO

from mozlog import commandline, formatters, handlers, reader, stdadapter, structuredlog


class LogHandler:
    def __init__(self):
        self.items = []

    def __call__(self, data):
        self.items.append(data)

    @property
    def last_item(self):
        return self.items[-1]

    @property
    def empty(self):
        return not self.items


class BaseStructuredTest(unittest.TestCase):
    def setUp(self):
        self.logger = structuredlog.StructuredLogger("test")
        self.handler = LogHandler()
        self.logger.add_handler(self.handler)

    def pop_last_item(self):
        return self.handler.items.pop()

    def assert_log_equals(self, expected, actual=None):
        if actual is None:
            actual = self.pop_last_item()

        all_expected = {"pid": os.getpid(), "thread": "MainThread", "source": "test"}
        specials = set(["time"])

        all_expected.update(expected)
        for key, value in all_expected.items():
            self.assertEqual(actual[key], value)

        self.assertEqual(set(all_expected.keys()) | specials, set(actual.keys()))


class TestStatusHandler(BaseStructuredTest):
    def setUp(self):
        super(TestStatusHandler, self).setUp()
        self.handler = handlers.StatusHandler()
        self.logger.add_handler(self.handler)

    def test_failure_run(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "sub1", status="PASS")
        self.logger.test_status("test1", "sub2", status="TIMEOUT")
        self.logger.test_status(
            "test1", "sub3", status="FAIL", expected="PASS", known_intermittent=["FAIL"]
        )
        self.logger.test_end("test1", status="OK")
        self.logger.suite_end()
        summary = self.handler.summarize()
        self.assertIn("TIMEOUT", summary.unexpected_statuses)
        self.assertEqual(1, summary.unexpected_statuses["TIMEOUT"])
        self.assertIn("PASS", summary.expected_statuses)
        self.assertEqual(1, summary.expected_statuses["PASS"])
        self.assertIn("OK", summary.expected_statuses)
        self.assertEqual(1, summary.expected_statuses["OK"])
        self.assertIn("FAIL", summary.expected_statuses)
        self.assertEqual(1, summary.expected_statuses["FAIL"])
        self.assertIn("FAIL", summary.known_intermittent_statuses)
        self.assertEqual(1, summary.known_intermittent_statuses["FAIL"])
        self.assertEqual(3, summary.action_counts["test_status"])
        self.assertEqual(1, summary.action_counts["test_end"])

    def test_precondition_failed_run(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", status="PRECONDITION_FAILED")
        self.logger.test_start("test2")
        self.logger.test_status("test2", "sub1", status="PRECONDITION_FAILED")
        self.logger.test_end("test2", status="OK")
        self.logger.suite_end()
        summary = self.handler.summarize()
        self.assertEqual(1, summary.expected_statuses["OK"])
        self.assertEqual(2, summary.unexpected_statuses["PRECONDITION_FAILED"])

    def test_error_run(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.error("ERRR!")
        self.logger.test_end("test1", status="OK")
        self.logger.test_start("test2")
        self.logger.test_end("test2", status="OK")
        self.logger.suite_end()
        summary = self.handler.summarize()
        self.assertIn("ERROR", summary.log_level_counts)
        self.assertEqual(1, summary.log_level_counts["ERROR"])
        self.assertIn("OK", summary.expected_statuses)
        self.assertEqual(2, summary.expected_statuses["OK"])


class TestSummaryHandler(BaseStructuredTest):
    def setUp(self):
        super(TestSummaryHandler, self).setUp()
        self.handler = handlers.SummaryHandler()
        self.logger.add_handler(self.handler)

    def test_failure_run(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "sub1", status="PASS")
        self.logger.test_status("test1", "sub2", status="TIMEOUT")
        self.logger.assertion_count("test1", 5, 1, 10)
        self.logger.assertion_count("test1", 5, 10, 15)
        self.logger.test_end("test1", status="OK")
        self.logger.suite_end()

        counts = self.handler.current["counts"]
        self.assertIn("timeout", counts["subtest"]["unexpected"])
        self.assertEqual(1, counts["subtest"]["unexpected"]["timeout"])
        self.assertIn("pass", counts["subtest"]["expected"])
        self.assertEqual(1, counts["subtest"]["expected"]["pass"])
        self.assertIn("ok", counts["test"]["expected"])
        self.assertEqual(1, counts["test"]["expected"]["ok"])
        self.assertIn("pass", counts["assert"]["unexpected"])
        self.assertEqual(1, counts["assert"]["unexpected"]["pass"])
        self.assertIn("fail", counts["assert"]["expected"])
        self.assertEqual(1, counts["assert"]["expected"]["fail"])

        logs = self.handler.current["unexpected_logs"]
        self.assertEqual(1, len(logs))
        self.assertIn("test1", logs)
        self.assertEqual(1, len(logs["test1"]))
        self.assertEqual("sub2", logs["test1"][0]["subtest"])

    def test_precondition_failed_run(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "sub1", status="PASS")
        self.logger.test_end("test1", status="PRECONDITION_FAILED")
        self.logger.test_start("test2")
        self.logger.test_status("test2", "sub1", status="PRECONDITION_FAILED")
        self.logger.test_status("test2", "sub2", status="PRECONDITION_FAILED")
        self.logger.test_end("test2", status="OK")
        self.logger.suite_end()

        counts = self.handler.current["counts"]
        self.assertIn("precondition_failed", counts["test"]["unexpected"])
        self.assertEqual(1, counts["test"]["unexpected"]["precondition_failed"])
        self.assertIn("pass", counts["subtest"]["expected"])
        self.assertEqual(1, counts["subtest"]["expected"]["pass"])
        self.assertIn("ok", counts["test"]["expected"])
        self.assertEqual(1, counts["test"]["expected"]["ok"])
        self.assertIn("precondition_failed", counts["subtest"]["unexpected"])
        self.assertEqual(2, counts["subtest"]["unexpected"]["precondition_failed"])


class TestStructuredLog(BaseStructuredTest):
    def test_suite_start(self):
        self.logger.suite_start(["test"], "logtest")
        self.assert_log_equals(
            {"action": "suite_start", "name": "logtest", "tests": {"default": ["test"]}}
        )
        self.logger.suite_end()

    def test_suite_end(self):
        self.logger.suite_start([])
        self.logger.suite_end()
        self.assert_log_equals({"action": "suite_end"})

    def test_add_subsuite(self):
        self.logger.suite_start([])
        self.logger.add_subsuite("other")
        self.assert_log_equals(
            {
                "action": "add_subsuite",
                "name": "other",
                "run_info": {"subsuite": "other"},
            }
        )
        self.logger.suite_end()

    def test_add_subsuite_duplicate(self):
        self.logger.suite_start([])
        self.logger.add_subsuite("other")
        # This should be a no-op
        self.logger.add_subsuite("other")
        self.assert_log_equals(
            {
                "action": "add_subsuite",
                "name": "other",
                "run_info": {"subsuite": "other"},
            }
        )
        self.assert_log_equals({"action": "suite_start", "tests": {"default": []}})

        self.logger.suite_end()

    def test_start(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.assert_log_equals({"action": "test_start", "test": "test1"})

        self.logger.test_start(("test1", "==", "test1-ref"), path="path/to/test")
        self.assert_log_equals(
            {
                "action": "test_start",
                "test": ("test1", "==", "test1-ref"),
                "path": "path/to/test",
            }
        )
        self.logger.suite_end()

    def test_start_inprogress(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_start("test1")
        self.assert_log_equals(
            {
                "action": "log",
                "message": "test_start for test1 logged while in progress.",
                "level": "ERROR",
            }
        )
        self.logger.suite_end()

    def test_start_inprogress_subsuite(self):
        self.logger.suite_start([])
        self.logger.add_subsuite("other")
        self.logger.test_start("test1")
        self.logger.test_start("test1", subsuite="other")
        self.assert_log_equals(
            {
                "action": "test_start",
                "test": "test1",
                "subsuite": "other",
            }
        )
        self.logger.suite_end()

    def test_status(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status(
            "test1", "subtest name", "fail", expected="FAIL", message="Test message"
        )
        self.assert_log_equals(
            {
                "action": "test_status",
                "subtest": "subtest name",
                "status": "FAIL",
                "message": "Test message",
                "test": "test1",
            }
        )
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_status_1(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "subtest name", "fail")
        self.assert_log_equals(
            {
                "action": "test_status",
                "subtest": "subtest name",
                "status": "FAIL",
                "expected": "PASS",
                "test": "test1",
            }
        )
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_status_2(self):
        self.assertRaises(
            ValueError,
            self.logger.test_status,
            "test1",
            "subtest name",
            "XXXUNKNOWNXXX",
        )

    def test_status_extra(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status(
            "test1", "subtest name", "FAIL", expected="PASS", extra={"data": 42}
        )
        self.assert_log_equals(
            {
                "action": "test_status",
                "subtest": "subtest name",
                "status": "FAIL",
                "expected": "PASS",
                "test": "test1",
                "extra": {"data": 42},
            }
        )
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_status_stack(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status(
            "test1",
            "subtest name",
            "FAIL",
            expected="PASS",
            stack="many\nlines\nof\nstack",
        )
        self.assert_log_equals(
            {
                "action": "test_status",
                "subtest": "subtest name",
                "status": "FAIL",
                "expected": "PASS",
                "test": "test1",
                "stack": "many\nlines\nof\nstack",
            }
        )
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_status_known_intermittent(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status(
            "test1", "subtest name", "fail", known_intermittent=["FAIL"]
        )
        self.assert_log_equals(
            {
                "action": "test_status",
                "subtest": "subtest name",
                "status": "FAIL",
                "expected": "PASS",
                "known_intermittent": ["FAIL"],
                "test": "test1",
            }
        )
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_status_not_started(self):
        self.logger.test_status("test_UNKNOWN", "subtest", "PASS")
        self.assertTrue(
            self.pop_last_item()["message"].startswith(
                "test_status for test_UNKNOWN logged while not in progress. Logged with data: {"
            )
        )

    def test_remove_optional_defaults(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status(
            "test1", "subtest name", "fail", message=None, stack=None
        )
        self.assert_log_equals(
            {
                "action": "test_status",
                "subtest": "subtest name",
                "status": "FAIL",
                "expected": "PASS",
                "test": "test1",
            }
        )
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_remove_optional_defaults_raw_log(self):
        self.logger.log_raw({"action": "suite_start", "tests": [1], "name": None})
        self.assert_log_equals({"action": "suite_start", "tests": {"default": ["1"]}})
        self.logger.suite_end()

    def test_end(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "fail", message="Test message")
        self.assert_log_equals(
            {
                "action": "test_end",
                "status": "FAIL",
                "expected": "OK",
                "message": "Test message",
                "test": "test1",
            }
        )
        self.logger.suite_end()

    def test_end_1(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "PASS", expected="PASS", extra={"data": 123})
        self.assert_log_equals(
            {
                "action": "test_end",
                "status": "PASS",
                "extra": {"data": 123},
                "test": "test1",
            }
        )
        self.logger.suite_end()

    def test_end_2(self):
        self.assertRaises(ValueError, self.logger.test_end, "test1", "XXXUNKNOWNXXX")

    def test_end_stack(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end(
            "test1", "PASS", expected="PASS", stack="many\nlines\nof\nstack"
        )
        self.assert_log_equals(
            {
                "action": "test_end",
                "status": "PASS",
                "test": "test1",
                "stack": "many\nlines\nof\nstack",
            }
        )
        self.logger.suite_end()

    def test_end_no_start(self):
        self.logger.test_end("test1", "PASS", expected="PASS")
        self.assertTrue(
            self.pop_last_item()["message"].startswith(
                "test_end for test1 logged while not in progress. Logged with data: {"
            )
        )
        self.logger.suite_end()

    def test_end_no_start_subsuite(self):
        self.logger.suite_start([])
        self.logger.add_subsuite("other")
        self.logger.test_start("test1", subsuite="other")
        self.logger.test_end("test1", "PASS", expected="PASS")
        self.assertTrue(
            self.pop_last_item()["message"].startswith(
                "test_end for test1 logged while not in progress. Logged with data: {"
            )
        )
        self.logger.test_end("test1", "OK", subsuite="other")
        self.assert_log_equals(
            {
                "action": "test_end",
                "status": "OK",
                "test": "test1",
                "subsuite": "other",
            }
        )
        self.logger.suite_end()

    def test_end_twice(self):
        self.logger.suite_start([])
        self.logger.test_start("test2")
        self.logger.test_end("test2", "PASS", expected="PASS")
        self.assert_log_equals(
            {"action": "test_end", "status": "PASS", "test": "test2"}
        )
        self.logger.test_end("test2", "PASS", expected="PASS")
        last_item = self.pop_last_item()
        self.assertEqual(last_item["action"], "log")
        self.assertEqual(last_item["level"], "ERROR")
        self.assertTrue(
            last_item["message"].startswith(
                "test_end for test2 logged while not in progress. Logged with data: {"
            )
        )
        self.logger.suite_end()

    def test_suite_start_twice(self):
        self.logger.suite_start([])
        self.assert_log_equals({"action": "suite_start", "tests": {"default": []}})
        self.logger.suite_start([])
        last_item = self.pop_last_item()
        self.assertEqual(last_item["action"], "log")
        self.assertEqual(last_item["level"], "ERROR")
        self.logger.suite_end()

    def test_suite_end_no_start(self):
        self.logger.suite_start([])
        self.assert_log_equals({"action": "suite_start", "tests": {"default": []}})
        self.logger.suite_end()
        self.assert_log_equals({"action": "suite_end"})
        self.logger.suite_end()
        last_item = self.pop_last_item()
        self.assertEqual(last_item["action"], "log")
        self.assertEqual(last_item["level"], "ERROR")

    def test_multiple_loggers_suite_start(self):
        logger1 = structuredlog.StructuredLogger("test")
        self.logger.suite_start([])
        logger1.suite_start([])
        last_item = self.pop_last_item()
        self.assertEqual(last_item["action"], "log")
        self.assertEqual(last_item["level"], "ERROR")

    def test_multiple_loggers_test_start(self):
        logger1 = structuredlog.StructuredLogger("test")
        self.logger.suite_start([])
        self.logger.test_start("test")
        logger1.test_start("test")
        last_item = self.pop_last_item()
        self.assertEqual(last_item["action"], "log")
        self.assertEqual(last_item["level"], "ERROR")

    def test_process(self):
        self.logger.process_output(1234, "test output")
        self.assert_log_equals(
            {"action": "process_output", "process": "1234", "data": "test output"}
        )

    def test_process_start(self):
        self.logger.process_start(1234)
        self.assert_log_equals({"action": "process_start", "process": "1234"})

    def test_process_exit(self):
        self.logger.process_exit(1234, 0)
        self.assert_log_equals(
            {"action": "process_exit", "process": "1234", "exitcode": 0}
        )

    def test_log(self):
        for level in ["critical", "error", "warning", "info", "debug"]:
            getattr(self.logger, level)("message")
            self.assert_log_equals(
                {"action": "log", "level": level.upper(), "message": "message"}
            )

    def test_logging_adapter(self):
        import logging

        logging.basicConfig(level="DEBUG")
        old_level = logging.root.getEffectiveLevel()
        logging.root.setLevel("DEBUG")

        std_logger = logging.getLogger("test")
        std_logger.setLevel("DEBUG")

        logger = stdadapter.std_logging_adapter(std_logger)

        try:
            for level in ["critical", "error", "warning", "info", "debug"]:
                getattr(logger, level)("message")
                self.assert_log_equals(
                    {"action": "log", "level": level.upper(), "message": "message"}
                )
        finally:
            logging.root.setLevel(old_level)

    def test_add_remove_handlers(self):
        handler = LogHandler()
        self.logger.add_handler(handler)
        self.logger.info("test1")

        self.assert_log_equals({"action": "log", "level": "INFO", "message": "test1"})

        self.assert_log_equals(
            {"action": "log", "level": "INFO", "message": "test1"},
            actual=handler.last_item,
        )

        self.logger.remove_handler(handler)
        self.logger.info("test2")

        self.assert_log_equals({"action": "log", "level": "INFO", "message": "test2"})

        self.assert_log_equals(
            {"action": "log", "level": "INFO", "message": "test1"},
            actual=handler.last_item,
        )

    def test_wrapper(self):
        file_like = structuredlog.StructuredLogFileLike(self.logger)

        file_like.write("line 1")

        self.assert_log_equals({"action": "log", "level": "INFO", "message": "line 1"})

        file_like.write("line 2\n")

        self.assert_log_equals({"action": "log", "level": "INFO", "message": "line 2"})

        file_like.write("line 3\r")

        self.assert_log_equals({"action": "log", "level": "INFO", "message": "line 3"})

        file_like.write("line 4\r\n")

        self.assert_log_equals({"action": "log", "level": "INFO", "message": "line 4"})

    def test_shutdown(self):
        # explicit shutdown
        log = structuredlog.StructuredLogger("test 1")
        log.add_handler(self.handler)
        log.info("line 1")
        self.assert_log_equals(
            {"action": "log", "level": "INFO", "message": "line 1", "source": "test 1"}
        )
        log.shutdown()
        self.assert_log_equals({"action": "shutdown", "source": "test 1"})
        with self.assertRaises(structuredlog.LoggerShutdownError):
            log.info("bad log")
        with self.assertRaises(structuredlog.LoggerShutdownError):
            log.log_raw({"action": "log", "level": "info", "message": "bad log"})

        # shutdown still applies to new instances
        del log
        log = structuredlog.StructuredLogger("test 1")
        with self.assertRaises(structuredlog.LoggerShutdownError):
            log.info("bad log")

        # context manager shutdown
        with structuredlog.StructuredLogger("test 2") as log:
            log.add_handler(self.handler)
            log.info("line 2")
            self.assert_log_equals(
                {
                    "action": "log",
                    "level": "INFO",
                    "message": "line 2",
                    "source": "test 2",
                }
            )
        self.assert_log_equals({"action": "shutdown", "source": "test 2"})

        # shutdown prevents logging across instances
        log1 = structuredlog.StructuredLogger("test 3")
        log2 = structuredlog.StructuredLogger("test 3", component="bar")
        log1.shutdown()
        with self.assertRaises(structuredlog.LoggerShutdownError):
            log2.info("line 3")


class TestTypeConversions(BaseStructuredTest):
    def test_raw(self):
        self.logger.log_raw({"action": "suite_start", "tests": [1], "time": "1234"})
        self.assert_log_equals(
            {"action": "suite_start", "tests": {"default": ["1"]}, "time": 1234}
        )
        self.logger.suite_end()

    def test_tuple(self):
        self.logger.suite_start([])
        self.logger.test_start(
            (
                b"\xf0\x90\x8d\x84\xf0\x90\x8c\xb4\xf0\x90"
                b"\x8d\x83\xf0\x90\x8d\x84".decode(),
                42,
                u"\u16a4",
            )
        )
        self.assert_log_equals(
            {
                "action": "test_start",
                "test": (u"\U00010344\U00010334\U00010343\U00010344", u"42", u"\u16a4"),
            }
        )
        self.logger.suite_end()

    def test_non_string_messages(self):
        self.logger.suite_start([])
        self.logger.info(1)
        self.assert_log_equals({"action": "log", "message": "1", "level": "INFO"})
        self.logger.info([1, (2, "3"), "s", "s" + chr(255)])
        self.assert_log_equals(
            {
                "action": "log",
                "message": "[1, (2, '3'), 's', 's\xff']",
                "level": "INFO",
            }
        )

        self.logger.suite_end()

    def test_utf8str_write(self):
        with tempfile.NamedTemporaryFile() as logfile:
            _fmt = formatters.TbplFormatter()
            _handler = handlers.StreamHandler(logfile, _fmt)
            self.logger.add_handler(_handler)
            self.logger.suite_start([])
            self.logger.info("☺")
            logfile.seek(0)
            data = logfile.readlines()[-1].strip()
            self.assertEqual(data.decode(), "☺")
            self.logger.suite_end()
            self.logger.remove_handler(_handler)

    def test_arguments(self):
        self.logger.info(message="test")
        self.assert_log_equals({"action": "log", "message": "test", "level": "INFO"})

        self.logger.suite_start([], run_info={})
        self.assert_log_equals(
            {"action": "suite_start", "tests": {"default": []}, "run_info": {}}
        )
        self.logger.test_start(test="test1")
        self.logger.test_status("subtest1", "FAIL", test="test1", status="PASS")
        self.assert_log_equals(
            {
                "action": "test_status",
                "test": "test1",
                "subtest": "subtest1",
                "status": "PASS",
                "expected": "FAIL",
            }
        )
        self.logger.process_output(123, "data", "test")
        self.assert_log_equals(
            {
                "action": "process_output",
                "process": "123",
                "command": "test",
                "data": "data",
            }
        )
        self.assertRaises(
            TypeError,
            self.logger.test_status,
            subtest="subtest2",
            status="FAIL",
            expected="PASS",
        )
        self.assertRaises(
            TypeError,
            self.logger.test_status,
            "test1",
            "subtest1",
            "PASS",
            "FAIL",
            "message",
            "stack",
            {},
            [],
            None,
            "unexpected",
        )
        self.assertRaises(TypeError, self.logger.test_status, "test1", test="test2")
        self.logger.suite_end()


class TestComponentFilter(BaseStructuredTest):
    def test_filter_component(self):
        component_logger = structuredlog.StructuredLogger(
            self.logger.name, "test_component"
        )
        component_logger.component_filter = handlers.LogLevelFilter(lambda x: x, "info")

        self.logger.debug("Test")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals({"action": "log", "level": "DEBUG", "message": "Test"})
        self.assertTrue(self.handler.empty)

        component_logger.info("Test 1")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals(
            {
                "action": "log",
                "level": "INFO",
                "message": "Test 1",
                "component": "test_component",
            }
        )

        component_logger.debug("Test 2")
        self.assertTrue(self.handler.empty)

        component_logger.component_filter = None

        component_logger.debug("Test 3")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals(
            {
                "action": "log",
                "level": "DEBUG",
                "message": "Test 3",
                "component": "test_component",
            }
        )

    def test_filter_default_component(self):
        component_logger = structuredlog.StructuredLogger(
            self.logger.name, "test_component"
        )

        self.logger.debug("Test")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals({"action": "log", "level": "DEBUG", "message": "Test"})

        self.logger.component_filter = handlers.LogLevelFilter(lambda x: x, "info")

        self.logger.debug("Test 1")
        self.assertTrue(self.handler.empty)

        component_logger.debug("Test 2")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals(
            {
                "action": "log",
                "level": "DEBUG",
                "message": "Test 2",
                "component": "test_component",
            }
        )

        self.logger.component_filter = None

        self.logger.debug("Test 3")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals({"action": "log", "level": "DEBUG", "message": "Test 3"})

    def test_filter_message_mutuate(self):
        def filter_mutate(msg):
            if msg["action"] == "log":
                msg["message"] = "FILTERED! %s" % msg["message"]
            return msg

        self.logger.component_filter = filter_mutate
        self.logger.debug("Test")
        self.assert_log_equals(
            {"action": "log", "level": "DEBUG", "message": "FILTERED! Test"}
        )
        self.logger.component_filter = None


class TestCommandline(unittest.TestCase):
    @contextlib.contextmanager
    def get_logfile(self):
        temp_file = tempfile.NamedTemporaryFile(delete=False)
        temp_file.close()
        try:
            yield temp_file
        finally:
            # This will fail on Windows, because file is not closed
            # in StreamHandler.
            os.unlink(temp_file.name)

    def loglines(self, logfile):
        close = False
        if logfile.closed:
            close = True
            logfile = open(logfile.name, "rb")
        try:
            logfile.seek(0)
            return [line.rstrip() for line in logfile.readlines()]
        finally:
            if close:
                logfile.close()

    def test_setup_logging(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser)
        args = parser.parse_args(["--log-raw=-"])
        logger = commandline.setup_logging("test_setup_logging", args, {})
        self.assertEqual(len(logger.handlers), 1)

    def test_setup_logging_optparse(self):
        parser = optparse.OptionParser()
        commandline.add_logging_group(parser)
        args, _ = parser.parse_args(["--log-raw=-"])
        logger = commandline.setup_logging("test_optparse", args, {})
        self.assertEqual(len(logger.handlers), 1)
        self.assertIsInstance(logger.handlers[0], handlers.StreamHandler)

    def test_limit_formatters(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser, include_formatters=["raw"])
        other_formatters = [fmt for fmt in commandline.log_formatters if fmt != "raw"]
        # check that every formatter except raw is not present
        for fmt in other_formatters:
            with self.assertRaises(SystemExit):
                parser.parse_args(["--log-%s=-" % fmt])
            with self.assertRaises(SystemExit):
                parser.parse_args(["--log-%s-level=error" % fmt])
        # raw is still ok
        args = parser.parse_args(["--log-raw=-"])
        logger = commandline.setup_logging("test_setup_logging2", args, {})
        self.assertEqual(len(logger.handlers), 1)

    def test_setup_logging_optparse_unicode(self):
        parser = optparse.OptionParser()
        commandline.add_logging_group(parser)
        args, _ = parser.parse_args([u"--log-raw=-"])
        logger = commandline.setup_logging("test_optparse_unicode", args, {})
        self.assertEqual(len(logger.handlers), 1)
        self.assertEqual(logger.handlers[0].stream, sys.stdout)
        self.assertIsInstance(logger.handlers[0], handlers.StreamHandler)

    @unittest.skip("Failed on Windows")
    def test_logging_defaultlevel(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser)

        with self.get_logfile() as logfile:
            args = parser.parse_args(["--log-tbpl=%s" % logfile.name])
            logger = commandline.setup_logging("test_logging_defaultlevel", args, {})
            logger.info("INFO message")
            logger.debug("DEBUG message")
            logger.error("ERROR message")
            # The debug level is not logged by default.
            self.assertEqual([b"INFO message", b"ERROR message"], self.loglines(logfile))

    @unittest.skip("Failed on Windows")
    def test_logging_errorlevel(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser)
        with self.get_logfile() as logfile:
            args = parser.parse_args(
                ["--log-tbpl=%s" % logfile.name, "--log-tbpl-level=error"]
            )
            logger = commandline.setup_logging("test_logging_errorlevel", args, {})
            logger.info("INFO message")
            logger.debug("DEBUG message")
            logger.error("ERROR message")

            # Only the error level and above were requested.
            self.assertEqual([b"ERROR message"], self.loglines(logfile))

    @unittest.skip("Failed on Windows")
    def test_logging_debuglevel(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser)
        with self.get_logfile() as logfile:
            args = parser.parse_args(
                ["--log-tbpl=%s" % logfile.name, "--log-tbpl-level=debug"]
            )
            logger = commandline.setup_logging("test_logging_debuglevel", args, {})
            logger.info("INFO message")
            logger.debug("DEBUG message")
            logger.error("ERROR message")
            # Requesting a lower log level than default works as expected.
            self.assertEqual(
                [b"INFO message", b"DEBUG message", b"ERROR message"], self.loglines(logfile)
            )

    def test_unused_options(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser)
        args = parser.parse_args(["--log-tbpl-level=error"])
        self.assertRaises(
            ValueError, commandline.setup_logging, "test_unused_options", args, {}
        )


class TestBuffer(BaseStructuredTest):
    def assert_log_equals(self, expected, actual=None):
        if actual is None:
            actual = self.pop_last_item()

        all_expected = {
            "pid": os.getpid(),
            "thread": "MainThread",
            "source": "testBuffer",
        }
        specials = set(["time"])

        all_expected.update(expected)
        for key, value in all_expected.items():
            self.assertEqual(actual[key], value)

        self.assertEqual(set(all_expected.keys()) | specials, set(actual.keys()))

    def setUp(self):
        self.logger = structuredlog.StructuredLogger("testBuffer")
        self.handler = handlers.BufferHandler(LogHandler(), message_limit=4)
        self.logger.add_handler(self.handler)

    def tearDown(self):
        self.logger.remove_handler(self.handler)

    def pop_last_item(self):
        return self.handler.inner.items.pop()

    def test_buffer_messages(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.send_message("buffer", "off")
        self.logger.test_status("test1", "sub1", status="PASS")
        # Even for buffered actions, the buffer does not interfere if
        # buffering is turned off.
        self.assert_log_equals(
            {
                "action": "test_status",
                "test": "test1",
                "status": "PASS",
                "subtest": "sub1",
            }
        )
        self.logger.send_message("buffer", "on")
        self.logger.test_status("test1", "sub2", status="PASS")
        self.logger.test_status("test1", "sub3", status="PASS")
        self.logger.test_status("test1", "sub4", status="PASS")
        self.logger.test_status("test1", "sub5", status="PASS")
        self.logger.test_status("test1", "sub6", status="PASS")
        self.logger.test_status("test1", "sub7", status="PASS")
        self.logger.test_end("test1", status="OK")
        self.logger.send_message("buffer", "clear")
        self.assert_log_equals({"action": "test_end", "test": "test1", "status": "OK"})
        self.logger.suite_end()

    def test_buffer_size(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "sub1", status="PASS")
        self.logger.test_status("test1", "sub2", status="PASS")
        self.logger.test_status("test1", "sub3", status="PASS")
        self.logger.test_status("test1", "sub4", status="PASS")
        self.logger.test_status("test1", "sub5", status="PASS")
        self.logger.test_status("test1", "sub6", status="PASS")
        self.logger.test_status("test1", "sub7", status="PASS")

        # No test status messages made it to the underlying handler.
        self.assert_log_equals({"action": "test_start", "test": "test1"})

        # The buffer's actual size never grows beyond the specified limit.
        self.assertEqual(len(self.handler._buffer), 4)

        self.logger.test_status("test1", "sub8", status="FAIL")
        # The number of messages deleted comes back in a list.
        self.assertEqual([4], self.logger.send_message("buffer", "flush"))

        # When the buffer is dumped, the failure is the last thing logged
        self.assert_log_equals(
            {
                "action": "test_status",
                "test": "test1",
                "subtest": "sub8",
                "status": "FAIL",
                "expected": "PASS",
            }
        )
        # Three additional messages should have been retained for context
        self.assert_log_equals(
            {
                "action": "test_status",
                "test": "test1",
                "status": "PASS",
                "subtest": "sub7",
            }
        )
        self.assert_log_equals(
            {
                "action": "test_status",
                "test": "test1",
                "status": "PASS",
                "subtest": "sub6",
            }
        )
        self.assert_log_equals(
            {
                "action": "test_status",
                "test": "test1",
                "status": "PASS",
                "subtest": "sub5",
            }
        )
        self.assert_log_equals({"action": "suite_start", "tests": {"default": []}})


class TestReader(unittest.TestCase):
    def to_file_like(self, obj):
        data_str = "\n".join(json.dumps(item) for item in obj)
        return StringIO(data_str)

    def test_read(self):
        data = [
            {"action": "action_0", "data": "data_0"},
            {"action": "action_1", "data": "data_1"},
        ]

        f = self.to_file_like(data)
        self.assertEqual(data, list(reader.read(f)))

    def test_imap_log(self):
        data = [
            {"action": "action_0", "data": "data_0"},
            {"action": "action_1", "data": "data_1"},
        ]

        f = self.to_file_like(data)

        def f_action_0(item):
            return ("action_0", item["data"])

        def f_action_1(item):
            return ("action_1", item["data"])

        res_iter = reader.imap_log(
            reader.read(f), {"action_0": f_action_0, "action_1": f_action_1}
        )
        self.assertEqual(
            [("action_0", "data_0"), ("action_1", "data_1")], list(res_iter)
        )

    def test_each_log(self):
        data = [
            {"action": "action_0", "data": "data_0"},
            {"action": "action_1", "data": "data_1"},
        ]

        f = self.to_file_like(data)

        count = {"action_0": 0, "action_1": 0}

        def f_action_0(item):
            count[item["action"]] += 1

        def f_action_1(item):
            count[item["action"]] += 2

        reader.each_log(
            reader.read(f), {"action_0": f_action_0, "action_1": f_action_1}
        )

        self.assertEqual({"action_0": 1, "action_1": 2}, count)

    def test_handler(self):
        data = [
            {"action": "action_0", "data": "data_0"},
            {"action": "action_1", "data": "data_1"},
        ]

        f = self.to_file_like(data)

        test = self

        class ReaderHandler(reader.LogHandler):
            def __init__(self):
                self.action_0_count = 0
                self.action_1_count = 0

            def action_0(self, item):
                test.assertEqual(item["action"], "action_0")
                self.action_0_count += 1

            def action_1(self, item):
                test.assertEqual(item["action"], "action_1")
                self.action_1_count += 1

        handler = ReaderHandler()
        reader.handle_log(reader.read(f), handler)

        self.assertEqual(handler.action_0_count, 1)
        self.assertEqual(handler.action_1_count, 1)


if __name__ == "__main__":
    import mozunit
    mozunit.main()
