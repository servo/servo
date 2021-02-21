import io
import os
import re
from typing import cast

import pytest
from _pytest.capture import CaptureManager
from _pytest.config import ExitCode
from _pytest.pytester import Testdir
from _pytest.terminal import TerminalReporter


def test_nothing_logged(testdir):
    testdir.makepyfile(
        """
        import sys

        def test_foo():
            sys.stdout.write('text going to stdout')
            sys.stderr.write('text going to stderr')
            assert False
        """
    )
    result = testdir.runpytest()
    assert result.ret == 1
    result.stdout.fnmatch_lines(["*- Captured stdout call -*", "text going to stdout"])
    result.stdout.fnmatch_lines(["*- Captured stderr call -*", "text going to stderr"])
    with pytest.raises(pytest.fail.Exception):
        result.stdout.fnmatch_lines(["*- Captured *log call -*"])


def test_messages_logged(testdir):
    testdir.makepyfile(
        """
        import sys
        import logging

        logger = logging.getLogger(__name__)

        def test_foo():
            sys.stdout.write('text going to stdout')
            sys.stderr.write('text going to stderr')
            logger.info('text going to logger')
            assert False
        """
    )
    result = testdir.runpytest("--log-level=INFO")
    assert result.ret == 1
    result.stdout.fnmatch_lines(["*- Captured *log call -*", "*text going to logger*"])
    result.stdout.fnmatch_lines(["*- Captured stdout call -*", "text going to stdout"])
    result.stdout.fnmatch_lines(["*- Captured stderr call -*", "text going to stderr"])


def test_root_logger_affected(testdir):
    testdir.makepyfile(
        """
        import logging
        logger = logging.getLogger()

        def test_foo():
            logger.info('info text ' + 'going to logger')
            logger.warning('warning text ' + 'going to logger')
            logger.error('error text ' + 'going to logger')

            assert 0
    """
    )
    log_file = testdir.tmpdir.join("pytest.log").strpath
    result = testdir.runpytest("--log-level=ERROR", "--log-file=pytest.log")
    assert result.ret == 1

    # The capture log calls in the stdout section only contain the
    # logger.error msg, because of --log-level=ERROR.
    result.stdout.fnmatch_lines(["*error text going to logger*"])
    stdout = result.stdout.str()
    assert "warning text going to logger" not in stdout
    assert "info text going to logger" not in stdout

    # The log file should contain the warning and the error log messages and
    # not the info one, because the default level of the root logger is
    # WARNING.
    assert os.path.isfile(log_file)
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "info text going to logger" not in contents
        assert "warning text going to logger" in contents
        assert "error text going to logger" in contents


def test_log_cli_level_log_level_interaction(testdir):
    testdir.makepyfile(
        """
        import logging
        logger = logging.getLogger()

        def test_foo():
            logger.debug('debug text ' + 'going to logger')
            logger.info('info text ' + 'going to logger')
            logger.warning('warning text ' + 'going to logger')
            logger.error('error text ' + 'going to logger')
            assert 0
    """
    )

    result = testdir.runpytest("--log-cli-level=INFO", "--log-level=ERROR")
    assert result.ret == 1

    result.stdout.fnmatch_lines(
        [
            "*-- live log call --*",
            "*INFO*info text going to logger",
            "*WARNING*warning text going to logger",
            "*ERROR*error text going to logger",
            "=* 1 failed in *=",
        ]
    )
    result.stdout.no_re_match_line("DEBUG")


def test_setup_logging(testdir):
    testdir.makepyfile(
        """
        import logging

        logger = logging.getLogger(__name__)

        def setup_function(function):
            logger.info('text going to logger from setup')

        def test_foo():
            logger.info('text going to logger from call')
            assert False
    """
    )
    result = testdir.runpytest("--log-level=INFO")
    assert result.ret == 1
    result.stdout.fnmatch_lines(
        [
            "*- Captured *log setup -*",
            "*text going to logger from setup*",
            "*- Captured *log call -*",
            "*text going to logger from call*",
        ]
    )


