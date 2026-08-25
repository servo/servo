# mypy: allow-untyped-defs
from __future__ import annotations

import fnmatch
import importlib
import io
import operator
from pathlib import Path
import queue
import re
import sys
import textwrap
from typing import Any
from typing import TYPE_CHECKING

import _pytest._code
from _pytest._code.code import ExceptionChainRepr
from _pytest._code.code import ExceptionInfo
from _pytest._code.code import FormattedExcinfo
from _pytest._io import TerminalWriter
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pathlib import bestrelpath
from _pytest.pathlib import import_path
from _pytest.pytester import LineMatcher
from _pytest.pytester import Pytester
import pytest


if TYPE_CHECKING:
    from _pytest._code.code import _TracebackStyle

if sys.version_info < (3, 11):
    from exceptiongroup import ExceptionGroup


@pytest.fixture
def limited_recursion_depth():
    before = sys.getrecursionlimit()
    sys.setrecursionlimit(150)
    yield
    sys.setrecursionlimit(before)


def test_excinfo_simple() -> None:
    try:
        raise ValueError
    except ValueError:
        info = _pytest._code.ExceptionInfo.from_current()
    assert info.type == ValueError


def test_excinfo_from_exc_info_simple() -> None:
    try:
        raise ValueError
    except ValueError as e:
        assert e.__traceback__ is not None
        info = _pytest._code.ExceptionInfo.from_exc_info((type(e), e, e.__traceback__))
    assert info.type == ValueError


def test_excinfo_from_exception_simple() -> None:
    try:
        raise ValueError
    except ValueError as e:
        assert e.__traceback__ is not None
        info = _pytest._code.ExceptionInfo.from_exception(e)
    assert info.type == ValueError


def test_excinfo_from_exception_missing_traceback_assertion() -> None:
    with pytest.raises(AssertionError, match=r"must have.*__traceback__"):
        _pytest._code.ExceptionInfo.from_exception(ValueError())


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
        f.__code__.co_firstlineno - 1 + 4,
        f.__code__.co_firstlineno - 1 + 1,
        g.__code__.co_firstlineno - 1 + 1,
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


class TestTraceback_f_g_h:
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

    def test_traceback_entry_getsource_in_construct(self):
        def xyz():
            try:
                raise ValueError
            except somenoname:  # type: ignore[name-defined] # noqa: F821
                pass  # pragma: no cover

        try:
            xyz()
        except NameError:
            excinfo = _pytest._code.ExceptionInfo.from_current()
        else:
            assert False, "did not raise NameError"

        tb = excinfo.traceback
        source = tb[-1].getsource()
        assert source is not None
        assert source.deindent().lines == [
            "def xyz():",
            "    try:",
            "        raise ValueError",
            "    except somenoname:  # type: ignore[name-defined] # noqa: F821",
        ]

    def test_traceback_cut(self) -> None:
        co = _pytest._code.Code.from_function(f)
        path, firstlineno = co.path, co.firstlineno
        assert isinstance(path, Path)
        traceback = self.excinfo.traceback
        newtraceback = traceback.cut(path=path, firstlineno=firstlineno)
        assert len(newtraceback) == 1
        newtraceback = traceback.cut(path=path, lineno=firstlineno + 2)
        assert len(newtraceback) == 1

    def test_traceback_cut_excludepath(self, pytester: Pytester) -> None:
        p = pytester.makepyfile("def f(): raise ValueError")
        with pytest.raises(ValueError) as excinfo:
            import_path(p, root=pytester.path, consider_namespace_packages=False).f()
        basedir = Path(pytest.__file__).parent
        newtraceback = excinfo.traceback.cut(excludepath=basedir)
        for x in newtraceback:
            assert isinstance(x.path, Path)
            assert basedir not in x.path.parents
        assert newtraceback[-1].frame.code.path == p

    def test_traceback_filter(self):
        traceback = self.excinfo.traceback
        ntraceback = traceback.filter(self.excinfo)
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
        ntraceback = traceback.filter(excinfo)
        print(f"old: {traceback!r}")
        print(f"new: {ntraceback!r}")

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

        excinfo = pytest.raises(RecursionError, f, 8)
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

    def test_traceback_no_recursion_index(self) -> None:
        def do_stuff() -> None:
            raise RuntimeError

        def reraise_me() -> None:
            import sys

            exc, val, tb = sys.exc_info()
            assert val is not None
            raise val.with_traceback(tb)

        def f(n: int) -> None:
            try:
                do_stuff()
            except BaseException:
                reraise_me()

        excinfo = pytest.raises(RuntimeError, f, 8)
        assert excinfo is not None
        traceback = excinfo.traceback
        recindex = traceback.recursionindex()
        assert recindex is None

    def test_traceback_messy_recursion(self):
        # XXX: simplified locally testable version
        decorator = pytest.importorskip("decorator").decorator

        def log(f, *k, **kw):
            print(f"{k} {kw}")
            f(*k, **kw)

        log = decorator(log)

        def fail():
            raise ValueError("")

        fail = log(log(fail))

        excinfo = pytest.raises(ValueError, fail)
        assert excinfo.traceback.recursionindex() is None

    def test_getreprcrash(self):
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
        reprcrash = excinfo._getreprcrash()
        assert reprcrash is not None
        co = _pytest._code.Code.from_function(h)
        assert reprcrash.path == str(co.path)
        assert reprcrash.lineno == co.firstlineno + 1 + 1

    def test_getreprcrash_empty(self):
        def g():
            __tracebackhide__ = True
            raise ValueError

        def f():
            __tracebackhide__ = True
            g()

        excinfo = pytest.raises(ValueError, f)
        assert excinfo._getreprcrash() is None


def test_excinfo_exconly():
    excinfo = pytest.raises(ValueError, h)
    assert excinfo.exconly().startswith("ValueError")
    with pytest.raises(ValueError) as excinfo:
        raise ValueError("hello\nworld")
    msg = excinfo.exconly(tryshort=True)
    assert msg.startswith("ValueError")
    assert msg.endswith("world")


