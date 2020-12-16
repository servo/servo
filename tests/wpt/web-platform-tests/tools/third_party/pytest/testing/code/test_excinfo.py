# -*- coding: utf-8 -*-
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import operator
import os
import sys
import textwrap

import py
import six
from six.moves import queue

import _pytest
import pytest
from _pytest._code.code import ExceptionChainRepr
from _pytest._code.code import ExceptionInfo
from _pytest._code.code import FormattedExcinfo
from _pytest._code.code import ReprExceptionInfo

try:
    import importlib
except ImportError:
    invalidate_import_caches = None
else:
    invalidate_import_caches = getattr(importlib, "invalidate_caches", None)

failsonjython = pytest.mark.xfail("sys.platform.startswith('java')")

pytest_version_info = tuple(map(int, pytest.__version__.split(".")[:3]))


@pytest.fixture
def limited_recursion_depth():
    before = sys.getrecursionlimit()
    sys.setrecursionlimit(150)
    yield
    sys.setrecursionlimit(before)


class TWMock(object):
    WRITE = object()

    def __init__(self):
        self.lines = []
        self.is_writing = False

    def sep(self, sep, line=None):
        self.lines.append((sep, line))

    def write(self, msg, **kw):
        self.lines.append((TWMock.WRITE, msg))

    def line(self, line, **kw):
        self.lines.append(line)

    def markup(self, text, **kw):
        return text

    def get_write_msg(self, idx):
        flag, msg = self.lines[idx]
        assert flag == TWMock.WRITE
        return msg

    fullwidth = 80


def test_excinfo_simple():
    try:
        raise ValueError
    except ValueError:
        info = _pytest._code.ExceptionInfo.from_current()
    assert info.type == ValueError


def test_excinfo_getstatement():
    def g():
        raise ValueError

    def f():
        g()

    try:
        f()
    except ValueError:
        excinfo = _pytest._code.ExceptionInfo.from_current()
    linenumbers = [
        _pytest._code.getrawcode(f).co_firstlineno - 1 + 4,
        _pytest._code.getrawcode(f).co_firstlineno - 1 + 1,
        _pytest._code.getrawcode(g).co_firstlineno - 1 + 1,
    ]
    values = list(excinfo.traceback)
    foundlinenumbers = [x.lineno for x in values]
    assert foundlinenumbers == linenumbers
    # for x in info:
    #    print "%s:%d  %s" %(x.path.relto(root), x.lineno, x.statement)
    # xxx


# testchain for getentries test below


def f():
    #
    raise ValueError
    #


def g():
    #
    __tracebackhide__ = True
    f()
    #


def h():
    #
    g()
    #


