# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
import collections
import os
import platform
import subprocess
import sys

from mozlog.formatters import base

DEFAULT_MOVE_UP_CODE = u"\x1b[A"
DEFAULT_CLEAR_EOL_CODE = u"\x1b[K"


class GroupingFormatter(base.BaseFormatter):
    """Formatter designed to produce unexpected test results grouped
    together in a readable format."""

    def __init__(self):
        super(GroupingFormatter, self).__init__()
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
        self.show_logs = False

        self.message_handler.register_message_handlers(
            "show_logs",
            {
                "on": self._enable_show_logs,
                "off": self._disable_show_logs,
            },
        )

        # TODO(mrobinson, 8313): We need to add support for Windows terminals here.
        if self.interactive:
            self.move_up, self.clear_eol = self.get_move_up_and_clear_eol_codes()
            if platform.system() != "Windows":
                self.line_width = int(
                    subprocess.check_output(["stty", "size"]).split()[1]
                )
            else:
                # Until we figure out proper Windows support,
                # this makes things work well enough to run.
                self.line_width = 80

        self.expected = {
            "OK": 0,
            "PASS": 0,
            "FAIL": 0,
            "PRECONDITION_FAILED": 0,
            "ERROR": 0,
            "TIMEOUT": 0,
            "SKIP": 0,
            "CRASH": 0,
        }

        self.unexpected_tests = {
            "OK": [],
            "PASS": [],
            "FAIL": [],
            "PRECONDITION_FAILED": [],
            "ERROR": [],
            "TIMEOUT": [],
            "CRASH": [],
        }

        # Follows the format of {(<subsuite>, <test>, <subtest>): <data>}, where
        # (<subsuite>, <test>, None) represents a top level test.
        self.known_intermittent_results = {}

    def _enable_show_logs(self):
        self.show_logs = True

    def _disable_show_logs(self):
        self.show_logs = False

    def get_move_up_and_clear_eol_codes(self):
        try:
            import blessed
        except ImportError:
            return DEFAULT_MOVE_UP_CODE, DEFAULT_CLEAR_EOL_CODE

        try:
            self.terminal = blessed.Terminal()
            return self.terminal.move_up, self.terminal.clear_eol
        except Exception as exception:
            sys.stderr.write(
                "GroupingFormatter: Could not get terminal "
                "control characters: %s\n" % exception
            )
            return DEFAULT_MOVE_UP_CODE, DEFAULT_CLEAR_EOL_CODE

    def text_to_erase_display(self):
        if not self.interactive or not self.current_display:
            return ""
        return (self.move_up + self.clear_eol) * self.current_display.count("\n")

    def generate_output(self, text=None, new_display=None):
        if not self.interactive:
            return text

        output = self.text_to_erase_display()
        if text:
            output += text
        if new_display is not None:
            self.current_display = new_display
        return output + self.current_display

    def build_status_line(self):
        if self.number_of_tests == 0:
            new_display = "  [%i] " % self.completed_tests
        else:
            new_display = "  [%i/%i] " % (self.completed_tests, self.number_of_tests)

        if self.running_tests:
            indent = " " * len(new_display)
            if self.interactive:
                max_width = self.line_width - len(new_display)
            else:
                max_width = sys.maxsize
            return (
                new_display +
                ("\n%s" % indent).join(
                    f"{self.get_test_name_output(subsuite, test_name)}"[:max_width]
                    for subsuite, test_name in self.running_tests.values()
                ) + "\n"
            )
        else:
            return new_display + "No tests running.\n"

    def suite_start(self, data):
        self.number_of_tests = sum(
            len(tests) for tests in data["tests"].values()
        )
        self.start_time = data["time"]

        if self.number_of_tests == 0:
            return "Running tests in %s\n\n" % data[u"source"]
        else:
            return "Running %i tests in %s\n\n" % (
                self.number_of_tests,
                data[u"source"],
            )

    def test_start(self, data):
        self.running_tests[data["thread"]] = (data.get("subsuite"), data["test"])
        return self.generate_output(text=None, new_display=self.build_status_line())

    def wrap_and_indent_lines(self, lines, indent):
        assert len(lines) > 0

        output = indent + u"\u25B6 %s\n" % lines[0]
        for line in lines[1:-1]:
            output += indent + u"\u2502 %s\n" % line
        if len(lines) > 1:
            output += indent + u"\u2514 %s\n" % lines[-1]
        return output

    def get_lines_for_unexpected_result(
        self, test_name, status, expected, message, stack
    ):
        # Test names sometimes contain control characters, which we want
        # to be printed in their raw form, and not their interpreted form.
        test_name = test_name.encode("unicode-escape").decode("utf-8")

        if expected:
            expected_text = u" [expected %s]" % expected
        else:
            expected_text = u""

        lines = [u"%s%s %s" % (status, expected_text, test_name)]
        if message:
            lines.append(u"  \u2192 %s" % message)
        if stack:
            lines.append("")
            lines += [stackline for stackline in stack.splitlines()]
        return lines

    def get_lines_for_known_intermittents(self, known_intermittent_results):
        lines = []

        for (subsuite, test, subtest), data in self.known_intermittent_results.items():
            status = data["status"]
            known_intermittent = ", ".join(data["known_intermittent"])
            expected = " [expected %s, known intermittent [%s]" % (
                data["expected"],
                known_intermittent,
            )
            lines += [
                u"%s%s %s%s"
                % (
                    status,
                    expected,
                    self.get_test_name_output(subsuite, test),
                    (", %s" % subtest) if subtest is not None else "",
                )
            ]
        output = self.wrap_and_indent_lines(lines, "  ") + "\n"
        return output

    def get_test_name_output(self, subsuite, test_name):
        # Generate human readable test name from subsuite and test_name.
        # Vendors can override this function to produce output in a different
        # format that suites their need.
        return f"{subsuite}:{test_name}" if subsuite else test_name

    def get_output_for_unexpected_subtests(self, subsuite, test_name, unexpected_subtests):
        if not unexpected_subtests:
            return ""

        def add_subtest_failure(lines, subtest, stack=None):
            lines += self.get_lines_for_unexpected_result(
                subtest.get("subtest", None),
                subtest.get("status", None),
                subtest.get("expected", None),
                subtest.get("message", None),
                stack,
            )

        def make_subtests_failure(subsuite, test_name, subtests, stack=None):
            lines = [u"Unexpected subtest result in %s:"
                     % self.get_test_name_output(subsuite, test_name)]
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
            if "stack" not in failure:
                output += make_subtests_failure(subsuite, test_name, [failure], None)
            else:
                failures_by_stack[failure["stack"]].append(failure)

        for (stack, failures) in failures_by_stack.items():
            output += make_subtests_failure(subsuite, test_name, failures, stack)
        return output

    def test_end(self, data):
        self.completed_tests += 1
        test_status = data["status"]
        test_name = data["test"]
        subsuite = data.get("subsuite")
        known_intermittent_statuses = data.get("known_intermittent", [])
        subtest_failures = self.subtest_failures.pop((subsuite, test_name), [])
        if "expected" in data and test_status not in known_intermittent_statuses:
            had_unexpected_test_result = True
        else:
            had_unexpected_test_result = False

        del self.running_tests[data["thread"]]
        new_display = self.build_status_line()

        if not had_unexpected_test_result and not subtest_failures:
            self.expected[test_status] += 1
            if self.interactive:
                return self.generate_output(text=None, new_display=new_display)
            else:
                return self.generate_output(
                    text="  %s\n" % self.get_test_name_output(subsuite, test_name),
                    new_display=new_display
                )

        if test_status in known_intermittent_statuses:
            self.known_intermittent_results[(subsuite, test_name, None)] = data

        # If the test crashed or timed out, we also include any process output,
        # because there is a good chance that the test produced a stack trace
        # or other error messages.
        if test_status in ("CRASH", "TIMEOUT"):
            stack = self.test_output[(subsuite, test_name)] + data.get("stack", "")
        else:
            stack = data.get("stack", None)

        output = ""
        if had_unexpected_test_result:
            self.unexpected_tests[test_status].append(data)
            lines = self.get_lines_for_unexpected_result(
                self.get_test_name_output(subsuite, test_name),
                test_status,
                data.get("expected", None),
                data.get("message", None),
                stack,
            )
            output += self.wrap_and_indent_lines(lines, "  ") + "\n"

        if subtest_failures:
            self.tests_with_failing_subtests.append((subsuite, test_name))
            output += self.get_output_for_unexpected_subtests(
                subsuite, test_name, subtest_failures
            )
        self.test_failure_text += output

        return self.generate_output(text=output, new_display=new_display)

    def test_status(self, data):
        if "expected" in data and data["status"] not in data.get(
            "known_intermittent", []
        ):
            key = (data.get("subsuite"), data["test"])
            self.subtest_failures[key].append(data)
        elif data["status"] in data.get("known_intermittent", []):
            key = (data.get("subsuite"), data["test"], data["subtest"])
            self.known_intermittent_results[key] = data

    def suite_end(self, data):
        self.end_time = data["time"]

        if not self.interactive:
            output = u"\n"
        else:
            output = ""

        output += u"Ran %i tests finished in %.1f seconds.\n" % (
            self.completed_tests,
            (self.end_time - self.start_time) / 1000.0,
        )
        output += u"  \u2022 %i ran as expected. %i tests skipped.\n" % (
            sum(self.expected.values()),
            self.expected["SKIP"],
        )
        if self.known_intermittent_results:
            output += u"  \u2022 %i known intermittent results.\n" % (
                len(self.known_intermittent_results)
            )

        def text_for_unexpected_list(text, section):
            tests = self.unexpected_tests[section]
            if not tests:
                return u""
            return u"  \u2022 %i tests %s\n" % (len(tests), text)

        output += text_for_unexpected_list(u"crashed unexpectedly", "CRASH")
        output += text_for_unexpected_list(u"had errors unexpectedly", "ERROR")
        output += text_for_unexpected_list(u"failed unexpectedly", "FAIL")
        output += text_for_unexpected_list(
            u"precondition failed unexpectedly", "PRECONDITION_FAILED"
        )
        output += text_for_unexpected_list(u"timed out unexpectedly", "TIMEOUT")
        output += text_for_unexpected_list(u"passed unexpectedly", "PASS")
        output += text_for_unexpected_list(u"unexpectedly okay", "OK")

        num_with_failing_subtests = len(self.tests_with_failing_subtests)
        if num_with_failing_subtests:
            output += (
                u"  \u2022 %i tests had unexpected subtest results\n"
                % num_with_failing_subtests
            )
        output += "\n"

        # Repeat failing test output, so that it is easier to find, since the
        # non-interactive version prints all the test names.
        if not self.interactive and self.test_failure_text:
            output += u"Tests with unexpected results:\n" + self.test_failure_text

        if self.known_intermittent_results:
            results = self.get_lines_for_known_intermittents(
                self.known_intermittent_results
            )
            output += u"Tests with known intermittent results:\n" + results

        return self.generate_output(text=output, new_display="")

    def process_output(self, data):
        if data["thread"] not in self.running_tests:
            return
        test_key = self.running_tests[data["thread"]]
        self.test_output[test_key] += data["data"] + "\n"

    def log(self, data):
        if data.get("component"):
            message = "%s %s %s" % (data["component"], data["level"], data["message"])
        else:
            message = "%s %s" % (data["level"], data["message"])
        if "stack" in data:
            message += "\n%s" % data["stack"]

        # We are logging messages that begin with STDERR, because that is how exceptions
        # in this formatter are indicated.
        if data["message"].startswith("STDERR"):
            return self.generate_output(text=message + "\n")

        if data["level"] in ("CRITICAL", "ERROR"):
            return self.generate_output(text=message + "\n")
        # Show all messages if show_logs switched on.
        if self.show_logs:
            return self.generate_output(text=message + "\n")
