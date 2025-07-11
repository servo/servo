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
from typing import DefaultDict, Dict, Optional, NotRequired, Union, TypedDict, Literal
from six import itervalues

DEFAULT_MOVE_UP_CODE = "\x1b[A"
DEFAULT_CLEAR_EOL_CODE = "\x1b[K"


@dataclass
class UnexpectedSubtestResult:
    path: str
    subtest: str
    actual: str
    expected: str
    message: str
    time: int
    stack: Optional[str]


@dataclass
class UnexpectedResult:
    path: str
    subsuite: Optional[str]
    actual: str
    expected: str
    message: str
    time: int
    stack: Optional[str]
    unexpected_subtest_results: list[UnexpectedSubtestResult] = field(default_factory=list)
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
                    lines += UnexpectedResult.to_lines(subtest, print_stack=False)
                lines += UnexpectedResult.to_lines(subtest_results[-1])
                return self.wrap_and_indent_lines(lines, "  ").splitlines()

            # Organize the failures by stack trace so we don't print the same stack trace
            # more than once. They are really tall and we don't want to flood the screen
            # with duplicate information.
            results_by_stack: DefaultDict[str | None, list[UnexpectedSubtestResult]] = collections.defaultdict(list)
            for subtest_result in self.unexpected_subtest_results:
                results_by_stack[subtest_result.stack].append(subtest_result)

            # Print stackless results first. They are all separate.
            if None in results_by_stack:
                output += make_subtests_failure(results_by_stack.pop(None))
            for subtest_results in results_by_stack.values():
                output += make_subtests_failure(subtest_results)

        return UnexpectedResult.wrap_and_indent_lines(output, "  ")

    @staticmethod
    def wrap_and_indent_lines(lines, indent: str):
        if not lines:
            return ""

        output = indent + "\u25b6 %s\n" % lines[0]
        for line in lines[1:-1]:
            output += indent + "\u2502 %s\n" % line
        if len(lines) > 1:
            output += indent + "\u2514 %s\n" % lines[-1]
        return output

    @staticmethod
    def to_lines(result: Union[UnexpectedSubtestResult, UnexpectedResult], print_stack=True) -> list[str]:
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


class GlobalTestData(TypedDict):
    action: str
    time: int
    thread: str
    pid: int
    source: str


Status = Literal["PASS", "FAIL", "PRECONDITION_FAILED", "TIMEOUT", "CRASH", "ASSERT", "SKIP", "OK", "ERROR"]


class SuiteStartData(GlobalTestData):
    tests: Dict
    name: NotRequired[str]
    run_info: NotRequired[Dict]
    version_info: NotRequired[Dict]
    device_info: NotRequired[Dict]


class TestStartData(GlobalTestData):
    test: str
    path: NotRequired[str]
    known_intermittent: Status
    subsuite: NotRequired[str]
    group: NotRequired[str]


class TestEndData(GlobalTestData):
    test: str
    status: Status
    expected: Status
    known_intermittent: Status
    message: NotRequired[str]
    stack: NotRequired[str]
    extra: NotRequired[str]
    subsuite: NotRequired[str]
    group: NotRequired[str]


class TestStatusData(TestEndData):
    subtest: str


