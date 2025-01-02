# encoding: utf-8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import signal
import unittest
import xml.etree.ElementTree as Et
from textwrap import dedent

import pytest
from mozlog.formatters import (
    GroupingFormatter,
    HTMLFormatter,
    MachFormatter,
    TbplFormatter,
    XUnitFormatter,
)
from mozlog.handlers import StreamHandler
from mozlog.structuredlog import StructuredLogger
from six import StringIO, ensure_text, unichr

FORMATS = {
    # A list of tuples consisting of (name, options, expected string).
    "PASS": [
        (
            "mach",
            {},
            dedent(
                """
             0:00.00 SUITE_START: running 3 tests
             0:00.00 TEST_START: test_foo
             0:00.00 TEST_END: OK
             0:00.00 TEST_START: test_bar
             0:00.00 TEST_END: Test OK. Subtests passed 1/1. Unexpected 0
             0:00.00 TEST_START: test_baz
             0:00.00 TEST_END: FAIL
             0:00.00 SUITE_END

            suite 1
            ~~~~~~~
            Ran 4 checks (1 subtests, 3 tests)
            Expected results: 4
            Unexpected results: 0
            OK
            """
            ).lstrip("\n"),
        ),
        (
            "mach",
            {"verbose": True},
            dedent(
                """
             0:00.00 SUITE_START: running 3 tests
             0:00.00 TEST_START: test_foo
             0:00.00 TEST_END: OK
             0:00.00 TEST_START: test_bar
             0:00.00 PASS a subtest
             0:00.00 TEST_END: Test OK. Subtests passed 1/1. Unexpected 0
             0:00.00 TEST_START: test_baz
             0:00.00 TEST_END: FAIL
             0:00.00 SUITE_END

            suite 1
            ~~~~~~~
            Ran 4 checks (1 subtests, 3 tests)
            Expected results: 4
            Unexpected results: 0
            OK
            """
            ).lstrip("\n"),
        ),
    ],
    "FAIL": [
        (
            "mach",
            {},
            dedent(
                """
             0:00.00 SUITE_START: running 3 tests
             0:00.00 TEST_START: test_foo
             0:00.00 TEST_END: FAIL, expected PASS - expected 0 got 1
             0:00.00 TEST_START: test_bar
             0:00.00 TEST_END: Test OK. Subtests passed 0/2. Unexpected 2
            FAIL a subtest - expected 0 got 1
                SimpleTest.is@SimpleTest/SimpleTest.js:312:5
                @caps/tests/mochitest/test_bug246699.html:53:1
            TIMEOUT another subtest
             0:00.00 TEST_START: test_baz
             0:00.00 TEST_END: PASS, expected FAIL
             0:00.00 SUITE_END

            suite 1
            ~~~~~~~
            Ran 5 checks (2 subtests, 3 tests)
            Expected results: 1
            Unexpected results: 4
              test: 2 (1 fail, 1 pass)
              subtest: 2 (1 fail, 1 timeout)

            Unexpected Results
            ------------------
            test_foo
              FAIL test_foo - expected 0 got 1
            test_bar
              FAIL a subtest - expected 0 got 1
                SimpleTest.is@SimpleTest/SimpleTest.js:312:5
                @caps/tests/mochitest/test_bug246699.html:53:1
              TIMEOUT another subtest
            test_baz
              UNEXPECTED-PASS test_baz
            """
            ).lstrip("\n"),
        ),
        (
            "mach",
            {"verbose": True},
            dedent(
                """
             0:00.00 SUITE_START: running 3 tests
             0:00.00 TEST_START: test_foo
             0:00.00 TEST_END: FAIL, expected PASS - expected 0 got 1
             0:00.00 TEST_START: test_bar
             0:00.00 FAIL a subtest - expected 0 got 1
                SimpleTest.is@SimpleTest/SimpleTest.js:312:5
                @caps/tests/mochitest/test_bug246699.html:53:1
             0:00.00 TIMEOUT another subtest
             0:00.00 TEST_END: Test OK. Subtests passed 0/2. Unexpected 2
             0:00.00 TEST_START: test_baz
             0:00.00 TEST_END: PASS, expected FAIL
             0:00.00 SUITE_END

            suite 1
            ~~~~~~~
            Ran 5 checks (2 subtests, 3 tests)
            Expected results: 1
            Unexpected results: 4
              test: 2 (1 fail, 1 pass)
              subtest: 2 (1 fail, 1 timeout)

            Unexpected Results
            ------------------
            test_foo
              FAIL test_foo - expected 0 got 1
            test_bar
              FAIL a subtest - expected 0 got 1
                SimpleTest.is@SimpleTest/SimpleTest.js:312:5
                @caps/tests/mochitest/test_bug246699.html:53:1
              TIMEOUT another subtest
            test_baz
              UNEXPECTED-PASS test_baz
            """
            ).lstrip("\n"),
        ),
    ],
    "PRECONDITION_FAILED": [
        (
            "mach",
            {},
            dedent(
                """
             0:00.00 SUITE_START: running 2 tests
             0:00.00 TEST_START: test_foo
             0:00.00 TEST_END: PRECONDITION_FAILED, expected OK
             0:00.00 TEST_START: test_bar
             0:00.00 TEST_END: Test OK. Subtests passed 1/2. Unexpected 1
            PRECONDITION_FAILED another subtest
             0:00.00 SUITE_END

            suite 1
            ~~~~~~~
            Ran 4 checks (2 subtests, 2 tests)
            Expected results: 2
            Unexpected results: 2
              test: 1 (1 precondition_failed)
              subtest: 1 (1 precondition_failed)

            Unexpected Results
            ------------------
            test_foo
              PRECONDITION_FAILED test_foo
            test_bar
              PRECONDITION_FAILED another subtest
            """
            ).lstrip("\n"),
        ),
        (
            "mach",
            {"verbose": True},
            dedent(
                """
             0:00.00 SUITE_START: running 2 tests
             0:00.00 TEST_START: test_foo
             0:00.00 TEST_END: PRECONDITION_FAILED, expected OK
             0:00.00 TEST_START: test_bar
             0:00.00 PASS a subtest
             0:00.00 PRECONDITION_FAILED another subtest
             0:00.00 TEST_END: Test OK. Subtests passed 1/2. Unexpected 1
             0:00.00 SUITE_END

            suite 1
            ~~~~~~~
            Ran 4 checks (2 subtests, 2 tests)
            Expected results: 2
            Unexpected results: 2
              test: 1 (1 precondition_failed)
              subtest: 1 (1 precondition_failed)

            Unexpected Results
            ------------------
            test_foo
              PRECONDITION_FAILED test_foo
            test_bar
              PRECONDITION_FAILED another subtest
            """
            ).lstrip("\n"),
        ),
    ],
    "KNOWN-INTERMITTENT": [
        (
            "mach",
            {},
            dedent(
                """
             0:00.00 SUITE_START: running 3 tests
             0:00.00 TEST_START: test_foo
             0:00.00 TEST_END: FAIL
            KNOWN-INTERMITTENT-FAIL test_foo
             0:00.00 TEST_START: test_bar
             0:00.00 TEST_END: Test OK. Subtests passed 1/1. Unexpected 0
            KNOWN-INTERMITTENT-PASS a subtest
             0:00.00 TEST_START: test_baz
             0:00.00 TEST_END: FAIL
             0:00.00 SUITE_END

            suite 1
            ~~~~~~~
            Ran 4 checks (1 subtests, 3 tests)
            Expected results: 4 (2 known intermittents)
            Unexpected results: 0

            Known Intermittent Results
            --------------------------
            test_foo
              KNOWN-INTERMITTENT-FAIL test_foo
            test_bar
              KNOWN-INTERMITTENT-PASS a subtest
            OK
            """
            ).lstrip("\n"),
        ),
        (
            "mach",
            {"verbose": True},
            dedent(
                """
             0:00.00 SUITE_START: running 3 tests
             0:00.00 TEST_START: test_foo
             0:00.00 TEST_END: FAIL
            KNOWN-INTERMITTENT-FAIL test_foo
             0:00.00 TEST_START: test_bar
             0:00.00 KNOWN-INTERMITTENT-PASS a subtest
             0:00.00 TEST_END: Test OK. Subtests passed 1/1. Unexpected 0
            KNOWN-INTERMITTENT-PASS a subtest
             0:00.00 TEST_START: test_baz
             0:00.00 TEST_END: FAIL
             0:00.00 SUITE_END

            suite 1
            ~~~~~~~
            Ran 4 checks (1 subtests, 3 tests)
            Expected results: 4 (2 known intermittents)
            Unexpected results: 0

            Known Intermittent Results
            --------------------------
            test_foo
              KNOWN-INTERMITTENT-FAIL test_foo
            test_bar
              KNOWN-INTERMITTENT-PASS a subtest
            OK
            """
            ).lstrip("\n"),
        ),
    ],
}


