# mypy: disable-error-code="attr-defined"
# mypy: disallow-untyped-defs
import logging
from typing import Iterator

from _pytest.logging import caplog_records_key
from _pytest.pytester import Pytester
import pytest


logger = logging.getLogger(__name__)
sublogger = logging.getLogger(__name__ + ".baz")


@pytest.fixture(autouse=True)
def cleanup_disabled_logging() -> Iterator[None]:
    """Simple fixture that ensures that a test doesn't disable logging.

    This is necessary because ``logging.disable()`` is global, so a test disabling logging
    and not cleaning up after will break every test that runs after it.

    This behavior was moved to a fixture so that logging will be un-disabled even if the test fails an assertion.
    """
    yield
    logging.disable(logging.NOTSET)


def test_fixture_help(pytester: Pytester) -> None:
    result = pytester.runpytest("--fixtures")
    result.stdout.fnmatch_lines(["*caplog*"])


def test_change_level(caplog: pytest.LogCaptureFixture) -> None:
    caplog.set_level(logging.INFO)
    logger.debug("handler DEBUG level")
    logger.info("handler INFO level")

    caplog.set_level(logging.CRITICAL, logger=sublogger.name)
    sublogger.warning("logger WARNING level")
    sublogger.critical("logger CRITICAL level")

    assert "DEBUG" not in caplog.text
    assert "INFO" in caplog.text
    assert "WARNING" not in caplog.text
    assert "CRITICAL" in caplog.text


def test_change_level_logging_disabled(caplog: pytest.LogCaptureFixture) -> None:
    logging.disable(logging.CRITICAL)
    assert logging.root.manager.disable == logging.CRITICAL
    caplog.set_level(logging.WARNING)
    logger.info("handler INFO level")
    logger.warning("handler WARNING level")

    caplog.set_level(logging.CRITICAL, logger=sublogger.name)
    sublogger.warning("logger SUB_WARNING level")
    sublogger.critical("logger SUB_CRITICAL level")

    assert "INFO" not in caplog.text
    assert "WARNING" in caplog.text
    assert "SUB_WARNING" not in caplog.text
    assert "SUB_CRITICAL" in caplog.text


