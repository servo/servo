# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import json
import sys
import time
import traceback
from multiprocessing import current_process
from threading import Lock, current_thread

from .logtypes import (
    Any,
    Boolean,
    Dict,
    Int,
    List,
    Nullable,
    Status,
    SubStatus,
    TestId,
    TestList,
    Tuple,
    Unicode,
    convertor_registry,
    log_action,
)

"""Structured Logging for recording test results.

Allowed actions, and subfields:
  suite_start
      tests  - List of test names
      name - Name for the suite
      run_info - Dictionary of run properties

  add_subsuite
      name - Name for the subsuite (must be unique)
      run_info - Updates to the suite run_info (optional)

  suite_end

  test_start
      test - ID for the test
      path - Relative path to test (optional)
      subsuite - Name of the subsuite to which test belongs (optional)

  test_end
      test - ID for the test
      status [PASS | FAIL | OK | ERROR | TIMEOUT | CRASH |
              ASSERT PRECONDITION_FAILED | SKIP] - test status
      expected [As for status] - Status that the test was expected to get,
                                 or absent if the test got the expected status
      extra - Dictionary of harness-specific extra information e.g. debug info
      known_intermittent - List of known intermittent statuses that should
                           not fail a test. eg. ['FAIL', 'TIMEOUT']
      subsuite - Name of the subsuite to which test belongs (optional)

  test_status
      test - ID for the test
      subtest - Name of the subtest
      status [PASS | FAIL | TIMEOUT |
              PRECONDITION_FAILED | NOTRUN | SKIP] - test status
      expected [As for status] - Status that the subtest was expected to get,
                                 or absent if the subtest got the expected status
      known_intermittent - List of known intermittent statuses that should
                           not fail a test. eg. ['FAIL', 'TIMEOUT']
      subsuite - Name of the subsuite to which test belongs (optional)

  process_output
      process - PID of the process
      command - Command line of the process
      data - Output data from the process
      test - ID of the test that the process was running (optional)
      subsuite - Name of the subsuite that the process was running (optional)

  assertion_count
      count - Number of assertions produced
      min_expected - Minimum expected number of assertions
      max_expected - Maximum expected number of assertions
      subsuite - Name of the subsuite for the tests that ran (optional)

  lsan_leak
      frames - List of stack frames from the leak report
      scope - An identifier for the set of tests run during the browser session
              (e.g. a directory name)
      allowed_match - A stack frame in the list that matched a rule meaning the
                      leak is expected
      subsuite - Name of the subsuite for the tests that ran (optional)

  lsan_summary
      bytes - Number of bytes leaked
      allocations - Number of allocations
      allowed - Boolean indicating whether all detected leaks matched allow rules
      subsuite - Name of the subsuite for the tests that ran (optional)

  mozleak_object
     process - Process that leaked
     bytes - Number of bytes that leaked
     name - Name of the object that leaked
     scope - An identifier for the set of tests run during the browser session
             (e.g. a directory name)
     allowed - Boolean indicating whether the leak was permitted
     subsuite - Name of the subsuite for the tests that ran (optional)

  log
      level [CRITICAL | ERROR | WARNING |
             INFO | DEBUG] - level of the logging message
      message - Message to log

Subfields for all messages:
      action - the action type of the current message
      time - the timestamp in ms since the epoch of the log message
      thread - name for the thread emitting the message
      pid - id of the python process in which the logger is running
      source - name for the source emitting the message
      component - name of the subcomponent emitting the message
"""

_default_logger_name = None


def get_default_logger(component=None):
    """Gets the default logger if available, optionally tagged with component
    name. Will return None if not yet set

    :param component: The component name to tag log messages with
    """
    global _default_logger_name

    if not _default_logger_name:
        return None

    return StructuredLogger(_default_logger_name, component=component)


def set_default_logger(default_logger):
    """Sets the default logger to logger.

    It can then be retrieved with :py:func:`get_default_logger`

    Note that :py:func:`~mozlog.commandline.setup_logging` will
    set a default logger for you, so there should be no need to call this
    function if you're using setting up logging that way (recommended).

    :param default_logger: The logger to set to default.
    """
    global _default_logger_name

    _default_logger_name = default_logger.name


log_levels = dict(
    (k.upper(), v)
    for v, k in enumerate(["critical", "error", "warning", "info", "debug"])
)

lint_levels = ["ERROR", "WARNING"]


def log_actions():
    """Returns the set of actions implemented by mozlog."""
    return set(convertor_registry.keys())


