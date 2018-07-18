# -*- coding: utf-8 -*-
import re
import os

import six

import pytest


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

    # the capture log calls in the stdout section only contain the
    # logger.error msg, because --log-level=ERROR
    result.stdout.fnmatch_lines(["*error text going to logger*"])
    with pytest.raises(pytest.fail.Exception):
        result.stdout.fnmatch_lines(["*warning text going to logger*"])
    with pytest.raises(pytest.fail.Exception):
        result.stdout.fnmatch_lines(["*info text going to logger*"])

    # the log file should contain the warning and the error log messages and
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
    assert "DEBUG" not in result.stdout.str()


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


def test_disable_log_capturing(testdir):
    testdir.makepyfile(
        """
        import sys
        import logging

        logger = logging.getLogger(__name__)

        def test_foo():
            sys.stdout.write('text going to stdout')
            logger.warning('catch me if you can!')
            sys.stderr.write('text going to stderr')
            assert False
        """
    )
    result = testdir.runpytest("--no-print-logs")
    print(result.stdout)
    assert result.ret == 1
    result.stdout.fnmatch_lines(["*- Captured stdout call -*", "text going to stdout"])
    result.stdout.fnmatch_lines(["*- Captured stderr call -*", "text going to stderr"])
    with pytest.raises(pytest.fail.Exception):
        result.stdout.fnmatch_lines(["*- Captured *log call -*"])


def test_disable_log_capturing_ini(testdir):
    testdir.makeini(
        """
        [pytest]
        log_print=False
        """
    )
    testdir.makepyfile(
        """
        import sys
        import logging

        logger = logging.getLogger(__name__)

        def test_foo():
            sys.stdout.write('text going to stdout')
            logger.warning('catch me if you can!')
            sys.stderr.write('text going to stderr')
            assert False
        """
    )
    result = testdir.runpytest()
    print(result.stdout)
    assert result.ret == 1
    result.stdout.fnmatch_lines(["*- Captured stdout call -*", "text going to stdout"])
    result.stdout.fnmatch_lines(["*- Captured stderr call -*", "text going to stderr"])
    with pytest.raises(pytest.fail.Exception):
        result.stdout.fnmatch_lines(["*- Captured *log call -*"])


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
                "test_log_cli_enabled_disabled.py* CRITICAL critical message logged by test",
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
            "test_log_cli_default_level.py*WARNING message will be shown*",
        ]
    )
    assert "INFO message won't be shown" not in result.stdout.str()
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
    assert re.search(
        r"(.+)live log teardown(.+)\n(.+)WARNING(.+)\n(.+)WARNING(.+)",
        result.stdout.str(),
        re.MULTILINE,
    ) is not None
    assert re.search(
        r"(.+)live log finish(.+)\n(.+)WARNING(.+)\n(.+)WARNING(.+)",
        result.stdout.str(),
        re.MULTILINE,
    ) is not None


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
            "test_log_cli_level.py*This log message will be shown",
            "PASSED",  # 'PASSED' on its own line because the log message prints a new line
        ]
    )
    assert "This log message won't be shown" not in result.stdout.str()

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0

    result = testdir.runpytest("-s", "--log-level=INFO")

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines(
        [
            "test_log_cli_level.py* This log message will be shown",
            "PASSED",  # 'PASSED' on its own line because the log message prints a new line
        ]
    )
    assert "This log message won't be shown" not in result.stdout.str()

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
            "test_log_cli_ini_level.py* This log message will be shown",
            "PASSED",  # 'PASSED' on its own line because the log message prints a new line
        ]
    )
    assert "This log message won't be shown" not in result.stdout.str()

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0


@pytest.mark.parametrize(
    "cli_args",
    ["", "--log-level=WARNING", "--log-file-level=WARNING", "--log-cli-level=WARNING"],
)
def test_log_cli_auto_enable(testdir, request, cli_args):
    """Check that live logs are enabled if --log-level or --log-cli-level is passed on the CLI.
    It should not be auto enabled if the same configs are set on the INI file.
    """
    testdir.makepyfile(
        """
        import pytest
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
        assert "INFO" not in result.stdout.str()
    else:
        result.stdout.fnmatch_lines(
            ["*test_log_cli_auto_enable*100%*", "=* 1 passed in *="]
        )
        assert "INFO" not in result.stdout.str()
        assert "WARNING" not in result.stdout.str()


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
    result.stdout.fnmatch_lines("* 1 passed in *")


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


@pytest.mark.parametrize("has_capture_manager", [True, False])
def test_live_logging_suspends_capture(has_capture_manager, request):
    """Test that capture manager is suspended when we emitting messages for live logging.

    This tests the implementation calls instead of behavior because it is difficult/impossible to do it using
    ``testdir`` facilities because they do their own capturing.

    We parametrize the test to also make sure _LiveLoggingStreamHandler works correctly if no capture manager plugin
    is installed.
    """
    import logging
    from functools import partial
    from _pytest.capture import CaptureManager
    from _pytest.logging import _LiveLoggingStreamHandler

    class MockCaptureManager:
        calls = []

        def suspend_global_capture(self):
            self.calls.append("suspend_global_capture")

        def resume_global_capture(self):
            self.calls.append("resume_global_capture")

    # sanity check
    assert CaptureManager.suspend_capture_item
    assert CaptureManager.resume_global_capture

    class DummyTerminal(six.StringIO):

        def section(self, *args, **kwargs):
            pass

    out_file = DummyTerminal()
    capture_manager = MockCaptureManager() if has_capture_manager else None
    handler = _LiveLoggingStreamHandler(out_file, capture_manager)
    handler.set_when("call")

    logger = logging.getLogger(__name__ + ".test_live_logging_suspends_capture")
    logger.addHandler(handler)
    request.addfinalizer(partial(logger.removeHandler, handler))

    logger.critical("some message")
    if has_capture_manager:
        assert (
            MockCaptureManager.calls
            == ["suspend_global_capture", "resume_global_capture"]
        )
    else:
        assert MockCaptureManager.calls == []
    assert out_file.getvalue() == "\nsome message\n"
