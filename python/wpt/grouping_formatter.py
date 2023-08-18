# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# This allows using types that are defined later in the file.
from __future__ import annotations

import collections
import os
import sys
import mozlog
import mozlog.formatters.base
import mozlog.reader

from dataclasses import dataclass, field
from typing import Dict, List, Optional, Any
from six import itervalues

DEFAULT_MOVE_UP_CODE = u"\x1b[A"
DEFAULT_CLEAR_EOL_CODE = u"\x1b[K"


@dataclass
class UnexpectedSubtestResult():
    path: str
    subtest: str
    actual: str
    expected: str
    message: str
    time: int
    stack: Optional[str]


@dataclass
class UnexpectedResult():
    path: str
    actual: str
    expected: str
    message: str
    time: int
    stack: Optional[str]
    unexpected_subtest_results: list[UnexpectedSubtestResult] = field(
        default_factory=list)
    issues: list[str] = field(default_factory=list)
    flaky: bool = False

    def __str__(self):
        output = UnexpectedResult.to_lines(self)

        if self.unexpected_subtest_results:
            def make_subtests_failure(subtest_results):
                # Test names sometimes contain control characters, which we want
                # to be printed in their raw form, and not their interpreted form.
                lines = []
                for subtest in subtest_results[:-1]:
                    lines += UnexpectedResult.to_lines(
                        subtest, print_stack=False)
                lines += UnexpectedResult.to_lines(subtest_results[-1])
                return self.wrap_and_indent_lines(lines, "  ").splitlines()

            # Organize the failures by stack trace so we don't print the same stack trace
            # more than once. They are really tall and we don't want to flood the screen
            # with duplicate information.
            results_by_stack = collections.defaultdict(list)
            for subtest_result in self.unexpected_subtest_results:
                results_by_stack[subtest_result.stack].append(subtest_result)

            # Print stackless results first. They are all separate.
            if None in results_by_stack:
                output += make_subtests_failure(results_by_stack.pop(None))
            for subtest_results in results_by_stack.values():
                output += make_subtests_failure(subtest_results)

        return UnexpectedResult.wrap_and_indent_lines(output, "  ")

    @staticmethod
    def wrap_and_indent_lines(lines, indent):
        if not lines:
            return ""

        output = indent + u"\u25B6 %s\n" % lines[0]
        for line in lines[1:-1]:
            output += indent + u"\u2502 %s\n" % line
        if len(lines) > 1:
            output += indent + u"\u2514 %s\n" % lines[-1]
        return output

    @staticmethod
    def to_lines(result: Any[UnexpectedSubtestResult, UnexpectedResult], print_stack=True):
        first_line = result.actual
        if result.expected != result.actual:
            first_line += f" [expected {result.expected}]"

        # Test names sometimes contain control characters, which we want
        # to be printed in their raw form, and not their interpreted form.
        title = result.subtest if isinstance(result, UnexpectedSubtestResult) else result.path
        first_line += f" {title.encode('unicode-escape').decode('utf-8')}"

        if isinstance(result, UnexpectedResult) and result.issues:
            first_line += f" ({', '.join([f'#{bug}' for bug in result.issues])})"

        lines = [first_line]
        if result.message:
            for message_line in result.message.splitlines():
                lines.append(f"  \u2192 {message_line}")
        if print_stack and result.stack:
            lines.append("")
            lines.extend(result.stack.splitlines())
        return lines