def ids(test):
    ids = []
    for value in FORMATS[test]:
        args = ", ".join(["{}={}".format(k, v) for k, v in value[1].items()])
        if args:
            args = "-{}".format(args)
        ids.append("{}{}".format(value[0], args))
    return ids


@pytest.fixture(autouse=True)
def timestamp(monkeypatch):
    def fake_time(*args, **kwargs):
        return 0

    monkeypatch.setattr(MachFormatter, "_time", fake_time)


@pytest.mark.parametrize("name,opts,expected", FORMATS["PASS"], ids=ids("PASS"))
def test_pass(get_logger, name, opts, expected):
    logger = get_logger(name, **opts)

    logger.suite_start(["test_foo", "test_bar", "test_baz"])
    logger.test_start("test_foo")
    logger.test_end("test_foo", "OK")
    logger.test_start("test_bar")
    logger.test_status("test_bar", "a subtest", "PASS")
    logger.test_end("test_bar", "OK")
    logger.test_start("test_baz")
    logger.test_end("test_baz", "FAIL", "FAIL", "expected 0 got 1")
    logger.suite_end()

    buf = logger.handlers[0].stream
    result = buf.getvalue()
    print("Dumping result for copy/paste:")
    print(result)
    assert result == expected


@pytest.mark.parametrize("name,opts,expected", FORMATS["FAIL"], ids=ids("FAIL"))
def test_fail(get_logger, name, opts, expected):
    stack = """
    SimpleTest.is@SimpleTest/SimpleTest.js:312:5
    @caps/tests/mochitest/test_bug246699.html:53:1
""".strip(
        "\n"
    )

    logger = get_logger(name, **opts)

    logger.suite_start(["test_foo", "test_bar", "test_baz"])
    logger.test_start("test_foo")
    logger.test_end("test_foo", "FAIL", "PASS", "expected 0 got 1")
    logger.test_start("test_bar")
    logger.test_status(
        "test_bar", "a subtest", "FAIL", "PASS", "expected 0 got 1", stack
    )
    logger.test_status("test_bar", "another subtest", "TIMEOUT")
    logger.test_end("test_bar", "OK")
    logger.test_start("test_baz")
    logger.test_end("test_baz", "PASS", "FAIL")
    logger.suite_end()

    buf = logger.handlers[0].stream
    result = buf.getvalue()
    print("Dumping result for copy/paste:")
    print(result)
    assert result == expected


