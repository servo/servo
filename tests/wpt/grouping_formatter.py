# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import collections
import os
import sys
import mozlog
import mozlog.formatters.base
import mozlog.reader

from typing import Dict, List, NamedTuple
from six import itervalues, iteritems

DEFAULT_MOVE_UP_CODE = u"\x1b[A"
DEFAULT_CLEAR_EOL_CODE = u"\x1b[K"


class UnexpectedResult(NamedTuple):
    test_name: str
    test_status: str
    output: str


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

    def wrap_and_indent_lines(self, lines, indent):
        assert(len(lines) > 0)

        output = indent + u"\u25B6 %s\n" % lines[0]
        for line in lines[1:-1]:
            output += indent + u"\u2502 %s\n" % line
        if len(lines) > 1:
            output += indent + u"\u2514 %s\n" % lines[-1]
        return output

    def get_lines_for_unexpected_result(self,
                                        test_name,
                                        status,
                                        expected,
                                        message,
                                        stack):
        # Test names sometimes contain control characters, which we want
        # to be printed in their raw form, and not their interpreted form.
        test_name = test_name.encode('unicode-escape')

        if expected:
            expected_text = f" [expected {expected}]"
        else:
            expected_text = u""

        lines = [f"{status}{expected_text} {test_name}"]
        if message:
            for message_line in message.splitlines():
                lines.append(f"  \u2192 {message_line}")
        if stack:
            lines.append("")
            lines.extend(stack.splitlines())
        return lines

    def get_output_for_unexpected_subtests(self, test_name, unexpected_subtests):
        if not unexpected_subtests:
            return ""

        def add_subtest_failure(lines, subtest, stack=None):
            lines += self.get_lines_for_unexpected_result(
                subtest.get('subtest', None),
                subtest.get('status', None),
                subtest.get('expected', None),
                subtest.get('message', None),
                stack)

        def make_subtests_failure(test_name, subtests, stack=None):
            lines = [u"Unexpected subtest result in %s:" % test_name]
            for subtest in subtests[:-1]:
                add_subtest_failure(lines, subtest, None)
            add_subtest_failure(lines, subtests[-1], stack)
            return self.wrap_and_indent_lines(lines, "  ")

        # Organize the failures by stack trace so we don't print the same stack trace
        # more than once. They are really tall and we don't want to flood the screen
        # with duplicate information.
        output = ""
        failures_by_stack = collections.defaultdict(list)
        for failure in unexpected_subtests:
            # Print stackless results first. They are all separate.
            if 'stack' not in failure:
                output += make_subtests_failure(test_name, [failure], None)
            else:
                failures_by_stack[failure['stack']].append(failure)

        for (stack, failures) in iteritems(failures_by_stack):
            output += make_subtests_failure(test_name, failures, stack)
        return output

    def test_end(self, data):
        self.completed_tests += 1
        test_status = data["status"]
        test_name = data["test"]
        had_unexpected_test_result = "expected" in data
        subtest_failures = self.subtest_failures.get(test_name, [])

        del self.running_tests[data['thread']]

        if not had_unexpected_test_result and not subtest_failures:
            self.expected[test_status] += 1
            return None

        # If the test crashed or timed out, we also include any process output,
        # because there is a good chance that the test produced a stack trace
        # or other error messages.
        if test_status in ("CRASH", "TIMEOUT"):
            stack = self.test_output[test_name] + data.get('stack', "")
        else:
            stack = data.get('stack', None)

        output = ""
        if had_unexpected_test_result:
            self.unexpected_tests[test_status].append(data)
            lines = self.get_lines_for_unexpected_result(
                test_name,
                test_status,
                data.get('expected', None),
                data.get('message', None),
                stack)
            output += self.wrap_and_indent_lines(lines, "  ")

        if subtest_failures:
            self.tests_with_failing_subtests.append(test_name)
            output += self.get_output_for_unexpected_subtests(test_name,
                                                              subtest_failures)
        self.unexpected_results.append(
            UnexpectedResult(test_name, test_status, output))
        return output

    def test_status(self, data):
        if "expected" in data:
            self.subtest_failures[data["test"]].append(data)

    def process_output(self, data):
        if data['thread'] not in self.running_tests:
            return
        test_name = self.running_tests[data['thread']]
        self.test_output[test_name] += data['data'] + "\n"

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
        output_for_unexpected_test = ServoHandler.test_end(self, data)
        if not output_for_unexpected_test:
            if self.interactive:
                return self.generate_output(new_display=self.build_status_line())
            else:
                return self.generate_output(text="%s%s\n" % (self.test_counter(), data["test"]))

        # Surround test output by newlines so that it is easier to read.
        output_for_unexpected_test = f"{output_for_unexpected_test}\n"
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
            output += "".join([result.output for result in self.unexpected_results])

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