class LoggerShutdownError(Exception):
    """Raised when attempting to log after logger.shutdown() has been called."""


class LoggerState(object):
    def __init__(self):
        self.reset()

    def reset(self):
        self.handlers = []
        self.subsuites = set()
        self.running_tests = set()
        self.suite_started = False
        self.component_states = {}
        self.has_shutdown = False


class ComponentState(object):
    def __init__(self):
        self.filter_ = None


class StructuredLogger(object):
    _lock = Lock()
    _logger_states = {}
    """Create a structured logger with the given name

    :param name: The name of the logger.
    :param component: A subcomponent that the logger belongs to (typically a library name)
    """

    def __init__(self, name, component=None):
        self.name = name
        self.component = component

        with self._lock:
            if name not in self._logger_states:
                self._logger_states[name] = LoggerState()

            if component not in self._logger_states[name].component_states:
                self._logger_states[name].component_states[component] = ComponentState()

        self._state = self._logger_states[name]
        self._component_state = self._state.component_states[component]

    def add_handler(self, handler):
        """Add a handler to the current logger"""
        self._state.handlers.append(handler)

    def remove_handler(self, handler):
        """Remove a handler from the current logger"""
        self._state.handlers.remove(handler)

    def reset_state(self):
        """Resets the logger to a brand new state. This means all handlers
        are removed, running tests are discarded and components are reset.
        """
        self._state.reset()
        self._component_state = self._state.component_states[
            self.component
        ] = ComponentState()

    def send_message(self, topic, command, *args):
        """Send a message to each handler configured for this logger. This
        part of the api is useful to those users requiring dynamic control
        of a handler's behavior.

        :param topic: The name used by handlers to subscribe to a message.
        :param command: The name of the command to issue.
        :param args: Any arguments known to the target for specialized
                     behavior.
        """
        rv = []
        for handler in self._state.handlers:
            if hasattr(handler, "message_handler"):
                rv += handler.message_handler.handle_message(topic, command, *args)
        return rv

    @property
    def handlers(self):
        """A list of handlers that will be called when a
        message is logged from this logger"""
        return self._state.handlers

    @property
    def component_filter(self):
        return self._component_state.filter_

    @component_filter.setter
    def component_filter(self, value):
        self._component_state.filter_ = value

    def log_raw(self, raw_data):
        if "action" not in raw_data:
            raise ValueError

        action = raw_data["action"]
        converted_data = convertor_registry[action].convert_known(**raw_data)
        for k, v in raw_data.items():
            if (
                k not in converted_data and
                k not in convertor_registry[action].optional_args
            ):
                converted_data[k] = v

        data = self._make_log_data(action, converted_data)

        if action in ("test_status", "test_end"):
            if (
                data["expected"] == data["status"] or
                data["status"] == "SKIP" or
                "expected" not in raw_data
            ):
                del data["expected"]

        if not self._ensure_suite_state(action, data):
            return

        self._handle_log(data)

    def _log_data(self, action, data=None):
        if data is None:
            data = {}

        if data.get("subsuite") and data["subsuite"] not in self._state.subsuites:
            self.error(f"Unrecognised subsuite {data['subsuite']}")
            return

        log_data = self._make_log_data(action, data)
        self._handle_log(log_data)

    def _handle_log(self, data):
        if self._state.has_shutdown:
            raise LoggerShutdownError(
                "{} action received after shutdown.".format(data["action"])
            )

        with self._lock:
            if self.component_filter:
                data = self.component_filter(data)
                if data is None:
                    return

            for handler in self.handlers:
                try:
                    handler(data)
                except Exception:
                    # Write the exception details directly to stderr because
                    # log() would call this method again which is currently locked.
                    print(
                        "%s: Failure calling log handler:" % __name__,
                        file=sys.__stderr__,
                    )
                    print(traceback.format_exc(), file=sys.__stderr__)

    def _make_log_data(self, action, data):
        all_data = {
            "action": action,
            "time": int(time.time() * 1000),
            "thread": current_thread().name,
            "pid": current_process().pid,
            "source": self.name,
        }
        if self.component:
            all_data["component"] = self.component
        all_data.update(data)
        return all_data

    def _ensure_suite_state(self, action, data):
        if action == "suite_start":
            if self._state.suite_started:
                # limit data to reduce unnecessary log bloat
                self.error(
                    "Got second suite_start message before suite_end. " +
                    "Logged with data: {}".format(json.dumps(data)[:100])
                )
                return False
            self._state.suite_started = True
        elif action == "suite_end":
            if not self._state.suite_started:
                self.error(
                    "Got suite_end message before suite_start. " +
                    "Logged with data: {}".format(json.dumps(data))
                )
                return False
            self._state.suite_started = False
        return True

    @log_action(
        TestList("tests"),
        Unicode("name", default=None, optional=True),
        Dict(Any, "run_info", default=None, optional=True),
        Dict(Any, "version_info", default=None, optional=True),
        Dict(Any, "device_info", default=None, optional=True),
        Dict(Any, "extra", default=None, optional=True),
    )
    def suite_start(self, data):
        """Log a suite_start message

        :param dict tests: Test identifiers that will be run in the suite, keyed by group name.
        :param str name: Optional name to identify the suite.
        :param dict run_info: Optional information typically provided by mozinfo.
        :param dict version_info: Optional target application version information provided
          by mozversion.
        :param dict device_info: Optional target device information provided by mozdevice.
        """
        if not self._ensure_suite_state("suite_start", data):
            return

        self._log_data("suite_start", data)

    @log_action(
        Unicode("name"),
        Dict(Any, "run_info", default=None, optional=True),
    )
    def add_subsuite(self, data):
        """Log a add_subsuite message

        :param str name: Name to identify the subsuite.
        :param dict run_info: Optional information about the subsuite. This updates the suite run_info.
        """
        if data["name"] in self._state.subsuites:
            return
        run_info = data.get("run_info", {"subsuite": data["name"]})
        if "subsuite" not in run_info:
            run_info = run_info.copy()
            run_info["subsuite"] = data["name"]
        data["run_info"] = run_info
        self._state.subsuites.add(data["name"])
        self._log_data("add_subsuite", data)

    @log_action(Dict(Any, "extra", default=None, optional=True))
    def suite_end(self, data):
        """Log a suite_end message"""
        if not self._ensure_suite_state("suite_end", data):
            return

        self._state.subsuites.clear()

        self._log_data("suite_end", data)

    @log_action(
        TestId("test"),
        Unicode("path", default=None, optional=True),
        Unicode("subsuite", default=None, optional=True),
    )
    def test_start(self, data):
        """Log a test_start message

        :param test: Identifier of the test that will run.
        :param path: Path to test relative to some base (typically the root of
                     the source tree).
        :param subsuite: Optional name of the subsuite to which the test belongs.
        """
        if not self._state.suite_started:
            self.error(
                "Got test_start message before suite_start for test %s" % data["test"]
            )
            return
        test_key = (data.get("subsuite"), data["test"])
        if test_key in self._state.running_tests:
            self.error("test_start for %s logged while in progress." % data["test"])
            return
        self._state.running_tests.add(test_key)
        self._log_data("test_start", data)

    @log_action(
        TestId("test"),
        Unicode("subtest"),
        SubStatus("status"),
        SubStatus("expected", default="PASS"),
        Unicode("message", default=None, optional=True),
        Unicode("stack", default=None, optional=True),
        Dict(Any, "extra", default=None, optional=True),
        List(SubStatus, "known_intermittent", default=None, optional=True),
        Unicode("subsuite", default=None, optional=True),
    )
    def test_status(self, data):
        """
        Log a test_status message indicating a subtest result. Tests that
        do not have subtests are not expected to produce test_status messages.

        :param test: Identifier of the test that produced the result.
        :param subtest: Name of the subtest.
        :param status: Status string indicating the subtest result
        :param expected: Status string indicating the expected subtest result.
        :param message: Optional string containing a message associated with the result.
        :param stack: Optional stack trace encountered during test execution.
        :param extra: Optional suite-specific data associated with the test result.
        :param known_intermittent: Optional list of string expected intermittent statuses
        :param subsuite: Optional name of the subsuite to which the test belongs.
        """

        if data["expected"] == data["status"] or data["status"] == "SKIP":
            del data["expected"]

        test_key = (data.get("subsuite"), data["test"])
        if test_key not in self._state.running_tests:
            self.error(
                "test_status for %s logged while not in progress. "
                "Logged with data: %s" % (data["test"], json.dumps(data))
            )
            return

        self._log_data("test_status", data)

    @log_action(
        TestId("test"),
        Status("status"),
        Status("expected", default="OK"),
        Unicode("message", default=None, optional=True),
        Unicode("stack", default=None, optional=True),
        Dict(Any, "extra", default=None, optional=True),
        List(Status, "known_intermittent", default=None, optional=True),
        Unicode("subsuite", default=None, optional=True),
    )
    def test_end(self, data):
        """
        Log a test_end message indicating that a test completed. For tests
        with subtests this indicates whether the overall test completed without
        errors. For tests without subtests this indicates the test result
        directly.

        :param test: Identifier of the test that produced the result.
        :param status: Status string indicating the test result
        :param expected: Status string indicating the expected test result.
        :param message: Optonal string containing a message associated with the result.
        :param stack: Optional stack trace encountered during test execution.
        :param extra: Optional suite-specific data associated with the test result.
        :param subsuite: Optional name of the subsuite to which the test belongs.
        """

        if data["expected"] == data["status"] or data["status"] == "SKIP":
            del data["expected"]

        test_key = (data.get("subsuite"), data["test"])
        if test_key not in self._state.running_tests:
            self.error(
                "test_end for %s logged while not in progress. "
                "Logged with data: %s" % (data["test"], json.dumps(data))
            )
        else:
            self._state.running_tests.remove(test_key)
            self._log_data("test_end", data)

    @log_action(
        Unicode("process"),
        Unicode("data"),
        Unicode("command", default=None, optional=True),
        TestId("test", default=None, optional=True),
        Unicode("subsuite", default=None, optional=True),
    )
    def process_output(self, data):
        """Log output from a managed process.

        :param process: A unique identifier for the process producing the output
                        (typically the pid)
        :param data: The output to log
        :param command: Optional string representing the full command line used to start
                        the process.
        :param test: Optional ID of the test which the process was running.
        :param subsuite: Optional name of the subsuite which the process was running.
        """
        self._log_data("process_output", data)

    @log_action(
        Unicode("process", default=None),
        Unicode("signature", default="[Unknown]"),
        TestId("test", default=None, optional=True),
        Unicode("minidump_path", default=None, optional=True),
        Unicode("minidump_extra", default=None, optional=True),
        Int("stackwalk_retcode", default=None, optional=True),
        Unicode("stackwalk_stdout", default=None, optional=True),
        Unicode("stackwalk_stderr", default=None, optional=True),
        Unicode("reason", default=None, optional=True),
        Unicode("java_stack", default=None, optional=True),
        Unicode("process_type", default=None, optional=True),
        List(Unicode, "stackwalk_errors", default=None),
        Unicode("subsuite", default=None, optional=True),
    )
    def crash(self, data):
        if data["stackwalk_errors"] is None:
            data["stackwalk_errors"] = []

        self._log_data("crash", data)

    @log_action(
        Unicode("primary", default=None), List(Unicode, "secondary", default=None)
    )
    def valgrind_error(self, data):
        self._log_data("valgrind_error", data)

    @log_action(
        Unicode("process"),
        Unicode("command", default=None, optional=True),
        Unicode("subsuite", default=None, optional=True),
    )
    def process_start(self, data):
        """Log start event of a process.

        :param process: A unique identifier for the process producing the
                        output (typically the pid)
        :param command: Optional string representing the full command line used to
                        start the process.
        :param subsuite: Optional name of the subsuite using the process.
        """
        self._log_data("process_start", data)

    @log_action(
        Unicode("process"),
        Int("exitcode"),
        Unicode("command", default=None, optional=True),
        Unicode("subsuite", default=None, optional=True),
    )
    def process_exit(self, data):
        """Log exit event of a process.

        :param process: A unique identifier for the process producing the
                        output (typically the pid)
        :param exitcode: the exit code
        :param command: Optional string representing the full command line used to
                        start the process.
        :param subsuite: Optional name of the subsuite using the process.
        """
        self._log_data("process_exit", data)

    @log_action(
        TestId("test"),
        Int("count"),
        Int("min_expected"),
        Int("max_expected"),
        Unicode("subsuite", default=None, optional=True),
    )
    def assertion_count(self, data):
        """Log count of assertions produced when running a test.

        :param count: Number of assertions produced
        :param min_expected: Minimum expected number of assertions
        :param max_expected: Maximum expected number of assertions
        :param subsuite: Optional name of the subsuite for the tests that ran
        """
        self._log_data("assertion_count", data)

    @log_action(
        List(Unicode, "frames"),
        Unicode("scope", optional=True, default=None),
        Unicode("allowed_match", optional=True, default=None),
        Unicode("subsuite", default=None, optional=True),
    )
    def lsan_leak(self, data):
        self._log_data("lsan_leak", data)

    @log_action(
        Int("bytes"),
        Int("allocations"),
        Boolean("allowed", optional=True, default=False),
        Unicode("subsuite", default=None, optional=True),
    )
    def lsan_summary(self, data):
        self._log_data("lsan_summary", data)

    @log_action(
        Unicode("process"),
        Int("bytes"),
        Unicode("name"),
        Unicode("scope", optional=True, default=None),
        Boolean("allowed", optional=True, default=False),
        Unicode("subsuite", default=None, optional=True),
    )
    def mozleak_object(self, data):
        self._log_data("mozleak_object", data)

    @log_action(
        Unicode("process"),
        Nullable(Int, "bytes"),
        Int("threshold"),
        List(Unicode, "objects"),
        Unicode("scope", optional=True, default=None),
        Boolean("induced_crash", optional=True, default=False),
        Boolean("ignore_missing", optional=True, default=False),
        Unicode("subsuite", default=None, optional=True),
    )
    def mozleak_total(self, data):
        self._log_data("mozleak_total", data)

    @log_action()
    def shutdown(self, data):
        """Shutdown the logger.

        This logs a 'shutdown' action after which any further attempts to use
        the logger will raise a :exc:`LoggerShutdownError`.

        This is also called implicitly from the destructor or
        when exiting the context manager.
        """
        self._log_data("shutdown", data)
        self._state.has_shutdown = True

    def __enter__(self):
        return self

    def __exit__(self, exc, val, tb):
        self.shutdown()


