# -*- coding: utf-8 -*-
import argparse
import json
import optparse
import os
import StringIO
import sys
import unittest
import xml.etree.ElementTree as ET

import mozfile

from mozlog.structured import (
    commandline,
    reader,
    structuredlog,
    stdadapter,
    handlers,
    formatters,
)


class TestHandler(object):
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
        self.handler = TestHandler()
        self.logger.add_handler(self.handler)

    def pop_last_item(self):
        return self.handler.items.pop()

    def assert_log_equals(self, expected, actual=None):
        if actual is None:
            actual = self.pop_last_item()

        all_expected = {"pid": os.getpid(),
                        "thread": "MainThread",
                        "source": "test"}
        specials = set(["time"])

        all_expected.update(expected)
        for key, value in all_expected.iteritems():
            self.assertEqual(actual[key], value)

        self.assertEquals(set(all_expected.keys()) | specials, set(actual.keys()))


class TestStatusHandler(BaseStructuredTest):
    def setUp(self):
        super(TestStatusHandler, self).setUp()
        self.handler = handlers.StatusHandler()
        self.logger.add_handler(self.handler)

    def test_failure_run(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "sub1", status='PASS')
        self.logger.test_status("test1", "sub2", status='TIMEOUT')
        self.logger.test_end("test1", status='OK')
        self.logger.suite_end()
        summary = self.handler.summarize()
        self.assertIn('TIMEOUT', summary.unexpected_statuses)
        self.assertEqual(1, summary.unexpected_statuses['TIMEOUT'])
        self.assertIn('PASS', summary.expected_statuses)
        self.assertEqual(1, summary.expected_statuses['PASS'])
        self.assertIn('OK', summary.expected_statuses)
        self.assertEqual(1, summary.expected_statuses['OK'])
        self.assertEqual(2, summary.action_counts['test_status'])
        self.assertEqual(1, summary.action_counts['test_end'])

    def test_error_run(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.error("ERRR!")
        self.logger.test_end("test1", status='OK')
        self.logger.test_start("test2")
        self.logger.test_end("test2", status='OK')
        self.logger.suite_end()
        summary = self.handler.summarize()
        self.assertIn('ERROR', summary.log_level_counts)
        self.assertEqual(1, summary.log_level_counts['ERROR'])
        self.assertIn('OK', summary.expected_statuses)
        self.assertEqual(2, summary.expected_statuses['OK'])


class TestStructuredLog(BaseStructuredTest):
    def test_suite_start(self):
        self.logger.suite_start(["test"])
        self.assert_log_equals({"action": "suite_start",
                                "tests":["test"]})
        self.logger.suite_end()

    def test_suite_end(self):
        self.logger.suite_start([])
        self.logger.suite_end()
        self.assert_log_equals({"action": "suite_end"})

    def test_start(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.assert_log_equals({"action": "test_start",
                                "test":"test1"})

        self.logger.test_start(("test1", "==", "test1-ref"), path="path/to/test")
        self.assert_log_equals({"action": "test_start",
                                "test":("test1", "==", "test1-ref"),
                                "path": "path/to/test"})
        self.logger.suite_end()

    def test_start_inprogress(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_start("test1")
        self.assert_log_equals({"action": "log",
                                "message": "test_start for test1 logged while in progress.",
                                "level": "ERROR"})
        self.logger.suite_end()

    def test_status(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "subtest name", "fail", expected="FAIL", message="Test message")
        self.assert_log_equals({"action": "test_status",
                                "subtest": "subtest name",
                                "status": "FAIL",
                                "message": "Test message",
                                "test":"test1"})
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_status_1(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "subtest name", "fail")
        self.assert_log_equals({"action": "test_status",
                                "subtest": "subtest name",
                                "status": "FAIL",
                                "expected": "PASS",
                                "test":"test1"})
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_status_2(self):
        self.assertRaises(ValueError, self.logger.test_status, "test1", "subtest name", "XXXUNKNOWNXXX")

    def test_status_extra(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "subtest name", "FAIL", expected="PASS", extra={"data": 42})
        self.assert_log_equals({"action": "test_status",
                                "subtest": "subtest name",
                                "status": "FAIL",
                                "expected": "PASS",
                                "test": "test1",
                                "extra": {"data":42}
                            })
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_status_stack(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_status("test1", "subtest name", "FAIL", expected="PASS", stack="many\nlines\nof\nstack")
        self.assert_log_equals({"action": "test_status",
                                "subtest": "subtest name",
                                "status": "FAIL",
                                "expected": "PASS",
                                "test": "test1",
                                "stack": "many\nlines\nof\nstack"
                            })
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

    def test_status_not_started(self):
        self.logger.test_status("test_UNKNOWN", "subtest", "PASS")
        self.assertTrue(self.pop_last_item()["message"].startswith(
            "test_status for test_UNKNOWN logged while not in progress. Logged with data: {"))

    def test_end(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "fail", message="Test message")
        self.assert_log_equals({"action": "test_end",
                                "status": "FAIL",
                                "expected": "OK",
                                "message": "Test message",
                                "test":"test1"})
        self.logger.suite_end()

    def test_end_1(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "PASS", expected="PASS", extra={"data":123})
        self.assert_log_equals({"action": "test_end",
                                "status": "PASS",
                                "extra": {"data": 123},
                                "test":"test1"})
        self.logger.suite_end()

    def test_end_2(self):
        self.assertRaises(ValueError, self.logger.test_end, "test1", "XXXUNKNOWNXXX")

    def test_end_stack(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "PASS", expected="PASS", stack="many\nlines\nof\nstack")
        self.assert_log_equals({"action": "test_end",
                                "status": "PASS",
                                "test": "test1",
                                "stack": "many\nlines\nof\nstack"
                            })
        self.logger.suite_end()

    def test_end_no_start(self):
        self.logger.test_end("test1", "PASS", expected="PASS")
        self.assertTrue(self.pop_last_item()["message"].startswith(
            "test_end for test1 logged while not in progress. Logged with data: {"))
        self.logger.suite_end()

    def test_end_twice(self):
        self.logger.suite_start([])
        self.logger.test_start("test2")
        self.logger.test_end("test2", "PASS", expected="PASS")
        self.assert_log_equals({"action": "test_end",
                                "status": "PASS",
                                "test": "test2"})
        self.logger.test_end("test2", "PASS", expected="PASS")
        last_item = self.pop_last_item()
        self.assertEquals(last_item["action"], "log")
        self.assertEquals(last_item["level"], "ERROR")
        self.assertTrue(last_item["message"].startswith(
            "test_end for test2 logged while not in progress. Logged with data: {"))
        self.logger.suite_end()

    def test_suite_start_twice(self):
        self.logger.suite_start([])
        self.assert_log_equals({"action": "suite_start",
                                "tests": []})
        self.logger.suite_start([])
        last_item = self.pop_last_item()
        self.assertEquals(last_item["action"], "log")
        self.assertEquals(last_item["level"], "ERROR")
        self.logger.suite_end()

    def test_suite_end_no_start(self):
        self.logger.suite_start([])
        self.assert_log_equals({"action": "suite_start",
                                "tests": []})
        self.logger.suite_end()
        self.assert_log_equals({"action": "suite_end"})
        self.logger.suite_end()
        last_item = self.pop_last_item()
        self.assertEquals(last_item["action"], "log")
        self.assertEquals(last_item["level"], "ERROR")

    def test_multiple_loggers_suite_start(self):
        logger1 = structuredlog.StructuredLogger("test")
        self.logger.suite_start([])
        logger1.suite_start([])
        last_item = self.pop_last_item()
        self.assertEquals(last_item["action"], "log")
        self.assertEquals(last_item["level"], "ERROR")

    def test_multiple_loggers_test_start(self):
        logger1 = structuredlog.StructuredLogger("test")
        self.logger.suite_start([])
        self.logger.test_start("test")
        logger1.test_start("test")
        last_item = self.pop_last_item()
        self.assertEquals(last_item["action"], "log")
        self.assertEquals(last_item["level"], "ERROR")

    def test_process(self):
        self.logger.process_output(1234, "test output")
        self.assert_log_equals({"action": "process_output",
                                "process": "1234",
                                "data": "test output"})

    def test_log(self):
        for level in ["critical", "error", "warning", "info", "debug"]:
            getattr(self.logger, level)("message")
            self.assert_log_equals({"action": "log",
                                    "level": level.upper(),
                                    "message": "message"})

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
                self.assert_log_equals({"action": "log",
                                        "level": level.upper(),
                                        "message": "message"})
        finally:
            logging.root.setLevel(old_level)

    def test_add_remove_handlers(self):
        handler = TestHandler()
        self.logger.add_handler(handler)
        self.logger.info("test1")

        self.assert_log_equals({"action": "log",
                                "level": "INFO",
                                "message": "test1"})

        self.assert_log_equals({"action": "log",
                                "level": "INFO",
                                "message": "test1"}, actual=handler.last_item)

        self.logger.remove_handler(handler)
        self.logger.info("test2")

        self.assert_log_equals({"action": "log",
                                "level": "INFO",
                                "message": "test2"})

        self.assert_log_equals({"action": "log",
                                "level": "INFO",
                                "message": "test1"}, actual=handler.last_item)

    def test_wrapper(self):
        file_like = structuredlog.StructuredLogFileLike(self.logger)

        file_like.write("line 1")

        self.assert_log_equals({"action": "log",
                                "level": "INFO",
                                "message": "line 1"})

        file_like.write("line 2\n")

        self.assert_log_equals({"action": "log",
                                "level": "INFO",
                                "message": "line 2"})

        file_like.write("line 3\r")

        self.assert_log_equals({"action": "log",
                                "level": "INFO",
                                "message": "line 3"})

        file_like.write("line 4\r\n")

        self.assert_log_equals({"action": "log",
                                "level": "INFO",
                                "message": "line 4"})


class TestTypeConversions(BaseStructuredTest):
    def test_raw(self):
        self.logger.log_raw({"action":"suite_start", "tests":[1], "time": "1234"})
        self.assert_log_equals({"action": "suite_start",
                                "tests":["1"],
                                "time": 1234})
        self.logger.suite_end()

    def test_tuple(self):
        self.logger.suite_start([])
        self.logger.test_start(("\xf0\x90\x8d\x84\xf0\x90\x8c\xb4\xf0\x90\x8d\x83\xf0\x90\x8d\x84", 42, u"\u16a4"))
        self.assert_log_equals({"action": "test_start",
                                "test": (u'\U00010344\U00010334\U00010343\U00010344', u"42", u"\u16a4")})
        self.logger.suite_end()

    def test_non_string_messages(self):
        self.logger.suite_start([])
        self.logger.info(1)
        self.assert_log_equals({"action": "log",
                                "message": "1",
                                "level": "INFO"})
        self.logger.info([1, (2, '3'), "s", "s" + chr(255)])
        self.assert_log_equals({"action": "log",
                                "message": "[1, (2, '3'), 's', 's\\xff']",
                                "level": "INFO"})
        self.logger.suite_end()

    def test_utf8str_write(self):
        with mozfile.NamedTemporaryFile() as logfile:
            _fmt = formatters.TbplFormatter()
            _handler = handlers.StreamHandler(logfile, _fmt)
            self.logger.add_handler(_handler)
            self.logger.suite_start([])
            self.logger.info("☺")
            logfile.seek(0)
            data = logfile.readlines()[-1].strip()
            self.assertEquals(data, "☺")
            self.logger.suite_end()

    def test_arguments(self):
        self.logger.info(message="test")
        self.assert_log_equals({"action": "log",
                                "message": "test",
                                "level": "INFO"})

        self.logger.suite_start([], {})
        self.assert_log_equals({"action": "suite_start",
                                "tests": [],
                                "run_info": {}})
        self.logger.test_start(test="test1")
        self.logger.test_status("subtest1", "FAIL", test="test1", status="PASS")
        self.assert_log_equals({"action": "test_status",
                                "test": "test1",
                                "subtest": "subtest1",
                                "status": "PASS",
                                "expected": "FAIL"})
        self.logger.process_output(123, "data", "test")
        self.assert_log_equals({"action": "process_output",
                                "process": "123",
                                "command": "test",
                                "data": "data"})
        self.assertRaises(TypeError, self.logger.test_status, subtest="subtest2",
                          status="FAIL", expected="PASS")
        self.assertRaises(TypeError, self.logger.test_status, "test1", "subtest1",
                          "PASS", "FAIL", "message", "stack", {}, "unexpected")
        self.assertRaises(TypeError, self.logger.test_status, "test1", test="test2")
        self.logger.suite_end()


class TestComponentFilter(BaseStructuredTest):
    def test_filter_component(self):
        component_logger = structuredlog.StructuredLogger(self.logger.name,
                                                          "test_component")
        component_logger.component_filter = handlers.LogLevelFilter(lambda x:x, "info")

        self.logger.debug("Test")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals({"action": "log",
                                "level": "DEBUG",
                                "message": "Test"})
        self.assertTrue(self.handler.empty)

        component_logger.info("Test 1")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals({"action": "log",
                                "level": "INFO",
                                "message": "Test 1",
                                "component": "test_component"})

        component_logger.debug("Test 2")
        self.assertTrue(self.handler.empty)

        component_logger.component_filter = None

        component_logger.debug("Test 3")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals({"action": "log",
                                "level": "DEBUG",
                                "message": "Test 3",
                                "component": "test_component"})

    def test_filter_default_component(self):
        component_logger = structuredlog.StructuredLogger(self.logger.name,
                                                          "test_component")

        self.logger.debug("Test")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals({"action": "log",
                                "level": "DEBUG",
                                "message": "Test"})

        self.logger.component_filter = handlers.LogLevelFilter(lambda x:x, "info")

        self.logger.debug("Test 1")
        self.assertTrue(self.handler.empty)

        component_logger.debug("Test 2")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals({"action": "log",
                                "level": "DEBUG",
                                "message": "Test 2",
                                "component": "test_component"})

        self.logger.component_filter = None

        self.logger.debug("Test 3")
        self.assertFalse(self.handler.empty)
        self.assert_log_equals({"action": "log",
                                "level": "DEBUG",
                                "message": "Test 3"})

    def test_filter_message_mutuate(self):
        def filter_mutate(msg):
            if msg["action"] == "log":
                msg["message"] = "FILTERED! %s" % msg["message"]
            return msg

        self.logger.component_filter = filter_mutate
        self.logger.debug("Test")
        self.assert_log_equals({"action": "log",
                                "level": "DEBUG",
                                "message": "FILTERED! Test"})
        self.logger.component_filter = None


class FormatterTest(unittest.TestCase):

    def setUp(self):
        self.position = 0
        self.logger = structuredlog.StructuredLogger("test_%s" % type(self).__name__)
        self.output_file = StringIO.StringIO()
        self.handler = handlers.StreamHandler(
            self.output_file, self.get_formatter())
        self.logger.add_handler(self.handler)

    def set_position(self, pos=None):
        if pos is None:
            pos = self.output_file.tell()
        self.position = pos

    def get_formatter(self):
        raise NotImplementedError("FormatterTest subclasses must implement get_formatter")

    @property
    def loglines(self):
        self.output_file.seek(self.position)
        return [line.rstrip() for line in self.output_file.readlines()]

class TestTBPLFormatter(FormatterTest):

    def get_formatter(self):
        return formatters.TbplFormatter()

    def test_unexpected_message(self):
        self.logger.suite_start([])
        self.logger.test_start("timeout_test")
        self.logger.test_end("timeout_test",
                             "TIMEOUT",
                             message="timed out")
        self.assertIn("TEST-UNEXPECTED-TIMEOUT | timeout_test | timed out",
                      self.loglines)
        self.logger.suite_end()

    def test_default_unexpected_end_message(self):
        self.logger.suite_start([])
        self.logger.test_start("timeout_test")
        self.logger.test_end("timeout_test",
                             "TIMEOUT")
        self.assertIn("TEST-UNEXPECTED-TIMEOUT | timeout_test | expected OK",
                      self.loglines)
        self.logger.suite_end()

    def test_default_unexpected_status_message(self):
        self.logger.suite_start([])
        self.logger.test_start("timeout_test")
        self.logger.test_status("timeout_test",
                                "subtest",
                                status="TIMEOUT")
        self.assertIn("TEST-UNEXPECTED-TIMEOUT | timeout_test | subtest - expected PASS",
                      self.loglines)
        self.logger.test_end("timeout_test", "OK")
        self.logger.suite_end()

    def test_single_newline(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.set_position()
        self.logger.test_status("test1", "subtest",
                                status="PASS",
                                expected="FAIL")
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

        # This sequence should not produce blanklines
        for line in self.loglines:
            self.assertNotEqual("", line, "No blank line should be present in: %s" %
                                self.loglines)


class TestMachFormatter(FormatterTest):

    def get_formatter(self):
        return formatters.MachFormatter(disable_colors=True)

    def test_summary(self):
        self.logger.suite_start([])

        #Some tests that pass
        self.logger.test_start("test1")
        self.logger.test_end("test1", status="PASS", expected="PASS")

        self.logger.test_start("test2")
        self.logger.test_end("test2", status="PASS", expected="TIMEOUT")

        self.logger.test_start("test3")
        self.logger.test_end("test3", status="FAIL", expected="PASS")

        self.set_position()
        self.logger.suite_end()

        self.assertIn("Ran 3 tests", self.loglines)
        self.assertIn("Expected results: 1", self.loglines)
        self.assertIn("Unexpected results: 2 (FAIL: 1, PASS: 1)", self.loglines)
        self.assertNotIn("test1", self.loglines)
        self.assertIn("PASS expected TIMEOUT test2", self.loglines)
        self.assertIn("FAIL test3", self.loglines)

    def test_summary_subtests(self):
        self.logger.suite_start([])

        self.logger.test_start("test1")
        self.logger.test_status("test1", "subtest1", status="PASS")
        self.logger.test_status("test1", "subtest2", status="FAIL")
        self.logger.test_end("test1", status="OK", expected="OK")

        self.logger.test_start("test2")
        self.logger.test_status("test2", "subtest1", status="TIMEOUT", expected="PASS")
        self.logger.test_end("test2", status="TIMEOUT", expected="OK")

        self.set_position()
        self.logger.suite_end()

        self.assertIn("Ran 5 tests (2 parents, 3 subtests)", self.loglines)
        self.assertIn("Expected results: 2", self.loglines)
        self.assertIn("Unexpected results: 3 (FAIL: 1, TIMEOUT: 2)", self.loglines)

    def test_summary_ok(self):
        self.logger.suite_start([])

        self.logger.test_start("test1")
        self.logger.test_status("test1", "subtest1", status="PASS")
        self.logger.test_status("test1", "subtest2", status="PASS")
        self.logger.test_end("test1", status="OK", expected="OK")

        self.logger.test_start("test2")
        self.logger.test_status("test2", "subtest1", status="PASS", expected="PASS")
        self.logger.test_end("test2", status="OK", expected="OK")

        self.set_position()
        self.logger.suite_end()

        self.assertIn("OK", self.loglines)
        self.assertIn("Expected results: 5", self.loglines)
        self.assertIn("Unexpected results: 0", self.loglines)


class TestXUnitFormatter(FormatterTest):

    def get_formatter(self):
        return formatters.XUnitFormatter()

    def log_as_xml(self):
        return ET.fromstring('\n'.join(self.loglines))

    def test_stacktrace_is_present(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "fail", message="Test message", stack='this\nis\na\nstack')
        self.logger.suite_end()

        root = self.log_as_xml()
        self.assertIn('this\nis\na\nstack', root.find('testcase/failure').text)

    def test_failure_message(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "fail", message="Test message")
        self.logger.suite_end()

        root = self.log_as_xml()
        self.assertEquals('Expected OK, got FAIL', root.find('testcase/failure').get('message'))

    def test_suite_attrs(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "ok", message="Test message")
        self.logger.suite_end()

        root = self.log_as_xml()
        self.assertEqual(root.get('skips'), '0')
        self.assertEqual(root.get('failures'), '0')
        self.assertEqual(root.get('errors'), '0')
        self.assertEqual(root.get('tests'), '1')
        self.assertEqual(root.get('time'), '0.00')

    def test_time_is_not_rounded(self):
        # call formatter directly, it is easier here
        formatter = self.get_formatter()
        formatter.suite_start(dict(time=55000))
        formatter.test_start(dict(time=55100))
        formatter.test_end(dict(time=55558, test='id', message='message', status='PASS'))
        xml_string = formatter.suite_end(dict(time=55559))

        root = ET.fromstring(xml_string)
        self.assertEqual(root.get('time'), '0.56')
        self.assertEqual(root.find('testcase').get('time'), '0.46')


class TestCommandline(unittest.TestCase):

    def setUp(self):
        self.logfile = mozfile.NamedTemporaryFile()

    @property
    def loglines(self):
        self.logfile.seek(0)
        return [line.rstrip() for line in self.logfile.readlines()]

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
        commandline.add_logging_group(parser, include_formatters=['raw'])
        other_formatters = [fmt for fmt in commandline.log_formatters
                            if fmt != 'raw']
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

    def test_logging_defaultlevel(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser)

        args = parser.parse_args(["--log-tbpl=%s" % self.logfile.name])
        logger = commandline.setup_logging("test_fmtopts", args, {})
        logger.info("INFO message")
        logger.debug("DEBUG message")
        logger.error("ERROR message")
        # The debug level is not logged by default.
        self.assertEqual(["INFO message",
                          "ERROR message"],
                         self.loglines)

    def test_logging_errorlevel(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser)
        args = parser.parse_args(["--log-tbpl=%s" % self.logfile.name, "--log-tbpl-level=error"])
        logger = commandline.setup_logging("test_fmtopts", args, {})
        logger.info("INFO message")
        logger.debug("DEBUG message")
        logger.error("ERROR message")

        # Only the error level and above were requested.
        self.assertEqual(["ERROR message"],
                         self.loglines)

    def test_logging_debuglevel(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser)
        args = parser.parse_args(["--log-tbpl=%s" % self.logfile.name, "--log-tbpl-level=debug"])
        logger = commandline.setup_logging("test_fmtopts", args, {})
        logger.info("INFO message")
        logger.debug("DEBUG message")
        logger.error("ERROR message")
        # Requesting a lower log level than default works as expected.
        self.assertEqual(["INFO message",
                          "DEBUG message",
                          "ERROR message"],
                         self.loglines)

    def test_unused_options(self):
        parser = argparse.ArgumentParser()
        commandline.add_logging_group(parser)
        args = parser.parse_args(["--log-tbpl-level=error"])
        self.assertRaises(ValueError, commandline.setup_logging, "test_fmtopts", args, {})

class TestBuffer(BaseStructuredTest):

    def assert_log_equals(self, expected, actual=None):
        if actual is None:
            actual = self.pop_last_item()

        all_expected = {"pid": os.getpid(),
                        "thread": "MainThread",
                        "source": "testBuffer"}
        specials = set(["time"])

        all_expected.update(expected)
        for key, value in all_expected.iteritems():
            self.assertEqual(actual[key], value)

        self.assertEquals(set(all_expected.keys()) | specials, set(actual.keys()))

    def setUp(self):
        self.logger = structuredlog.StructuredLogger("testBuffer")
        self.handler = handlers.BufferHandler(TestHandler(), message_limit=4)
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
        self.assert_log_equals({"action": "test_status",
                                "test": "test1",
                                "status": "PASS",
                                "subtest": "sub1"})
        self.logger.send_message("buffer", "on")
        self.logger.test_status("test1", "sub2", status="PASS")
        self.logger.test_status("test1", "sub3", status="PASS")
        self.logger.test_status("test1", "sub4", status="PASS")
        self.logger.test_status("test1", "sub5", status="PASS")
        self.logger.test_status("test1", "sub6", status="PASS")
        self.logger.test_status("test1", "sub7", status="PASS")
        self.logger.test_end("test1", status="OK")
        self.logger.send_message("buffer", "clear")
        self.assert_log_equals({"action": "test_end",
                                "test": "test1",
                                "status": "OK"})
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
        self.assert_log_equals({"action": "test_start",
                                "test": "test1"})

        # The buffer's actual size never grows beyond the specified limit.
        self.assertEquals(len(self.handler._buffer), 4)

        self.logger.test_status("test1", "sub8", status="FAIL")
        # The number of messages deleted comes back in a list.
        self.assertEquals([4], self.logger.send_message("buffer", "flush"))

        # When the buffer is dumped, the failure is the last thing logged
        self.assert_log_equals({"action": "test_status",
                                "test": "test1",
                                "subtest": "sub8",
                                "status": "FAIL",
                                "expected": "PASS"})
        # Three additional messages should have been retained for context
        self.assert_log_equals({"action": "test_status",
                                "test": "test1",
                                "status": "PASS",
                                "subtest": "sub7"})
        self.assert_log_equals({"action": "test_status",
                                "test": "test1",
                                "status": "PASS",
                                "subtest": "sub6"})
        self.assert_log_equals({"action": "test_status",
                                "test": "test1",
                                "status": "PASS",
                                "subtest": "sub5"})
        self.assert_log_equals({"action": "suite_start",
                                "tests": []})


class TestReader(unittest.TestCase):
    def to_file_like(self, obj):
        data_str = "\n".join(json.dumps(item) for item in obj)
        return StringIO.StringIO(data_str)

    def test_read(self):
        data = [{"action": "action_0", "data": "data_0"},
                {"action": "action_1", "data": "data_1"}]

        f = self.to_file_like(data)
        self.assertEquals(data, list(reader.read(f)))

    def test_imap_log(self):
        data = [{"action": "action_0", "data": "data_0"},
                {"action": "action_1", "data": "data_1"}]

        f = self.to_file_like(data)

        def f_action_0(item):
            return ("action_0", item["data"])

        def f_action_1(item):
            return ("action_1", item["data"])

        res_iter = reader.imap_log(reader.read(f),
                                   {"action_0": f_action_0,
                                    "action_1": f_action_1})
        self.assertEquals([("action_0", "data_0"), ("action_1", "data_1")],
                          list(res_iter))

    def test_each_log(self):
        data = [{"action": "action_0", "data": "data_0"},
                {"action": "action_1", "data": "data_1"}]

        f = self.to_file_like(data)

        count = {"action_0":0,
                 "action_1":0}

        def f_action_0(item):
            count[item["action"]] += 1

        def f_action_1(item):
            count[item["action"]] += 2

        reader.each_log(reader.read(f),
                        {"action_0": f_action_0,
                         "action_1": f_action_1})

        self.assertEquals({"action_0":1, "action_1":2}, count)

    def test_handler(self):
        data = [{"action": "action_0", "data": "data_0"},
                {"action": "action_1", "data": "data_1"}]

        f = self.to_file_like(data)

        test = self
        class ReaderTestHandler(reader.LogHandler):
            def __init__(self):
                self.action_0_count = 0
                self.action_1_count = 0

            def action_0(self, item):
                test.assertEquals(item["action"], "action_0")
                self.action_0_count += 1

            def action_1(self, item):
                test.assertEquals(item["action"], "action_1")
                self.action_1_count += 1

        handler = ReaderTestHandler()
        reader.handle_log(reader.read(f), handler)

        self.assertEquals(handler.action_0_count, 1)
        self.assertEquals(handler.action_1_count, 1)

if __name__ == "__main__":
    unittest.main()