class ServoHandler(mozlog.reader.LogHandler):
    """LogHandler designed to collect unexpected results for use by
    script or by the ServoFormatter output formatter."""

    number_of_tests: int
    completed_tests: int
    need_to_erase_last_line: int
    running_tests: Dict[str, str]
    test_output: DefaultDict[str, str]
    subtest_failures: DefaultDict[str, list]
    tests_with_failing_subtests: list
    unexpected_results: list
    expected: Dict[str, int]
    unexpected_tests: Dict[str, list]
    suite_start_time: int

    def __init__(self, detect_flakes=False) -> None:
        """
        Flake detection assumes first suite is actual run
        and rest of the suites are retry-unexpected for flakes detection.
        """
        self.detect_flakes = detect_flakes
        self.currently_detecting_flakes = False
        self.reset_state()

    def reset_state(self) -> None:
        self.number_of_tests = 0
        self.completed_tests = 0
        self.need_to_erase_last_line = False
        self.running_tests = {}
        if self.currently_detecting_flakes:
            return
        self.currently_detecting_flakes = False
        self.test_output = collections.defaultdict(str)
        self.subtest_failures = collections.defaultdict(list)
        self.tests_with_failing_subtests = []
        self.unexpected_results = []

        self.expected = {
            "OK": 0,
            "PASS": 0,
            "FAIL": 0,
            "ERROR": 0,
            "TIMEOUT": 0,
            "SKIP": 0,
            "CRASH": 0,
            "PRECONDITION_FAILED": 0,
        }

        self.unexpected_tests = {
            "OK": [],
            "PASS": [],
            "FAIL": [],
            "ERROR": [],
            "TIMEOUT": [],
            "CRASH": [],
            "PRECONDITION_FAILED": [],
        }

    def any_stable_unexpected(self) -> bool:
        return any(not unexpected.flaky for unexpected in self.unexpected_results)

    def suite_start(self, data: SuiteStartData) -> Optional[str]:
        # If there were any unexpected results and we are starting another suite, assume
        # that this suite has been launched to detect intermittent tests.
        # TODO: Support running more than a single suite at once.
        if self.unexpected_results:
            self.currently_detecting_flakes = True
        self.reset_state()

        self.number_of_tests = sum(len(tests) for tests in itervalues(data["tests"]))
        self.suite_start_time = data["time"]

    def suite_end(self, data) -> Optional[str]:
        pass

    def test_start(self, data: TestStartData) -> Optional[str]:
        self.running_tests[data["thread"]] = data["test"]

    @staticmethod
    def data_was_for_expected_result(data):
        if "expected" not in data:
            return True
        return "known_intermittent" in data and data["status"] in data["known_intermittent"]

    def test_end(self, data: TestEndData) -> Union[UnexpectedResult, str, None]:
        self.completed_tests += 1
        test_status = data["status"]
        test_path = data["test"]
        test_subsuite = data["subsuite"]
        del self.running_tests[data["thread"]]

        had_expected_test_result = self.data_was_for_expected_result(data)
        subtest_failures = self.subtest_failures.pop(test_path, [])
        test_output = self.test_output.pop(test_path, "")

        if had_expected_test_result and not subtest_failures:
            if not self.currently_detecting_flakes:
                self.expected[test_status] += 1
            else:
                # When `retry_unexpected` is passed and we are currently detecting flaky tests
                # we assume that this suite only runs tests that have already been run and are
                # in the list of unexpected results.
                for unexpected in self.unexpected_results:
                    if unexpected.path == test_path:
                        unexpected.flaky = True
                        break

            return None

        # If we are currently detecting flakes and a test still had an unexpected
        # result, it's enough to simply return the unexpected result. It isn't
        # necessary to update any of the test counting data structures.
        if self.currently_detecting_flakes:
            return UnexpectedResult(
                test_path,
                test_subsuite,
                test_status,
                data.get("expected", test_status),
                data.get("message", ""),
                data["time"],
                "",
                subtest_failures,
            )

        # If the test crashed or timed out, we also include any process output,
        # because there is a good chance that the test produced a stack trace
        # or other error messages.
        stack = data.get("stack", None)
        if test_status in ("CRASH", "TIMEOUT"):
            stack = f"\n{stack}" if stack else ""
            stack = f"{test_output}{stack}"

        result = UnexpectedResult(
            test_path,
            test_subsuite,
            test_status,
            data.get("expected", test_status),
            data.get("message", ""),
            data["time"],
            stack,
            subtest_failures,
        )

        if not had_expected_test_result:
            self.unexpected_tests[result.actual].append(data)
        if subtest_failures:
            self.tests_with_failing_subtests.append(data)

        self.unexpected_results.append(result)
        return result

    def test_status(self, data: TestStatusData) -> None:
        if self.data_was_for_expected_result(data):
            return
        self.subtest_failures[data["test"]].append(
            UnexpectedSubtestResult(
                data["test"],
                data["subtest"],
                data["status"],
                data["expected"],
                data.get("message", ""),
                data["time"],
                data.get("stack", None),
            )
        )

    def process_output(self, data) -> None:
        if "test" in data:
            self.test_output[data["test"]] += data["data"] + "\n"

    def log(self, data) -> str | None:
        pass


