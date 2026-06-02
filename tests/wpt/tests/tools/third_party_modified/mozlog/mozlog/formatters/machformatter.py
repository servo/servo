# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


import time
from functools import reduce

from mozterm import Terminal

from ..handlers import SummaryHandler
from . import base
from .process import strstatus
from .tbplformatter import TbplFormatter

color_dict = {
    "log_test_status_fail": "red",
    "log_process_output": "blue",
    "log_test_status_pass": "green",
    "log_test_status_unexpected_fail": "red",
    "log_test_status_known_intermittent": "yellow",
    "time": "cyan",
    "action": "yellow",
    "pid": "cyan",
    "heading": "bold_yellow",
    "sub_heading": "yellow",
    "error": "red",
    "warning": "yellow",
    "bold": "bold",
    "grey": "grey",
    "normal": "normal",
    "bright_black": "bright_black",
}

DEFAULT = "\x1b(B\x1b[m"


def format_seconds(total):
    """Format number of seconds to MM:SS.DD form."""
    minutes, seconds = divmod(total, 60)
    return "%2d:%05.2f" % (minutes, seconds)


class TerminalColors(object):
    def __init__(self, term, color_dict):
        for key, value in color_dict.items():
            attribute = getattr(term, value)
            # In Blessed, these attributes aren't always callable. We can assume
            # that if they're not, they're just the raw ANSI Escape Sequences.
            # This TerminalColors class is basically just a lookup table for
            # what function to call to format/color an input string a certain way.
            # So if the attribute above is a callable, we can just proceed, but
            # if it's not, we need to create our own function that prepends the
            # raw ANSI Escape Sequences to the input string, so that everything
            # has the same behavior. We append DEFAULT to reset to no formatting
            # at the end of our string, to prevent text that comes afterwards
            # from inheriting the prepended formatting.
            if not callable(attribute):

                def apply_formatting(text):
                    return attribute + text + DEFAULT

                attribute = apply_formatting
            setattr(self, key, attribute)