def test_teardown_logging(testdir):
    testdir.makepyfile(
        """
        import logging

        logger = logging.getLogger(__name__)

        def test_foo():
            logger.info('text going to logger from call')

        def teardown_function(function):
            logger.info('text going to logger from teardown')
            assert False
        """
    )
    result = testdir.runpytest("--log-level=INFO")
    assert result.ret == 1
    result.stdout.fnmatch_lines(
        [
            "*- Captured *log call -*",
            "*text going to logger from call*",
            "*- Captured *log teardown -*",
            "*text going to logger from teardown*",
        ]
    )


@pytest.mark.parametrize("enabled", [True, False])
def test_log_cli_enabled_disabled(testdir, enabled):
    msg = "critical message logged by test"
    testdir.makepyfile(
        """
        import logging
        def test_log_cli():
            logging.critical("{}")
    """.format(
            msg
        )
    )
    if enabled:
        testdir.makeini(
            """
            [pytest]
            log_cli=true
        """
        )
    result = testdir.runpytest()
    if enabled:
        result.stdout.fnmatch_lines(
            [
                "test_log_cli_enabled_disabled.py::test_log_cli ",
                "*-- live log call --*",
                "CRITICAL *test_log_cli_enabled_disabled.py* critical message logged by test",
                "PASSED*",
            ]
        )
    else:
        assert msg not in result.stdout.str()


def test_log_cli_default_level(testdir):
    # Default log file level
    testdir.makepyfile(
        """
        import pytest
        import logging
        def test_log_cli(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_cli_handler.level == logging.NOTSET
            logging.getLogger('catchlog').info("INFO message won't be shown")
            logging.getLogger('catchlog').warning("WARNING message will be shown")
    """
    )
    testdir.makeini(
        """
        [pytest]
        log_cli=true
    """
    )

    result = testdir.runpytest()

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines(
        [
            "test_log_cli_default_level.py::test_log_cli ",
            "WARNING*test_log_cli_default_level.py* message will be shown*",
        ]
    )
    result.stdout.no_fnmatch_line("*INFO message won't be shown*")
    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0


def test_log_cli_default_level_multiple_tests(testdir, request):
    """Ensure we reset the first newline added by the live logger between tests"""
    filename = request.node.name + ".py"
    testdir.makepyfile(
        """
        import logging

        def test_log_1():
            logging.warning("log message from test_log_1")

        def test_log_2():
            logging.warning("log message from test_log_2")
    """
    )
    testdir.makeini(
        """
        [pytest]
        log_cli=true
    """
    )

    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "{}::test_log_1 ".format(filename),
            "*WARNING*log message from test_log_1*",
            "PASSED *50%*",
            "{}::test_log_2 ".format(filename),
            "*WARNING*log message from test_log_2*",
            "PASSED *100%*",
            "=* 2 passed in *=",
        ]
    )


def test_log_cli_default_level_sections(testdir, request):
    """Check that with live logging enable we are printing the correct headers during
    start/setup/call/teardown/finish."""
    filename = request.node.name + ".py"
    testdir.makeconftest(
        """
        import pytest
        import logging

        def pytest_runtest_logstart():
            logging.warning('>>>>> START >>>>>')

        def pytest_runtest_logfinish():
            logging.warning('<<<<< END <<<<<<<')
    """
    )

    testdir.makepyfile(
        """
        import pytest
        import logging

        @pytest.fixture
        def fix(request):
            logging.warning("log message from setup of {}".format(request.node.name))
            yield
            logging.warning("log message from teardown of {}".format(request.node.name))

        def test_log_1(fix):
            logging.warning("log message from test_log_1")

        def test_log_2(fix):
            logging.warning("log message from test_log_2")
    """
    )
    testdir.makeini(
        """
        [pytest]
        log_cli=true
    """
    )

    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "{}::test_log_1 ".format(filename),
            "*-- live log start --*",
            "*WARNING* >>>>> START >>>>>*",
            "*-- live log setup --*",
            "*WARNING*log message from setup of test_log_1*",
            "*-- live log call --*",
            "*WARNING*log message from test_log_1*",
            "PASSED *50%*",
            "*-- live log teardown --*",
            "*WARNING*log message from teardown of test_log_1*",
            "*-- live log finish --*",
            "*WARNING* <<<<< END <<<<<<<*",
            "{}::test_log_2 ".format(filename),
            "*-- live log start --*",
            "*WARNING* >>>>> START >>>>>*",
            "*-- live log setup --*",
            "*WARNING*log message from setup of test_log_2*",
            "*-- live log call --*",
            "*WARNING*log message from test_log_2*",
            "PASSED *100%*",
            "*-- live log teardown --*",
            "*WARNING*log message from teardown of test_log_2*",
            "*-- live log finish --*",
            "*WARNING* <<<<< END <<<<<<<*",
            "=* 2 passed in *=",
        ]
    )


