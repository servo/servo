# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

"""Provides interface to deal with pytest.

Usage::

    session = webdriver.client.Session("127.0.0.1", "4444", "/")
    harness_result = ("OK", None)
    subtest_results = pytestrunner.run("/path/to/test", session.url)
    return (harness_result, subtest_results)
"""

import errno
import shutil
import tempfile

from . import fixtures


pytest = None


def do_delayed_imports():
    global pytest
    import pytest


def run(path, session, timeout=0):
    """Run Python test at ``path`` in pytest.  The provided ``session``
    is exposed as a fixture available in the scope of the test functions.

    :param path: Path to the test file.
    :param session: WebDriver session to expose.
    :param timeout: Duration before interrupting potentially hanging
        tests.  If 0, there is no timeout.

    :returns: List of subtest results, which are tuples of (test id,
        status, message, stacktrace).
    """

    if pytest is None:
        do_delayed_imports()

    recorder = SubtestResultRecorder()
    plugins = [recorder,
               fixtures.Session(session)]

    # TODO(ato): Deal with timeouts

    with TemporaryDirectory() as cache:
        pytest.main(["--strict",  # turn warnings into errors
                     "--verbose",  # show each individual subtest
                     "--capture", "no",  # enable stdout/stderr from tests
                     "--basetemp", cache,  # temporary directory
                     path],
                    plugins=plugins)

    return recorder.results


class SubtestResultRecorder(object):
    def __init__(self):
        self.results = []

    def pytest_runtest_logreport(self, report):
        if report.passed and report.when == "call":
            self.record_pass(report)
        elif report.failed:
            if report.when != "call":
                self.record_error(report)
            else:
                self.record_fail(report)
        elif report.skipped:
            self.record_skip(report)

    def record_pass(self, report):
        self.record(report.nodeid, "PASS")

    def record_fail(self, report):
        self.record(report.nodeid, "FAIL", stack=report.longrepr)

    def record_error(self, report):
        # error in setup/teardown
        if report.when != "call":
            message = "%s error" % report.when
        self.record(report.nodeid, "ERROR", message, report.longrepr)

    def record_skip(self, report):
        self.record(report.nodeid, "ERROR",
                    "In-test skip decorators are disallowed, "
                    "please use WPT metadata to ignore tests.")

    def record(self, test, status, message=None, stack=None):
        if stack is not None:
            stack = str(stack)
        new_result = (test, status, message, stack)
        self.results.append(new_result)


class TemporaryDirectory(object):
    def __enter__(self):
        self.path = tempfile.mkdtemp(prefix="pytest-")
        return self.path

    def __exit__(self, *args):
        try:
            shutil.rmtree(self.path)
        except OSError as e:
            # no such file or directory
            if e.errno != errno.ENOENT:
                raise