def test_excinfo_repr_str() -> None:
    excinfo1 = pytest.raises(ValueError, h)
    assert repr(excinfo1) == "<ExceptionInfo ValueError() tblen=4>"
    assert str(excinfo1) == "<ExceptionInfo ValueError() tblen=4>"

    class CustomException(Exception):
        def __repr__(self):
            return "custom_repr"

    def raises() -> None:
        raise CustomException()

    excinfo2 = pytest.raises(CustomException, raises)
    assert repr(excinfo2) == "<ExceptionInfo custom_repr tblen=2>"
    assert str(excinfo2) == "<ExceptionInfo custom_repr tblen=2>"


def test_excinfo_for_later() -> None:
    e = ExceptionInfo[BaseException].for_later()
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
    # TODO: Since Python 3.13b1 under pytest-xdist, the * is `import
    # sys;exec(eval(sys.stdin.readline()))` (execnet bootstrap code)
    # instead of `???` like before. Is this OK?
    fnmatch.fnmatch(s, "  File '<string>':1 in <module>\n  *\n")


def test_excinfo_no_python_sourcecode(tmp_path: Path) -> None:
    # XXX: simplified locally testable version
    tmp_path.joinpath("test.txt").write_text("{{ h()}}:", encoding="utf-8")

    jinja2 = pytest.importorskip("jinja2")
    loader = jinja2.FileSystemLoader(str(tmp_path))
    env = jinja2.Environment(loader=loader)
    template = env.get_template("test.txt")
    excinfo = pytest.raises(ValueError, template.render, h=h)
    for item in excinfo.traceback:
        print(item)  # XXX: for some reason jinja.Template.render is printed in full
        _ = item.source  # shouldn't fail
        if isinstance(item.path, Path) and item.path.name == "test.txt":
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


def test_codepath_Queue_example() -> None:
    try:
        queue.Queue().get(timeout=0.001)
    except queue.Empty:
        excinfo = _pytest._code.ExceptionInfo.from_current()
    entry = excinfo.traceback[-1]
    path = entry.path
    assert isinstance(path, Path)
    assert path.name.lower() == "queue.py"
    assert path.exists()


def test_match_succeeds():
    with pytest.raises(ZeroDivisionError) as excinfo:
        _ = 0 // 0
    excinfo.match(r".*zero.*")


