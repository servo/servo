# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import time
from collections import defaultdict

try:
    import blessings
except ImportError:
    blessings = None

import base

def format_seconds(total):
    """Format number of seconds to MM:SS.DD form."""
    minutes, seconds = divmod(total, 60)
    return '%2d:%05.2f' % (minutes, seconds)

class NullTerminal(object):
    def __getattr__(self, name):
        return self._id

    def _id(self, value):
        return value

class MachFormatter(base.BaseFormatter):
    def __init__(self, start_time=None, write_interval=False, write_times=True,
                 terminal=None, disable_colors=False):

        if disable_colors:
            terminal = None
        elif terminal is None and blessings is not None:
            terminal = blessings.Terminal()

        if start_time is None:
            start_time = time.time()
        start_time = int(start_time * 1000)
        self.start_time = start_time
        self.write_interval = write_interval
        self.write_times = write_times
        self.status_buffer = {}
        self.has_unexpected = {}
        self.last_time = None
        self.terminal = terminal
        self.verbose = False
        self._known_pids = set()

        self.summary_values = {"tests": 0,
                               "subtests": 0,
                               "expected": 0,
                               "unexpected": defaultdict(int),
                               "skipped": 0}
        self.summary_unexpected = []

    def __call__(self, data):
        s = base.BaseFormatter.__call__(self, data)
        if s is None:
            return

        time = format_seconds(self._time(data))
        action = data["action"].upper()
        thread = data["thread"]

        # Not using the NullTerminal here is a small optimisation to cut the number of
        # function calls
        if self.terminal is not None:
            test = self._get_test_id(data)

            time = self.terminal.blue(time)

            color = None

            if data["action"] == "test_end":
                if "expected" not in data and not self.has_unexpected[test]:
                    color = self.terminal.green
                else:
                    color = self.terminal.red
            elif data["action"] in ("suite_start", "suite_end",
                                    "test_start", "test_status"):
                color = self.terminal.yellow
            elif data["action"] == "crash":
                color = self.terminal.red

            if color is not None:
                action = color(action)

        return "%s %s: %s %s\n" % (time, action, thread, s)

    def _get_test_id(self, data):
        test_id = data.get("test")
        if isinstance(test_id, list):
            test_id = tuple(test_id)
        return test_id

    def _get_file_name(self, test_id):
        if isinstance(test_id, (str, unicode)):
            return test_id

        if isinstance(test_id, tuple):
            return "".join(test_id)

        assert False, "unexpected test_id"

    def suite_start(self, data):
        self.summary_values = {"tests": 0,
                               "subtests": 0,
                               "expected": 0,
                               "unexpected": defaultdict(int),
                               "skipped": 0}
        self.summary_unexpected = []
        return "%i" % len(data["tests"])

    def suite_end(self, data):
        term = self.terminal if self.terminal is not None else NullTerminal()

        heading = "Summary"
        rv = ["", heading, "=" * len(heading), ""]

        has_subtests = self.summary_values["subtests"] > 0

        if has_subtests:
            rv.append("Ran %i tests (%i parents, %i subtests)" %
                      (self.summary_values["tests"] + self.summary_values["subtests"],
                       self.summary_values["tests"],
                       self.summary_values["subtests"]))
        else:
            rv.append("Ran %i tests" % self.summary_values["tests"])

        rv.append("Expected results: %i" % self.summary_values["expected"])

        unexpected_count = sum(self.summary_values["unexpected"].values())
        if unexpected_count > 0:
            unexpected_str = " (%s)" % ", ".join("%s: %i" % (key, value) for key, value in
                                              sorted(self.summary_values["unexpected"].items()))
        else:
            unexpected_str = ""

        rv.append("Unexpected results: %i%s" % (unexpected_count, unexpected_str))

        if self.summary_values["skipped"] > 0:
            rv.append("Skipped: %i" % self.summary_values["skipped"])
        rv.append("")

        if not self.summary_values["unexpected"]:
            rv.append(term.green("OK"))
        else:
            heading = "Unexpected Results"
            rv.extend([heading, "=" * len(heading), ""])
            if has_subtests:
                for test_id, results in self.summary_unexpected:
                    test = self._get_file_name(test_id)
                    rv.extend([test, "-" * len(test)])
                    for name, status, expected, message in results:
                        if name is None:
                            name = "[Parent]"
                        rv.append("%s %s" % (self.format_expected(status, expected), name))
            else:
                for test_id, results in self.summary_unexpected:
                    test = self._get_file_name(test_id)
                    assert len(results) == 1
                    name, status, expected, messge = results[0]
                    assert name is None
                    rv.append("%s %s" % (self.format_expected(status, expected), test))

        return "\n".join(rv)

    def format_expected(self, status, expected):
        term = self.terminal if self.terminal is not None else NullTerminal()
        if status == "ERROR":
            color = term.red
        else:
            color = term.yellow

        if expected in ("PASS", "OK"):
            return color(status)

        return color("%s expected %s" % (status, expected))

    def test_start(self, data):
        self.summary_values["tests"] += 1
        return "%s" % (self._get_test_id(data),)

    def test_end(self, data):
        subtests = self._get_subtest_data(data)
        unexpected = subtests["unexpected"]

        message = data.get("message", "")
        if "stack" in data:
            stack = data["stack"]
            if stack and stack[-1] != "\n":
                stack += "\n"
            message = stack + message

        if "expected" in data:
            parent_unexpected = True
            expected_str = ", expected %s" % data["expected"]
            unexpected.append((None, data["status"], data["expected"],
                               message))
        else:
            parent_unexpected = False
            expected_str = ""

        test = self._get_test_id(data)

        if unexpected:
            self.summary_unexpected.append((test, unexpected))
        self._update_summary(data)

        #Reset the counts to 0
        self.status_buffer[test] = {"count": 0, "unexpected": [], "pass": 0}
        self.has_unexpected[test] = bool(unexpected)

        if subtests["count"] != 0:
            rv = "Harness %s%s. Subtests passed %i/%i. Unexpected %s" % (
                data["status"], expected_str, subtests["pass"], subtests["count"],
                len(unexpected))
        else:
            rv = "%s%s" % (data["status"], expected_str)

        if unexpected:
            rv += "\n"
            if len(unexpected) == 1 and parent_unexpected:
                rv += "%s" % unexpected[0][-1]
            else:
                for name, status, expected, message in unexpected:
                    if name is None:
                        name = "[Parent]"
                    expected_str = "Expected %s, got %s" % (expected, status)
                    rv += "%s\n" % ("\n".join([name, "-" * len(name), expected_str, message]))
                rv = rv[:-1]
        return rv

    def test_status(self, data):
        self.summary_values["subtests"] += 1

        test = self._get_test_id(data)
        if test not in self.status_buffer:
            self.status_buffer[test] = {"count": 0, "unexpected": [], "pass": 0}
        self.status_buffer[test]["count"] += 1

        message = data.get("message", "")
        if "stack" in data:
            if message:
                message += "\n"
            message += data["stack"]

        if data["status"] == "PASS":
            self.status_buffer[test]["pass"] += 1

        self._update_summary(data)

        rv = None
        status, subtest = data["status"], data["subtest"]
        unexpected = "expected" in data
        if self.verbose:
            if self.terminal is not None:
                status = (self.terminal.red if unexpected else self.terminal.green)(status)
            rv = " ".join([subtest, status, message])
        elif unexpected:
            # We only append an unexpected summary if it was not logged
            # directly by verbose mode.
            self.status_buffer[test]["unexpected"].append((subtest,
                                                           status,
                                                           data["expected"],
                                                           message))
        return rv

    def _update_summary(self, data):
        if "expected" in data:
            self.summary_values["unexpected"][data["status"]] += 1
        elif data["status"] == "SKIP":
            self.summary_values["skipped"] += 1
        else:
            self.summary_values["expected"] += 1

    def process_output(self, data):
        rv = []

        if "command" in data and data["process"] not in self._known_pids:
            self._known_pids.add(data["process"])
            rv.append('(pid:%s) Full command: %s' % (data["process"], data["command"]))

        rv.append('(pid:%s) "%s"' % (data["process"], data["data"]))
        return "\n".join(rv)

    def crash(self, data):
        test = self._get_test_id(data)

        if data.get("stackwalk_returncode", 0) != 0 and not data.get("stackwalk_stderr"):
            success = True
        else:
            success = False

        rv = ["pid:%s. Test:%s. Minidump anaylsed:%s. Signature:[%s]" %
              (data.get("pid", None), test, success, data["signature"])]

        if data.get("minidump_path"):
            rv.append("Crash dump filename: %s" % data["minidump_path"])

        if data.get("stackwalk_returncode", 0) != 0:
            rv.append("minidump_stackwalk exited with return code %d" %
                      data["stackwalk_returncode"])

        if data.get("stackwalk_stderr"):
            rv.append("stderr from minidump_stackwalk:")
            rv.append(data["stackwalk_stderr"])
        elif data.get("stackwalk_stdout"):
            rv.append(data["stackwalk_stdout"])

        if data.get("stackwalk_errors"):
            rv.extend(data.get("stackwalk_errors"))

        rv = "\n".join(rv)
        if not rv[-1] == "\n":
            rv += "\n"

        return rv



    def log(self, data):
        level = data.get("level").upper()

        if self.terminal is not None:
            if level in ("CRITICAL", "ERROR"):
                level = self.terminal.red(level)
            elif level == "WARNING":
                level = self.terminal.yellow(level)
            elif level == "INFO":
                level = self.terminal.blue(level)

        if data.get('component'):
            rv = " ".join([data["component"], level, data["message"]])
        else:
            rv = "%s %s" % (level, data["message"])

        if "stack" in data:
            rv += "\n%s" % data["stack"]

        return rv

    def _get_subtest_data(self, data):
        test = self._get_test_id(data)
        return self.status_buffer.get(test, {"count": 0, "unexpected": [], "pass": 0})

    def _time(self, data):
        entry_time = data["time"]
        if self.write_interval and self.last_time is not None:
            t = entry_time - self.last_time
            self.last_time = entry_time
        else:
            t = entry_time - self.start_time

        return t / 1000.

