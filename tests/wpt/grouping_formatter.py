# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

from mozlog.formatters import base
import collections
import json
import os
import sys
import subprocess
import platform
from six import itervalues, iteritems

DEFAULT_MOVE_UP_CODE = u"\x1b[A"
DEFAULT_CLEAR_EOL_CODE = u"\x1b[K"


class ServoFormatter(base.BaseFormatter):
    """Formatter designed to produce unexpected test results grouped
       together in a readable format."""
    def __init__(self):
        self.number_of_tests = 0
        self.completed_tests = 0
        self.need_to_erase_last_line = False
        self.current_display = ""
        self.running_tests = {}
        self.test_output = collections.defaultdict(str)
        self.subtest_failures = collections.defaultdict(list)
        self.test_failure_text = ""
        self.tests_with_failing_subtests = []
        self.interactive = os.isatty(sys.stdout.fileno())

        # TODO(mrobinson, 8313): We need to add support for Windows terminals here.
        if self.interactive:
            self.move_up, self.clear_eol = self.get_move_up_and_clear_eol_codes()
            if platform.system() != "Windows":
                self.line_width = int(subprocess.check_output(['stty', 'size']).split()[1])
            else:
                # Until we figure out proper Windows support, this makes things work well enough to run.
                self.line_width = 80

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

    def get_move_up_and_clear_eol_codes(self):
        try:
            import blessings
        except ImportError:
            return DEFAULT_MOVE_UP_CODE, DEFAULT_CLEAR_EOL_CODE

        try:
            self.terminal = blessings.Terminal()
            return self.terminal.move_up, self.terminal.clear_eol
        except Exception as exception:
            sys.stderr.write("GroupingFormatter: Could not get terminal "
                             "control characters: %s\n" % exception)
            return DEFAULT_MOVE_UP_CODE, DEFAULT_CLEAR_EOL_CODE

    def text_to_erase_display(self):
        if not self.interactive or not self.current_display:
            return ""
        return ((self.move_up + self.clear_eol)
                * self.current_display.count('\n'))

    def generate_output(self, text=None, new_display=None, unexpected_in_test=None):
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
        self.number_of_tests = sum(len(tests) for tests in itervalues(data["tests"]))
        self.start_time = data["time"]

        if self.number_of_tests == 0:
            return "Running tests in %s\n\n" % data[u'source']
        else:
            return "Running %i tests in %s\n\n" % (self.number_of_tests, data[u'source'])

    def test_start(self, data):
        self.running_tests[data['thread']] = data['test']
        if self.interactive:
            return self.generate_output(new_display=self.build_status_line())

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
            expected_text = u" [expected %s]" % expected
        else:
            expected_text = u""

        lines = [u"%s%s %s" % (status, expected_text, test_name)]
        if message:
            for message_line in message.splitlines():
                lines.append(u"  \u2192 %s" % message_line)
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
            return self.wrap_and_indent_lines(lines, "  ") + "\n"

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
        subtest_failures = self.subtest_failures.pop(test_name, [])

        del self.running_tests[data['thread']]

        if not had_unexpected_test_result and not subtest_failures:
            self.expected[test_status] += 1
            if self.interactive:
                new_display = self.build_status_line()
                return self.generate_output(new_display=new_display)
            else:
                return self.generate_output(text="%s%s\n" % (self.test_counter(), test_name))

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
            output += self.wrap_and_indent_lines(lines, "  ") + "\n"

        if subtest_failures:
            self.tests_with_failing_subtests.append(test_name)
            output += self.get_output_for_unexpected_subtests(test_name,
                                                              subtest_failures)
        self.test_failure_text += output

        new_display = self.build_status_line()
        return self.generate_output(text=output, new_display=new_display,
                                    unexpected_in_test=test_name)

    def test_status(self, data):
        if "expected" in data:
            self.subtest_failures[data["test"]].append(data)

    def suite_end(self, data):
        self.end_time = data["time"]

        if not self.interactive:
            output = u"\n"
        else:
            output = ""

        output += u"Ran %i tests finished in %.1f seconds.\n" % (
            self.completed_tests, (self.end_time - self.start_time) / 1000)
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
        if not self.interactive and self.test_failure_text:
            output += u"Tests with unexpected results:\n" + self.test_failure_text

        return self.generate_output(text=output, new_display="")

    def process_output(self, data):
        if data['thread'] not in self.running_tests:
            return
        test_name = self.running_tests[data['thread']]
        self.test_output[test_name] += data['data'] + "\n"

    def log(self, data):
        # We are logging messages that begin with STDERR, because that is how exceptions
        # in this formatter are indicated.
        if data['message'].startswith('STDERR'):
            return self.generate_output(text=data['message'] + "\n")

        if data['level'] in ('CRITICAL', 'ERROR'):
            return self.generate_output(text=data['message'] + "\n")


class ServoJsonFormatter(ServoFormatter):
    def suite_start(self, data):
        ServoFormatter.suite_start(self, data)
        # Don't forward the return value

    def generate_output(self, text=None, new_display=None, unexpected_in_test=None):
        if unexpected_in_test:
            return "%s\n" % json.dumps({"test": unexpected_in_test, "output": text})

    def log(self, _):
        return