def test_change_level_undo(pytester: Pytester) -> None:
    """Ensure that 'set_level' is undone after the end of the test.

    Tests the logging output themselves (affected both by logger and handler levels).
    """
    pytester.makepyfile(
        """
        import logging

        def test1(caplog):
            caplog.set_level(logging.INFO)
            # using + operator here so fnmatch_lines doesn't match the code in the traceback
            logging.info('log from ' + 'test1')
            assert 0

        def test2(caplog):
            # using + operator here so fnmatch_lines doesn't match the code in the traceback
            logging.info('log from ' + 'test2')
            assert 0
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*log from test1*", "*2 failed in *"])
    result.stdout.no_fnmatch_line("*log from test2*")


def test_change_disabled_level_undo(pytester: Pytester) -> None:
    """Ensure that '_force_enable_logging' in 'set_level' is undone after the end of the test.

    Tests the logging output themselves (affected by disabled logging level).
    """
    pytester.makepyfile(
        """
        import logging

        def test1(caplog):
            logging.disable(logging.CRITICAL)
            caplog.set_level(logging.INFO)
            # using + operator here so fnmatch_lines doesn't match the code in the traceback
            logging.info('log from ' + 'test1')
            assert 0

        def test2(caplog):
            # using + operator here so fnmatch_lines doesn't match the code in the traceback
            # use logging.warning because we need a level that will show up if logging.disabled
            # isn't reset to ``CRITICAL`` after test1.
            logging.warning('log from ' + 'test2')
            assert 0
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*log from test1*", "*2 failed in *"])
    result.stdout.no_fnmatch_line("*log from test2*")


def test_change_level_undoes_handler_level(pytester: Pytester) -> None:
    """Ensure that 'set_level' is undone after the end of the test (handler).

    Issue #7569. Tests the handler level specifically.
    """
    pytester.makepyfile(
        """
        import logging

        def test1(caplog):
            assert caplog.handler.level == 0
            caplog.set_level(9999)
            caplog.set_level(41)
            assert caplog.handler.level == 41

        def test2(caplog):
            assert caplog.handler.level == 0

        def test3(caplog):
            assert caplog.handler.level == 0
            caplog.set_level(43)
            assert caplog.handler.level == 43
    """
    )
    result = pytester.runpytest()
    result.assert_outcomes(passed=3)


def test_with_statement_at_level(caplog: pytest.LogCaptureFixture) -> None:
    with caplog.at_level(logging.INFO):
        logger.debug("handler DEBUG level")
        logger.info("handler INFO level")

        with caplog.at_level(logging.CRITICAL, logger=sublogger.name):
            sublogger.warning("logger WARNING level")
            sublogger.critical("logger CRITICAL level")

    assert "DEBUG" not in caplog.text
    assert "INFO" in caplog.text
    assert "WARNING" not in caplog.text
    assert "CRITICAL" in caplog.text


def test_with_statement_at_level_logging_disabled(
    caplog: pytest.LogCaptureFixture,
) -> None:
    logging.disable(logging.CRITICAL)
    assert logging.root.manager.disable == logging.CRITICAL
    with caplog.at_level(logging.WARNING):
        logger.debug("handler DEBUG level")
        logger.info("handler INFO level")
        logger.warning("handler WARNING level")
        logger.error("handler ERROR level")
        logger.critical("handler CRITICAL level")

        assert logging.root.manager.disable == logging.INFO

        with caplog.at_level(logging.CRITICAL, logger=sublogger.name):
            sublogger.warning("logger SUB_WARNING level")
            sublogger.critical("logger SUB_CRITICAL level")

    assert "DEBUG" not in caplog.text
    assert "INFO" not in caplog.text
    assert "WARNING" in caplog.text
    assert "ERROR" in caplog.text
    assert " CRITICAL" in caplog.text
    assert "SUB_WARNING" not in caplog.text
    assert "SUB_CRITICAL" in caplog.text
    assert logging.root.manager.disable == logging.CRITICAL


def test_with_statement_filtering(caplog: pytest.LogCaptureFixture) -> None:
    class TestFilter(logging.Filter):
        def filter(self, record: logging.LogRecord) -> bool:
            record.msg = "filtered handler call"
            return True

    with caplog.at_level(logging.INFO):
        with caplog.filtering(TestFilter()):
            logger.info("handler call")
        logger.info("handler call")

    filtered_tuple, unfiltered_tuple = caplog.record_tuples
    assert filtered_tuple == ("test_fixture", 20, "filtered handler call")
    assert unfiltered_tuple == ("test_fixture", 20, "handler call")


@pytest.mark.parametrize(
    "level_str,expected_disable_level",
    [
        ("CRITICAL", logging.ERROR),
        ("ERROR", logging.WARNING),
        ("WARNING", logging.INFO),
        ("INFO", logging.DEBUG),
        ("DEBUG", logging.NOTSET),
        ("NOTSET", logging.NOTSET),
        ("NOTVALIDLEVEL", logging.NOTSET),
    ],
)
def test_force_enable_logging_level_string(
    caplog: pytest.LogCaptureFixture, level_str: str, expected_disable_level: int
) -> None:
    """Test _force_enable_logging using a level string.

    ``expected_disable_level`` is one level below ``level_str`` because the disabled log level
    always needs to be *at least* one level lower than the level that caplog is trying to capture.
    """
    test_logger = logging.getLogger("test_str_level_force_enable")
    # Emulate a testing environment where all logging is disabled.
    logging.disable(logging.CRITICAL)
    # Make sure all logging is disabled.
    assert not test_logger.isEnabledFor(logging.CRITICAL)
    # Un-disable logging for `level_str`.
    caplog._force_enable_logging(level_str, test_logger)
    # Make sure that the disabled level is now one below the requested logging level.
    # We don't use `isEnabledFor` here because that also checks the level set by
    # `logging.setLevel()` which is irrelevant to `logging.disable()`.
    assert test_logger.manager.disable == expected_disable_level


def test_log_access(caplog: pytest.LogCaptureFixture) -> None:
    caplog.set_level(logging.INFO)
    logger.info("boo %s", "arg")
    assert caplog.records[0].levelname == "INFO"
    assert caplog.records[0].msg == "boo %s"
    assert "boo arg" in caplog.text


def test_messages(caplog: pytest.LogCaptureFixture) -> None:
    caplog.set_level(logging.INFO)
    logger.info("boo %s", "arg")
    logger.info("bar %s\nbaz %s", "arg1", "arg2")
    assert "boo arg" == caplog.messages[0]
    assert "bar arg1\nbaz arg2" == caplog.messages[1]
    assert caplog.text.count("\n") > len(caplog.messages)
    assert len(caplog.text.splitlines()) > len(caplog.messages)

    try:
        raise Exception("test")
    except Exception:
        logger.exception("oops")

    assert "oops" in caplog.text
    assert "oops" in caplog.messages[-1]
    # Tracebacks are stored in the record and not added until the formatter or handler.
    assert "Exception" in caplog.text
    assert "Exception" not in caplog.messages[-1]


def test_record_tuples(caplog: pytest.LogCaptureFixture) -> None:
    caplog.set_level(logging.INFO)
    logger.info("boo %s", "arg")

    assert caplog.record_tuples == [(__name__, logging.INFO, "boo arg")]


def test_unicode(caplog: pytest.LogCaptureFixture) -> None:
    caplog.set_level(logging.INFO)
    logger.info("bū")
    assert caplog.records[0].levelname == "INFO"
    assert caplog.records[0].msg == "bū"
    assert "bū" in caplog.text


def test_clear(caplog: pytest.LogCaptureFixture) -> None:
    caplog.set_level(logging.INFO)
    logger.info("bū")
    assert len(caplog.records)
    assert caplog.text
    caplog.clear()
    assert not len(caplog.records)
    assert not caplog.text


@pytest.fixture
def logging_during_setup_and_teardown(
    caplog: pytest.LogCaptureFixture,
) -> Iterator[None]:
    caplog.set_level("INFO")
    logger.info("a_setup_log")
    yield
    logger.info("a_teardown_log")
    assert [x.message for x in caplog.get_records("teardown")] == ["a_teardown_log"]


def private_assert_caplog_records_is_setup_call(
    caplog: pytest.LogCaptureFixture,
) -> None:
    # This reaches into private API, don't use this type of thing in real tests!
    caplog_records = caplog._item.stash[caplog_records_key]
    assert set(caplog_records) == {"setup", "call"}


def test_captures_for_all_stages(
    caplog: pytest.LogCaptureFixture, logging_during_setup_and_teardown: None
) -> None:
    assert not caplog.records
    assert not caplog.get_records("call")
    logger.info("a_call_log")
    assert [x.message for x in caplog.get_records("call")] == ["a_call_log"]

    assert [x.message for x in caplog.get_records("setup")] == ["a_setup_log"]

    private_assert_caplog_records_is_setup_call(caplog)


def test_clear_for_call_stage(
    caplog: pytest.LogCaptureFixture, logging_during_setup_and_teardown: None
) -> None:
    logger.info("a_call_log")
    assert [x.message for x in caplog.get_records("call")] == ["a_call_log"]
    assert [x.message for x in caplog.get_records("setup")] == ["a_setup_log"]
    private_assert_caplog_records_is_setup_call(caplog)

    caplog.clear()

    assert caplog.get_records("call") == []
    assert [x.message for x in caplog.get_records("setup")] == ["a_setup_log"]
    private_assert_caplog_records_is_setup_call(caplog)

    logging.info("a_call_log_after_clear")
    assert [x.message for x in caplog.get_records("call")] == ["a_call_log_after_clear"]
    assert [x.message for x in caplog.get_records("setup")] == ["a_setup_log"]
    private_assert_caplog_records_is_setup_call(caplog)


def test_ini_controls_global_log_level(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest
        import logging
        def test_log_level_override(request, caplog):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_level == logging.ERROR
            logger = logging.getLogger('catchlog')
            logger.warning("WARNING message won't be shown")
            logger.error("ERROR message will be shown")
            assert 'WARNING' not in caplog.text
            assert 'ERROR' in caplog.text
    """
    )
    pytester.makeini(
        """
        [pytest]
        log_level=ERROR
    """
    )

    result = pytester.runpytest()
    # make sure that we get a '0' exit code for the testsuite
    assert result.ret == 0


def test_can_override_global_log_level(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest
        import logging
        def test_log_level_override(request, caplog):
            logger = logging.getLogger('catchlog')
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_level == logging.WARNING

            logger.info("INFO message won't be shown")

            caplog.set_level(logging.INFO, logger.name)

            with caplog.at_level(logging.DEBUG, logger.name):
                logger.debug("DEBUG message will be shown")

            logger.debug("DEBUG message won't be shown")

            with caplog.at_level(logging.CRITICAL, logger.name):
                logger.warning("WARNING message won't be shown")

            logger.debug("DEBUG message won't be shown")
            logger.info("INFO message will be shown")

            assert "message won't be shown" not in caplog.text
    """
    )
    pytester.makeini(
        """
        [pytest]
        log_level=WARNING
    """
    )

    result = pytester.runpytest()
    assert result.ret == 0


def test_captures_despite_exception(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest
        import logging
        def test_log_level_override(request, caplog):
            logger = logging.getLogger('catchlog')
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_level == logging.WARNING

            logger.error("ERROR message " + "will be shown")

            with caplog.at_level(logging.DEBUG, logger.name):
                logger.debug("DEBUG message " + "won't be shown")
                raise Exception()
    """
    )
    pytester.makeini(
        """
        [pytest]
        log_level=WARNING
    """
    )

    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*ERROR message will be shown*"])
    result.stdout.no_fnmatch_line("*DEBUG message won't be shown*")
    assert result.ret == 1


def test_log_report_captures_according_to_config_option_upon_failure(
    pytester: Pytester,
) -> None:
    """Test that upon failure:
    (1) `caplog` succeeded to capture the DEBUG message and assert on it => No `Exception` is raised.
    (2) The `DEBUG` message does NOT appear in the `Captured log call` report.
    (3) The stdout, `INFO`, and `WARNING` messages DO appear in the test reports due to `--log-level=INFO`.
    """
    pytester.makepyfile(
        """
        import pytest
        import logging

        def function_that_logs():
            logging.debug('DEBUG log ' + 'message')
            logging.info('INFO log ' + 'message')
            logging.warning('WARNING log ' + 'message')
            print('Print ' + 'message')

        def test_that_fails(request, caplog):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_level == logging.INFO

            with caplog.at_level(logging.DEBUG):
                function_that_logs()

            if 'DEBUG log ' + 'message' not in caplog.text:
                raise Exception('caplog failed to ' + 'capture DEBUG')

            assert False
    """
    )

    result = pytester.runpytest("--log-level=INFO")
    result.stdout.no_fnmatch_line("*Exception: caplog failed to capture DEBUG*")
    result.stdout.no_fnmatch_line("*DEBUG log message*")
    result.stdout.fnmatch_lines(
        ["*Print message*", "*INFO log message*", "*WARNING log message*"]
    )
    assert result.ret == 1
