
import py
import os, sys
from py._io import terminalwriter
import codecs
import pytest

def test_get_terminal_width():
    x = py.io.get_terminal_width
    assert x == terminalwriter.get_terminal_width

def test_getdimensions(monkeypatch):
    fcntl = py.test.importorskip("fcntl")
    import struct
    l = []
    monkeypatch.setattr(fcntl, 'ioctl', lambda *args: l.append(args))
    try:
        terminalwriter._getdimensions()
    except (TypeError, struct.error):
        pass
    assert len(l) == 1
    assert l[0][0] == 1

def test_terminal_width_COLUMNS(monkeypatch):
    """ Dummy test for get_terminal_width
    """
    fcntl = py.test.importorskip("fcntl")
    monkeypatch.setattr(fcntl, 'ioctl', lambda *args: int('x'))
    monkeypatch.setenv('COLUMNS', '42')
    assert terminalwriter.get_terminal_width() == 42
    monkeypatch.delenv('COLUMNS', raising=False)

def test_terminalwriter_defaultwidth_80(monkeypatch):
    monkeypatch.setattr(terminalwriter, '_getdimensions', lambda: 0/0)
    monkeypatch.delenv('COLUMNS', raising=False)
    tw = py.io.TerminalWriter()
    assert tw.fullwidth == 80

def test_terminalwriter_getdimensions_bogus(monkeypatch):
    monkeypatch.setattr(terminalwriter, '_getdimensions', lambda: (10,10))
    monkeypatch.delenv('COLUMNS', raising=False)
    tw = py.io.TerminalWriter()
    assert tw.fullwidth == 80

def test_terminalwriter_getdimensions_emacs(monkeypatch):
    # emacs terminal returns (0,0) but set COLUMNS properly
    monkeypatch.setattr(terminalwriter, '_getdimensions', lambda: (0,0))
    monkeypatch.setenv('COLUMNS', '42')
    tw = py.io.TerminalWriter()
    assert tw.fullwidth == 42

def test_terminalwriter_computes_width(monkeypatch):
    monkeypatch.setattr(terminalwriter, 'get_terminal_width', lambda: 42)
    tw = py.io.TerminalWriter()
    assert tw.fullwidth == 42

def test_terminalwriter_default_instantiation():
    tw = py.io.TerminalWriter(stringio=True)
    assert hasattr(tw, 'stringio')

def test_terminalwriter_dumb_term_no_markup(monkeypatch):
    monkeypatch.setattr(os, 'environ', {'TERM': 'dumb', 'PATH': ''})
    class MyFile:
        closed = False
        def isatty(self):
            return True
    monkeypatch.setattr(sys, 'stdout', MyFile())
    try:
        assert sys.stdout.isatty()
        tw = py.io.TerminalWriter()
        assert not tw.hasmarkup
    finally:
        monkeypatch.undo()

def test_terminalwriter_file_unicode(tmpdir):
    f = codecs.open(str(tmpdir.join("xyz")), "wb", "utf8")
    tw = py.io.TerminalWriter(file=f)
    assert tw.encoding == "utf8"

def test_unicode_encoding():
    msg = py.builtin._totext('b\u00f6y', 'utf8')
    for encoding in 'utf8', 'latin1':
        l = []
        tw = py.io.TerminalWriter(l.append, encoding=encoding)
        tw.line(msg)
        assert l[0].strip() == msg.encode(encoding)

@pytest.mark.parametrize("encoding", ["ascii"])
def test_unicode_on_file_with_ascii_encoding(tmpdir, monkeypatch, encoding):
    msg = py.builtin._totext('hell\xf6', "latin1")
    #pytest.raises(UnicodeEncodeError, lambda: bytes(msg))
    f = codecs.open(str(tmpdir.join("x")), "w", encoding)
    tw = py.io.TerminalWriter(f)
    tw.line(msg)
    f.close()
    s = tmpdir.join("x").open("rb").read().strip()
    assert encoding == "ascii"
    assert s == msg.encode("unicode-escape")