def _log_func(level_name):
    @log_action(Unicode("message"), Any("exc_info", default=False))
    def log(self, data):
        exc_info = data.pop("exc_info", None)
        if exc_info:
            if not isinstance(exc_info, tuple):
                exc_info = sys.exc_info()
            if exc_info != (None, None, None):
                bt = traceback.format_exception(*exc_info)
                data["stack"] = "\n".join(bt)

        data["level"] = level_name
        self._log_data("log", data)

    log.__doc__ = (
        """Log a message with level %s

:param message: The string message to log
:param exc_info: Either a boolean indicating whether to include a traceback
                 derived from sys.exc_info() or a three-item tuple in the
                 same format as sys.exc_info() containing exception information
                 to log.
"""
        % level_name
    )
    log.__name__ = str(level_name).lower()
    return log


def _lint_func(level_name):
    @log_action(
        Unicode("path"),
        Unicode("message", default=""),
        Int("lineno", default=0),
        Int("column", default=None, optional=True),
        Unicode("hint", default=None, optional=True),
        Unicode("source", default=None, optional=True),
        Unicode("rule", default=None, optional=True),
        Tuple((Int, Int), "lineoffset", default=None, optional=True),
        Unicode("linter", default=None, optional=True),
    )
    def lint(self, data):
        data["level"] = level_name
        self._log_data("lint", data)

    lint.__doc__ = """Log an error resulting from a failed lint check

        :param linter: name of the linter that flagged this error
        :param path: path to the file containing the error
        :param message: text describing the error
        :param lineno: line number that contains the error
        :param column: column containing the error
        :param hint: suggestion for fixing the error (optional)
        :param source: source code context of the error (optional)
        :param rule: name of the rule that was violated (optional)
        :param lineoffset: denotes an error spans multiple lines, of the form
                           (<lineno offset>, <num lines>) (optional)
        """
    lint.__name__ = str("lint_%s" % level_name)
    return lint


# Create all the methods on StructuredLog for log/lint levels
for level_name in log_levels:
    setattr(StructuredLogger, level_name.lower(), _log_func(level_name))

for level_name in lint_levels:
    level_name = level_name.lower()
    name = "lint_%s" % level_name
    setattr(StructuredLogger, name, _lint_func(level_name))


class StructuredLogFileLike(object):
    """Wrapper for file-like objects to redirect writes to logger
    instead. Each call to `write` becomes a single log entry of type `log`.

    When using this it is important that the callees i.e. the logging
    handlers do not themselves try to write to the wrapped file as this
    will cause infinite recursion.

    :param logger: `StructuredLogger` to which to redirect the file write operations.
    :param level: log level to use for each write.
    :param prefix: String prefix to prepend to each log entry.
    """

    def __init__(self, logger, level="info", prefix=None):
        self.logger = logger
        self.log_func = getattr(self.logger, level)
        self.prefix = prefix

    def write(self, data):
        if data.endswith("\n"):
            data = data[:-1]
        if data.endswith("\r"):
            data = data[:-1]
        if self.prefix is not None:
            data = "%s: %s" % (self.prefix, data)
        self.log_func(data)

    def flush(self):
        pass