def test_live_logs_unknown_sections(testdir, request):
    """Check that with live logging enable we are printing the correct headers during
    start/setup/call/teardown/finish."""
    filename = request.node.name + ".py"
    testdir.makeconftest(
        """
        import pytest
        import logging

        def pytest_runtest_protocol(item, nextitem):
            logging.warning('Unknown Section!')

        def pytest_runtest_logstart():
            logging.warning('>>>>> START >>>>>')

        def pytest_runtest_logfinish():
            logging.warning('<<<<< END <<<<<<<')
    """
    )

    testdir.makepyfile(
        """
        import pytest
        import logging

        @pytest.fixture
        def fix(request):
            logging.warning("log message from setup of {}".format(request.node.name))
            yield
            logging.warning("log message from teardown of {}".format(request.node.name))

        def test_log_1(fix):
            logging.warning("log message from test_log_1")

    """
    )
    testdir.makeini(
        """
        [pytest]
        log_cli=true
    """
    )

    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*WARNING*Unknown Section*",
            "{}::test_log_1 ".format(filename),
            "*WARNING* >>>>> START >>>>>*",
            "*-- live log setup --*",
            "*WARNING*log message from setup of test_log_1*",
            "*-- live log call --*",
            "*WARNING*log message from test_log_1*",
            "PASSED *100%*",
            "*-- live log teardown --*",
            "*WARNING*log message from teardown of test_log_1*",
            "*WARNING* <<<<< END <<<<<<<*",
            "=* 1 passed in *=",
        ]
    )


def test_sections_single_new_line_after_test_outcome(testdir, request):
    """Check that only a single new line is written between log messages during
    teardown/finish."""
    filename = request.node.name + ".py"
    testdir.makeconftest(
        """
        import pytest
        import logging

        def pytest_runtest_logstart():
            logging.warning('>>>>> START >>>>>')

        def pytest_runtest_logfinish():
            logging.warning('<<<<< END <<<<<<<')
            logging.warning('<<<<< END <<<<<<<')
    """
    )

    testdir.makepyfile(
        """
        import pytest
        import logging

        @pytest.fixture
        def fix(request):
            logging.warning("log message from setup of {}".format(request.node.name))
            yield
            logging.warning("log message from teardown of {}".format(request.node.name))
            logging.warning("log message from teardown of {}".format(request.node.name))

        def test_log_1(fix):
            logging.warning("log message from test_log_1")
    """
    )
    testdir.makeini(
        """
        [pytest]
        log_cli=true
    """
    )

    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "{}::test_log_1 ".format(filename),
            "*-- live log start --*",
            "*WARNING* >>>>> START >>>>>*",
            "*-- live log setup --*",
            "*WARNING*log message from setup of test_log_1*",
            "*-- live log call --*",
            "*WARNING*log message from test_log_1*",
            "PASSED *100%*",
            "*-- live log teardown --*",
            "*WARNING*log message from teardown of test_log_1*",
            "*-- live log finish --*",
            "*WARNING* <<<<< END <<<<<<<*",
            "*WARNING* <<<<< END <<<<<<<*",
            "=* 1 passed in *=",
        ]
    )
    assert (
        re.search(
            r"(.+)live log teardown(.+)\nWARNING(.+)\nWARNING(.+)",
            result.stdout.str(),
            re.MULTILINE,
        )
        is not None
    )
    assert (
        re.search(
            r"(.+)live log finish(.+)\nWARNING(.+)\nWARNING(.+)",
            result.stdout.str(),
            re.MULTILINE,
        )
        is not None
    )


