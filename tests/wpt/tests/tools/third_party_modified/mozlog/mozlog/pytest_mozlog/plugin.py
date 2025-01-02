# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import time

import pytest

import mozlog


def pytest_addoption(parser):
    # We can't simply use mozlog.commandline.add_logging_group(parser) here because
    # Pytest's parser doesn't have the add_argument_group method Mozlog expects.
    group = parser.getgroup("mozlog")

    for name, (_class, _help) in mozlog.commandline.log_formatters.items():
        group.addoption("--log-{0}".format(name), action="append", help=_help)

    formatter_options = mozlog.commandline.fmt_options.items()
    for name, (_class, _help, formatters, action) in formatter_options:
        for formatter in formatters:
            if formatter in mozlog.commandline.log_formatters:
                group.addoption(
                    "--log-{0}-{1}".format(formatter, name), action=action, help=_help
                )


def pytest_configure(config):
    # If using pytest-xdist for parallelization, only register plugin on master process
    if not hasattr(config, "slaveinput"):
        config.pluginmanager.register(MozLog())


class MozLog(object):
    def __init__(self):
        self._started = False
        self.results = {}
        self.start_time = int(time.time() * 1000)  # in ms for Mozlog compatibility

    def _log_suite_start(self, tests):
        if not self._started:
            # As this is called for each node when using pytest-xdist, we want
            # to avoid logging multiple suite_start messages.
            self.logger.suite_start(
                tests=tests, time=self.start_time, run_info=self.run_info
            )
            self._started = True

    def pytest_configure(self, config):
        mozlog.commandline.setup_logging(
            "pytest",
            config.known_args_namespace,
            defaults={},
            allow_unused_options=True,
        )
        self.logger = mozlog.get_default_logger(component="pytest")

    def pytest_sessionstart(self, session):
        """Called before test collection; records suite start time to log later"""
        self.start_time = int(time.time() * 1000)  # in ms for Mozlog compatibility
        self.run_info = getattr(session.config, "_metadata", None)

    def pytest_collection_finish(self, session):
        """Called after test collection is completed, just before tests are run (suite start)"""
        self._log_suite_start([item.nodeid for item in session.items])

    @pytest.mark.optionalhook
    def pytest_xdist_node_collection_finished(self, node, ids):
        """Called after each pytest-xdist node collection is completed"""
        self._log_suite_start(ids)

    def pytest_sessionfinish(self, session, exitstatus):
        self.logger.suite_end()

    def pytest_runtest_logstart(self, nodeid, location):
        self.logger.test_start(test=nodeid)

    def pytest_runtest_logreport(self, report):
        """Called 3 times per test (setup, call, teardown), indicated by report.when"""
        test = report.nodeid
        status = expected = "PASS"
        message = stack = None
        if hasattr(report, "wasxfail"):
            expected = "FAIL"
        if report.failed or report.outcome == "rerun":
            status = "FAIL" if report.when == "call" else "ERROR"
        if report.skipped:
            status = "SKIP" if not hasattr(report, "wasxfail") else "FAIL"
        if report.longrepr is not None:
            longrepr = report.longrepr
            if isinstance(longrepr, str):
                # When using pytest-xdist, longrepr is serialised as a str
                message = stack = longrepr
                if longrepr.startswith("[XPASS(strict)]"):
                    # Strict expected failures have an outcome of failed when
                    # they unexpectedly pass.
                    expected, status = ("FAIL", "PASS")
            elif hasattr(longrepr, "reprcrash"):
                # For failures, longrepr is a ReprExceptionInfo
                crash = longrepr.reprcrash
                message = "{0} (line {1})".format(crash.message, crash.lineno)
                stack = longrepr.reprtraceback
            elif hasattr(longrepr, "errorstring"):
                message = longrepr.errorstring
                stack = longrepr.errorstring
            elif hasattr(longrepr, "__getitem__") and len(longrepr) == 3:
                # For skips, longrepr is a tuple of (file, lineno, reason)
                message = report.longrepr[-1]
            else:
                raise ValueError(
                    "Unable to convert longrepr to message:\ntype %s\nfields: %s"
                    % (longrepr.__class__, dir(longrepr))
                )
        if status != expected or expected != "PASS":
            self.results[test] = (status, expected, message, stack)
        if report.outcome == "rerun" or report.when == "teardown":
            defaults = ("PASS", "PASS", None, None)
            status, expected, message, stack = self.results.get(test, defaults)
            self.logger.test_end(
                test=test,
                status=status,
                expected=expected,
                message=message,
                stack=stack,
            )