class ServoFormatter(mozlog.formatters.base.BaseFormatter, ServoHandler):
    """Formatter designed to produce unexpected test results grouped
    together in a readable format."""

    current_display: str
    interactive: bool
    number_skipped: int
    move_up: str
    clear_eol: str

    def __init__(self) -> None:
        ServoHandler.__init__(self)
        self.current_display = ""
        self.interactive = os.isatty(sys.stdout.fileno())
        self.number_skipped = 0

        if self.interactive:
            self.line_width = os.get_terminal_size().columns
            self.move_up = DEFAULT_MOVE_UP_CODE
            self.clear_eol = DEFAULT_CLEAR_EOL_CODE

            try:
                import blessed

                self.terminal = blessed.Terminal()
                self.move_up = self.terminal.move_up
                self.clear_eol = self.terminal.clear_eol
            except Exception as exception:
                sys.stderr.write("GroupingFormatter: Could not get terminal control characters: %s\n" % exception)

    def text_to_erase_display(self) -> str:
        if not self.interactive or not self.current_display:
            return ""
        return (self.move_up + self.clear_eol) * self.current_display.count("\n")

    def generate_output(self, text=None, new_display=None) -> str | None:
        if not self.interactive:
            return text

        output = self.text_to_erase_display()
        if text:
            output += text
        if new_display is not None:
            self.current_display = new_display
        return output + self.current_display

    def test_counter(self) -> str:
        if self.number_of_tests == 0:
            return "  [%i] " % self.completed_tests
        else:
            return "  [%i/%i] " % (self.completed_tests, self.number_of_tests)

    def build_status_line(self) -> str:
        new_display = self.test_counter()

        if self.running_tests:
            indent = " " * len(new_display)
            if self.interactive:
                max_width = self.line_width - len(new_display)
            else:
                max_width = sys.maxsize
            return new_display + ("\n%s" % indent).join(val[:max_width] for val in self.running_tests.values()) + "\n"
        else:
            return new_display + "No tests running.\n"

    def suite_start(self, data) -> str:
        ServoHandler.suite_start(self, data)
        maybe_flakes_msg = " to detect flaky tests" if self.currently_detecting_flakes else ""
        if self.number_of_tests == 0:
            return f"Running tests in {data['source']}{maybe_flakes_msg}\n\n"
        else:
            return f"Running {self.number_of_tests} tests in {data['source']}{maybe_flakes_msg}\n\n"

    def test_start(self, data) -> str | None:
        ServoHandler.test_start(self, data)
        if self.interactive:
            return self.generate_output(new_display=self.build_status_line())

    def test_end(self, data) -> str | None:
        unexpected_result = ServoHandler.test_end(self, data)
        if unexpected_result:
            # Surround test output by newlines so that it is easier to read.
            output_for_unexpected_test = f"{unexpected_result}\n"
            return self.generate_output(text=output_for_unexpected_test, new_display=self.build_status_line())

        # Print reason that tests are skipped.
        if data["status"] == "SKIP":
            self.number_skipped += 1
            lines = [f"SKIP {data['test']}", f"{data.get('message', '')}\n"]
            output_for_skipped_test = UnexpectedResult.wrap_and_indent_lines(lines, indent="  ")
            return self.generate_output(text=output_for_skipped_test, new_display=self.build_status_line())

        if self.interactive:
            return self.generate_output(new_display=self.build_status_line())
        else:
            return self.generate_output(text="%s%s\n" % (self.test_counter(), data["test"]))

    def test_status(self, data) -> None:
        ServoHandler.test_status(self, data)

    def suite_end(self, data) -> str | None:
        ServoHandler.suite_end(self, data)
        if not self.interactive:
            output = "\n"
        else:
            output = ""

        output += "Ran %i tests finished in %.1f seconds.\n" % (
            self.completed_tests,
            (data["time"] - self.suite_start_time) / 1000,
        )

        # Sum the number of expected test results from each category
        expected_test_results = sum(self.expected.values())
        output += f"  \u2022 {expected_test_results} ran as expected.\n"
        if self.number_skipped:
            output += f"    \u2022 {self.number_skipped} skipped.\n"

        def text_for_unexpected_list(text: str, section: str) -> str:
            tests = self.unexpected_tests[section]
            if not tests:
                return ""
            return "  \u2022 %i tests %s\n" % (len(tests), text)

        output += text_for_unexpected_list("crashed unexpectedly", "CRASH")
        output += text_for_unexpected_list("had errors unexpectedly", "ERROR")
        output += text_for_unexpected_list("failed unexpectedly", "FAIL")
        output += text_for_unexpected_list("precondition failed unexpectedly", "PRECONDITION_FAILED")
        output += text_for_unexpected_list("timed out unexpectedly", "TIMEOUT")
        output += text_for_unexpected_list("passed unexpectedly", "PASS")
        output += text_for_unexpected_list("unexpectedly okay", "OK")

        num_with_failing_subtests = len(self.tests_with_failing_subtests)
        if num_with_failing_subtests:
            output += "  \u2022 %i tests had unexpected subtest results\n" % num_with_failing_subtests
        output += "\n"

        # Repeat failing test output, so that it is easier to find, since the
        # non-interactive version prints all the test names.
        if not self.interactive and self.unexpected_results:
            output += "Tests with unexpected results:\n"
            output += "".join([str(result) for result in self.unexpected_results])

        return self.generate_output(text=output, new_display="")

    def process_output(self, data) -> None:
        ServoHandler.process_output(self, data)

    def log(self, data) -> str | None:
        ServoHandler.log(self, data)

        # We are logging messages that begin with STDERR, because that is how exceptions
        # in this formatter are indicated.
        if data["message"].startswith("STDERR"):
            return self.generate_output(text=data["message"] + "\n")

        if data["level"] in ("CRITICAL", "ERROR"):
            return self.generate_output(text=data["message"] + "\n")