def test_log_cli_level(testdir):
    # Default log file level
    testdir.makepyfile(
        """
        import pytest
        import logging
        def test_log_cli(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_cli_handler.level == logging.INFO
            logging.getLogger('catchlog').debug("This log message won't be shown")
            logging.getLogger('catchlog').info("This log message will be shown")
            print('PASSED')
    """
    )
    testdir.makeini(
        """
        [pytest]
        log_cli=true
    """
    )

    result = testdir.runpytest("-s", "--log-cli-level=INFO")

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines(
        [
            "*test_log_cli_level.py*This log message will be shown",
            "PASSED",  # 'PASSED' on its own line because the log message prints a new line
        ]
    )
    result.stdout.no_fnmatch_line("*This log message won't be shown*")

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0

    result = testdir.runpytest("-s", "--log-level=INFO")

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines(
        [
            "*test_log_cli_level.py* This log message will be shown",
            "PASSED",  # 'PASSED' on its own line because the log message prints a new line
        ]
    )
    result.stdout.no_fnmatch_line("*This log message won't be shown*")

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0


def test_log_cli_ini_level(testdir):
    testdir.makeini(
        """
        [pytest]
        log_cli=true
        log_cli_level = INFO
        """
    )
    testdir.makepyfile(
        """
        import pytest
        import logging
        def test_log_cli(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_cli_handler.level == logging.INFO
            logging.getLogger('catchlog').debug("This log message won't be shown")
            logging.getLogger('catchlog').info("This log message will be shown")
            print('PASSED')
    """
    )

    result = testdir.runpytest("-s")

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines(
        [
            "*test_log_cli_ini_level.py* This log message will be shown",
            "PASSED",  # 'PASSED' on its own line because the log message prints a new line
        ]
    )
    result.stdout.no_fnmatch_line("*This log message won't be shown*")

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0


@pytest.mark.parametrize(
    "cli_args",
    ["", "--log-level=WARNING", "--log-file-level=WARNING", "--log-cli-level=WARNING"],
)
def test_log_cli_auto_enable(testdir, cli_args):
    """Check that live logs are enabled if --log-level or --log-cli-level is passed on the CLI.
    It should not be auto enabled if the same configs are set on the INI file.
    """
    testdir.makepyfile(
        """
        import logging

        def test_log_1():
            logging.info("log message from test_log_1 not to be shown")
            logging.warning("log message from test_log_1")

    """
    )
    testdir.makeini(
        """
        [pytest]
        log_level=INFO
        log_cli_level=INFO
    """
    )

    result = testdir.runpytest(cli_args)
    stdout = result.stdout.str()
    if cli_args == "--log-cli-level=WARNING":
        result.stdout.fnmatch_lines(
            [
                "*::test_log_1 ",
                "*-- live log call --*",
                "*WARNING*log message from test_log_1*",
                "PASSED *100%*",
                "=* 1 passed in *=",
            ]
        )
        assert "INFO" not in stdout
    else:
        result.stdout.fnmatch_lines(
            ["*test_log_cli_auto_enable*100%*", "=* 1 passed in *="]
        )
        assert "INFO" not in stdout
        assert "WARNING" not in stdout


def test_log_file_cli(testdir):
    # Default log file level
    testdir.makepyfile(
        """
        import pytest
        import logging
        def test_log_file(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_file_handler.level == logging.WARNING
            logging.getLogger('catchlog').info("This log message won't be shown")
            logging.getLogger('catchlog').warning("This log message will be shown")
            print('PASSED')
    """
    )

    log_file = testdir.tmpdir.join("pytest.log").strpath

    result = testdir.runpytest(
        "-s", "--log-file={}".format(log_file), "--log-file-level=WARNING"
    )

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines(["test_log_file_cli.py PASSED"])

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "This log message will be shown" in contents
        assert "This log message won't be shown" not in contents


