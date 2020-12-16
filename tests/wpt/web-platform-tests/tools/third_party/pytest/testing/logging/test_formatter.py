# -*- coding: utf-8 -*-
import logging

import py.io
import six

import pytest
from _pytest.logging import ColoredLevelFormatter


def test_coloredlogformatter():
    logfmt = "%(filename)-25s %(lineno)4d %(levelname)-8s %(message)s"

    record = logging.LogRecord(
        name="dummy",
        level=logging.INFO,
        pathname="dummypath",
        lineno=10,
        msg="Test Message",
        args=(),
        exc_info=False,
    )

    class ColorConfig(object):
        class option(object):
            pass

    tw = py.io.TerminalWriter()
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


@pytest.mark.skipif(
    six.PY2, reason="Formatter classes don't support format styles in PY2"
)
def test_multiline_message():
    from _pytest.logging import PercentStyleMultiline

    logfmt = "%(filename)-25s %(lineno)4d %(levelname)-8s %(message)s"

    record = logging.LogRecord(
        name="dummy",
        level=logging.INFO,
        pathname="dummypath",
        lineno=10,
        msg="Test Message line1\nline2",
        args=(),
        exc_info=False,
    )
    # this is called by logging.Formatter.format
    record.message = record.getMessage()

    style = PercentStyleMultiline(logfmt)
    output = style.format(record)
    assert output == (
        "dummypath                   10 INFO     Test Message line1\n"
        "                                        line2"
    )