class TestTraceback_f_g_h(object):
    def setup_method(self, method):
        try:
            h()
        except ValueError:
            self.excinfo = _pytest._code.ExceptionInfo.from_current()

    def test_traceback_entries(self):
        tb = self.excinfo.traceback
        entries = list(tb)
        assert len(tb) == 4  # maybe fragile test
        assert len(entries) == 4  # maybe fragile test
        names = ["f", "g", "h"]
        for entry in entries:
            try:
                names.remove(entry.frame.code.name)
            except ValueError:
                pass
        assert not names

    def test_traceback_entry_getsource(self):
        tb = self.excinfo.traceback
        s = str(tb[-1].getsource())
        assert s.startswith("def f():")
        assert s.endswith("raise ValueError")

    @failsonjython
    def test_traceback_entry_getsource_in_construct(self):
        source = _pytest._code.Source(
            """\
            def xyz():
                try:
                    raise ValueError
                except somenoname:
                    pass
            xyz()
            """
        )
        try:
            exec(source.compile())
        except NameError:
            tb = _pytest._code.ExceptionInfo.from_current().traceback
            print(tb[-1].getsource())
            s = str(tb[-1].getsource())
            assert s.startswith("def xyz():\n    try:")
            assert s.strip().endswith("except somenoname:")

    def test_traceback_cut(self):
        co = _pytest._code.Code(f)
        path, firstlineno = co.path, co.firstlineno
        traceback = self.excinfo.traceback
        newtraceback = traceback.cut(path=path, firstlineno=firstlineno)
        assert len(newtraceback) == 1
        newtraceback = traceback.cut(path=path, lineno=firstlineno + 2)
        assert len(newtraceback) == 1

    def test_traceback_cut_excludepath(self, testdir):
        p = testdir.makepyfile("def f(): raise ValueError")
        with pytest.raises(ValueError) as excinfo:
            p.pyimport().f()
        basedir = py.path.local(pytest.__file__).dirpath()
        newtraceback = excinfo.traceback.cut(excludepath=basedir)
        for x in newtraceback:
            if hasattr(x, "path"):
                assert not py.path.local(x.path).relto(basedir)
        assert newtraceback[-1].frame.code.path == p

    def test_traceback_filter(self):
        traceback = self.excinfo.traceback
        ntraceback = traceback.filter()
        assert len(ntraceback) == len(traceback) - 1

    @pytest.mark.parametrize(
        "tracebackhide, matching",
        [
            (lambda info: True, True),
            (lambda info: False, False),
            (operator.methodcaller("errisinstance", ValueError), True),
            (operator.methodcaller("errisinstance", IndexError), False),
        ],
    )
    def test_traceback_filter_selective(self, tracebackhide, matching):
        def f():
            #
            raise ValueError
            #

        def g():
            #
            __tracebackhide__ = tracebackhide
            f()
            #

        def h():
            #
            g()
            #

        excinfo = pytest.raises(ValueError, h)
        traceback = excinfo.traceback
        ntraceback = traceback.filter()
        print("old: {!r}".format(traceback))
        print("new: {!r}".format(ntraceback))

        if matching:
            assert len(ntraceback) == len(traceback) - 2
        else:
            # -1 because of the __tracebackhide__ in pytest.raises
            assert len(ntraceback) == len(traceback) - 1

    def test_traceback_recursion_index(self):
        def f(n):
            if n < 10:
                n += 1
            f(n)

        excinfo = pytest.raises(RuntimeError, f, 8)
        traceback = excinfo.traceback
        recindex = traceback.recursionindex()
        assert recindex == 3

    def test_traceback_only_specific_recursion_errors(self, monkeypatch):
        def f(n):
            if n == 0:
                raise RuntimeError("hello")
            f(n - 1)

        excinfo = pytest.raises(RuntimeError, f, 25)
        monkeypatch.delattr(excinfo.traceback.__class__, "recursionindex")
        repr = excinfo.getrepr()
        assert "RuntimeError: hello" in str(repr.reprcrash)

    def test_traceback_no_recursion_index(self):
        def do_stuff():
            raise RuntimeError

        def reraise_me():
            import sys

            exc, val, tb = sys.exc_info()
            six.reraise(exc, val, tb)

        def f(n):
            try:
                do_stuff()
            except:  # noqa
                reraise_me()

        excinfo = pytest.raises(RuntimeError, f, 8)
        traceback = excinfo.traceback
        recindex = traceback.recursionindex()
        assert recindex is None

    def test_traceback_messy_recursion(self):
        # XXX: simplified locally testable version
        decorator = pytest.importorskip("decorator").decorator

        def log(f, *k, **kw):
            print("{} {}".format(k, kw))
            f(*k, **kw)

        log = decorator(log)

        def fail():
            raise ValueError("")

        fail = log(log(fail))

        excinfo = pytest.raises(ValueError, fail)
        assert excinfo.traceback.recursionindex() is None

    def test_traceback_getcrashentry(self):
        def i():
            __tracebackhide__ = True
            raise ValueError

        def h():
            i()

        def g():
            __tracebackhide__ = True
            h()

        def f():
            g()

        excinfo = pytest.raises(ValueError, f)
        tb = excinfo.traceback
        entry = tb.getcrashentry()
        co = _pytest._code.Code(h)
        assert entry.frame.code.path == co.path
        assert entry.lineno == co.firstlineno + 1
        assert entry.frame.code.name == "h"

    def test_traceback_getcrashentry_empty(self):
        def g():
            __tracebackhide__ = True
            raise ValueError

        def f():
            __tracebackhide__ = True
            g()

        excinfo = pytest.raises(ValueError, f)
        tb = excinfo.traceback
        entry = tb.getcrashentry()
        co = _pytest._code.Code(g)
        assert entry.frame.code.path == co.path
        assert entry.lineno == co.firstlineno + 2
        assert entry.frame.code.name == "g"


def test_excinfo_exconly():
    excinfo = pytest.raises(ValueError, h)
    assert excinfo.exconly().startswith("ValueError")
    with pytest.raises(ValueError) as excinfo:
        raise ValueError("hello\nworld")
    msg = excinfo.exconly(tryshort=True)
    assert msg.startswith("ValueError")
    assert msg.endswith("world")


def test_excinfo_repr():
    excinfo = pytest.raises(ValueError, h)
    s = repr(excinfo)
    assert s == "<ExceptionInfo ValueError tblen=4>"


def test_excinfo_str():
    excinfo = pytest.raises(ValueError, h)
    s = str(excinfo)
    assert s.startswith(__file__[:-9])  # pyc file and $py.class
    assert s.endswith("ValueError")
    assert len(s.split(":")) >= 3  # on windows it's 4


def test_excinfo_for_later():
    e = ExceptionInfo.for_later()
    assert "for raises" in repr(e)
    assert "for raises" in str(e)


def test_excinfo_errisinstance():
    excinfo = pytest.raises(ValueError, h)
    assert excinfo.errisinstance(ValueError)


def test_excinfo_no_sourcecode():
    try:
        exec("raise ValueError()")
    except ValueError:
        excinfo = _pytest._code.ExceptionInfo.from_current()
    s = str(excinfo.traceback[-1])
    assert s == "  File '<string>':1 in <module>\n  ???\n"


def test_excinfo_no_python_sourcecode(tmpdir):
    # XXX: simplified locally testable version
    tmpdir.join("test.txt").write("{{ h()}}:")

    jinja2 = pytest.importorskip("jinja2")
    loader = jinja2.FileSystemLoader(str(tmpdir))
    env = jinja2.Environment(loader=loader)
    template = env.get_template("test.txt")
    excinfo = pytest.raises(ValueError, template.render, h=h)
    for item in excinfo.traceback:
        print(item)  # XXX: for some reason jinja.Template.render is printed in full
        item.source  # shouldnt fail
        if item.path.basename == "test.txt":
            assert str(item.source) == "{{ h()}}:"


def test_entrysource_Queue_example():
    try:
        queue.Queue().get(timeout=0.001)
    except queue.Empty:
        excinfo = _pytest._code.ExceptionInfo.from_current()
    entry = excinfo.traceback[-1]
    source = entry.getsource()
    assert source is not None
    s = str(source).strip()
    assert s.startswith("def get")


