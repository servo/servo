# mypy: allow-untyped-defs
import os
from pathlib import Path
import re
import sys
import textwrap
from typing import Dict
from typing import Generator
from typing import Type

from _pytest.monkeypatch import MonkeyPatch
from _pytest.pytester import Pytester
import pytest


@pytest.fixture
def mp() -> Generator[MonkeyPatch, None, None]:
    cwd = os.getcwd()
    sys_path = list(sys.path)
    yield MonkeyPatch()
    sys.path[:] = sys_path
    os.chdir(cwd)


def test_setattr() -> None:
    class A:
        x = 1

    monkeypatch = MonkeyPatch()
    pytest.raises(AttributeError, monkeypatch.setattr, A, "notexists", 2)
    monkeypatch.setattr(A, "y", 2, raising=False)
    assert A.y == 2  # type: ignore
    monkeypatch.undo()
    assert not hasattr(A, "y")

    monkeypatch = MonkeyPatch()
    monkeypatch.setattr(A, "x", 2)
    assert A.x == 2
    monkeypatch.setattr(A, "x", 3)
    assert A.x == 3
    monkeypatch.undo()
    assert A.x == 1

    A.x = 5
    monkeypatch.undo()  # double-undo makes no modification
    assert A.x == 5

    with pytest.raises(TypeError):
        monkeypatch.setattr(A, "y")  # type: ignore[call-overload]


class TestSetattrWithImportPath:
    def test_string_expression(self, monkeypatch: MonkeyPatch) -> None:
        with monkeypatch.context() as mp:
            mp.setattr("os.path.abspath", lambda x: "hello2")
            assert os.path.abspath("123") == "hello2"

    def test_string_expression_class(self, monkeypatch: MonkeyPatch) -> None:
        with monkeypatch.context() as mp:
            mp.setattr("_pytest.config.Config", 42)
            import _pytest

            assert _pytest.config.Config == 42  # type: ignore

    def test_unicode_string(self, monkeypatch: MonkeyPatch) -> None:
        with monkeypatch.context() as mp:
            mp.setattr("_pytest.config.Config", 42)
            import _pytest

            assert _pytest.config.Config == 42  # type: ignore
            mp.delattr("_pytest.config.Config")

    def test_wrong_target(self, monkeypatch: MonkeyPatch) -> None:
        with pytest.raises(TypeError):
            monkeypatch.setattr(None, None)  # type: ignore[call-overload]

    def test_unknown_import(self, monkeypatch: MonkeyPatch) -> None:
        with pytest.raises(ImportError):
            monkeypatch.setattr("unkn123.classx", None)

    def test_unknown_attr(self, monkeypatch: MonkeyPatch) -> None:
        with pytest.raises(AttributeError):
            monkeypatch.setattr("os.path.qweqwe", None)

    def test_unknown_attr_non_raising(self, monkeypatch: MonkeyPatch) -> None:
        # https://github.com/pytest-dev/pytest/issues/746
        with monkeypatch.context() as mp:
            mp.setattr("os.path.qweqwe", 42, raising=False)
            assert os.path.qweqwe == 42  # type: ignore

    def test_delattr(self, monkeypatch: MonkeyPatch) -> None:
        with monkeypatch.context() as mp:
            mp.delattr("os.path.abspath")
            assert not hasattr(os.path, "abspath")
            mp.undo()
            assert os.path.abspath  # type:ignore[truthy-function]


def test_delattr() -> None:
    class A:
        x = 1

    monkeypatch = MonkeyPatch()
    monkeypatch.delattr(A, "x")
    assert not hasattr(A, "x")
    monkeypatch.undo()
    assert A.x == 1

    monkeypatch = MonkeyPatch()
    monkeypatch.delattr(A, "x")
    pytest.raises(AttributeError, monkeypatch.delattr, A, "y")
    monkeypatch.delattr(A, "y", raising=False)
    monkeypatch.setattr(A, "x", 5, raising=False)
    assert A.x == 5
    monkeypatch.undo()
    assert A.x == 1


def test_setitem() -> None:
    d = {"x": 1}
    monkeypatch = MonkeyPatch()
    monkeypatch.setitem(d, "x", 2)
    monkeypatch.setitem(d, "y", 1700)
    monkeypatch.setitem(d, "y", 1700)
    assert d["x"] == 2
    assert d["y"] == 1700
    monkeypatch.setitem(d, "x", 3)
    assert d["x"] == 3
    monkeypatch.undo()
    assert d["x"] == 1
    assert "y" not in d
    d["x"] = 5
    monkeypatch.undo()
    assert d["x"] == 5