def test_log_file_cli_level(testdir):
    # Default log file level
    testdir.makepyfile(
        """
        import pytest
        import logging
        def test_log_file(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_file_handler.level == logging.INFO
            logging.getLogger('catchlog').debug("This log message won't be shown")
            logging.getLogger('catchlog').info("This log message will be shown")
            print('PASSED')
    """
    )

    log_file = testdir.tmpdir.join("pytest.log").strpath

    result = testdir.runpytest(
        "-s", "--log-file={}".format(log_file), "--log-file-level=INFO"
    )

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines(["test_log_file_cli_level.py PASSED"])

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "This log message will be shown" in contents
        assert "This log message won't be shown" not in contents


def test_log_level_not_changed_by_default(testdir):
    testdir.makepyfile(
        """
        import logging
        def test_log_file():
            assert logging.getLogger().level == logging.WARNING
    """
    )
    result = testdir.runpytest("-s")
    result.stdout.fnmatch_lines(["* 1 passed in *"])


def test_log_file_ini(testdir):
    log_file = testdir.tmpdir.join("pytest.log").strpath

    testdir.makeini(
        """
        [pytest]
        log_file={}
        log_file_level=WARNING
        """.format(
            log_file
        )
    )
    testdir.makepyfile(
        """
        import pytest
        import logging
        def test_log_file(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_file_handler.level == logging.WARNING
            logging.getLogger('catchlog').info("This log message won't be shown")
            logging.getLogger('catchlog').warning("This log message will be shown")
            print('PASSED')
    """
    )

    result = testdir.runpytest("-s")

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines(["test_log_file_ini.py PASSED"])

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "This log message will be shown" in contents
        assert "This log message won't be shown" not in contents


def test_log_file_ini_level(testdir):
    log_file = testdir.tmpdir.join("pytest.log").strpath

    testdir.makeini(
        """
        [pytest]
        log_file={}
        log_file_level = INFO
        """.format(
            log_file
        )
    )
    testdir.makepyfile(
        """
        import pytest
        import logging
        def test_log_file(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_file_handler.level == logging.INFO
            logging.getLogger('catchlog').debug("This log message won't be shown")
            logging.getLogger('catchlog').info("This log message will be shown")
            print('PASSED')
    """
    )

    result = testdir.runpytest("-s")

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines(["test_log_file_ini_level.py PASSED"])

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "This log message will be shown" in contents
        assert "This log message won't be shown" not in contents


def test_log_file_unicode(testdir):
    log_file = testdir.tmpdir.join("pytest.log").strpath

    testdir.makeini(
        """
        [pytest]
        log_file={}
        log_file_level = INFO
        """.format(
            log_file
        )
    )
    testdir.makepyfile(
        """\
        import logging

        def test_log_file():
            logging.getLogger('catchlog').info("Normal message")
            logging.getLogger('catchlog').info("├")
            logging.getLogger('catchlog').info("Another normal message")
        """
    )

    result = testdir.runpytest()

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file, encoding="utf-8") as rfh:
        contents = rfh.read()
        assert "Normal message" in contents
        assert "├" in contents
        assert "Another normal message" in contents


