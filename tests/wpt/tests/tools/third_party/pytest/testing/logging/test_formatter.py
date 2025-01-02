import logging
from typing import Any

from _pytest._io import TerminalWriter
from _pytest.logging import ColoredLevelFormatter


def test_coloredlogformatter() -> None:
    logfmt = "%(filename)-25s %(lineno)4d %(levelname)-8s %(message)s"

    record = logging.LogRecord(
        name="dummy",
        level=logging.INFO,
        pathname="dummypath",
        lineno=10,
        msg="Test Message",
        args=(),
        exc_info=None,
    )

    tw = TerminalWriter()
    tw.hasmarkup = True
    formatter = ColoredLevelFormatter(tw, logfmt)
    output = formatter.format(record)
    assert output == (
        "dummypath                   10 \x1b[32mINFO    \x1b[0m Test Message"
    )

    tw.hasmarkup = False
    formatter = ColoredLevelFormatter(tw, logfmt)
    output = formatter.format(record)
    assert output == ("dummypath                   10 INFO     Test Message")


def test_coloredlogformatter_with_width_precision() -> None:
    logfmt = "%(filename)-25s %(lineno)4d %(levelname)-8.8s %(message)s"

    record = logging.LogRecord(
        name="dummy",
        level=logging.INFO,
        pathname="dummypath",
        lineno=10,
        msg="Test Message",
        args=(),
        exc_info=None,
    )

    tw = TerminalWriter()
    tw.hasmarkup = True
    formatter = ColoredLevelFormatter(tw, logfmt)
    output = formatter.format(record)
    assert output == (
        "dummypath                   10 \x1b[32mINFO    \x1b[0m Test Message"
    )

    tw.hasmarkup = False
    formatter = ColoredLevelFormatter(tw, logfmt)
    output = formatter.format(record)
    assert output == ("dummypath                   10 INFO     Test Message")


def test_multiline_message() -> None:
    from _pytest.logging import PercentStyleMultiline

    logfmt = "%(filename)-25s %(lineno)4d %(levelname)-8s %(message)s"

    record: Any = logging.LogRecord(
        name="dummy",
        level=logging.INFO,
        pathname="dummypath",
        lineno=10,
        msg="Test Message line1\nline2",
        args=(),
        exc_info=None,
    )
    # this is called by logging.Formatter.format
    record.message = record.getMessage()

    ai_on_style = PercentStyleMultiline(logfmt, True)
    output = ai_on_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\n"
        "                                        line2"
    )

    ai_off_style = PercentStyleMultiline(logfmt, False)
    output = ai_off_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\nline2"
    )

    ai_none_style = PercentStyleMultiline(logfmt, None)
    output = ai_none_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\nline2"
    )

    record.auto_indent = False
    output = ai_on_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\nline2"
    )

    record.auto_indent = True
    output = ai_off_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\n"
        "                                        line2"
    )

    record.auto_indent = "False"
    output = ai_on_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\nline2"
    )

    record.auto_indent = "True"
    output = ai_off_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\n"
        "                                        line2"
    )

    # bad string values default to False
    record.auto_indent = "junk"
    output = ai_off_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\nline2"
    )

    # anything other than string or int will default to False
    record.auto_indent = dict()
    output = ai_off_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\nline2"
    )

    record.auto_indent = "5"
    output = ai_off_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\n     line2"
    )

    record.auto_indent = 5
    output = ai_off_style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\n     line2"
    )


def test_colored_short_level() -> None:
    logfmt = "%(levelname).1s %(message)s"

    record = logging.LogRecord(
        name="dummy",
        level=logging.INFO,
        pathname="dummypath",
        lineno=10,
        msg="Test Message",
        args=(),
        exc_info=None,
    )

    class ColorConfig:
        class option:
            pass

    tw = TerminalWriter()
    tw.hasmarkup = True
    formatter = ColoredLevelFormatter(tw, logfmt)
    output = formatter.format(record)
    # the I (of INFO) is colored
    assert output == ("\x1b[32mI\x1b[0m Test Message")