def test_setitem_deleted_meanwhile() -> None:
    d: Dict[str, object] = {}
    monkeypatch = MonkeyPatch()
    monkeypatch.setitem(d, "x", 2)
    del d["x"]
    monkeypatch.undo()
    assert not d


@pytest.mark.parametrize("before", [True, False])
def test_setenv_deleted_meanwhile(before: bool) -> None:
    key = "qwpeoip123"
    if before:
        os.environ[key] = "world"
    monkeypatch = MonkeyPatch()
    monkeypatch.setenv(key, "hello")
    del os.environ[key]
    monkeypatch.undo()
    if before:
        assert os.environ[key] == "world"
        del os.environ[key]
    else:
        assert key not in os.environ


def test_delitem() -> None:
    d: Dict[str, object] = {"x": 1}
    monkeypatch = MonkeyPatch()
    monkeypatch.delitem(d, "x")
    assert "x" not in d
    monkeypatch.delitem(d, "y", raising=False)
    pytest.raises(KeyError, monkeypatch.delitem, d, "y")
    assert not d
    monkeypatch.setitem(d, "y", 1700)
    assert d["y"] == 1700
    d["hello"] = "world"
    monkeypatch.setitem(d, "x", 1500)
    assert d["x"] == 1500
    monkeypatch.undo()
    assert d == {"hello": "world", "x": 1}


def test_setenv() -> None:
    monkeypatch = MonkeyPatch()
    with pytest.warns(pytest.PytestWarning):
        monkeypatch.setenv("XYZ123", 2)  # type: ignore[arg-type]
    import os

    assert os.environ["XYZ123"] == "2"
    monkeypatch.undo()
    assert "XYZ123" not in os.environ


def test_delenv() -> None:
    name = "xyz1234"
    assert name not in os.environ
    monkeypatch = MonkeyPatch()
    pytest.raises(KeyError, monkeypatch.delenv, name, raising=True)
    monkeypatch.delenv(name, raising=False)
    monkeypatch.undo()
    os.environ[name] = "1"
    try:
        monkeypatch = MonkeyPatch()
        monkeypatch.delenv(name)
        assert name not in os.environ
        monkeypatch.setenv(name, "3")
        assert os.environ[name] == "3"
        monkeypatch.undo()
        assert os.environ[name] == "1"
    finally:
        if name in os.environ:
            del os.environ[name]


class TestEnvironWarnings:
    """
    os.environ keys and values should be native strings, otherwise it will cause problems with other modules (notably
    subprocess). On Python 2 os.environ accepts anything without complaining, while Python 3 does the right thing
    and raises an error.
    """

    VAR_NAME = "PYTEST_INTERNAL_MY_VAR"

    def test_setenv_non_str_warning(self, monkeypatch: MonkeyPatch) -> None:
        value = 2
        msg = (
            "Value of environment variable PYTEST_INTERNAL_MY_VAR type should be str, "
            "but got 2 (type: int); converted to str implicitly"
        )
        with pytest.warns(pytest.PytestWarning, match=re.escape(msg)):
            monkeypatch.setenv(str(self.VAR_NAME), value)  # type: ignore[arg-type]


def test_setenv_prepend() -> None:
    import os

    monkeypatch = MonkeyPatch()
    monkeypatch.setenv("XYZ123", "2", prepend="-")
    monkeypatch.setenv("XYZ123", "3", prepend="-")
    assert os.environ["XYZ123"] == "3-2"
    monkeypatch.undo()
    assert "XYZ123" not in os.environ


def test_monkeypatch_plugin(pytester: Pytester) -> None:
    reprec = pytester.inline_runsource(
        """
        def test_method(monkeypatch):
            assert monkeypatch.__class__.__name__ == "MonkeyPatch"
    """
    )
    res = reprec.countoutcomes()
    assert tuple(res) == (1, 0, 0), res


def test_syspath_prepend(mp: MonkeyPatch) -> None:
    old = list(sys.path)
    mp.syspath_prepend("world")
    mp.syspath_prepend("hello")
    assert sys.path[0] == "hello"
    assert sys.path[1] == "world"
    mp.undo()
    assert sys.path == old
    mp.undo()
    assert sys.path == old


def test_syspath_prepend_double_undo(mp: MonkeyPatch) -> None:
    old_syspath = sys.path[:]
    try:
        mp.syspath_prepend("hello world")
        mp.undo()
        sys.path.append("more hello world")
        mp.undo()
        assert sys.path[-1] == "more hello world"
    finally:
        sys.path[:] = old_syspath


def test_chdir_with_path_local(mp: MonkeyPatch, tmp_path: Path) -> None:
    mp.chdir(tmp_path)
    assert os.getcwd() == str(tmp_path)


def test_chdir_with_str(mp: MonkeyPatch, tmp_path: Path) -> None:
    mp.chdir(str(tmp_path))
    assert os.getcwd() == str(tmp_path)