@pytest.mark.parametrize("has_capture_manager", [True, False])
def test_live_logging_suspends_capture(has_capture_manager: bool, request) -> None:
    """Test that capture manager is suspended when we emitting messages for live logging.

    This tests the implementation calls instead of behavior because it is difficult/impossible to do it using
    ``testdir`` facilities because they do their own capturing.

    We parametrize the test to also make sure _LiveLoggingStreamHandler works correctly if no capture manager plugin
    is installed.
    """
    import logging
    import contextlib
    from functools import partial
    from _pytest.logging import _LiveLoggingStreamHandler

    class MockCaptureManager:
        calls = []

        @contextlib.contextmanager
        def global_and_fixture_disabled(self):
            self.calls.append("enter disabled")
            yield
            self.calls.append("exit disabled")

    class DummyTerminal(io.StringIO):
        def section(self, *args, **kwargs):
            pass

    out_file = cast(TerminalReporter, DummyTerminal())
    capture_manager = (
        cast(CaptureManager, MockCaptureManager()) if has_capture_manager else None
    )
    handler = _LiveLoggingStreamHandler(out_file, capture_manager)
    handler.set_when("call")

    logger = logging.getLogger(__name__ + ".test_live_logging_suspends_capture")
    logger.addHandler(handler)
    request.addfinalizer(partial(logger.removeHandler, handler))

    logger.critical("some message")
    if has_capture_manager:
        assert MockCaptureManager.calls == ["enter disabled", "exit disabled"]
    else:
        assert MockCaptureManager.calls == []
    assert cast(io.StringIO, out_file).getvalue() == "\nsome message\n"


def test_collection_live_logging(testdir):
    testdir.makepyfile(
        """
        import logging

        logging.getLogger().info("Normal message")
    """
    )

    result = testdir.runpytest("--log-cli-level=INFO")
    result.stdout.fnmatch_lines(
        ["*--- live log collection ---*", "*Normal message*", "collected 0 items"]
    )


@pytest.mark.parametrize("verbose", ["", "-q", "-qq"])
def test_collection_collect_only_live_logging(testdir, verbose):
    testdir.makepyfile(
        """
        def test_simple():
            pass
    """
    )

    result = testdir.runpytest("--collect-only", "--log-cli-level=INFO", verbose)

    expected_lines = []

    if not verbose:
        expected_lines.extend(
            [
                "*collected 1 item*",
                "*<Module test_collection_collect_only_live_logging.py>*",
                "*no tests ran*",
            ]
        )
    elif verbose == "-q":
        result.stdout.no_fnmatch_line("*collected 1 item**")
        expected_lines.extend(
            [
                "*test_collection_collect_only_live_logging.py::test_simple*",
                "no tests ran in [0-9].[0-9][0-9]s",
            ]
        )
    elif verbose == "-qq":
        result.stdout.no_fnmatch_line("*collected 1 item**")
        expected_lines.extend(["*test_collection_collect_only_live_logging.py: 1*"])

    result.stdout.fnmatch_lines(expected_lines)


def test_collection_logging_to_file(testdir):
    log_file = testdir.tmpdir.join("pytest.log").strpath

    testdir.makeini(
        """
        [pytest]
        log_file={}
        log_file_level = INFO
        """.format(
            log_file
        )
    )

    testdir.makepyfile(
        """
        import logging

        logging.getLogger().info("Normal message")

        def test_simple():
            logging.getLogger().debug("debug message in test_simple")
            logging.getLogger().info("info message in test_simple")
    """
    )

    result = testdir.runpytest()

    result.stdout.no_fnmatch_line("*--- live log collection ---*")

    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file, encoding="utf-8") as rfh:
        contents = rfh.read()
        assert "Normal message" in contents
        assert "debug message in test_simple" not in contents
        assert "info message in test_simple" in contents