class MachFormatter(base.BaseFormatter):
    def __init__(
        self,
        start_time=None,
        write_interval=False,
        write_times=True,
        terminal=None,
        disable_colors=False,
        summary_on_shutdown=False,
        verbose=False,
        enable_screenshot=False,
        **kwargs
    ):
        super(MachFormatter, self).__init__(**kwargs)

        if start_time is None:
            start_time = time.time()
        start_time = int(start_time * 1000)
        self.start_time = start_time
        self.write_interval = write_interval
        self.write_times = write_times
        self.status_buffer = {}
        self.has_unexpected = {}
        self.last_time = None
        self.color_formatter = TerminalColors(
            Terminal(disable_styling=disable_colors), color_dict
        )
        self.verbose = verbose
        self._known_pids = set()
        self.tbpl_formatter = None
        self.enable_screenshot = enable_screenshot
        self.summary = SummaryHandler()
        self.summary_on_shutdown = summary_on_shutdown

        message_handlers = {
            "colors": {
                "on": self._enable_colors,
                "off": self._disable_colors,
            },
            "summary_on_shutdown": {
                "on": self._enable_summary_on_shutdown,
                "off": self._disable_summary_on_shutdown,
            },
        }

        for topic, handlers in message_handlers.items():
            self.message_handler.register_message_handlers(topic, handlers)

    def __call__(self, data):
        self.summary(data)

        s = super(MachFormatter, self).__call__(data)
        if s is None:
            return

        time = self.color_formatter.time(format_seconds(self._time(data)))
        return "%s %s\n" % (time, s)

    def _enable_colors(self):
        self.disable_colors = False

    def _disable_colors(self):
        self.disable_colors = True

    def _enable_summary_on_shutdown(self):
        self.summary_on_shutdown = True

    def _disable_summary_on_shutdown(self):
        self.summary_on_shutdown = False

    def _get_test_id(self, data):
        test_id = data.get("test")
        if isinstance(test_id, list):
            test_id = tuple(test_id)
        return test_id

    def _get_file_name(self, test_id):
        if isinstance(test_id, str):
            return test_id

        if isinstance(test_id, tuple):
            return "".join(test_id)

        assert False, "unexpected test_id"

    def suite_start(self, data):
        num_tests = reduce(lambda x, y: x + len(y), data["tests"].values(), 0)
        action = self.color_formatter.action(data["action"].upper())
        name = ""
        if "name" in data:
            name = " %s -" % (data["name"],)
        return "%s:%s running %i tests" % (action, name, num_tests)

    def suite_end(self, data):
        action = self.color_formatter.action(data["action"].upper())
        rv = [action]
        if not self.summary_on_shutdown:
            rv.append(
                self._format_suite_summary(
                    self.summary.current_suite, self.summary.current
                )
            )
        return "\n".join(rv)

    def _format_expected(self, status, expected, known_intermittent=[]):
        if status == expected:
            color = self.color_formatter.log_test_status_pass
            if expected not in ("PASS", "OK"):
                color = self.color_formatter.log_test_status_fail
                status = "EXPECTED-%s" % status
        else:
            if status in known_intermittent:
                color = self.color_formatter.log_test_status_known_intermittent
                status = "KNOWN-INTERMITTENT-%s" % status
            else:
                color = self.color_formatter.log_test_status_fail
                if status in ("PASS", "OK"):
                    status = "UNEXPECTED-%s" % status
        return color(status)

    def _format_status(self, test, data):
        name = data.get("subtest", test)
        rv = "%s %s" % (
            self._format_expected(
                data["status"],
                data.get("expected", data["status"]),
                data.get("known_intermittent", []),
            ),
            name,
        )
        if "message" in data:
            rv += " - %s" % data["message"]
        if "stack" in data:
            rv += self._format_stack(data["stack"])
        return rv

    def _format_stack(self, stack):
        return "\n%s\n" % self.color_formatter.bright_black(stack.strip("\n"))

    def _format_suite_summary(self, suite, summary):
        count = summary["counts"]
        logs = summary["unexpected_logs"]
        intermittent_logs = summary["intermittent_logs"]
        harness_errors = summary["harness_errors"]

        rv = [
            "",
            self.color_formatter.sub_heading(suite),
            self.color_formatter.sub_heading("~" * len(suite)),
        ]

        # Format check counts
        checks = self.summary.aggregate("count", count)
        rv.append(
            "Ran {} checks ({})".format(
                sum(checks.values()),
                ", ".join(
                    ["{} {}s".format(v, k) for k, v in sorted(checks.items()) if v]
                ),
            )
        )

        # Format expected counts
        checks = self.summary.aggregate("expected", count, include_skip=False)
        intermittent_checks = self.summary.aggregate(
            "known_intermittent", count, include_skip=False
        )
        intermittents = sum(intermittent_checks.values())
        known = (
            " ({} known intermittents)".format(intermittents) if intermittents else ""
        )
        rv.append("Expected results: {}{}".format(sum(checks.values()), known))

        # Format skip counts
        skip_tests = count["test"]["expected"]["skip"]
        skip_subtests = count["subtest"]["expected"]["skip"]
        if skip_tests:
            skipped = "Skipped: {} tests".format(skip_tests)
            if skip_subtests:
                skipped = "{}, {} subtests".format(skipped, skip_subtests)
            rv.append(skipped)

        # Format unexpected counts
        checks = self.summary.aggregate("unexpected", count)
        unexpected_count = sum(checks.values())
        rv.append("Unexpected results: {}".format(unexpected_count))
        if unexpected_count:
            for key in ("test", "subtest", "assert"):
                if not count[key]["unexpected"]:
                    continue
                status_str = ", ".join(
                    [
                        "{} {}".format(n, s)
                        for s, n in sorted(count[key]["unexpected"].items())
                    ]
                )
                rv.append(
                    "  {}: {} ({})".format(
                        key, sum(count[key]["unexpected"].values()), status_str
                    )
                )

        # Format intermittents
        if intermittents > 0:
            heading = "Known Intermittent Results"
            rv.extend(
                [
                    "",
                    self.color_formatter.heading(heading),
                    self.color_formatter.heading("-" * len(heading)),
                ]
            )
            if count["subtest"]["count"]:
                for test_id, results in intermittent_logs.items():
                    test = self._get_file_name(test_id)
                    rv.append(self.color_formatter.bold(test))
                    for data in results:
                        rv.append("  %s" % self._format_status(test, data).rstrip())
            else:
                for test_id, results in intermittent_logs.items():
                    test = self._get_file_name(test_id)
                    for data in results:
                        assert "subtest" not in data
                        rv.append(self._format_status(test, data).rstrip())

        # Format status
        testfailed = any(
            count[key]["unexpected"] for key in ("test", "subtest", "assert")
        )
        if not testfailed and not harness_errors:
            rv.append(self.color_formatter.log_test_status_pass("OK"))
        else:
            # Format test failures
            heading = "Unexpected Results"
            rv.extend(
                [
                    "",
                    self.color_formatter.heading(heading),
                    self.color_formatter.heading("-" * len(heading)),
                ]
            )
            if count["subtest"]["count"]:
                for test_id, results in logs.items():
                    test = self._get_file_name(test_id)
                    rv.append(self.color_formatter.bold(test))
                    for data in results:
                        rv.append("  %s" % self._format_status(test, data).rstrip())
            else:
                for test_id, results in logs.items():
                    test = self._get_file_name(test_id)
                    for data in results:
                        assert "subtest" not in data
                        rv.append(self._format_status(test, data).rstrip())

            # Format harness errors
            if harness_errors:
                for data in harness_errors:
                    rv.append(self.log(data))

        return "\n".join(rv)

    def test_start(self, data):
        action = self.color_formatter.action(data["action"].upper())
        return "%s: %s" % (action, self._get_test_id(data))

    def test_end(self, data):
        subtests = self._get_subtest_data(data)

        if "expected" in data and data["status"] not in data.get(
            "known_intermittent", []
        ):
            parent_unexpected = True
            expected_str = ", expected %s" % data["expected"]
        else:
            parent_unexpected = False
            expected_str = ""

        has_screenshots = "reftest_screenshots" in data.get("extra", {})

        test = self._get_test_id(data)

        # Reset the counts to 0
        self.status_buffer[test] = {"count": 0, "unexpected": 0, "pass": 0}
        self.has_unexpected[test] = bool(subtests["unexpected"])

        if subtests["count"] != 0:
            rv = "Test %s%s. Subtests passed %i/%i. Unexpected %s" % (
                data["status"],
                expected_str,
                subtests["pass"],
                subtests["count"],
                subtests["unexpected"],
            )
        else:
            rv = "%s%s" % (data["status"], expected_str)

        unexpected = self.summary.current["unexpected_logs"].get(data["test"])
        if unexpected:
            if len(unexpected) == 1 and parent_unexpected:
                message = unexpected[0].get("message", "")
                if message:
                    rv += " - %s" % message
                if "stack" in data:
                    rv += self._format_stack(data["stack"])
            elif not self.verbose:
                rv += "\n"
                for d in unexpected:
                    rv += self._format_status(data["test"], d)

        intermittents = self.summary.current["intermittent_logs"].get(data["test"])
        if intermittents:
            rv += "\n"
            for d in intermittents:
                rv += self._format_status(data["test"], d)

        if "expected" not in data and not bool(subtests["unexpected"]):
            color = self.color_formatter.log_test_status_pass
        else:
            color = self.color_formatter.log_test_status_unexpected_fail

        action = color(data["action"].upper())
        rv = "%s: %s" % (action, rv)
        if has_screenshots and self.enable_screenshot:
            if self.tbpl_formatter is None:
                self.tbpl_formatter = TbplFormatter()
            # Create TBPL-like output that can be pasted into the reftest analyser
            rv = "\n".join((rv, self.tbpl_formatter.test_end(data)))
        return rv

    def valgrind_error(self, data):
        rv = " " + data["primary"] + "\n"
        for line in data["secondary"]:
            rv = rv + line + "\n"

        return rv

    def lsan_leak(self, data):
        allowed = data.get("allowed_match")
        if allowed:
            prefix = self.color_formatter.log_test_status_fail("FAIL")
        else:
            prefix = self.color_formatter.log_test_status_unexpected_fail(
                "UNEXPECTED-FAIL"
            )

        return "%s LeakSanitizer: leak at %s" % (prefix, ", ".join(data["frames"]))

    def lsan_summary(self, data):
        allowed = data.get("allowed", False)
        if allowed:
            prefix = self.color_formatter.warning("WARNING")
        else:
            prefix = self.color_formatter.error("ERROR")

        return (
            "%s | LeakSanitizer | "
            "SUMMARY: AddressSanitizer: %d byte(s) leaked in %d allocation(s)."
            % (prefix, data["bytes"], data["allocations"])
        )

    def mozleak_object(self, data):
        data_log = data.copy()
        data_log["level"] = "INFO"
        data_log["message"] = "leakcheck: %s leaked %d %s" % (
            data["process"],
            data["bytes"],
            data["name"],
        )
        return self.log(data_log)

    def mozleak_total(self, data):
        if data["bytes"] is None:
            # We didn't see a line with name 'TOTAL'
            if data.get("induced_crash", False):
                data_log = data.copy()
                data_log["level"] = "INFO"
                data_log["message"] = (
                    "leakcheck: %s deliberate crash and thus no leak log\n"
                    % data["process"]
                )
                return self.log(data_log)
            if data.get("ignore_missing", False):
                return (
                    "%s ignoring missing output line for total leaks\n"
                    % data["process"]
                )

            status = self.color_formatter.log_test_status_pass("FAIL")
            return "%s leakcheck: " "%s missing output line for total leaks!\n" % (
                status,
                data["process"],
            )

        if data["bytes"] == 0:
            return "%s leakcheck: %s no leaks detected!\n" % (
                self.color_formatter.log_test_status_pass("PASS"),
                data["process"],
            )

        message = "leakcheck: %s %d bytes leaked\n" % (data["process"], data["bytes"])

        # data["bytes"] will include any expected leaks, so it can be off
        # by a few thousand bytes.
        failure = data["bytes"] > data["threshold"]
        status = (
            self.color_formatter.log_test_status_unexpected_fail("UNEXPECTED-FAIL")
            if failure
            else self.color_formatter.log_test_status_fail("FAIL")
        )
        return "%s %s\n" % (status, message)

    def test_status(self, data):
        test = self._get_test_id(data)
        if test not in self.status_buffer:
            self.status_buffer[test] = {"count": 0, "unexpected": 0, "pass": 0}
        self.status_buffer[test]["count"] += 1

        if data["status"] == "PASS":
            self.status_buffer[test]["pass"] += 1

        if "expected" in data and data["status"] not in data.get(
            "known_intermittent", []
        ):
            self.status_buffer[test]["unexpected"] += 1

        if self.verbose:
            return self._format_status(test, data).rstrip("\n")

    def assertion_count(self, data):
        if data["min_expected"] <= data["count"] <= data["max_expected"]:
            return

        if data["min_expected"] != data["max_expected"]:
            expected = "%i to %i" % (data["min_expected"], data["max_expected"])
        else:
            expected = "%i" % data["min_expected"]

        action = self.color_formatter.log_test_status_fail("ASSERT")
        return "%s: Assertion count %i, expected %s assertions\n" % (
            action,
            data["count"],
            expected,
        )

    def process_output(self, data):
        rv = []

        pid = data["process"]
        if pid.isdigit():
            pid = "pid:%s" % pid
        pid = self.color_formatter.pid(pid)

        if "command" in data and data["process"] not in self._known_pids:
            self._known_pids.add(data["process"])
            rv.append("%s Full command: %s" % (pid, data["command"]))

        rv.append("%s %s" % (pid, data["data"]))
        return "\n".join(rv)

    def crash(self, data):
        test = self._get_test_id(data)

        if data.get("stackwalk_returncode", 0) != 0 and not data.get(
            "stackwalk_stderr"
        ):
            success = True
        else:
            success = False

        rv = [
            "pid:%s. Process type: %s. Test:%s. Minidump analysed:%s. Signature:[%s]"
            % (
                data.get("pid", "unknown"),
                data.get("process_type", None),
                test,
                success,
                data["signature"],
            )
        ]

        if data.get("java_stack"):
            rv.append("Java exception: %s" % data["java_stack"])
        else:
            if data.get("reason"):
                rv.append("Mozilla crash reason: %s" % data["reason"])

            if data.get("minidump_path"):
                rv.append("Crash dump filename: %s" % data["minidump_path"])

            if data.get("stackwalk_returncode", 0) != 0:
                rv.append(
                    "minidump-stackwalk exited with return code %d"
                    % data["stackwalk_returncode"]
                )

            if data.get("stackwalk_stderr"):
                rv.append("stderr from minidump-stackwalk:")
                rv.append(data["stackwalk_stderr"])
            elif data.get("stackwalk_stdout"):
                rv.append(data["stackwalk_stdout"])

            if data.get("stackwalk_errors"):
                rv.extend(data.get("stackwalk_errors"))

        rv = "\n".join(rv)
        if not rv[-1] == "\n":
            rv += "\n"

        action = self.color_formatter.action(data["action"].upper())
        return "%s: %s" % (action, rv)

    def process_start(self, data):
        rv = "Started process `%s`" % data["process"]
        desc = data.get("command")
        if desc:
            rv = "%s (%s)" % (rv, desc)
        return rv

    def process_exit(self, data):
        return "%s: %s" % (data["process"], strstatus(data["exitcode"]))

    def log(self, data):
        level = data.get("level").upper()

        if level in ("CRITICAL", "ERROR"):
            level = self.color_formatter.error(level)
        elif level == "WARNING":
            level = self.color_formatter.warning(level)
        elif level == "INFO":
            level = self.color_formatter.log_process_output(level)

        if data.get("component"):
            rv = " ".join([data["component"], level, data["message"]])
        else:
            rv = "%s %s" % (level, data["message"])

        if "stack" in data:
            rv += "\n%s" % data["stack"]

        return rv

    def lint(self, data):
        fmt = (
            "{path}  {c1}{lineno}{column}  {c2}{level}{normal}  {message}"
            "  {c1}{rule}({linter}){normal}"
        )
        message = fmt.format(
            path=data["path"],
            normal=self.color_formatter.normal,
            c1=self.color_formatter.grey,
            c2=self.color_formatter.error
            if data["level"] == "error"
            else (self.color_formatter.log_test_status_fail),
            lineno=str(data["lineno"]),
            column=(":" + str(data["column"])) if data.get("column") else "",
            level=data["level"],
            message=data["message"],
            rule="{} ".format(data["rule"]) if data.get("rule") else "",
            linter=data["linter"].lower() if data.get("linter") else "",
        )

        return message

    def shutdown(self, data):
        if not self.summary_on_shutdown:
            return

        heading = "Overall Summary"
        rv = [
            "",
            self.color_formatter.heading(heading),
            self.color_formatter.heading("=" * len(heading)),
        ]
        for suite, summary in self.summary:
            rv.append(self._format_suite_summary(suite, summary))
        return "\n".join(rv)

    def _get_subtest_data(self, data):
        test = self._get_test_id(data)
        return self.status_buffer.get(test, {"count": 0, "unexpected": 0, "pass": 0})

    def _time(self, data):
        entry_time = data["time"]
        if self.write_interval and self.last_time is not None:
            t = entry_time - self.last_time
            self.last_time = entry_time
        else:
            t = entry_time - self.start_time

        return t / 1000.0