def test_codepath_Queue_example():
    try:
        queue.Queue().get(timeout=0.001)
    except queue.Empty:
        excinfo = _pytest._code.ExceptionInfo.from_current()
    entry = excinfo.traceback[-1]
    path = entry.path
    assert isinstance(path, py.path.local)
    assert path.basename.lower() == "queue.py"
    assert path.check()


def test_match_succeeds():
    with pytest.raises(ZeroDivisionError) as excinfo:
        0 // 0
    excinfo.match(r".*zero.*")


def test_match_raises_error(testdir):
    testdir.makepyfile(
        """
        import pytest
        def test_division_zero():
            with pytest.raises(ZeroDivisionError) as excinfo:
                0 / 0
            excinfo.match(r'[123]+')
    """
    )
    result = testdir.runpytest()
    assert result.ret != 0
    result.stdout.fnmatch_lines(["*AssertionError*Pattern*[123]*not found*"])


class TestFormattedExcinfo(object):
    @pytest.fixture
    def importasmod(self, request, _sys_snapshot):
        def importasmod(source):
            source = textwrap.dedent(source)
            tmpdir = request.getfixturevalue("tmpdir")
            modpath = tmpdir.join("mod.py")
            tmpdir.ensure("__init__.py")
            modpath.write(source)
            if invalidate_import_caches is not None:
                invalidate_import_caches()
            return modpath.pyimport()

        return importasmod

    def excinfo_from_exec(self, source):
        source = _pytest._code.Source(source).strip()
        try:
            exec(source.compile())
        except KeyboardInterrupt:
            raise
        except:  # noqa
            return _pytest._code.ExceptionInfo.from_current()
        assert 0, "did not raise"

    def test_repr_source(self):
        pr = FormattedExcinfo()
        source = _pytest._code.Source(
            """\
            def f(x):
                pass
            """
        ).strip()
        pr.flow_marker = "|"
        lines = pr.get_source(source, 0)
        assert len(lines) == 2
        assert lines[0] == "|   def f(x):"
        assert lines[1] == "        pass"

    def test_repr_source_excinfo(self):
        """ check if indentation is right """
        pr = FormattedExcinfo()
        excinfo = self.excinfo_from_exec(
            """
                def f():
                    assert 0
                f()
        """
        )
        pr = FormattedExcinfo()
        source = pr._getentrysource(excinfo.traceback[-1])
        lines = pr.get_source(source, 1, excinfo)
        assert lines == ["    def f():", ">       assert 0", "E       AssertionError"]

    def test_repr_source_not_existing(self):
        pr = FormattedExcinfo()
        co = compile("raise ValueError()", "", "exec")
        try:
            exec(co)
        except ValueError:
            excinfo = _pytest._code.ExceptionInfo.from_current()
        repr = pr.repr_excinfo(excinfo)
        assert repr.reprtraceback.reprentries[1].lines[0] == ">   ???"
        if sys.version_info[0] >= 3:
            assert repr.chain[0][0].reprentries[1].lines[0] == ">   ???"

    def test_repr_many_line_source_not_existing(self):
        pr = FormattedExcinfo()
        co = compile(
            """
a = 1
raise ValueError()
""",
            "",
            "exec",
        )
        try:
            exec(co)
        except ValueError:
            excinfo = _pytest._code.ExceptionInfo.from_current()
        repr = pr.repr_excinfo(excinfo)
        assert repr.reprtraceback.reprentries[1].lines[0] == ">   ???"
        if sys.version_info[0] >= 3:
            assert repr.chain[0][0].reprentries[1].lines[0] == ">   ???"

    def test_repr_source_failing_fullsource(self):
        pr = FormattedExcinfo()

        class FakeCode(object):
            class raw(object):
                co_filename = "?"

            path = "?"
            firstlineno = 5

            def fullsource(self):
                return None

            fullsource = property(fullsource)

        class FakeFrame(object):
            code = FakeCode()
            f_locals = {}
            f_globals = {}

        class FakeTracebackEntry(_pytest._code.Traceback.Entry):
            def __init__(self, tb, excinfo=None):
                self.lineno = 5 + 3

            @property
            def frame(self):
                return FakeFrame()

        class Traceback(_pytest._code.Traceback):
            Entry = FakeTracebackEntry

        class FakeExcinfo(_pytest._code.ExceptionInfo):
            typename = "Foo"
            value = Exception()

            def __init__(self):
                pass

            def exconly(self, tryshort):
                return "EXC"

            def errisinstance(self, cls):
                return False

        excinfo = FakeExcinfo()

        class FakeRawTB(object):
            tb_next = None

        tb = FakeRawTB()
        excinfo.traceback = Traceback(tb)

        fail = IOError()
        repr = pr.repr_excinfo(excinfo)
        assert repr.reprtraceback.reprentries[0].lines[0] == ">   ???"
        if sys.version_info[0] >= 3:
            assert repr.chain[0][0].reprentries[0].lines[0] == ">   ???"

        fail = py.error.ENOENT  # noqa
        repr = pr.repr_excinfo(excinfo)
        assert repr.reprtraceback.reprentries[0].lines[0] == ">   ???"
        if sys.version_info[0] >= 3:
            assert repr.chain[0][0].reprentries[0].lines[0] == ">   ???"

    def test_repr_local(self):
        p = FormattedExcinfo(showlocals=True)
        loc = {"y": 5, "z": 7, "x": 3, "@x": 2, "__builtins__": {}}
        reprlocals = p.repr_locals(loc)
        assert reprlocals.lines
        assert reprlocals.lines[0] == "__builtins__ = <builtins>"
        assert reprlocals.lines[1] == "x          = 3"
        assert reprlocals.lines[2] == "y          = 5"
        assert reprlocals.lines[3] == "z          = 7"

    def test_repr_local_with_error(self):
        class ObjWithErrorInRepr:
            def __repr__(self):
                raise NotImplementedError

        p = FormattedExcinfo(showlocals=True, truncate_locals=False)
        loc = {"x": ObjWithErrorInRepr(), "__builtins__": {}}
        reprlocals = p.repr_locals(loc)
        assert reprlocals.lines
        assert reprlocals.lines[0] == "__builtins__ = <builtins>"
        assert '[NotImplementedError("") raised in repr()]' in reprlocals.lines[1]

    def test_repr_local_with_exception_in_class_property(self):
        class ExceptionWithBrokenClass(Exception):
            @property
            def __class__(self):
                raise TypeError("boom!")

        class ObjWithErrorInRepr:
            def __repr__(self):
                raise ExceptionWithBrokenClass()

        p = FormattedExcinfo(showlocals=True, truncate_locals=False)
        loc = {"x": ObjWithErrorInRepr(), "__builtins__": {}}
        reprlocals = p.repr_locals(loc)
        assert reprlocals.lines
        assert reprlocals.lines[0] == "__builtins__ = <builtins>"
        assert '[ExceptionWithBrokenClass("") raised in repr()]' in reprlocals.lines[1]

    def test_repr_local_truncated(self):
        loc = {"l": [i for i in range(10)]}
        p = FormattedExcinfo(showlocals=True)
        truncated_reprlocals = p.repr_locals(loc)
        assert truncated_reprlocals.lines
        assert truncated_reprlocals.lines[0] == "l          = [0, 1, 2, 3, 4, 5, ...]"

        q = FormattedExcinfo(showlocals=True, truncate_locals=False)
        full_reprlocals = q.repr_locals(loc)
        assert full_reprlocals.lines
        assert full_reprlocals.lines[0] == "l          = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"

    def test_repr_tracebackentry_lines(self, importasmod):
        mod = importasmod(
            """
            def func1():
                raise ValueError("hello\\nworld")
        """
        )
        excinfo = pytest.raises(ValueError, mod.func1)
        excinfo.traceback = excinfo.traceback.filter()
        p = FormattedExcinfo()
        reprtb = p.repr_traceback_entry(excinfo.traceback[-1])

        # test as intermittent entry
        lines = reprtb.lines
        assert lines[0] == "    def func1():"
        assert lines[1] == '>       raise ValueError("hello\\nworld")'

        # test as last entry
        p = FormattedExcinfo(showlocals=True)
        repr_entry = p.repr_traceback_entry(excinfo.traceback[-1], excinfo)
        lines = repr_entry.lines
        assert lines[0] == "    def func1():"
        assert lines[1] == '>       raise ValueError("hello\\nworld")'
        assert lines[2] == "E       ValueError: hello"
        assert lines[3] == "E       world"
        assert not lines[4:]

        loc = repr_entry.reprlocals is not None
        loc = repr_entry.reprfileloc
        assert loc.path == mod.__file__
        assert loc.lineno == 3
        # assert loc.message == "ValueError: hello"

    def test_repr_tracebackentry_lines2(self, importasmod):
        mod = importasmod(
            """
            def func1(m, x, y, z):
                raise ValueError("hello\\nworld")
        """
        )
        excinfo = pytest.raises(ValueError, mod.func1, "m" * 90, 5, 13, "z" * 120)
        excinfo.traceback = excinfo.traceback.filter()
        entry = excinfo.traceback[-1]
        p = FormattedExcinfo(funcargs=True)
        reprfuncargs = p.repr_args(entry)
        assert reprfuncargs.args[0] == ("m", repr("m" * 90))
        assert reprfuncargs.args[1] == ("x", "5")
        assert reprfuncargs.args[2] == ("y", "13")
        assert reprfuncargs.args[3] == ("z", repr("z" * 120))

        p = FormattedExcinfo(funcargs=True)
        repr_entry = p.repr_traceback_entry(entry)
        assert repr_entry.reprfuncargs.args == reprfuncargs.args
        tw = TWMock()
        repr_entry.toterminal(tw)
        assert tw.lines[0] == "m = " + repr("m" * 90)
        assert tw.lines[1] == "x = 5, y = 13"
        assert tw.lines[2] == "z = " + repr("z" * 120)

    def test_repr_tracebackentry_lines_var_kw_args(self, importasmod):
        mod = importasmod(
            """
            def func1(x, *y, **z):
                raise ValueError("hello\\nworld")
        """
        )
        excinfo = pytest.raises(ValueError, mod.func1, "a", "b", c="d")
        excinfo.traceback = excinfo.traceback.filter()
        entry = excinfo.traceback[-1]
        p = FormattedExcinfo(funcargs=True)
        reprfuncargs = p.repr_args(entry)
        assert reprfuncargs.args[0] == ("x", repr("a"))
        assert reprfuncargs.args[1] == ("y", repr(("b",)))
        assert reprfuncargs.args[2] == ("z", repr({"c": "d"}))

        p = FormattedExcinfo(funcargs=True)
        repr_entry = p.repr_traceback_entry(entry)
        assert repr_entry.reprfuncargs.args == reprfuncargs.args
        tw = TWMock()
        repr_entry.toterminal(tw)
        assert tw.lines[0] == "x = 'a', y = ('b',), z = {'c': 'd'}"

    def test_repr_tracebackentry_short(self, importasmod):
        mod = importasmod(
            """
            def func1():
                raise ValueError("hello")
            def entry():
                func1()
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)
        p = FormattedExcinfo(style="short")
        reprtb = p.repr_traceback_entry(excinfo.traceback[-2])
        lines = reprtb.lines
        basename = py.path.local(mod.__file__).basename
        assert lines[0] == "    func1()"
        assert basename in str(reprtb.reprfileloc.path)
        assert reprtb.reprfileloc.lineno == 5

        # test last entry
        p = FormattedExcinfo(style="short")
        reprtb = p.repr_traceback_entry(excinfo.traceback[-1], excinfo)
        lines = reprtb.lines
        assert lines[0] == '    raise ValueError("hello")'
        assert lines[1] == "E   ValueError: hello"
        assert basename in str(reprtb.reprfileloc.path)
        assert reprtb.reprfileloc.lineno == 3

    def test_repr_tracebackentry_no(self, importasmod):
        mod = importasmod(
            """
            def func1():
                raise ValueError("hello")
            def entry():
                func1()
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)
        p = FormattedExcinfo(style="no")
        p.repr_traceback_entry(excinfo.traceback[-2])

        p = FormattedExcinfo(style="no")
        reprentry = p.repr_traceback_entry(excinfo.traceback[-1], excinfo)
        lines = reprentry.lines
        assert lines[0] == "E   ValueError: hello"
        assert not lines[1:]

    def test_repr_traceback_tbfilter(self, importasmod):
        mod = importasmod(
            """
            def f(x):
                raise ValueError(x)
            def entry():
                f(0)
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)
        p = FormattedExcinfo(tbfilter=True)
        reprtb = p.repr_traceback(excinfo)
        assert len(reprtb.reprentries) == 2
        p = FormattedExcinfo(tbfilter=False)
        reprtb = p.repr_traceback(excinfo)
        assert len(reprtb.reprentries) == 3

    def test_traceback_short_no_source(self, importasmod, monkeypatch):
        mod = importasmod(
            """
            def func1():
                raise ValueError("hello")
            def entry():
                func1()
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)
        from _pytest._code.code import Code

        monkeypatch.setattr(Code, "path", "bogus")
        excinfo.traceback[0].frame.code.path = "bogus"
        p = FormattedExcinfo(style="short")
        reprtb = p.repr_traceback_entry(excinfo.traceback[-2])
        lines = reprtb.lines
        last_p = FormattedExcinfo(style="short")
        last_reprtb = last_p.repr_traceback_entry(excinfo.traceback[-1], excinfo)
        last_lines = last_reprtb.lines
        monkeypatch.undo()
        assert lines[0] == "    func1()"

        assert last_lines[0] == '    raise ValueError("hello")'
        assert last_lines[1] == "E   ValueError: hello"

    def test_repr_traceback_and_excinfo(self, importasmod):
        mod = importasmod(
            """
            def f(x):
                raise ValueError(x)
            def entry():
                f(0)
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)

        for style in ("long", "short"):
            p = FormattedExcinfo(style=style)
            reprtb = p.repr_traceback(excinfo)
            assert len(reprtb.reprentries) == 2
            assert reprtb.style == style
            assert not reprtb.extraline
            repr = p.repr_excinfo(excinfo)
            assert repr.reprtraceback
            assert len(repr.reprtraceback.reprentries) == len(reprtb.reprentries)
            if sys.version_info[0] >= 3:
                assert repr.chain[0][0]
                assert len(repr.chain[0][0].reprentries) == len(reprtb.reprentries)
            assert repr.reprcrash.path.endswith("mod.py")
            assert repr.reprcrash.message == "ValueError: 0"

    def test_repr_traceback_with_invalid_cwd(self, importasmod, monkeypatch):
        mod = importasmod(
            """
            def f(x):
                raise ValueError(x)
            def entry():
                f(0)
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)

        p = FormattedExcinfo()

        def raiseos():
            raise OSError(2)

        monkeypatch.setattr(os, "getcwd", raiseos)
        assert p._makepath(__file__) == __file__
        p.repr_traceback(excinfo)

    def test_repr_excinfo_addouterr(self, importasmod):
        mod = importasmod(
            """
            def entry():
                raise ValueError()
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)
        repr = excinfo.getrepr()
        repr.addsection("title", "content")
        twmock = TWMock()
        repr.toterminal(twmock)
        assert twmock.lines[-1] == "content"
        assert twmock.lines[-2] == ("-", "title")

    def test_repr_excinfo_reprcrash(self, importasmod):
        mod = importasmod(
            """
            def entry():
                raise ValueError()
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)
        repr = excinfo.getrepr()
        assert repr.reprcrash.path.endswith("mod.py")
        assert repr.reprcrash.lineno == 3
        assert repr.reprcrash.message == "ValueError"
        assert str(repr.reprcrash).endswith("mod.py:3: ValueError")

    def test_repr_traceback_recursion(self, importasmod):
        mod = importasmod(
            """
            def rec2(x):
                return rec1(x+1)
            def rec1(x):
                return rec2(x-1)
            def entry():
                rec1(42)
        """
        )
        excinfo = pytest.raises(RuntimeError, mod.entry)

        for style in ("short", "long", "no"):
            p = FormattedExcinfo(style="short")
            reprtb = p.repr_traceback(excinfo)
            assert reprtb.extraline == "!!! Recursion detected (same locals & position)"
            assert str(reprtb)

    def test_reprexcinfo_getrepr(self, importasmod):
        mod = importasmod(
            """
            def f(x):
                raise ValueError(x)
            def entry():
                f(0)
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)

        for style in ("short", "long", "no"):
            for showlocals in (True, False):
                repr = excinfo.getrepr(style=style, showlocals=showlocals)
                if sys.version_info[0] < 3:
                    assert isinstance(repr, ReprExceptionInfo)
                assert repr.reprtraceback.style == style
                if sys.version_info[0] >= 3:
                    assert isinstance(repr, ExceptionChainRepr)
                    for repr in repr.chain:
                        assert repr[0].style == style

    def test_reprexcinfo_unicode(self):
        from _pytest._code.code import TerminalRepr

        class MyRepr(TerminalRepr):
            def toterminal(self, tw):
                tw.line(u"я")

        x = six.text_type(MyRepr())
        assert x == u"я"

    def test_toterminal_long(self, importasmod):
        mod = importasmod(
            """
            def g(x):
                raise ValueError(x)
            def f():
                g(3)
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        excinfo.traceback = excinfo.traceback.filter()
        repr = excinfo.getrepr()
        tw = TWMock()
        repr.toterminal(tw)
        assert tw.lines[0] == ""
        tw.lines.pop(0)
        assert tw.lines[0] == "    def f():"
        assert tw.lines[1] == ">       g(3)"
        assert tw.lines[2] == ""
        line = tw.get_write_msg(3)
        assert line.endswith("mod.py")
        assert tw.lines[4] == (":5: ")
        assert tw.lines[5] == ("_ ", None)
        assert tw.lines[6] == ""
        assert tw.lines[7] == "    def g(x):"
        assert tw.lines[8] == ">       raise ValueError(x)"
        assert tw.lines[9] == "E       ValueError: 3"
        assert tw.lines[10] == ""
        line = tw.get_write_msg(11)
        assert line.endswith("mod.py")
        assert tw.lines[12] == ":3: ValueError"

    def test_toterminal_long_missing_source(self, importasmod, tmpdir):
        mod = importasmod(
            """
            def g(x):
                raise ValueError(x)
            def f():
                g(3)
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        tmpdir.join("mod.py").remove()
        excinfo.traceback = excinfo.traceback.filter()
        repr = excinfo.getrepr()
        tw = TWMock()
        repr.toterminal(tw)
        assert tw.lines[0] == ""
        tw.lines.pop(0)
        assert tw.lines[0] == ">   ???"
        assert tw.lines[1] == ""
        line = tw.get_write_msg(2)
        assert line.endswith("mod.py")
        assert tw.lines[3] == ":5: "
        assert tw.lines[4] == ("_ ", None)
        assert tw.lines[5] == ""
        assert tw.lines[6] == ">   ???"
        assert tw.lines[7] == "E   ValueError: 3"
        assert tw.lines[8] == ""
        line = tw.get_write_msg(9)
        assert line.endswith("mod.py")
        assert tw.lines[10] == ":3: ValueError"

    def test_toterminal_long_incomplete_source(self, importasmod, tmpdir):
        mod = importasmod(
            """
            def g(x):
                raise ValueError(x)
            def f():
                g(3)
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        tmpdir.join("mod.py").write("asdf")
        excinfo.traceback = excinfo.traceback.filter()
        repr = excinfo.getrepr()
        tw = TWMock()
        repr.toterminal(tw)
        assert tw.lines[0] == ""
        tw.lines.pop(0)
        assert tw.lines[0] == ">   ???"
        assert tw.lines[1] == ""
        line = tw.get_write_msg(2)
        assert line.endswith("mod.py")
        assert tw.lines[3] == ":5: "
        assert tw.lines[4] == ("_ ", None)
        assert tw.lines[5] == ""
        assert tw.lines[6] == ">   ???"
        assert tw.lines[7] == "E   ValueError: 3"
        assert tw.lines[8] == ""
        line = tw.get_write_msg(9)
        assert line.endswith("mod.py")
        assert tw.lines[10] == ":3: ValueError"

    def test_toterminal_long_filenames(self, importasmod):
        mod = importasmod(
            """
            def f():
                raise ValueError()
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        tw = TWMock()
        path = py.path.local(mod.__file__)
        old = path.dirpath().chdir()
        try:
            repr = excinfo.getrepr(abspath=False)
            repr.toterminal(tw)
            x = py.path.local().bestrelpath(path)
            if len(x) < len(str(path)):
                msg = tw.get_write_msg(-2)
                assert msg == "mod.py"
                assert tw.lines[-1] == ":3: ValueError"

            repr = excinfo.getrepr(abspath=True)
            repr.toterminal(tw)
            msg = tw.get_write_msg(-2)
            assert msg == path
            line = tw.lines[-1]
            assert line == ":3: ValueError"
        finally:
            old.chdir()

    @pytest.mark.parametrize(
        "reproptions",
        [
            {
                "style": style,
                "showlocals": showlocals,
                "funcargs": funcargs,
                "tbfilter": tbfilter,
            }
            for style in ("long", "short", "no")
            for showlocals in (True, False)
            for tbfilter in (True, False)
            for funcargs in (True, False)
        ],
    )
    def test_format_excinfo(self, importasmod, reproptions):
        mod = importasmod(
            """
            def g(x):
                raise ValueError(x)
            def f():
                g(3)
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        tw = py.io.TerminalWriter(stringio=True)
        repr = excinfo.getrepr(**reproptions)
        repr.toterminal(tw)
        assert tw.stringio.getvalue()

    def test_traceback_repr_style(self, importasmod):
        mod = importasmod(
            """
            def f():
                g()
            def g():
                h()
            def h():
                i()
            def i():
                raise ValueError()
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        excinfo.traceback = excinfo.traceback.filter()
        excinfo.traceback[1].set_repr_style("short")
        excinfo.traceback[2].set_repr_style("short")
        r = excinfo.getrepr(style="long")
        tw = TWMock()
        r.toterminal(tw)
        for line in tw.lines:
            print(line)
        assert tw.lines[0] == ""
        assert tw.lines[1] == "    def f():"
        assert tw.lines[2] == ">       g()"
        assert tw.lines[3] == ""
        msg = tw.get_write_msg(4)
        assert msg.endswith("mod.py")
        assert tw.lines[5] == ":3: "
        assert tw.lines[6] == ("_ ", None)
        tw.get_write_msg(7)
        assert tw.lines[8].endswith("in g")
        assert tw.lines[9] == "    h()"
        tw.get_write_msg(10)
        assert tw.lines[11].endswith("in h")
        assert tw.lines[12] == "    i()"
        assert tw.lines[13] == ("_ ", None)
        assert tw.lines[14] == ""
        assert tw.lines[15] == "    def i():"
        assert tw.lines[16] == ">       raise ValueError()"
        assert tw.lines[17] == "E       ValueError"
        assert tw.lines[18] == ""
        msg = tw.get_write_msg(19)
        msg.endswith("mod.py")
        assert tw.lines[20] == ":9: ValueError"

    @pytest.mark.skipif("sys.version_info[0] < 3")
    def test_exc_chain_repr(self, importasmod):
        mod = importasmod(
            """
            class Err(Exception):
                pass
            def f():
                try:
                    g()
                except Exception as e:
                    raise Err() from e
                finally:
                    h()
            def g():
                raise ValueError()

            def h():
                raise AttributeError()
        """
        )
        excinfo = pytest.raises(AttributeError, mod.f)
        r = excinfo.getrepr(style="long")
        tw = TWMock()
        r.toterminal(tw)
        for line in tw.lines:
            print(line)
        assert tw.lines[0] == ""
        assert tw.lines[1] == "    def f():"
        assert tw.lines[2] == "        try:"
        assert tw.lines[3] == ">           g()"
        assert tw.lines[4] == ""
        line = tw.get_write_msg(5)
        assert line.endswith("mod.py")
        assert tw.lines[6] == ":6: "
        assert tw.lines[7] == ("_ ", None)
        assert tw.lines[8] == ""
        assert tw.lines[9] == "    def g():"
        assert tw.lines[10] == ">       raise ValueError()"
        assert tw.lines[11] == "E       ValueError"
        assert tw.lines[12] == ""
        line = tw.get_write_msg(13)
        assert line.endswith("mod.py")
        assert tw.lines[14] == ":12: ValueError"
        assert tw.lines[15] == ""
        assert (
            tw.lines[16]
            == "The above exception was the direct cause of the following exception:"
        )
        assert tw.lines[17] == ""
        assert tw.lines[18] == "    def f():"
        assert tw.lines[19] == "        try:"
        assert tw.lines[20] == "            g()"
        assert tw.lines[21] == "        except Exception as e:"
        assert tw.lines[22] == ">           raise Err() from e"
        assert tw.lines[23] == "E           test_exc_chain_repr0.mod.Err"
        assert tw.lines[24] == ""
        line = tw.get_write_msg(25)
        assert line.endswith("mod.py")
        assert tw.lines[26] == ":8: Err"
        assert tw.lines[27] == ""
        assert (
            tw.lines[28]
            == "During handling of the above exception, another exception occurred:"
        )
        assert tw.lines[29] == ""
        assert tw.lines[30] == "    def f():"
        assert tw.lines[31] == "        try:"
        assert tw.lines[32] == "            g()"
        assert tw.lines[33] == "        except Exception as e:"
        assert tw.lines[34] == "            raise Err() from e"
        assert tw.lines[35] == "        finally:"
        assert tw.lines[36] == ">           h()"
        assert tw.lines[37] == ""
        line = tw.get_write_msg(38)
        assert line.endswith("mod.py")
        assert tw.lines[39] == ":10: "
        assert tw.lines[40] == ("_ ", None)
        assert tw.lines[41] == ""
        assert tw.lines[42] == "    def h():"
        assert tw.lines[43] == ">       raise AttributeError()"
        assert tw.lines[44] == "E       AttributeError"
        assert tw.lines[45] == ""
        line = tw.get_write_msg(46)
        assert line.endswith("mod.py")
        assert tw.lines[47] == ":15: AttributeError"

    @pytest.mark.skipif("sys.version_info[0] < 3")
    @pytest.mark.parametrize("mode", ["from_none", "explicit_suppress"])
    def test_exc_repr_chain_suppression(self, importasmod, mode):
        """Check that exc repr does not show chained exceptions in Python 3.
        - When the exception is raised with "from None"
        - Explicitly suppressed with "chain=False" to ExceptionInfo.getrepr().
        """
        raise_suffix = " from None" if mode == "from_none" else ""
        mod = importasmod(
            """
            def f():
                try:
                    g()
                except Exception:
                    raise AttributeError(){raise_suffix}
            def g():
                raise ValueError()
        """.format(
                raise_suffix=raise_suffix
            )
        )
        excinfo = pytest.raises(AttributeError, mod.f)
        r = excinfo.getrepr(style="long", chain=mode != "explicit_suppress")
        tw = TWMock()
        r.toterminal(tw)
        for line in tw.lines:
            print(line)
        assert tw.lines[0] == ""
        assert tw.lines[1] == "    def f():"
        assert tw.lines[2] == "        try:"
        assert tw.lines[3] == "            g()"
        assert tw.lines[4] == "        except Exception:"
        assert tw.lines[5] == ">           raise AttributeError(){}".format(
            raise_suffix
        )
        assert tw.lines[6] == "E           AttributeError"
        assert tw.lines[7] == ""
        line = tw.get_write_msg(8)
        assert line.endswith("mod.py")
        assert tw.lines[9] == ":6: AttributeError"
        assert len(tw.lines) == 10

    @pytest.mark.skipif("sys.version_info[0] < 3")
    @pytest.mark.parametrize(
        "reason, description",
        [
            (
                "cause",
                "The above exception was the direct cause of the following exception:",
            ),
            (
                "context",
                "During handling of the above exception, another exception occurred:",
            ),
        ],
    )
    def test_exc_chain_repr_without_traceback(self, importasmod, reason, description):
        """
        Handle representation of exception chains where one of the exceptions doesn't have a
        real traceback, such as those raised in a subprocess submitted by the multiprocessing
        module (#1984).
        """
        from _pytest.pytester import LineMatcher

        exc_handling_code = " from e" if reason == "cause" else ""
        mod = importasmod(
            """
            def f():
                try:
                    g()
                except Exception as e:
                    raise RuntimeError('runtime problem'){exc_handling_code}
            def g():
                raise ValueError('invalid value')
        """.format(
                exc_handling_code=exc_handling_code
            )
        )

        with pytest.raises(RuntimeError) as excinfo:
            mod.f()

        # emulate the issue described in #1984
        attr = "__%s__" % reason
        getattr(excinfo.value, attr).__traceback__ = None

        r = excinfo.getrepr()
        tw = py.io.TerminalWriter(stringio=True)
        tw.hasmarkup = False
        r.toterminal(tw)

        matcher = LineMatcher(tw.stringio.getvalue().splitlines())
        matcher.fnmatch_lines(
            [
                "ValueError: invalid value",
                description,
                "* except Exception as e:",
                "> * raise RuntimeError('runtime problem')" + exc_handling_code,
                "E *RuntimeError: runtime problem",
            ]
        )

    @pytest.mark.skipif("sys.version_info[0] < 3")
    def test_exc_chain_repr_cycle(self, importasmod):
        mod = importasmod(
            """
            class Err(Exception):
                pass
            def fail():
                return 0 / 0
            def reraise():
                try:
                    fail()
                except ZeroDivisionError as e:
                    raise Err() from e
            def unreraise():
                try:
                    reraise()
                except Err as e:
                    raise e.__cause__
        """
        )
        excinfo = pytest.raises(ZeroDivisionError, mod.unreraise)
        r = excinfo.getrepr(style="short")
        tw = TWMock()
        r.toterminal(tw)
        out = "\n".join(line for line in tw.lines if isinstance(line, str))
        expected_out = textwrap.dedent(
            """\
            :13: in unreraise
                reraise()
            :10: in reraise
                raise Err() from e
            E   test_exc_chain_repr_cycle0.mod.Err

            During handling of the above exception, another exception occurred:
            :15: in unreraise
                raise e.__cause__
            :8: in reraise
                fail()
            :5: in fail
                return 0 / 0
            E   ZeroDivisionError: division by zero"""
        )
        assert out == expected_out


@pytest.mark.parametrize("style", ["short", "long"])
@pytest.mark.parametrize("encoding", [None, "utf8", "utf16"])
def test_repr_traceback_with_unicode(style, encoding):
    msg = u"☹"
    if encoding is not None:
        msg = msg.encode(encoding)
    try:
        raise RuntimeError(msg)
    except RuntimeError:
        e_info = ExceptionInfo.from_current()
    formatter = FormattedExcinfo(style=style)
    repr_traceback = formatter.repr_traceback(e_info)
    assert repr_traceback is not None


def test_cwd_deleted(testdir):
    testdir.makepyfile(
        """
        def test(tmpdir):
            tmpdir.chdir()
            tmpdir.remove()
            assert False
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["* 1 failed in *"])
    assert "INTERNALERROR" not in result.stdout.str() + result.stderr.str()


@pytest.mark.usefixtures("limited_recursion_depth")
def test_exception_repr_extraction_error_on_recursion():
    """
    Ensure we can properly detect a recursion error even
    if some locals raise error on comparison (#2459).
    """
    from _pytest.pytester import LineMatcher

    class numpy_like(object):
        def __eq__(self, other):
            if type(other) is numpy_like:
                raise ValueError(
                    "The truth value of an array "
                    "with more than one element is ambiguous."
                )

    def a(x):
        return b(numpy_like())

    def b(x):
        return a(numpy_like())

    with pytest.raises(RuntimeError) as excinfo:
        a(numpy_like())

    matcher = LineMatcher(str(excinfo.getrepr()).splitlines())
    matcher.fnmatch_lines(
        [
            "!!! Recursion error detected, but an error occurred locating the origin of recursion.",
            "*The following exception happened*",
            "*ValueError: The truth value of an array*",
        ]
    )


@pytest.mark.usefixtures("limited_recursion_depth")
def test_no_recursion_index_on_recursion_error():
    """
    Ensure that we don't break in case we can't find the recursion index
    during a recursion error (#2486).
    """

    class RecursionDepthError(object):
        def __getattr__(self, attr):
            return getattr(self, "_" + attr)

    with pytest.raises(RuntimeError) as excinfo:
        RecursionDepthError().trigger
    assert "maximum recursion" in str(excinfo.getrepr())