def test_log_in_hooks(testdir):
    log_file = testdir.tmpdir.join("pytest.log").strpath

    testdir.makeini(
        """
        [pytest]
        log_file={}
        log_file_level = INFO
        log_cli=true
        """.format(
            log_file
        )
    )
    testdir.makeconftest(
        """
        import logging

        def pytest_runtestloop(session):
            logging.info('runtestloop')

        def pytest_sessionstart(session):
            logging.info('sessionstart')

        def pytest_sessionfinish(session, exitstatus):
            logging.info('sessionfinish')
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*sessionstart*", "*runtestloop*", "*sessionfinish*"])
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "sessionstart" in contents
        assert "runtestloop" in contents
        assert "sessionfinish" in contents


def test_log_in_runtest_logreport(testdir):
    log_file = testdir.tmpdir.join("pytest.log").strpath

    testdir.makeini(
        """
        [pytest]
        log_file={}
        log_file_level = INFO
        log_cli=true
        """.format(
            log_file
        )
    )
    testdir.makeconftest(
        """
        import logging
        logger = logging.getLogger(__name__)

        def pytest_runtest_logreport(report):
            logger.info("logreport")
    """
    )
    testdir.makepyfile(
        """
            def test_first():
                assert True
        """
    )
    testdir.runpytest()
    with open(log_file) as rfh:
        contents = rfh.read()
        assert contents.count("logreport") == 3


def test_log_set_path(testdir):
    report_dir_base = testdir.tmpdir.strpath

    testdir.makeini(
        """
        [pytest]
        log_file_level = DEBUG
        log_cli=true
        """
    )
    testdir.makeconftest(
        """
            import os
            import pytest
            @pytest.hookimpl(hookwrapper=True, tryfirst=True)
            def pytest_runtest_setup(item):
                config = item.config
                logging_plugin = config.pluginmanager.get_plugin("logging-plugin")
                report_file = os.path.join({}, item._request.node.name)
                logging_plugin.set_log_path(report_file)
                yield
        """.format(
            repr(report_dir_base)
        )
    )
    testdir.makepyfile(
        """
            import logging
            logger = logging.getLogger("testcase-logger")
            def test_first():
                logger.info("message from test 1")
                assert True

            def test_second():
                logger.debug("message from test 2")
                assert True
        """
    )
    testdir.runpytest()
    with open(os.path.join(report_dir_base, "test_first")) as rfh:
        content = rfh.read()
        assert "message from test 1" in content

    with open(os.path.join(report_dir_base, "test_second")) as rfh:
        content = rfh.read()
        assert "message from test 2" in content


def test_colored_captured_log(testdir):
    """Test that the level names of captured log messages of a failing test
    are colored."""
    testdir.makepyfile(
        """
        import logging

        logger = logging.getLogger(__name__)

        def test_foo():
            logger.info('text going to logger from call')
            assert False
        """
    )
    result = testdir.runpytest("--log-level=INFO", "--color=yes")
    assert result.ret == 1
    result.stdout.fnmatch_lines(
        [
            "*-- Captured log call --*",
            "\x1b[32mINFO    \x1b[0m*text going to logger from call",
        ]
    )


def test_colored_ansi_esc_caplogtext(testdir):
    """Make sure that caplog.text does not contain ANSI escape sequences."""
    testdir.makepyfile(
        """
        import logging

        logger = logging.getLogger(__name__)

        def test_foo(caplog):
            logger.info('text going to logger from call')
            assert '\x1b' not in caplog.text
        """
    )
    result = testdir.runpytest("--log-level=INFO", "--color=yes")
    assert result.ret == 0


def test_logging_emit_error(testdir: Testdir) -> None:
    """An exception raised during emit() should fail the test.

    The default behavior of logging is to print "Logging error"
    to stderr with the call stack and some extra details.

    pytest overrides this behavior to propagate the exception.
    """
    testdir.makepyfile(
        """
        import logging

        def test_bad_log():
            logging.warning('oops', 'first', 2)
        """
    )
    result = testdir.runpytest()
    result.assert_outcomes(failed=1)
    result.stdout.fnmatch_lines(
        [
            "====* FAILURES *====",
            "*not all arguments converted during string formatting*",
        ]
    )


def test_logging_emit_error_supressed(testdir: Testdir) -> None:
    """If logging is configured to silently ignore errors, pytest
    doesn't propagate errors either."""
    testdir.makepyfile(
        """
        import logging

        def test_bad_log(monkeypatch):
            monkeypatch.setattr(logging, 'raiseExceptions', False)
            logging.warning('oops', 'first', 2)
        """
    )
    result = testdir.runpytest()
    result.assert_outcomes(passed=1)


def test_log_file_cli_subdirectories_are_successfully_created(testdir):
    path = testdir.makepyfile(""" def test_logger(): pass """)
    expected = os.path.join(os.path.dirname(str(path)), "foo", "bar")
    result = testdir.runpytest("--log-file=foo/bar/logf.log")
    assert "logf.log" in os.listdir(expected)
    assert result.ret == ExitCode.OK