win32 = int(sys.platform == "win32")
class TestTerminalWriter:
    def pytest_generate_tests(self, metafunc):
        if "tw" in metafunc.funcargnames:
            metafunc.addcall(id="path", param="path")
            metafunc.addcall(id="stringio", param="stringio")
            metafunc.addcall(id="callable", param="callable")
    def pytest_funcarg__tw(self, request):
        if request.param == "path":
            tmpdir = request.getfuncargvalue("tmpdir")
            p = tmpdir.join("tmpfile")
            f = codecs.open(str(p), 'w+', encoding='utf8')
            tw = py.io.TerminalWriter(f)
            def getlines():
                tw._file.flush()
                return codecs.open(str(p), 'r',
                    encoding='utf8').readlines()
        elif request.param == "stringio":
            tw = py.io.TerminalWriter(stringio=True)
            def getlines():
                tw.stringio.seek(0)
                return tw.stringio.readlines()
        elif request.param == "callable":
            writes = []
            tw = py.io.TerminalWriter(writes.append)
            def getlines():
                io = py.io.TextIO()
                io.write("".join(writes))
                io.seek(0)
                return io.readlines()
        tw.getlines = getlines
        tw.getvalue = lambda: "".join(getlines())
        return tw

    def test_line(self, tw):
        tw.line("hello")
        l = tw.getlines()
        assert len(l) == 1
        assert l[0] == "hello\n"

    def test_line_unicode(self, tw):
        for encoding in 'utf8', 'latin1':
            tw._encoding = encoding
            msg = py.builtin._totext('b\u00f6y', 'utf8')
            tw.line(msg)
            l = tw.getlines()
            assert l[0] == msg + "\n"

    def test_sep_no_title(self, tw):
        tw.sep("-", fullwidth=60)
        l = tw.getlines()
        assert len(l) == 1
        assert l[0] == "-" * (60-win32) + "\n"

    def test_sep_with_title(self, tw):
        tw.sep("-", "hello", fullwidth=60)
        l = tw.getlines()
        assert len(l) == 1
        assert l[0] == "-" * 26 + " hello " + "-" * (27-win32) + "\n"

    @py.test.mark.skipif("sys.platform == 'win32'")
    def test__escaped(self, tw):
        text2 = tw._escaped("hello", (31))
        assert text2.find("hello") != -1

    @py.test.mark.skipif("sys.platform == 'win32'")
    def test_markup(self, tw):
        for bold in (True, False):
            for color in ("red", "green"):
                text2 = tw.markup("hello", **{color: True, 'bold': bold})
                assert text2.find("hello") != -1
        py.test.raises(ValueError, "tw.markup('x', wronkw=3)")
        py.test.raises(ValueError, "tw.markup('x', wronkw=0)")

    def test_line_write_markup(self, tw):
        tw.hasmarkup = True
        tw.line("x", bold=True)
        tw.write("x\n", red=True)
        l = tw.getlines()
        if sys.platform != "win32":
            assert len(l[0]) >= 2, l
            assert len(l[1]) >= 2, l

    def test_attr_fullwidth(self, tw):
        tw.sep("-", "hello", fullwidth=70)
        tw.fullwidth = 70
        tw.sep("-", "hello")
        l = tw.getlines()
        assert len(l[0]) == len(l[1])

    def test_reline(self, tw):
        tw.line("hello")
        tw.hasmarkup = False
        pytest.raises(ValueError, lambda: tw.reline("x"))
        tw.hasmarkup = True
        tw.reline("0 1 2")
        tw.getlines()
        l = tw.getvalue().split("\n")
        assert len(l) == 2
        tw.reline("0 1 3")
        l = tw.getvalue().split("\n")
        assert len(l) == 2
        assert l[1].endswith("0 1 3\r")
        tw.line("so")
        l = tw.getvalue().split("\n")
        assert len(l) == 3
        assert l[-1] == ""
        assert l[1] == ("0 1 2\r0 1 3\rso   ")
        assert l[0] == "hello"


def test_terminal_with_callable_write_and_flush():
    l = set()
    class fil:
        flush = lambda self: l.add("1")
        write = lambda self, x: l.add("1")
        __call__ = lambda self, x: l.add("2")

    tw = py.io.TerminalWriter(fil())
    tw.line("hello")
    assert l == set(["1"])
    del fil.flush
    l.clear()
    tw = py.io.TerminalWriter(fil())
    tw.line("hello")
    assert l == set(["2"])


def test_chars_on_current_line():
    tw = py.io.TerminalWriter(stringio=True)

    written = []

    def write_and_check(s, expected):
        tw.write(s, bold=True)
        written.append(s)
        assert tw.chars_on_current_line == expected
        assert tw.stringio.getvalue() == ''.join(written)

    write_and_check('foo', 3)
    write_and_check('bar', 6)
    write_and_check('\n', 0)
    write_and_check('\n', 0)
    write_and_check('\n\n\n', 0)
    write_and_check('\nfoo', 3)
    write_and_check('\nfbar\nhello', 5)
    write_and_check('10', 7)


@pytest.mark.skipif(sys.platform == "win32", reason="win32 has no native ansi")
def test_attr_hasmarkup():
    tw = py.io.TerminalWriter(stringio=True)
    assert not tw.hasmarkup
    tw.hasmarkup = True
    tw.line("hello", bold=True)
    s = tw.stringio.getvalue()
    assert len(s) > len("hello\n")
    assert '\x1b[1m' in s
    assert '\x1b[0m' in s

@pytest.mark.skipif(sys.platform == "win32", reason="win32 has no native ansi")
def test_ansi_print():
    # we have no easy way to construct a file that
    # represents a terminal
    f = py.io.TextIO()
    f.isatty = lambda: True
    py.io.ansi_print("hello", 0x32, file=f)
    text2 = f.getvalue()
    assert text2.find("hello") != -1
    assert len(text2) >= len("hello\n")
    assert '\x1b[50m' in text2
    assert '\x1b[0m' in text2

def test_should_do_markup_PY_COLORS_eq_1(monkeypatch):
    monkeypatch.setitem(os.environ, 'PY_COLORS', '1')
    tw = py.io.TerminalWriter(stringio=True)
    assert tw.hasmarkup
    tw.line("hello", bold=True)
    s = tw.stringio.getvalue()
    assert len(s) > len("hello\n")
    assert '\x1b[1m' in s
    assert '\x1b[0m' in s

def test_should_do_markup_PY_COLORS_eq_0(monkeypatch):
    monkeypatch.setitem(os.environ, 'PY_COLORS', '0')
    f = py.io.TextIO()
    f.isatty = lambda: True
    tw = py.io.TerminalWriter(file=f)
    assert not tw.hasmarkup
    tw.line("hello", bold=True)
    s = f.getvalue()
    assert s == "hello\n"