@pytest.mark.parametrize(
    "name,opts,expected", FORMATS["PRECONDITION_FAILED"], ids=ids("PRECONDITION_FAILED")
)
def test_precondition_failed(get_logger, name, opts, expected):
    logger = get_logger(name, **opts)

    logger.suite_start(["test_foo", "test_bar"])
    logger.test_start("test_foo")
    logger.test_end("test_foo", "PRECONDITION_FAILED")
    logger.test_start("test_bar")
    logger.test_status("test_bar", "a subtest", "PASS")
    logger.test_status("test_bar", "another subtest", "PRECONDITION_FAILED")
    logger.test_end("test_bar", "OK")
    logger.suite_end()

    buf = logger.handlers[0].stream
    result = buf.getvalue()
    print("Dumping result for copy/paste:")
    print(result)
    assert result == expected


@pytest.mark.parametrize(
    "name,opts,expected", FORMATS["KNOWN-INTERMITTENT"], ids=ids("KNOWN-INTERMITTENT")
)
def test_known_intermittent(get_logger, name, opts, expected):
    logger = get_logger(name, **opts)

    logger.suite_start(["test_foo", "test_bar", "test_baz"])
    logger.test_start("test_foo")
    logger.test_end("test_foo", "FAIL", "PASS", known_intermittent=["FAIL"])
    logger.test_start("test_bar")
    logger.test_status(
        "test_bar", "a subtest", "PASS", "FAIL", known_intermittent=["PASS"]
    )
    logger.test_end("test_bar", "OK")
    logger.test_start("test_baz")
    logger.test_end(
        "test_baz", "FAIL", "FAIL", "expected 0 got 1", known_intermittent=["PASS"]
    )
    logger.suite_end()

    buf = logger.handlers[0].stream
    result = buf.getvalue()
    print("Dumping result for copy/paste:")
    print(result)
    assert result == expected