class ServoHandler(mozlog.reader.LogHandler):
    """LogHandler designed to collect unexpected results for use by
       script or by the ServoFormatter output formatter."""
    def __init__(self):
        self.reset_state()

    def reset_state(self):
        self.number_of_tests = 0
        self.completed_tests = 0
        self.need_to_erase_last_line = False
        self.running_tests: Dict[str, str] = {}
        self.test_output = collections.defaultdict(str)
        self.subtest_failures = collections.defaultdict(list)
        self.tests_with_failing_subtests = []
        self.unexpected_results: List[UnexpectedResult] = []

        self.expected = {
            'OK': 0,
            'PASS': 0,
            'FAIL': 0,
            'ERROR': 0,
            'TIMEOUT': 0,
            'SKIP': 0,
            'CRASH': 0,
            'PRECONDITION_FAILED': 0,
        }

        self.unexpected_tests = {
            'OK': [],
            'PASS': [],
            'FAIL': [],
            'ERROR': [],
            'TIMEOUT': [],
            'CRASH': [],
            'PRECONDITION_FAILED': [],
        }

    def suite_start(self, data):
        self.reset_state()
        self.number_of_tests = sum(len(tests) for tests in itervalues(data["tests"]))
        self.suite_start_time = data["time"]

    def suite_end(self, _):
        pass

    def test_start(self, data):
        self.running_tests[data['thread']] = data['test']

    @staticmethod
    def data_was_for_expected_result(data):
        if "expected" not in data:
            return True
        return "known_intermittent" in data \
            and data["status"] in data["known_intermittent"]

    def test_end(self, data: dict) -> Optional[UnexpectedResult]:
        self.completed_tests += 1
        test_status = data["status"]
        test_path = data["test"]
        del self.running_tests[data['thread']]

        had_expected_test_result = self.data_was_for_expected_result(data)
        subtest_failures = self.subtest_failures.pop(test_path, [])
        if had_expected_test_result and not subtest_failures:
            self.expected[test_status] += 1
            return None

        # If the test crashed or timed out, we also include any process output,
        # because there is a good chance that the test produced a stack trace
        # or other error messages.
        stack = data.get("stack", None)
        if test_status in ("CRASH", "TIMEOUT"):
            stack = f"\n{stack}" if stack else ""
            stack = f"{self.test_output[test_path]}{stack}"

        result = UnexpectedResult(
            test_path,
            test_status,
            data.get("expected", test_status),
            data.get("message", ""),
            data["time"],
            stack,
            subtest_failures
        )

        if not had_expected_test_result:
            self.unexpected_tests[result.actual].append(data)
        if subtest_failures:
            self.tests_with_failing_subtests.append(data)

        self.unexpected_results.append(result)
        return result

    def test_status(self, data: dict):
        if self.data_was_for_expected_result(data):
            return
        self.subtest_failures[data["test"]].append(UnexpectedSubtestResult(
            data["test"],
            data["subtest"],
            data["status"],
            data["expected"],
            data.get("message", ""),
            data["time"],
            data.get('stack', None),
        ))

    def process_output(self, data):
        if 'test' in data:
            self.test_output[data['test']] += data['data'] + "\n"

    def log(self, _):
        pass


