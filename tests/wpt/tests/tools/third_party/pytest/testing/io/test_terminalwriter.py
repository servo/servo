# mypy: allow-untyped-defs
import io
import os
from pathlib import Path
import re
import shutil
import sys
from typing import Generator
from typing import Optional
from unittest import mock

from _pytest._io import terminalwriter
from _pytest.monkeypatch import MonkeyPatch
import pytest


# These tests were initially copied from py 1.8.1.


def test_terminal_width_COLUMNS(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setenv("COLUMNS", "42")
    assert terminalwriter.get_terminal_width() == 42
    monkeypatch.delenv("COLUMNS", raising=False)


def test_terminalwriter_width_bogus(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setattr(shutil, "get_terminal_size", mock.Mock(return_value=(10, 10)))
    monkeypatch.delenv("COLUMNS", raising=False)
    tw = terminalwriter.TerminalWriter()
    assert tw.fullwidth == 80


def test_terminalwriter_computes_width(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setattr(terminalwriter, "get_terminal_width", lambda: 42)
    tw = terminalwriter.TerminalWriter()
    assert tw.fullwidth == 42


def test_terminalwriter_dumb_term_no_markup(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setattr(os, "environ", {"TERM": "dumb", "PATH": ""})

    class MyFile:
        closed = False

        def isatty(self):
            return True

    with monkeypatch.context() as m:
        m.setattr(sys, "stdout", MyFile())
        assert sys.stdout.isatty()
        tw = terminalwriter.TerminalWriter()
        assert not tw.hasmarkup


def test_terminalwriter_not_unicode() -> None:
    """If the file doesn't support Unicode, the string is unicode-escaped (#7475)."""
    buffer = io.BytesIO()
    file = io.TextIOWrapper(buffer, encoding="cp1252")
    tw = terminalwriter.TerminalWriter(file)
    tw.write("hello 🌀 wôrld אבג", flush=True)
    assert buffer.getvalue() == rb"hello \U0001f300 w\xf4rld \u05d0\u05d1\u05d2"


win32 = int(sys.platform == "win32")


class TestTerminalWriter:
    @pytest.fixture(params=["path", "stringio"])
    def tw(
        self, request, tmp_path: Path
    ) -> Generator[terminalwriter.TerminalWriter, None, None]:
        if request.param == "path":
            p = tmp_path.joinpath("tmpfile")
            f = open(str(p), "w+", encoding="utf8")
            tw = terminalwriter.TerminalWriter(f)

            def getlines():
                f.flush()
                with open(str(p), encoding="utf8") as fp:
                    return fp.readlines()

        elif request.param == "stringio":
            f = io.StringIO()
            tw = terminalwriter.TerminalWriter(f)

            def getlines():
                f.seek(0)
                return f.readlines()

        tw.getlines = getlines  # type: ignore
        tw.getvalue = lambda: "".join(getlines())  # type: ignore

        with f:
            yield tw

    def test_line(self, tw) -> None:
        tw.line("hello")
        lines = tw.getlines()
        assert len(lines) == 1
        assert lines[0] == "hello\n"

    def test_line_unicode(self, tw) -> None:
        msg = "b\u00f6y"
        tw.line(msg)
        lines = tw.getlines()
        assert lines[0] == msg + "\n"

    def test_sep_no_title(self, tw) -> None:
        tw.sep("-", fullwidth=60)
        lines = tw.getlines()
        assert len(lines) == 1
        assert lines[0] == "-" * (60 - win32) + "\n"

    def test_sep_with_title(self, tw) -> None:
        tw.sep("-", "hello", fullwidth=60)
        lines = tw.getlines()
        assert len(lines) == 1
        assert lines[0] == "-" * 26 + " hello " + "-" * (27 - win32) + "\n"

    def test_sep_longer_than_width(self, tw) -> None:
        tw.sep("-", "a" * 10, fullwidth=5)
        (line,) = tw.getlines()
        # even though the string is wider than the line, still have a separator
        assert line == "- aaaaaaaaaa -\n"

    @pytest.mark.skipif(sys.platform == "win32", reason="win32 has no native ansi")
    @pytest.mark.parametrize("bold", (True, False))
    @pytest.mark.parametrize("color", ("red", "green"))
    def test_markup(self, tw, bold: bool, color: str) -> None:
        text = tw.markup("hello", **{color: True, "bold": bold})
        assert "hello" in text

    def test_markup_bad(self, tw) -> None:
        with pytest.raises(ValueError):
            tw.markup("x", wronkw=3)
        with pytest.raises(ValueError):
            tw.markup("x", wronkw=0)

    def test_line_write_markup(self, tw) -> None:
        tw.hasmarkup = True
        tw.line("x", bold=True)
        tw.write("x\n", red=True)
        lines = tw.getlines()
        if sys.platform != "win32":
            assert len(lines[0]) >= 2, lines
            assert len(lines[1]) >= 2, lines

    def test_attr_fullwidth(self, tw) -> None:
        tw.sep("-", "hello", fullwidth=70)
        tw.fullwidth = 70
        tw.sep("-", "hello")
        lines = tw.getlines()
        assert len(lines[0]) == len(lines[1])


@pytest.mark.skipif(sys.platform == "win32", reason="win32 has no native ansi")
def test_attr_hasmarkup() -> None:
    file = io.StringIO()
    tw = terminalwriter.TerminalWriter(file)
    assert not tw.hasmarkup
    tw.hasmarkup = True
    tw.line("hello", bold=True)
    s = file.getvalue()
    assert len(s) > len("hello\n")
    assert "\x1b[1m" in s
    assert "\x1b[0m" in s


def assert_color(expected: bool, default: Optional[bool] = None) -> None:
    file = io.StringIO()
    if default is None:
        default = not expected
    file.isatty = lambda: default  # type: ignore
    tw = terminalwriter.TerminalWriter(file=file)
    assert tw.hasmarkup is expected
    tw.line("hello", bold=True)
    s = file.getvalue()
    if expected:
        assert len(s) > len("hello\n")
        assert "\x1b[1m" in s
        assert "\x1b[0m" in s
    else:
        assert s == "hello\n"


def test_should_do_markup_PY_COLORS_eq_1(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setitem(os.environ, "PY_COLORS", "1")
    assert_color(True)


def test_should_not_do_markup_PY_COLORS_eq_0(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setitem(os.environ, "PY_COLORS", "0")
    assert_color(False)


def test_should_not_do_markup_NO_COLOR(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setitem(os.environ, "NO_COLOR", "1")
    assert_color(False)


def test_should_do_markup_FORCE_COLOR(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setitem(os.environ, "FORCE_COLOR", "1")
    assert_color(True)


@pytest.mark.parametrize(
    ["NO_COLOR", "FORCE_COLOR", "expected"],
    [
        ("1", "1", False),
        ("", "1", True),
        ("1", "", False),
    ],
)
def test_NO_COLOR_and_FORCE_COLOR(
    monkeypatch: MonkeyPatch,
    NO_COLOR: str,
    FORCE_COLOR: str,
    expected: bool,
) -> None:
    monkeypatch.setitem(os.environ, "NO_COLOR", NO_COLOR)
    monkeypatch.setitem(os.environ, "FORCE_COLOR", FORCE_COLOR)
    assert_color(expected)


def test_empty_NO_COLOR_and_FORCE_COLOR_ignored(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setitem(os.environ, "NO_COLOR", "")
    monkeypatch.setitem(os.environ, "FORCE_COLOR", "")
    assert_color(True, True)
    assert_color(False, False)


class TestTerminalWriterLineWidth:
    def test_init(self) -> None:
        tw = terminalwriter.TerminalWriter()
        assert tw.width_of_current_line == 0

    def test_update(self) -> None:
        tw = terminalwriter.TerminalWriter()
        tw.write("hello world")
        assert tw.width_of_current_line == 11

    def test_update_with_newline(self) -> None:
        tw = terminalwriter.TerminalWriter()
        tw.write("hello\nworld")
        assert tw.width_of_current_line == 5

    def test_update_with_wide_text(self) -> None:
        tw = terminalwriter.TerminalWriter()
        tw.write("乇乂ㄒ尺卂 ㄒ卄丨匚匚")
        assert tw.width_of_current_line == 21  # 5*2 + 1 + 5*2

    def test_composed(self) -> None:
        tw = terminalwriter.TerminalWriter()
        text = "café food"
        assert len(text) == 9
        tw.write(text)
        assert tw.width_of_current_line == 9

    def test_combining(self) -> None:
        tw = terminalwriter.TerminalWriter()
        text = "café food"
        assert len(text) == 10
        tw.write(text)
        assert tw.width_of_current_line == 9


@pytest.mark.parametrize(
    ("has_markup", "code_highlight", "expected"),
    [
        pytest.param(
            True,
            True,
            "{reset}{kw}assert{hl-reset} {number}0{hl-reset}{endline}\n",
            id="with markup and code_highlight",
        ),
        pytest.param(
            True,
            False,
            "assert 0\n",
            id="with markup but no code_highlight",
        ),
        pytest.param(
            False,
            True,
            "assert 0\n",
            id="without markup but with code_highlight",
        ),
        pytest.param(
            False,
            False,
            "assert 0\n",
            id="neither markup nor code_highlight",
        ),
    ],
)
def test_code_highlight(has_markup, code_highlight, expected, color_mapping):
    f = io.StringIO()
    tw = terminalwriter.TerminalWriter(f)
    tw.hasmarkup = has_markup
    tw.code_highlight = code_highlight
    tw._write_source(["assert 0"])

    assert f.getvalue().splitlines(keepends=True) == color_mapping.format([expected])

    with pytest.raises(
        ValueError,
        match=re.escape("indents size (2) should have same size as lines (1)"),
    ):
        tw._write_source(["assert 0"], [" ", " "])


def test_highlight_empty_source() -> None:
    """Don't crash trying to highlight empty source code.

    Issue #11758.
    """
    f = io.StringIO()
    tw = terminalwriter.TerminalWriter(f)
    tw.hasmarkup = True
    tw.code_highlight = True
    tw._write_source([])

    assert f.getvalue() == ""