class FormatterTest(unittest.TestCase):
    def setUp(self):
        self.position = 0
        self.logger = StructuredLogger("test_%s" % type(self).__name__)
        self.output_file = StringIO()
        self.handler = StreamHandler(self.output_file, self.get_formatter())
        self.logger.add_handler(self.handler)

    def set_position(self, pos=None):
        if pos is None:
            pos = self.output_file.tell()
        self.position = pos

    def get_formatter(self):
        raise NotImplementedError(
            "FormatterTest subclasses must implement get_formatter"
        )

    @property
    def loglines(self):
        self.output_file.seek(self.position)
        return [ensure_text(line.rstrip()) for line in self.output_file.readlines()]


class TestHTMLFormatter(FormatterTest):
    def get_formatter(self):
        return HTMLFormatter()

    def test_base64_string(self):
        self.logger.suite_start([])
        self.logger.test_start("string_test")
        self.logger.test_end("string_test", "FAIL", extra={"data": "foobar"})
        self.logger.suite_end()
        self.assertIn("data:text/html;charset=utf-8;base64,Zm9vYmFy", self.loglines[-3])

    def test_base64_unicode(self):
        self.logger.suite_start([])
        self.logger.test_start("unicode_test")
        self.logger.test_end("unicode_test", "FAIL", extra={"data": unichr(0x02A9)})
        self.logger.suite_end()
        self.assertIn("data:text/html;charset=utf-8;base64,yqk=", self.loglines[-3])

    def test_base64_other(self):
        self.logger.suite_start([])
        self.logger.test_start("int_test")
        self.logger.test_end("int_test", "FAIL", extra={"data": {"foo": "bar"}})
        self.logger.suite_end()
        self.assertIn(
            "data:text/html;charset=utf-8;base64,eyJmb28iOiAiYmFyIn0=",
            self.loglines[-3],
        )