class ServoFormatter(mozlog.formatters.base.BaseFormatter, ServoHandler):
    """Formatter designed to produce unexpected test results grouped
       together in a readable format."""
    def __init__(self):
        ServoHandler.__init__(self)
        self.current_display = ""
        self.interactive = os.isatty(sys.stdout.fileno())

        if self.interactive:
            self.line_width = os.get_terminal_size().columns
            self.move_up = DEFAULT_MOVE_UP_CODE
            self.clear_eol = DEFAULT_CLEAR_EOL_CODE

            try:
                import blessings
                self.terminal = blessings.Terminal()
                self.move_up = self.terminal.move_up
                self.clear_eol = self.terminal.clear_eol
            except Exception as exception:
                sys.stderr.write("GroupingFormatter: Could not get terminal "
                                 "control characters: %s\n" % exception)

    def text_to_erase_display(self):
        if not self.interactive or not self.current_display:
            return ""
        return ((self.move_up + self.clear_eol)
                * self.current_display.count('\n'))

    def generate_output(self, text=None, new_display=None):
        if not self.interactive:
            return text

        output = self.text_to_erase_display()
        if text:
            output += text
        if new_display is not None:
            self.current_display = new_display
        return output + self.current_display

    def test_counter(self):
        if self.number_of_tests == 0:
            return "  [%i] " % self.completed_tests
        else:
            return "  [%i/%i] " % (self.completed_tests, self.number_of_tests)

    def build_status_line(self):
        new_display = self.test_counter()

        if self.running_tests:
            indent = " " * len(new_display)
            if self.interactive:
                max_width = self.line_width - len(new_display)
            else:
                max_width = sys.maxsize
            return new_display + ("\n%s" % indent).join(
                val[:max_width] for val in self.running_tests.values()) + "\n"
        else:
            return new_display + "No tests running.\n"

    def suite_start(self, data):
        ServoHandler.suite_start(self, data)
        if self.number_of_tests == 0:
            return "Running tests in %s\n\n" % data[u'source']
        else:
            return "Running %i tests in %s\n\n" % (self.number_of_tests, data[u'source'])

    def test_start(self, data):
        ServoHandler.test_start(self, data)
        if self.interactive:
            return self.generate_output(new_display=self.build_status_line())

    def test_end(self, data):
        unexpected_result = ServoHandler.test_end(self, data)
        if not unexpected_result:
            if self.interactive:
                return self.generate_output(new_display=self.build_status_line())
            else:
                return self.generate_output(text="%s%s\n" % (self.test_counter(), data["test"]))

        # Surround test output by newlines so that it is easier to read.
        output_for_unexpected_test = f"{unexpected_result}\n"
        return self.generate_output(text=output_for_unexpected_test,
                                    new_display=self.build_status_line())

    def test_status(self, data):
        ServoHandler.test_status(self, data)

    def suite_end(self, data):
        ServoHandler.suite_end(self, data)
        if not self.interactive:
            output = u"\n"
        else:
            output = ""

        output += u"Ran %i tests finished in %.1f seconds.\n" % (
            self.completed_tests, (data["time"] - self.suite_start_time) / 1000)
        output += u"  \u2022 %i ran as expected. %i tests skipped.\n" % (
            sum(self.expected.values()), self.expected['SKIP'])

        def text_for_unexpected_list(text, section):
            tests = self.unexpected_tests[section]
            if not tests:
                return u""
            return u"  \u2022 %i tests %s\n" % (len(tests), text)

        output += text_for_unexpected_list(u"crashed unexpectedly", 'CRASH')
        output += text_for_unexpected_list(u"had errors unexpectedly", 'ERROR')
        output += text_for_unexpected_list(u"failed unexpectedly", 'FAIL')
        output += text_for_unexpected_list(u"precondition failed unexpectedly", 'PRECONDITION_FAILED')
        output += text_for_unexpected_list(u"timed out unexpectedly", 'TIMEOUT')
        output += text_for_unexpected_list(u"passed unexpectedly", 'PASS')
        output += text_for_unexpected_list(u"unexpectedly okay", 'OK')

        num_with_failing_subtests = len(self.tests_with_failing_subtests)
        if num_with_failing_subtests:
            output += (u"  \u2022 %i tests had unexpected subtest results\n"
                       % num_with_failing_subtests)
        output += "\n"

        # Repeat failing test output, so that it is easier to find, since the
        # non-interactive version prints all the test names.
        if not self.interactive and self.unexpected_results:
            output += u"Tests with unexpected results:\n"
            output += "".join([str(result)
                              for result in self.unexpected_results])

        return self.generate_output(text=output, new_display="")

    def process_output(self, data):
        ServoHandler.process_output(self, data)

    def log(self, data):
        ServoHandler.log(self, data)

        # We are logging messages that begin with STDERR, because that is how exceptions
        # in this formatter are indicated.
        if data['message'].startswith('STDERR'):
            return self.generate_output(text=data['message'] + "\n")

        if data['level'] in ('CRITICAL', 'ERROR'):
            return self.generate_output(text=data['message'] + "\n")
