# mypy: allow-untyped-defs

"""
Provides interface to deal with pytest.

Usage::

    session = webdriver.client.Session("127.0.0.1", "4444", "/")
    harness_result = ("OK", None)
    subtest_results = pytestrunner.run("/path/to/test", session.url)
    return (harness_result, subtest_results)
"""

import errno
import json
import os
import shutil
import tempfile
from collections import OrderedDict


pytest = None


def do_delayed_imports():
    global pytest
    import pytest


def run(path, server_config, session_config, timeout=0, environ=None):
    """
    Run Python test at ``path`` in pytest.  The provided ``session``
    is exposed as a fixture available in the scope of the test functions.

    :param path: Path to the test file.
    :param session_config: dictionary of host, port,capabilities parameters
    to pass through to the webdriver session
    :param timeout: Duration before interrupting potentially hanging
        tests.  If 0, there is no timeout.

    :returns: (<harness result>, [<subtest result>, ...]),
        where <subtest result> is (test id, status, message, stacktrace).
    """
    if pytest is None:
        do_delayed_imports()

    old_environ = os.environ.copy()
    try:
        with TemporaryDirectory() as cache:
            config_path = os.path.join(cache, "wd_config.json")
            os.environ["WDSPEC_CONFIG_FILE"] = config_path

            config = session_config.copy()
            config["wptserve"] = server_config.as_dict()

            with open(config_path, "w") as f:
                json.dump(config, f)

            if environ:
                os.environ.update(environ)

            harness = HarnessResultRecorder()
            subtests = SubtestResultRecorder()

            try:
                basetemp = os.path.join(cache, "pytest")
                pytest.main(["--strict-markers",  # turn function marker warnings into errors
                             "-vv",  # show each individual subtest and full failure logs
                             "--capture", "no",  # enable stdout/stderr from tests
                             "--basetemp", basetemp,  # temporary directory
                             "--showlocals",  # display contents of variables in local scope
                             "-p", "no:mozlog",  # use the WPT result recorder
                             "-p", "no:cacheprovider",  # disable state preservation across invocations
                             "-o=console_output_style=classic",  # disable test progress bar
                             path],
                            plugins=[harness, subtests])
            except Exception as e:
                harness.outcome = ("INTERNAL-ERROR", str(e))

    finally:
        os.environ = old_environ

    subtests_results = [(key,) + value for (key, value) in subtests.results.items()]
    return (harness.outcome, subtests_results)


class HarnessResultRecorder:
    outcomes = {
        "failed": "ERROR",
        "passed": "OK",
        "skipped": "SKIP",
    }

    def __init__(self):
        # we are ok unless told otherwise
        self.outcome = ("OK", None)

    def pytest_collectreport(self, report):
        harness_result = self.outcomes[report.outcome]
        self.outcome = (harness_result, None)


class SubtestResultRecorder:
    def __init__(self):
        self.results = OrderedDict()

    def pytest_runtest_logreport(self, report):
        if report.passed and report.when == "call":
            self.record_pass(report)
        elif report.failed:
            # pytest outputs the stacktrace followed by an error message prefixed
            # with "E   ", e.g.
            #
            #        def test_example():
            #  >         assert "fuu" in "foobar"
            #  > E       AssertionError: assert 'fuu' in 'foobar'
            message = ""
            for line in report.longreprtext.splitlines():
                if line.startswith("E   "):
                    message = line[1:].strip()
                    break

            if report.when != "call":
                self.record_error(report, message)
            else:
                self.record_fail(report, message)
        elif report.skipped:
            self.record_skip(report)

    def record_pass(self, report):
        self.record(report.nodeid, "PASS")

    def record_fail(self, report, message):
        self.record(report.nodeid, "FAIL", message=message, stack=report.longrepr)

    def record_error(self, report, message):
        # error in setup/teardown
        message = f"{report.when} error: {message}"
        self.record(report.nodeid, "ERROR", message, report.longrepr)

    def record_skip(self, report):
        self.record(report.nodeid, "ERROR",
                    "In-test skip decorators are disallowed, "
                    "please use WPT metadata to ignore tests.")

    def record(self, test, status, message=None, stack=None):
        if stack is not None:
            stack = str(stack)
        # Ensure we get a single result per subtest; pytest will sometimes
        # call pytest_runtest_logreport more than once per test e.g. if
        # it fails and then there's an error during teardown.
        subtest_id = test.split("::")[-1]
        if subtest_id in self.results and status == "PASS":
            # This shouldn't happen, but never overwrite an existing result with PASS
            return
        new_result = (status, message, stack)
        self.results[subtest_id] = new_result


class TemporaryDirectory:
    def __enter__(self):
        self.path = tempfile.mkdtemp(prefix="wdspec-")
        return self.path

    def __exit__(self, *args):
        try:
            shutil.rmtree(self.path)
        except OSError as e:
            # no such file or directory
            if e.errno != errno.ENOENT:
                raise