class TestTBPLFormatter(FormatterTest):
    def get_formatter(self):
        return TbplFormatter()

    def test_unexpected_message(self):
        self.logger.suite_start([])
        self.logger.test_start("timeout_test")
        self.logger.test_end("timeout_test", "TIMEOUT", message="timed out")
        self.assertIn(
            "TEST-UNEXPECTED-TIMEOUT | timeout_test | timed out", self.loglines
        )
        self.logger.suite_end()

    def test_default_unexpected_end_message(self):
        self.logger.suite_start([])
        self.logger.test_start("timeout_test")
        self.logger.test_end("timeout_test", "TIMEOUT")
        self.assertIn(
            "TEST-UNEXPECTED-TIMEOUT | timeout_test | expected OK", self.loglines
        )
        self.logger.suite_end()

    def test_default_unexpected_status_message(self):
        self.logger.suite_start([])
        self.logger.test_start("timeout_test")
        self.logger.test_status("timeout_test", "subtest", status="TIMEOUT")
        self.assertIn(
            "TEST-UNEXPECTED-TIMEOUT | timeout_test | subtest - expected PASS",
            self.loglines,
        )
        self.logger.test_end("timeout_test", "OK")
        self.logger.suite_end()

    def test_known_intermittent_end(self):
        self.logger.suite_start([])
        self.logger.test_start("intermittent_test")
        self.logger.test_end(
            "intermittent_test",
            status="FAIL",
            expected="PASS",
            known_intermittent=["FAIL"],
        )
        # test_end log format:
        # "TEST-KNOWN-INTERMITTENT-<STATUS> | <test> | took <duration>ms"
        # where duration may be different each time
        self.assertIn(
            "TEST-KNOWN-INTERMITTENT-FAIL | intermittent_test | took ", self.loglines[2]
        )
        self.assertIn("ms", self.loglines[2])
        self.logger.suite_end()

    def test_known_intermittent_status(self):
        self.logger.suite_start([])
        self.logger.test_start("intermittent_test")
        self.logger.test_status(
            "intermittent_test",
            "subtest",
            status="FAIL",
            expected="PASS",
            known_intermittent=["FAIL"],
        )
        self.assertIn(
            "TEST-KNOWN-INTERMITTENT-FAIL | intermittent_test | subtest", self.loglines
        )
        self.logger.test_end("intermittent_test", "OK")
        self.logger.suite_end()

    def test_single_newline(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.set_position()
        self.logger.test_status("test1", "subtest", status="PASS", expected="FAIL")
        self.logger.test_end("test1", "OK")
        self.logger.suite_end()

        # This sequence should not produce blanklines
        for line in self.loglines:
            self.assertNotEqual("", line)

    def test_process_exit(self):
        self.logger.process_exit(1234, 0)
        self.assertIn("TEST-INFO | 1234: exit 0", self.loglines)

    @unittest.skipUnless(os.name == "posix", "posix only")
    def test_process_exit_with_sig(self):
        # subprocess return code is negative when process
        # has been killed by signal on posix.
        self.logger.process_exit(1234, -signal.SIGTERM)
        self.assertIn("TEST-INFO | 1234: killed by SIGTERM", self.loglines)


class TestTBPLFormatterWithShutdown(FormatterTest):
    def get_formatter(self):
        return TbplFormatter(summary_on_shutdown=True)

    def test_suite_summary_on_shutdown(self):
        self.logger.suite_start([])
        self.logger.test_start("summary_test")
        self.logger.test_status(
            "summary_test", "subtest", "FAIL", "PASS", known_intermittent=["FAIL"]
        )
        self.logger.test_end("summary_test", "FAIL", "OK", known_intermittent=["FAIL"])
        self.logger.suite_end()
        self.logger.shutdown()

        self.assertIn("suite 1: 2/2 (2 known intermittent tests)", self.loglines)
        self.assertIn("Known Intermittent tests:", self.loglines)
        self.assertIn(
            "TEST-KNOWN-INTERMITTENT-FAIL | summary_test | subtest", self.loglines
        )


class TestMachFormatter(FormatterTest):
    def get_formatter(self):
        return MachFormatter(disable_colors=True)

    def test_summary(self):
        self.logger.suite_start([])

        # Some tests that pass
        self.logger.test_start("test1")
        self.logger.test_end("test1", status="PASS", expected="PASS")

        self.logger.test_start("test2")
        self.logger.test_end("test2", status="PASS", expected="TIMEOUT")

        self.logger.test_start("test3")
        self.logger.test_end("test3", status="FAIL", expected="PASS")

        self.set_position()
        self.logger.suite_end()

        self.assertIn("Ran 3 checks (3 tests)", self.loglines)
        self.assertIn("Expected results: 1", self.loglines)
        self.assertIn(
            """
Unexpected results: 2
  test: 2 (1 fail, 1 pass)
""".strip(),
            "\n".join(self.loglines),
        )
        self.assertNotIn("test1", self.loglines)
        self.assertIn("UNEXPECTED-PASS test2", self.loglines)
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

        self.assertIn("Ran 5 checks (3 subtests, 2 tests)", self.loglines)
        self.assertIn("Expected results: 2", self.loglines)
        self.assertIn(
            """
Unexpected results: 3
  test: 1 (1 timeout)
  subtest: 2 (1 fail, 1 timeout)
""".strip(),
            "\n".join(self.loglines),
        )

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

    def test_process_start(self):
        self.logger.process_start(1234)
        self.assertIn("Started process `1234`", self.loglines[0])

    def test_process_start_with_command(self):
        self.logger.process_start(1234, command="test cmd")
        self.assertIn("Started process `1234` (test cmd)", self.loglines[0])

    def test_process_exit(self):
        self.logger.process_exit(1234, 0)
        self.assertIn("1234: exit 0", self.loglines[0])

    @unittest.skipUnless(os.name == "posix", "posix only")
    def test_process_exit_with_sig(self):
        # subprocess return code is negative when process
        # has been killed by signal on posix.
        self.logger.process_exit(1234, -signal.SIGTERM)
        self.assertIn("1234: killed by SIGTERM", self.loglines[0])


class TestGroupingFormatter(FormatterTest):
    def get_formatter(self):
        return GroupingFormatter()

    def test_results_total(self):
        self.logger.suite_start([])

        self.logger.test_start("test1")
        self.logger.test_status("test1", "subtest1", status="PASS")
        self.logger.test_status("test1", "subtest1", status="PASS")
        self.logger.test_end("test1", status="OK")

        self.logger.test_start("test2")
        self.logger.test_status(
            "test2",
            "subtest2",
            status="FAIL",
            expected="PASS",
            known_intermittent=["FAIL"],
        )
        self.logger.test_end("test2", status="FAIL", expected="OK")

        self.set_position()
        self.logger.suite_end()

        self.assertIn("Ran 2 tests finished in 0.0 seconds.", self.loglines)
        self.assertIn("  \u2022 1 ran as expected. 0 tests skipped.", self.loglines)
        self.assertIn("  \u2022 1 known intermittent results.", self.loglines)
        self.assertIn("  \u2022 1 tests failed unexpectedly", self.loglines)
        self.assertIn("  \u25B6 FAIL [expected OK] test2", self.loglines)
        self.assertIn(
            "  \u25B6 FAIL [expected PASS, known intermittent [FAIL] test2, subtest2",
            self.loglines,
        )


class TestXUnitFormatter(FormatterTest):
    def get_formatter(self):
        return XUnitFormatter()

    def log_as_xml(self):
        return Et.fromstring("\n".join(self.loglines))

    def test_stacktrace_is_present(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end(
            "test1", "fail", message="Test message", stack="this\nis\na\nstack"
        )
        self.logger.suite_end()

        root = self.log_as_xml()
        self.assertIn("this\nis\na\nstack", root.find("testcase/failure").text)

    def test_failure_message(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "fail", message="Test message")
        self.logger.suite_end()

        root = self.log_as_xml()
        self.assertEqual(
            "Expected OK, got FAIL", root.find("testcase/failure").get("message")
        )

    def test_suite_attrs(self):
        self.logger.suite_start([])
        self.logger.test_start("test1")
        self.logger.test_end("test1", "ok", message="Test message")
        self.logger.suite_end()

        root = self.log_as_xml()
        self.assertEqual(root.get("skips"), "0")
        self.assertEqual(root.get("failures"), "0")
        self.assertEqual(root.get("errors"), "0")
        self.assertEqual(root.get("tests"), "1")

    def test_time_is_not_rounded(self):
        # call formatter directly, it is easier here
        formatter = self.get_formatter()
        formatter.suite_start(dict(time=55000))
        formatter.test_start(dict(time=55100))
        formatter.test_end(
            dict(time=55558, test="id", message="message", status="PASS")
        )
        xml_string = formatter.suite_end(dict(time=55559))

        root = Et.fromstring(xml_string)
        self.assertEqual(root.get("time"), "0.56")
        self.assertEqual(root.find("testcase").get("time"), "0.46")


if __name__ == "__main__":
    import mozunit
    mozunit.main()
