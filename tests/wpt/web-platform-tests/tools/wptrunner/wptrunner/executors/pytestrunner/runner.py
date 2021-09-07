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
            os.environ["WD_HOST"] = session_config["host"]
            os.environ["WD_PORT"] = str(session_config["port"])
            os.environ["WD_CAPABILITIES"] = json.dumps(session_config["capabilities"])

            config_path = os.path.join(cache, "wd_server_config.json")
            os.environ["WD_SERVER_CONFIG_FILE"] = config_path
            with open(config_path, "w") as f:
                json.dump(server_config.as_dict(), f)

            if environ:
                os.environ.update(environ)

            harness = HarnessResultRecorder()
            subtests = SubtestResultRecorder()

            try:
                pytest.main(["--strict",  # turn warnings into errors
                             "-vv",  # show each individual subtest and full failure logs
                             "--capture", "no",  # enable stdout/stderr from tests
                             "--basetemp", cache,  # temporary directory
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

    return (harness.outcome, subtests.results)


class HarnessResultRecorder(object):
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


class SubtestResultRecorder(object):
    def __init__(self):
        self.results = []

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
        message = "{} error: {}".format(report.when, message)
        self.record(report.nodeid, "ERROR", message, report.longrepr)

    def record_skip(self, report):
        self.record(report.nodeid, "ERROR",
                    "In-test skip decorators are disallowed, "
                    "please use WPT metadata to ignore tests.")

    def record(self, test, status, message=None, stack=None):
        if stack is not None:
            stack = str(stack)
        new_result = (test.split("::")[-1], status, message, stack)
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