def test_chdir_undo(mp: MonkeyPatch, tmp_path: Path) -> None:
    cwd = os.getcwd()
    mp.chdir(tmp_path)
    mp.undo()
    assert os.getcwd() == cwd


def test_chdir_double_undo(mp: MonkeyPatch, tmp_path: Path) -> None:
    mp.chdir(str(tmp_path))
    mp.undo()
    os.chdir(tmp_path)
    mp.undo()
    assert os.getcwd() == str(tmp_path)


def test_issue185_time_breaks(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import time
        def test_m(monkeypatch):
            def f():
                raise Exception
            monkeypatch.setattr(time, "time", f)
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(
        """
        *1 passed*
    """
    )


def test_importerror(pytester: Pytester) -> None:
    p = pytester.mkpydir("package")
    p.joinpath("a.py").write_text(
        textwrap.dedent(
            """\
        import doesnotexist

        x = 1
    """
        ),
        encoding="utf-8",
    )
    pytester.path.joinpath("test_importerror.py").write_text(
        textwrap.dedent(
            """\
        def test_importerror(monkeypatch):
            monkeypatch.setattr('package.a.x', 2)
    """
        ),
        encoding="utf-8",
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(
        """
        *import error in package.a: No module named 'doesnotexist'*
    """
    )


class Sample:
    @staticmethod
    def hello() -> bool:
        return True


class SampleInherit(Sample):
    pass


@pytest.mark.parametrize(
    "Sample",
    [Sample, SampleInherit],
    ids=["new", "new-inherit"],
)
def test_issue156_undo_staticmethod(Sample: Type[Sample]) -> None:
    monkeypatch = MonkeyPatch()

    monkeypatch.setattr(Sample, "hello", None)
    assert Sample.hello is None

    monkeypatch.undo()  # type: ignore[unreachable]
    assert Sample.hello()


def test_undo_class_descriptors_delattr() -> None:
    class SampleParent:
        @classmethod
        def hello(_cls):
            pass

        @staticmethod
        def world():
            pass

    class SampleChild(SampleParent):
        pass

    monkeypatch = MonkeyPatch()

    original_hello = SampleChild.hello
    original_world = SampleChild.world
    monkeypatch.delattr(SampleParent, "hello")
    monkeypatch.delattr(SampleParent, "world")
    assert getattr(SampleParent, "hello", None) is None
    assert getattr(SampleParent, "world", None) is None

    monkeypatch.undo()
    assert original_hello == SampleChild.hello
    assert original_world == SampleChild.world


def test_issue1338_name_resolving() -> None:
    pytest.importorskip("requests")
    monkeypatch = MonkeyPatch()
    try:
        monkeypatch.delattr("requests.sessions.Session.request")
    finally:
        monkeypatch.undo()


def test_context() -> None:
    monkeypatch = MonkeyPatch()

    import functools
    import inspect

    with monkeypatch.context() as m:
        m.setattr(functools, "partial", 3)
        assert not inspect.isclass(functools.partial)
    assert inspect.isclass(functools.partial)


def test_context_classmethod() -> None:
    class A:
        x = 1

    with MonkeyPatch.context() as m:
        m.setattr(A, "x", 2)
        assert A.x == 2
    assert A.x == 1


@pytest.mark.filterwarnings(r"ignore:.*\bpkg_resources\b:DeprecationWarning")
def test_syspath_prepend_with_namespace_packages(
    pytester: Pytester, monkeypatch: MonkeyPatch
) -> None:
    for dirname in "hello", "world":
        d = pytester.mkdir(dirname)
        ns = d.joinpath("ns_pkg")
        ns.mkdir()
        ns.joinpath("__init__.py").write_text(
            "__import__('pkg_resources').declare_namespace(__name__)", encoding="utf-8"
        )
        lib = ns.joinpath(dirname)
        lib.mkdir()
        lib.joinpath("__init__.py").write_text(
            "def check(): return %r" % dirname, encoding="utf-8"
        )

    monkeypatch.syspath_prepend("hello")
    import ns_pkg.hello

    assert ns_pkg.hello.check() == "hello"

    with pytest.raises(ImportError):
        import ns_pkg.world

    # Prepending should call fixup_namespace_packages.
    monkeypatch.syspath_prepend("world")
    import ns_pkg.world

    assert ns_pkg.world.check() == "world"

    # Should invalidate caches via importlib.invalidate_caches.
    modules_tmpdir = pytester.mkdir("modules_tmpdir")
    monkeypatch.syspath_prepend(str(modules_tmpdir))
    modules_tmpdir.joinpath("main_app.py").write_text("app = True", encoding="utf-8")
    from main_app import app  # noqa: F401