def test_match_raises_error(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest
        def test_division_zero():
            with pytest.raises(ZeroDivisionError) as excinfo:
                0 / 0
            excinfo.match(r'[123]+')
    """
    )
    result = pytester.runpytest("--tb=short")
    assert result.ret != 0

    match = [
        r"E .* AssertionError: Regex pattern did not match.",
        r"E .* Regex: '\[123\]\+'",
        r"E .* Input: 'division by zero'",
    ]
    result.stdout.re_match_lines(match)
    result.stdout.no_fnmatch_line("*__tracebackhide__ = True*")

    result = pytester.runpytest("--fulltrace")
    assert result.ret != 0
    result.stdout.re_match_lines([r".*__tracebackhide__ = True.*", *match])


class TestGroupContains:
    def test_contains_exception_type(self) -> None:
        exc_group = ExceptionGroup("", [RuntimeError()])
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert exc_info.group_contains(RuntimeError)

    def test_doesnt_contain_exception_type(self) -> None:
        exc_group = ExceptionGroup("", [ValueError()])
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert not exc_info.group_contains(RuntimeError)

    def test_contains_exception_match(self) -> None:
        exc_group = ExceptionGroup("", [RuntimeError("exception message")])
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert exc_info.group_contains(RuntimeError, match=r"^exception message$")

    def test_doesnt_contain_exception_match(self) -> None:
        exc_group = ExceptionGroup("", [RuntimeError("message that will not match")])
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert not exc_info.group_contains(RuntimeError, match=r"^exception message$")

    def test_contains_exception_type_unlimited_depth(self) -> None:
        exc_group = ExceptionGroup("", [ExceptionGroup("", [RuntimeError()])])
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert exc_info.group_contains(RuntimeError)

    def test_contains_exception_type_at_depth_1(self) -> None:
        exc_group = ExceptionGroup("", [RuntimeError()])
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert exc_info.group_contains(RuntimeError, depth=1)

    def test_doesnt_contain_exception_type_past_depth(self) -> None:
        exc_group = ExceptionGroup("", [ExceptionGroup("", [RuntimeError()])])
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert not exc_info.group_contains(RuntimeError, depth=1)

    def test_contains_exception_type_specific_depth(self) -> None:
        exc_group = ExceptionGroup("", [ExceptionGroup("", [RuntimeError()])])
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert exc_info.group_contains(RuntimeError, depth=2)

    def test_contains_exception_match_unlimited_depth(self) -> None:
        exc_group = ExceptionGroup(
            "", [ExceptionGroup("", [RuntimeError("exception message")])]
        )
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert exc_info.group_contains(RuntimeError, match=r"^exception message$")

    def test_contains_exception_match_at_depth_1(self) -> None:
        exc_group = ExceptionGroup("", [RuntimeError("exception message")])
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert exc_info.group_contains(
            RuntimeError, match=r"^exception message$", depth=1
        )

    def test_doesnt_contain_exception_match_past_depth(self) -> None:
        exc_group = ExceptionGroup(
            "", [ExceptionGroup("", [RuntimeError("exception message")])]
        )
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert not exc_info.group_contains(
            RuntimeError, match=r"^exception message$", depth=1
        )

    def test_contains_exception_match_specific_depth(self) -> None:
        exc_group = ExceptionGroup(
            "", [ExceptionGroup("", [RuntimeError("exception message")])]
        )
        with pytest.raises(ExceptionGroup) as exc_info:
            raise exc_group
        assert exc_info.group_contains(
            RuntimeError, match=r"^exception message$", depth=2
        )


class TestFormattedExcinfo:
    @pytest.fixture
    def importasmod(self, tmp_path: Path, _sys_snapshot):
        def importasmod(source):
            source = textwrap.dedent(source)
            modpath = tmp_path.joinpath("mod.py")
            tmp_path.joinpath("__init__.py").touch()
            modpath.write_text(source, encoding="utf-8")
            importlib.invalidate_caches()
            return import_path(
                modpath, root=tmp_path, consider_namespace_packages=False
            )

        return importasmod

    def test_repr_source(self):
        pr = FormattedExcinfo()
        source = _pytest._code.Source(
            """\
            def f(x):
                pass
            """
        ).strip()
        pr.flow_marker = "|"  # type: ignore[misc]
        lines = pr.get_source(source, 0)
        assert len(lines) == 2
        assert lines[0] == "|   def f(x):"
        assert lines[1] == "        pass"

    def test_repr_source_out_of_bounds(self):
        pr = FormattedExcinfo()
        source = _pytest._code.Source(
            """\
            def f(x):
                pass
            """
        ).strip()
        pr.flow_marker = "|"  # type: ignore[misc]

        lines = pr.get_source(source, 100)
        assert len(lines) == 1
        assert lines[0] == "|   ???"

        lines = pr.get_source(source, -100)
        assert len(lines) == 1
        assert lines[0] == "|   ???"

    def test_repr_source_excinfo(self) -> None:
        """Check if indentation is right."""
        try:

            def f():
                _ = 1 / 0

            f()

        except BaseException:
            excinfo = _pytest._code.ExceptionInfo.from_current()
        else:
            assert False, "did not raise"

        pr = FormattedExcinfo()
        source = pr._getentrysource(excinfo.traceback[-1])
        assert source is not None
        lines = pr.get_source(source, 1, excinfo)
        for line in lines:
            print(line)
        assert lines == [
            "    def f():",
            ">       _ = 1 / 0",
            "E       ZeroDivisionError: division by zero",
        ]

    def test_repr_source_not_existing(self):
        pr = FormattedExcinfo()
        co = compile("raise ValueError()", "", "exec")
        try:
            exec(co)
        except ValueError:
            excinfo = _pytest._code.ExceptionInfo.from_current()
        repr = pr.repr_excinfo(excinfo)
        assert repr.reprtraceback.reprentries[1].lines[0] == ">   ???"
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
        assert repr.chain[0][0].reprentries[1].lines[0] == ">   ???"

    def test_repr_source_failing_fullsource(self, monkeypatch) -> None:
        pr = FormattedExcinfo()

        try:
            _ = 1 / 0
        except ZeroDivisionError:
            excinfo = ExceptionInfo.from_current()

        with monkeypatch.context() as m:
            m.setattr(_pytest._code.Code, "fullsource", property(lambda self: None))
            repr = pr.repr_excinfo(excinfo)

        assert repr.reprtraceback.reprentries[0].lines[0] == ">   ???"
        assert repr.chain[0][0].reprentries[0].lines[0] == ">   ???"

    def test_repr_local(self) -> None:
        p = FormattedExcinfo(showlocals=True)
        loc = {"y": 5, "z": 7, "x": 3, "@x": 2, "__builtins__": {}}
        reprlocals = p.repr_locals(loc)
        assert reprlocals is not None
        assert reprlocals.lines
        assert reprlocals.lines[0] == "__builtins__ = <builtins>"
        assert reprlocals.lines[1] == "x          = 3"
        assert reprlocals.lines[2] == "y          = 5"
        assert reprlocals.lines[3] == "z          = 7"

    def test_repr_local_with_error(self) -> None:
        class ObjWithErrorInRepr:
            def __repr__(self):
                raise NotImplementedError

        p = FormattedExcinfo(showlocals=True, truncate_locals=False)
        loc = {"x": ObjWithErrorInRepr(), "__builtins__": {}}
        reprlocals = p.repr_locals(loc)
        assert reprlocals is not None
        assert reprlocals.lines
        assert reprlocals.lines[0] == "__builtins__ = <builtins>"
        assert "[NotImplementedError() raised in repr()]" in reprlocals.lines[1]

    def test_repr_local_with_exception_in_class_property(self) -> None:
        class ExceptionWithBrokenClass(Exception):
            # Type ignored because it's bypassed intentionally.
            @property  # type: ignore
            def __class__(self):
                raise TypeError("boom!")

        class ObjWithErrorInRepr:
            def __repr__(self):
                raise ExceptionWithBrokenClass()

        p = FormattedExcinfo(showlocals=True, truncate_locals=False)
        loc = {"x": ObjWithErrorInRepr(), "__builtins__": {}}
        reprlocals = p.repr_locals(loc)
        assert reprlocals is not None
        assert reprlocals.lines
        assert reprlocals.lines[0] == "__builtins__ = <builtins>"
        assert "[ExceptionWithBrokenClass() raised in repr()]" in reprlocals.lines[1]

    def test_repr_local_truncated(self) -> None:
        loc = {"l": [i for i in range(10)]}
        p = FormattedExcinfo(showlocals=True)
        truncated_reprlocals = p.repr_locals(loc)
        assert truncated_reprlocals is not None
        assert truncated_reprlocals.lines
        assert truncated_reprlocals.lines[0] == "l          = [0, 1, 2, 3, 4, 5, ...]"

        q = FormattedExcinfo(showlocals=True, truncate_locals=False)
        full_reprlocals = q.repr_locals(loc)
        assert full_reprlocals is not None
        assert full_reprlocals.lines
        assert full_reprlocals.lines[0] == "l          = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"

    def test_repr_tracebackentry_lines(self, importasmod) -> None:
        mod = importasmod(
            """
            def func1():
                raise ValueError("hello\\nworld")
        """
        )
        excinfo = pytest.raises(ValueError, mod.func1)
        excinfo.traceback = excinfo.traceback.filter(excinfo)
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

        loc = repr_entry.reprfileloc
        assert loc is not None
        assert loc.path == mod.__file__
        assert loc.lineno == 3
        # assert loc.message == "ValueError: hello"

    def test_repr_tracebackentry_lines2(self, importasmod, tw_mock) -> None:
        mod = importasmod(
            """
            def func1(m, x, y, z):
                raise ValueError("hello\\nworld")
        """
        )
        excinfo = pytest.raises(ValueError, mod.func1, "m" * 90, 5, 13, "z" * 120)
        excinfo.traceback = excinfo.traceback.filter(excinfo)
        entry = excinfo.traceback[-1]
        p = FormattedExcinfo(funcargs=True)
        reprfuncargs = p.repr_args(entry)
        assert reprfuncargs is not None
        assert reprfuncargs.args[0] == ("m", repr("m" * 90))
        assert reprfuncargs.args[1] == ("x", "5")
        assert reprfuncargs.args[2] == ("y", "13")
        assert reprfuncargs.args[3] == ("z", repr("z" * 120))

        p = FormattedExcinfo(funcargs=True)
        repr_entry = p.repr_traceback_entry(entry)
        assert repr_entry.reprfuncargs is not None
        assert repr_entry.reprfuncargs.args == reprfuncargs.args
        repr_entry.toterminal(tw_mock)
        assert tw_mock.lines[0] == "m = " + repr("m" * 90)
        assert tw_mock.lines[1] == "x = 5, y = 13"
        assert tw_mock.lines[2] == "z = " + repr("z" * 120)

    def test_repr_tracebackentry_lines_var_kw_args(self, importasmod, tw_mock) -> None:
        mod = importasmod(
            """
            def func1(x, *y, **z):
                raise ValueError("hello\\nworld")
        """
        )
        excinfo = pytest.raises(ValueError, mod.func1, "a", "b", c="d")
        excinfo.traceback = excinfo.traceback.filter(excinfo)
        entry = excinfo.traceback[-1]
        p = FormattedExcinfo(funcargs=True)
        reprfuncargs = p.repr_args(entry)
        assert reprfuncargs is not None
        assert reprfuncargs.args[0] == ("x", repr("a"))
        assert reprfuncargs.args[1] == ("y", repr(("b",)))
        assert reprfuncargs.args[2] == ("z", repr({"c": "d"}))

        p = FormattedExcinfo(funcargs=True)
        repr_entry = p.repr_traceback_entry(entry)
        assert repr_entry.reprfuncargs
        assert repr_entry.reprfuncargs.args == reprfuncargs.args
        repr_entry.toterminal(tw_mock)
        assert tw_mock.lines[0] == "x = 'a', y = ('b',), z = {'c': 'd'}"

    def test_repr_tracebackentry_short(self, importasmod) -> None:
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
        basename = Path(mod.__file__).name
        assert lines[0] == "    func1()"
        assert reprtb.reprfileloc is not None
        assert basename in str(reprtb.reprfileloc.path)
        assert reprtb.reprfileloc.lineno == 5

        # test last entry
        p = FormattedExcinfo(style="short")
        reprtb = p.repr_traceback_entry(excinfo.traceback[-1], excinfo)
        lines = reprtb.lines
        assert lines[0] == '    raise ValueError("hello")'
        assert lines[1] == "E   ValueError: hello"
        assert reprtb.reprfileloc is not None
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

    def test_traceback_short_no_source(
        self,
        importasmod,
        monkeypatch: pytest.MonkeyPatch,
    ) -> None:
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

        with monkeypatch.context() as mp:
            mp.setattr(Code, "path", "bogus")
            p = FormattedExcinfo(style="short")
            reprtb = p.repr_traceback_entry(excinfo.traceback[-2])
            lines = reprtb.lines
            last_p = FormattedExcinfo(style="short")
            last_reprtb = last_p.repr_traceback_entry(excinfo.traceback[-1], excinfo)
            last_lines = last_reprtb.lines
        assert lines[0] == "    func1()"

        assert last_lines[0] == '    raise ValueError("hello")'
        assert last_lines[1] == "E   ValueError: hello"

    def test_repr_traceback_and_excinfo(self, importasmod) -> None:
        mod = importasmod(
            """
            def f(x):
                raise ValueError(x)
            def entry():
                f(0)
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)

        styles: tuple[_TracebackStyle, ...] = ("long", "short")
        for style in styles:
            p = FormattedExcinfo(style=style)
            reprtb = p.repr_traceback(excinfo)
            assert len(reprtb.reprentries) == 2
            assert reprtb.style == style
            assert not reprtb.extraline
            repr = p.repr_excinfo(excinfo)
            assert repr.reprtraceback
            assert len(repr.reprtraceback.reprentries) == len(reprtb.reprentries)

            assert repr.chain[0][0]
            assert len(repr.chain[0][0].reprentries) == len(reprtb.reprentries)
            assert repr.reprcrash is not None
            assert repr.reprcrash.path.endswith("mod.py")
            assert repr.reprcrash.message == "ValueError: 0"

    def test_repr_traceback_with_invalid_cwd(self, importasmod, monkeypatch) -> None:
        mod = importasmod(
            """
            def f(x):
                raise ValueError(x)
            def entry():
                f(0)
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)

        p = FormattedExcinfo(abspath=False)

        raised = 0

        orig_path_cwd = Path.cwd

        def raiseos():
            nonlocal raised
            upframe = sys._getframe().f_back
            assert upframe is not None
            if upframe.f_code.co_name == "_makepath":
                # Only raise with expected calls, but not via e.g. inspect for
                # py38-windows.
                raised += 1
                raise OSError(2, "custom_oserror")
            return orig_path_cwd()

        monkeypatch.setattr(Path, "cwd", raiseos)
        assert p._makepath(Path(__file__)) == __file__
        assert raised == 1
        repr_tb = p.repr_traceback(excinfo)

        matcher = LineMatcher(str(repr_tb).splitlines())
        matcher.fnmatch_lines(
            [
                "def entry():",
                ">       f(0)",
                "",
                f"{mod.__file__}:5: ",
                "_ _ *",
                "",
                "    def f(x):",
                ">       raise ValueError(x)",
                "E       ValueError: 0",
                "",
                f"{mod.__file__}:3: ValueError",
            ]
        )
        assert raised == 3

    def test_repr_excinfo_addouterr(self, importasmod, tw_mock):
        mod = importasmod(
            """
            def entry():
                raise ValueError()
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)
        repr = excinfo.getrepr()
        repr.addsection("title", "content")
        repr.toterminal(tw_mock)
        assert tw_mock.lines[-1] == "content"
        assert tw_mock.lines[-2] == ("-", "title")

    def test_repr_excinfo_reprcrash(self, importasmod) -> None:
        mod = importasmod(
            """
            def entry():
                raise ValueError()
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)
        repr = excinfo.getrepr()
        assert repr.reprcrash is not None
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

    def test_reprexcinfo_getrepr(self, importasmod) -> None:
        mod = importasmod(
            """
            def f(x):
                raise ValueError(x)
            def entry():
                f(0)
        """
        )
        excinfo = pytest.raises(ValueError, mod.entry)

        styles: tuple[_TracebackStyle, ...] = ("short", "long", "no")
        for style in styles:
            for showlocals in (True, False):
                repr = excinfo.getrepr(style=style, showlocals=showlocals)
                assert repr.reprtraceback.style == style

                assert isinstance(repr, ExceptionChainRepr)
                for r in repr.chain:
                    assert r[0].style == style

    def test_reprexcinfo_unicode(self):
        from _pytest._code.code import TerminalRepr

        class MyRepr(TerminalRepr):
            def toterminal(self, tw: TerminalWriter) -> None:
                tw.line("я")

        x = str(MyRepr())
        assert x == "я"

    def test_toterminal_long(self, importasmod, tw_mock):
        mod = importasmod(
            """
            def g(x):
                raise ValueError(x)
            def f():
                g(3)
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        excinfo.traceback = excinfo.traceback.filter(excinfo)
        repr = excinfo.getrepr()
        repr.toterminal(tw_mock)
        assert tw_mock.lines[0] == ""
        tw_mock.lines.pop(0)
        assert tw_mock.lines[0] == "    def f():"
        assert tw_mock.lines[1] == ">       g(3)"
        assert tw_mock.lines[2] == ""
        line = tw_mock.get_write_msg(3)
        assert line.endswith("mod.py")
        assert tw_mock.lines[4] == (":5: ")
        assert tw_mock.lines[5] == ("_ ", None)
        assert tw_mock.lines[6] == ""
        assert tw_mock.lines[7] == "    def g(x):"
        assert tw_mock.lines[8] == ">       raise ValueError(x)"
        assert tw_mock.lines[9] == "E       ValueError: 3"
        assert tw_mock.lines[10] == ""
        line = tw_mock.get_write_msg(11)
        assert line.endswith("mod.py")
        assert tw_mock.lines[12] == ":3: ValueError"

    def test_toterminal_long_missing_source(
        self, importasmod, tmp_path: Path, tw_mock
    ) -> None:
        mod = importasmod(
            """
            def g(x):
                raise ValueError(x)
            def f():
                g(3)
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        tmp_path.joinpath("mod.py").unlink()
        excinfo.traceback = excinfo.traceback.filter(excinfo)
        repr = excinfo.getrepr()
        repr.toterminal(tw_mock)
        assert tw_mock.lines[0] == ""
        tw_mock.lines.pop(0)
        assert tw_mock.lines[0] == ">   ???"
        assert tw_mock.lines[1] == ""
        line = tw_mock.get_write_msg(2)
        assert line.endswith("mod.py")
        assert tw_mock.lines[3] == ":5: "
        assert tw_mock.lines[4] == ("_ ", None)
        assert tw_mock.lines[5] == ""
        assert tw_mock.lines[6] == ">   ???"
        assert tw_mock.lines[7] == "E   ValueError: 3"
        assert tw_mock.lines[8] == ""
        line = tw_mock.get_write_msg(9)
        assert line.endswith("mod.py")
        assert tw_mock.lines[10] == ":3: ValueError"

    def test_toterminal_long_incomplete_source(
        self, importasmod, tmp_path: Path, tw_mock
    ) -> None:
        mod = importasmod(
            """
            def g(x):
                raise ValueError(x)
            def f():
                g(3)
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        tmp_path.joinpath("mod.py").write_text("asdf", encoding="utf-8")
        excinfo.traceback = excinfo.traceback.filter(excinfo)
        repr = excinfo.getrepr()
        repr.toterminal(tw_mock)
        assert tw_mock.lines[0] == ""
        tw_mock.lines.pop(0)
        assert tw_mock.lines[0] == ">   ???"
        assert tw_mock.lines[1] == ""
        line = tw_mock.get_write_msg(2)
        assert line.endswith("mod.py")
        assert tw_mock.lines[3] == ":5: "
        assert tw_mock.lines[4] == ("_ ", None)
        assert tw_mock.lines[5] == ""
        assert tw_mock.lines[6] == ">   ???"
        assert tw_mock.lines[7] == "E   ValueError: 3"
        assert tw_mock.lines[8] == ""
        line = tw_mock.get_write_msg(9)
        assert line.endswith("mod.py")
        assert tw_mock.lines[10] == ":3: ValueError"

    def test_toterminal_long_filenames(
        self, importasmod, tw_mock, monkeypatch: MonkeyPatch
    ) -> None:
        mod = importasmod(
            """
            def f():
                raise ValueError()
        """
        )
        excinfo = pytest.raises(ValueError, mod.f)
        path = Path(mod.__file__)
        monkeypatch.chdir(path.parent)
        repr = excinfo.getrepr(abspath=False)
        repr.toterminal(tw_mock)
        x = bestrelpath(Path.cwd(), path)
        if len(x) < len(str(path)):
            msg = tw_mock.get_write_msg(-2)
            assert msg == "mod.py"
            assert tw_mock.lines[-1] == ":3: ValueError"

        repr = excinfo.getrepr(abspath=True)
        repr.toterminal(tw_mock)
        msg = tw_mock.get_write_msg(-2)
        assert msg == str(path)
        line = tw_mock.lines[-1]
        assert line == ":3: ValueError"

    @pytest.mark.parametrize(
        "reproptions",
        [
            pytest.param(
                {
                    "style": style,
                    "showlocals": showlocals,
                    "funcargs": funcargs,
                    "tbfilter": tbfilter,
                },
                id=f"style={style},showlocals={showlocals},funcargs={funcargs},tbfilter={tbfilter}",
            )
            for style in ["long", "short", "line", "no", "native", "value", "auto"]
            for showlocals in (True, False)
            for tbfilter in (True, False)
            for funcargs in (True, False)
        ],
    )
    def test_format_excinfo(self, reproptions: dict[str, Any]) -> None:
        def bar():
            assert False, "some error"

        def foo():
            bar()

        # using inline functions as opposed to importasmod so we get source code lines
        # in the tracebacks (otherwise getinspect doesn't find the source code).
        with pytest.raises(AssertionError) as excinfo:
            foo()
        file = io.StringIO()
        tw = TerminalWriter(file=file)
        repr = excinfo.getrepr(**reproptions)
        repr.toterminal(tw)
        assert file.getvalue()

    def test_traceback_repr_style(self, importasmod, tw_mock):
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
        excinfo.traceback = excinfo.traceback.filter(excinfo)
        excinfo.traceback = _pytest._code.Traceback(
            entry if i not in (1, 2) else entry.with_repr_style("short")
            for i, entry in enumerate(excinfo.traceback)
        )
        r = excinfo.getrepr(style="long")
        r.toterminal(tw_mock)
        for line in tw_mock.lines:
            print(line)
        assert tw_mock.lines[0] == ""
        assert tw_mock.lines[1] == "    def f():"
        assert tw_mock.lines[2] == ">       g()"
        assert tw_mock.lines[3] == ""
        msg = tw_mock.get_write_msg(4)
        assert msg.endswith("mod.py")
        assert tw_mock.lines[5] == ":3: "
        assert tw_mock.lines[6] == ("_ ", None)
        tw_mock.get_write_msg(7)
        assert tw_mock.lines[8].endswith("in g")
        assert tw_mock.lines[9] == "    h()"
        tw_mock.get_write_msg(10)
        assert tw_mock.lines[11].endswith("in h")
        assert tw_mock.lines[12] == "    i()"
        assert tw_mock.lines[13] == ("_ ", None)
        assert tw_mock.lines[14] == ""
        assert tw_mock.lines[15] == "    def i():"
        assert tw_mock.lines[16] == ">       raise ValueError()"
        assert tw_mock.lines[17] == "E       ValueError"
        assert tw_mock.lines[18] == ""
        msg = tw_mock.get_write_msg(19)
        msg.endswith("mod.py")
        assert tw_mock.lines[20] == ":9: ValueError"

    def test_exc_chain_repr(self, importasmod, tw_mock):
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
        r.toterminal(tw_mock)
        for line in tw_mock.lines:
            print(line)
        assert tw_mock.lines[0] == ""
        assert tw_mock.lines[1] == "    def f():"
        assert tw_mock.lines[2] == "        try:"
        assert tw_mock.lines[3] == ">           g()"
        assert tw_mock.lines[4] == ""
        line = tw_mock.get_write_msg(5)
        assert line.endswith("mod.py")
        assert tw_mock.lines[6] == ":6: "
        assert tw_mock.lines[7] == ("_ ", None)
        assert tw_mock.lines[8] == ""
        assert tw_mock.lines[9] == "    def g():"
        assert tw_mock.lines[10] == ">       raise ValueError()"
        assert tw_mock.lines[11] == "E       ValueError"
        assert tw_mock.lines[12] == ""
        line = tw_mock.get_write_msg(13)
        assert line.endswith("mod.py")
        assert tw_mock.lines[14] == ":12: ValueError"
        assert tw_mock.lines[15] == ""
        assert (
            tw_mock.lines[16]
            == "The above exception was the direct cause of the following exception:"
        )
        assert tw_mock.lines[17] == ""
        assert tw_mock.lines[18] == "    def f():"
        assert tw_mock.lines[19] == "        try:"
        assert tw_mock.lines[20] == "            g()"
        assert tw_mock.lines[21] == "        except Exception as e:"
        assert tw_mock.lines[22] == ">           raise Err() from e"
        assert tw_mock.lines[23] == "E           test_exc_chain_repr0.mod.Err"
        assert tw_mock.lines[24] == ""
        line = tw_mock.get_write_msg(25)
        assert line.endswith("mod.py")
        assert tw_mock.lines[26] == ":8: Err"
        assert tw_mock.lines[27] == ""
        assert (
            tw_mock.lines[28]
            == "During handling of the above exception, another exception occurred:"
        )
        assert tw_mock.lines[29] == ""
        assert tw_mock.lines[30] == "    def f():"
        assert tw_mock.lines[31] == "        try:"
        assert tw_mock.lines[32] == "            g()"
        assert tw_mock.lines[33] == "        except Exception as e:"
        assert tw_mock.lines[34] == "            raise Err() from e"
        assert tw_mock.lines[35] == "        finally:"
        assert tw_mock.lines[36] == ">           h()"
        assert tw_mock.lines[37] == ""
        line = tw_mock.get_write_msg(38)
        assert line.endswith("mod.py")
        assert tw_mock.lines[39] == ":10: "
        assert tw_mock.lines[40] == ("_ ", None)
        assert tw_mock.lines[41] == ""
        assert tw_mock.lines[42] == "    def h():"
        assert tw_mock.lines[43] == ">       raise AttributeError()"
        assert tw_mock.lines[44] == "E       AttributeError"
        assert tw_mock.lines[45] == ""
        line = tw_mock.get_write_msg(46)
        assert line.endswith("mod.py")
        assert tw_mock.lines[47] == ":15: AttributeError"

    @pytest.mark.parametrize("mode", ["from_none", "explicit_suppress"])
    def test_exc_repr_chain_suppression(self, importasmod, mode, tw_mock):
        """Check that exc repr does not show chained exceptions in Python 3.
        - When the exception is raised with "from None"
        - Explicitly suppressed with "chain=False" to ExceptionInfo.getrepr().
        """
        raise_suffix = " from None" if mode == "from_none" else ""
        mod = importasmod(
            f"""
            def f():
                try:
                    g()
                except Exception:
                    raise AttributeError(){raise_suffix}
            def g():
                raise ValueError()
        """
        )
        excinfo = pytest.raises(AttributeError, mod.f)
        r = excinfo.getrepr(style="long", chain=mode != "explicit_suppress")
        r.toterminal(tw_mock)
        for line in tw_mock.lines:
            print(line)
        assert tw_mock.lines[0] == ""
        assert tw_mock.lines[1] == "    def f():"
        assert tw_mock.lines[2] == "        try:"
        assert tw_mock.lines[3] == "            g()"
        assert tw_mock.lines[4] == "        except Exception:"
        assert tw_mock.lines[5] == f">           raise AttributeError(){raise_suffix}"
        assert tw_mock.lines[6] == "E           AttributeError"
        assert tw_mock.lines[7] == ""
        line = tw_mock.get_write_msg(8)
        assert line.endswith("mod.py")
        assert tw_mock.lines[9] == ":6: AttributeError"
        assert len(tw_mock.lines) == 10

    @pytest.mark.parametrize(
        "reason, description",
        [
            pytest.param(
                "cause",
                "The above exception was the direct cause of the following exception:",
                id="cause",
            ),
            pytest.param(
                "context",
                "During handling of the above exception, another exception occurred:",
                id="context",
            ),
        ],
    )
    def test_exc_chain_repr_without_traceback(self, importasmod, reason, description):
        """
        Handle representation of exception chains where one of the exceptions doesn't have a
        real traceback, such as those raised in a subprocess submitted by the multiprocessing
        module (#1984).
        """
        exc_handling_code = " from e" if reason == "cause" else ""
        mod = importasmod(
            f"""
            def f():
                try:
                    g()
                except Exception as e:
                    raise RuntimeError('runtime problem'){exc_handling_code}
            def g():
                raise ValueError('invalid value')
        """
        )

        with pytest.raises(RuntimeError) as excinfo:
            mod.f()

        # emulate the issue described in #1984
        attr = "__%s__" % reason
        getattr(excinfo.value, attr).__traceback__ = None

        r = excinfo.getrepr()
        file = io.StringIO()
        tw = TerminalWriter(file=file)
        tw.hasmarkup = False
        r.toterminal(tw)

        matcher = LineMatcher(file.getvalue().splitlines())
        matcher.fnmatch_lines(
            [
                "ValueError: invalid value",
                description,
                "* except Exception as e:",
                "> * raise RuntimeError('runtime problem')" + exc_handling_code,
                "E *RuntimeError: runtime problem",
            ]
        )

    def test_exc_chain_repr_cycle(self, importasmod, tw_mock):
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
        r.toterminal(tw_mock)
        out = "\n".join(line for line in tw_mock.lines if isinstance(line, str))
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

    def test_exec_type_error_filter(self, importasmod):
        """See #7742"""
        mod = importasmod(
            """\
            def f():
                exec("a = 1", {}, [])
            """
        )
        with pytest.raises(TypeError) as excinfo:
            mod.f()
        # previously crashed with `AttributeError: list has no attribute get`
        excinfo.traceback.filter(excinfo)


@pytest.mark.parametrize("style", ["short", "long"])
@pytest.mark.parametrize("encoding", [None, "utf8", "utf16"])
def test_repr_traceback_with_unicode(style, encoding):
    if encoding is None:
        msg: str | bytes = "☹"
    else:
        msg = "☹".encode(encoding)
    try:
        raise RuntimeError(msg)
    except RuntimeError:
        e_info = ExceptionInfo.from_current()
    formatter = FormattedExcinfo(style=style)
    repr_traceback = formatter.repr_traceback(e_info)
    assert repr_traceback is not None


def test_cwd_deleted(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import os

        def test(tmp_path):
            os.chdir(tmp_path)
            tmp_path.unlink()
            assert False
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["* 1 failed in *"])
    result.stdout.no_fnmatch_line("*INTERNALERROR*")
    result.stderr.no_fnmatch_line("*INTERNALERROR*")


def test_regression_negative_line_index(pytester: Pytester) -> None:
    """
    With Python 3.10 alphas, there was an INTERNALERROR reported in
    https://github.com/pytest-dev/pytest/pull/8227
    This test ensures it does not regress.
    """
    pytester.makepyfile(
        """
        import ast
        import pytest


        def test_literal_eval():
            with pytest.raises(ValueError, match="^$"):
                ast.literal_eval("pytest")
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["* 1 failed in *"])
    result.stdout.no_fnmatch_line("*INTERNALERROR*")
    result.stderr.no_fnmatch_line("*INTERNALERROR*")


@pytest.mark.usefixtures("limited_recursion_depth")
def test_exception_repr_extraction_error_on_recursion():
    """
    Ensure we can properly detect a recursion error even
    if some locals raise error on comparison (#2459).
    """

    class numpy_like:
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

    class RecursionDepthError:
        def __getattr__(self, attr):
            return getattr(self, "_" + attr)

    with pytest.raises(RuntimeError) as excinfo:
        _ = RecursionDepthError().trigger
    assert "maximum recursion" in str(excinfo.getrepr())


def _exceptiongroup_common(
    pytester: Pytester,
    outer_chain: str,
    inner_chain: str,
    native: bool,
) -> None:
    pre_raise = "exceptiongroup." if not native else ""
    pre_catch = pre_raise if sys.version_info < (3, 11) else ""
    filestr = f"""
    {"import exceptiongroup" if not native else ""}
    import pytest

    def f(): raise ValueError("From f()")
    def g(): raise BaseException("From g()")

    def inner(inner_chain):
        excs = []
        for callback in [f, g]:
            try:
                callback()
            except BaseException as err:
                excs.append(err)
        if excs:
            if inner_chain == "none":
                raise {pre_raise}BaseExceptionGroup("Oops", excs)
            try:
                raise SyntaxError()
            except SyntaxError as e:
                if inner_chain == "from":
                    raise {pre_raise}BaseExceptionGroup("Oops", excs) from e
                else:
                    raise {pre_raise}BaseExceptionGroup("Oops", excs)

    def outer(outer_chain, inner_chain):
        try:
            inner(inner_chain)
        except {pre_catch}BaseExceptionGroup as e:
            if outer_chain == "none":
                raise
            if outer_chain == "from":
                raise IndexError() from e
            else:
                raise IndexError()


    def test():
        outer("{outer_chain}", "{inner_chain}")
    """
    pytester.makepyfile(test_excgroup=filestr)
    result = pytester.runpytest()
    match_lines = []
    if inner_chain in ("another", "from"):
        match_lines.append(r"SyntaxError: <no detail available>")

    match_lines += [
        r"  + Exception Group Traceback (most recent call last):",
        rf"  \| {pre_catch}BaseExceptionGroup: Oops \(2 sub-exceptions\)",
        r"    \| ValueError: From f\(\)",
        r"    \| BaseException: From g\(\)",
        r"=* short test summary info =*",
    ]
    if outer_chain in ("another", "from"):
        match_lines.append(r"FAILED test_excgroup.py::test - IndexError")
    else:
        match_lines.append(
            rf"FAILED test_excgroup.py::test - {pre_catch}BaseExceptionGroup: Oops \(2.*"
        )
    result.stdout.re_match_lines(match_lines)


@pytest.mark.skipif(
    sys.version_info < (3, 11), reason="Native ExceptionGroup not implemented"
)
@pytest.mark.parametrize("outer_chain", ["none", "from", "another"])
@pytest.mark.parametrize("inner_chain", ["none", "from", "another"])
def test_native_exceptiongroup(pytester: Pytester, outer_chain, inner_chain) -> None:
    _exceptiongroup_common(pytester, outer_chain, inner_chain, native=True)


@pytest.mark.parametrize("outer_chain", ["none", "from", "another"])
@pytest.mark.parametrize("inner_chain", ["none", "from", "another"])
def test_exceptiongroup(pytester: Pytester, outer_chain, inner_chain) -> None:
    # with py>=3.11 does not depend on exceptiongroup, though there is a toxenv for it
    pytest.importorskip("exceptiongroup")
    _exceptiongroup_common(pytester, outer_chain, inner_chain, native=False)


@pytest.mark.parametrize("tbstyle", ("long", "short", "auto", "line", "native"))
def test_all_entries_hidden(pytester: Pytester, tbstyle: str) -> None:
    """Regression test for #10903."""
    pytester.makepyfile(
        """
        def test():
            __tracebackhide__ = True
            1 / 0
    """
    )
    result = pytester.runpytest("--tb", tbstyle)
    assert result.ret == 1
    if tbstyle != "line":
        result.stdout.fnmatch_lines(["*ZeroDivisionError: division by zero"])
    if tbstyle not in ("line", "native"):
        result.stdout.fnmatch_lines(["All traceback entries are hidden.*"])


def test_hidden_entries_of_chained_exceptions_are_not_shown(pytester: Pytester) -> None:
    """Hidden entries of chained exceptions are not shown (#1904)."""
    p = pytester.makepyfile(
        """
        def g1():
            __tracebackhide__ = True
            str.does_not_exist

        def f3():
            __tracebackhide__ = True
            1 / 0

        def f2():
            try:
                f3()
            except Exception:
                g1()

        def f1():
            __tracebackhide__ = True
            f2()

        def test():
            f1()
        """
    )
    result = pytester.runpytest(str(p), "--tb=short")
    assert result.ret == 1
    result.stdout.fnmatch_lines(
        [
            "*.py:11: in f2",
            "    f3()",
            "E   ZeroDivisionError: division by zero",
            "",
            "During handling of the above exception, another exception occurred:",
            "*.py:20: in test",
            "    f1()",
            "*.py:13: in f2",
            "    g1()",
            "E   AttributeError:*'does_not_exist'",
        ],
        consecutive=True,
    )


def add_note(err: BaseException, msg: str) -> None:
    """Adds a note to an exception inplace."""
    if sys.version_info < (3, 11):
        err.__notes__ = [*getattr(err, "__notes__", []), msg]  # type: ignore[attr-defined]
    else:
        err.add_note(msg)


@pytest.mark.parametrize(
    "error,notes,match",
    [
        (Exception("test"), [], "test"),
        (AssertionError("foo"), ["bar"], "bar"),
        (AssertionError("foo"), ["bar", "baz"], "bar"),
        (AssertionError("foo"), ["bar", "baz"], "baz"),
        (ValueError("foo"), ["bar", "baz"], re.compile(r"bar\nbaz", re.MULTILINE)),
        (ValueError("foo"), ["bar", "baz"], re.compile(r"BAZ", re.IGNORECASE)),
    ],
)
def test_check_error_notes_success(
    error: Exception, notes: list[str], match: str
) -> None:
    for note in notes:
        add_note(error, note)

    with pytest.raises(Exception, match=match):
        raise error


@pytest.mark.parametrize(
    "error, notes, match",
    [
        (Exception("test"), [], "foo"),
        (AssertionError("foo"), ["bar"], "baz"),
        (AssertionError("foo"), ["bar"], "foo\nbaz"),
    ],
)
def test_check_error_notes_failure(
    error: Exception, notes: list[str], match: str
) -> None:
    for note in notes:
        add_note(error, note)

    with pytest.raises(AssertionError):
        with pytest.raises(type(error), match=match):
            raise error
