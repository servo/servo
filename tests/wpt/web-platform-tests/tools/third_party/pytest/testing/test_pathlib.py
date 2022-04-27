import os.path
import pickle
import sys
import unittest.mock
from pathlib import Path
from textwrap import dedent
from types import ModuleType
from typing import Any
from typing import Generator

import pytest
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pathlib import bestrelpath
from _pytest.pathlib import commonpath
from _pytest.pathlib import ensure_deletable
from _pytest.pathlib import fnmatch_ex
from _pytest.pathlib import get_extended_length_path_str
from _pytest.pathlib import get_lock_path
from _pytest.pathlib import import_path
from _pytest.pathlib import ImportPathMismatchError
from _pytest.pathlib import insert_missing_modules
from _pytest.pathlib import maybe_delete_a_numbered_dir
from _pytest.pathlib import module_name_from_path
from _pytest.pathlib import resolve_package_path
from _pytest.pathlib import symlink_or_skip
from _pytest.pathlib import visit
from _pytest.tmpdir import TempPathFactory


class TestFNMatcherPort:
    """Test our port of py.common.FNMatcher (fnmatch_ex)."""

    if sys.platform == "win32":
        drv1 = "c:"
        drv2 = "d:"
    else:
        drv1 = "/c"
        drv2 = "/d"

    @pytest.mark.parametrize(
        "pattern, path",
        [
            ("*.py", "foo.py"),
            ("*.py", "bar/foo.py"),
            ("test_*.py", "foo/test_foo.py"),
            ("tests/*.py", "tests/foo.py"),
            (f"{drv1}/*.py", f"{drv1}/foo.py"),
            (f"{drv1}/foo/*.py", f"{drv1}/foo/foo.py"),
            ("tests/**/test*.py", "tests/foo/test_foo.py"),
            ("tests/**/doc/test*.py", "tests/foo/bar/doc/test_foo.py"),
            ("tests/**/doc/**/test*.py", "tests/foo/doc/bar/test_foo.py"),
        ],
    )
    def test_matching(self, pattern: str, path: str) -> None:
        assert fnmatch_ex(pattern, path)

    def test_matching_abspath(self) -> None:
        abspath = os.path.abspath(os.path.join("tests/foo.py"))
        assert fnmatch_ex("tests/foo.py", abspath)

    @pytest.mark.parametrize(
        "pattern, path",
        [
            ("*.py", "foo.pyc"),
            ("*.py", "foo/foo.pyc"),
            ("tests/*.py", "foo/foo.py"),
            (f"{drv1}/*.py", f"{drv2}/foo.py"),
            (f"{drv1}/foo/*.py", f"{drv2}/foo/foo.py"),
            ("tests/**/test*.py", "tests/foo.py"),
            ("tests/**/test*.py", "foo/test_foo.py"),
            ("tests/**/doc/test*.py", "tests/foo/bar/doc/foo.py"),
            ("tests/**/doc/test*.py", "tests/foo/bar/test_foo.py"),
        ],
    )
    def test_not_matching(self, pattern: str, path: str) -> None:
        assert not fnmatch_ex(pattern, path)


class TestImportPath:
    """

    Most of the tests here were copied from py lib's tests for "py.local.path.pyimport".

    Having our own pyimport-like function is inline with removing py.path dependency in the future.
    """

    @pytest.fixture(scope="session")
    def path1(self, tmp_path_factory: TempPathFactory) -> Generator[Path, None, None]:
        path = tmp_path_factory.mktemp("path")
        self.setuptestfs(path)
        yield path
        assert path.joinpath("samplefile").exists()

    def setuptestfs(self, path: Path) -> None:
        # print "setting up test fs for", repr(path)
        samplefile = path / "samplefile"
        samplefile.write_text("samplefile\n")

        execfile = path / "execfile"
        execfile.write_text("x=42")

        execfilepy = path / "execfile.py"
        execfilepy.write_text("x=42")

        d = {1: 2, "hello": "world", "answer": 42}
        path.joinpath("samplepickle").write_bytes(pickle.dumps(d, 1))

        sampledir = path / "sampledir"
        sampledir.mkdir()
        sampledir.joinpath("otherfile").touch()

        otherdir = path / "otherdir"
        otherdir.mkdir()
        otherdir.joinpath("__init__.py").touch()

        module_a = otherdir / "a.py"
        module_a.write_text("from .b import stuff as result\n")
        module_b = otherdir / "b.py"
        module_b.write_text('stuff="got it"\n')
        module_c = otherdir / "c.py"
        module_c.write_text(
            dedent(
                """
            import pluggy;
            import otherdir.a
            value = otherdir.a.result
        """
            )
        )
        module_d = otherdir / "d.py"
        module_d.write_text(
            dedent(
                """
            import pluggy;
            from otherdir import a
            value2 = a.result
        """
            )
        )

    def test_smoke_test(self, path1: Path) -> None:
        obj = import_path(path1 / "execfile.py", root=path1)
        assert obj.x == 42  # type: ignore[attr-defined]
        assert obj.__name__ == "execfile"

    def test_renamed_dir_creates_mismatch(
        self, tmp_path: Path, monkeypatch: MonkeyPatch
    ) -> None:
        tmp_path.joinpath("a").mkdir()
        p = tmp_path.joinpath("a", "test_x123.py")
        p.touch()
        import_path(p, root=tmp_path)
        tmp_path.joinpath("a").rename(tmp_path.joinpath("b"))
        with pytest.raises(ImportPathMismatchError):
            import_path(tmp_path.joinpath("b", "test_x123.py"), root=tmp_path)

        # Errors can be ignored.
        monkeypatch.setenv("PY_IGNORE_IMPORTMISMATCH", "1")
        import_path(tmp_path.joinpath("b", "test_x123.py"), root=tmp_path)

        # PY_IGNORE_IMPORTMISMATCH=0 does not ignore error.
        monkeypatch.setenv("PY_IGNORE_IMPORTMISMATCH", "0")
        with pytest.raises(ImportPathMismatchError):
            import_path(tmp_path.joinpath("b", "test_x123.py"), root=tmp_path)

    def test_messy_name(self, tmp_path: Path) -> None:
        # https://bitbucket.org/hpk42/py-trunk/issue/129
        path = tmp_path / "foo__init__.py"
        path.touch()
        module = import_path(path, root=tmp_path)
        assert module.__name__ == "foo__init__"

    def test_dir(self, tmp_path: Path) -> None:
        p = tmp_path / "hello_123"
        p.mkdir()
        p_init = p / "__init__.py"
        p_init.touch()
        m = import_path(p, root=tmp_path)
        assert m.__name__ == "hello_123"
        m = import_path(p_init, root=tmp_path)
        assert m.__name__ == "hello_123"

    def test_a(self, path1: Path) -> None:
        otherdir = path1 / "otherdir"
        mod = import_path(otherdir / "a.py", root=path1)
        assert mod.result == "got it"  # type: ignore[attr-defined]
        assert mod.__name__ == "otherdir.a"

    def test_b(self, path1: Path) -> None:
        otherdir = path1 / "otherdir"
        mod = import_path(otherdir / "b.py", root=path1)
        assert mod.stuff == "got it"  # type: ignore[attr-defined]
        assert mod.__name__ == "otherdir.b"

    def test_c(self, path1: Path) -> None:
        otherdir = path1 / "otherdir"
        mod = import_path(otherdir / "c.py", root=path1)
        assert mod.value == "got it"  # type: ignore[attr-defined]

    def test_d(self, path1: Path) -> None:
        otherdir = path1 / "otherdir"
        mod = import_path(otherdir / "d.py", root=path1)
        assert mod.value2 == "got it"  # type: ignore[attr-defined]

    def test_import_after(self, tmp_path: Path) -> None:
        tmp_path.joinpath("xxxpackage").mkdir()
        tmp_path.joinpath("xxxpackage", "__init__.py").touch()
        mod1path = tmp_path.joinpath("xxxpackage", "module1.py")
        mod1path.touch()
        mod1 = import_path(mod1path, root=tmp_path)
        assert mod1.__name__ == "xxxpackage.module1"
        from xxxpackage import module1

        assert module1 is mod1

    def test_check_filepath_consistency(
        self, monkeypatch: MonkeyPatch, tmp_path: Path
    ) -> None:
        name = "pointsback123"
        p = tmp_path.joinpath(name + ".py")
        p.touch()
        for ending in (".pyc", ".pyo"):
            mod = ModuleType(name)
            pseudopath = tmp_path.joinpath(name + ending)
            pseudopath.touch()
            mod.__file__ = str(pseudopath)
            monkeypatch.setitem(sys.modules, name, mod)
            newmod = import_path(p, root=tmp_path)
            assert mod == newmod
        monkeypatch.undo()
        mod = ModuleType(name)
        pseudopath = tmp_path.joinpath(name + "123.py")
        pseudopath.touch()
        mod.__file__ = str(pseudopath)
        monkeypatch.setitem(sys.modules, name, mod)
        with pytest.raises(ImportPathMismatchError) as excinfo:
            import_path(p, root=tmp_path)
        modname, modfile, orig = excinfo.value.args
        assert modname == name
        assert modfile == str(pseudopath)
        assert orig == p
        assert issubclass(ImportPathMismatchError, ImportError)

    def test_issue131_on__init__(self, tmp_path: Path) -> None:
        # __init__.py files may be namespace packages, and thus the
        # __file__ of an imported module may not be ourselves
        # see issue
        tmp_path.joinpath("proja").mkdir()
        p1 = tmp_path.joinpath("proja", "__init__.py")
        p1.touch()
        tmp_path.joinpath("sub", "proja").mkdir(parents=True)
        p2 = tmp_path.joinpath("sub", "proja", "__init__.py")
        p2.touch()
        m1 = import_path(p1, root=tmp_path)
        m2 = import_path(p2, root=tmp_path)
        assert m1 == m2

    def test_ensuresyspath_append(self, tmp_path: Path) -> None:
        root1 = tmp_path / "root1"
        root1.mkdir()
        file1 = root1 / "x123.py"
        file1.touch()
        assert str(root1) not in sys.path
        import_path(file1, mode="append", root=tmp_path)
        assert str(root1) == sys.path[-1]
        assert str(root1) not in sys.path[:-1]

    def test_invalid_path(self, tmp_path: Path) -> None:
        with pytest.raises(ImportError):
            import_path(tmp_path / "invalid.py", root=tmp_path)

    @pytest.fixture
    def simple_module(self, tmp_path: Path) -> Path:
        fn = tmp_path / "_src/tests/mymod.py"
        fn.parent.mkdir(parents=True)
        fn.write_text("def foo(x): return 40 + x")
        return fn

    def test_importmode_importlib(self, simple_module: Path, tmp_path: Path) -> None:
        """`importlib` mode does not change sys.path."""
        module = import_path(simple_module, mode="importlib", root=tmp_path)
        assert module.foo(2) == 42  # type: ignore[attr-defined]
        assert str(simple_module.parent) not in sys.path
        assert module.__name__ in sys.modules
        assert module.__name__ == "_src.tests.mymod"
        assert "_src" in sys.modules
        assert "_src.tests" in sys.modules

    def test_importmode_twice_is_different_module(
        self, simple_module: Path, tmp_path: Path
    ) -> None:
        """`importlib` mode always returns a new module."""
        module1 = import_path(simple_module, mode="importlib", root=tmp_path)
        module2 = import_path(simple_module, mode="importlib", root=tmp_path)
        assert module1 is not module2

    def test_no_meta_path_found(
        self, simple_module: Path, monkeypatch: MonkeyPatch, tmp_path: Path
    ) -> None:
        """Even without any meta_path should still import module."""
        monkeypatch.setattr(sys, "meta_path", [])
        module = import_path(simple_module, mode="importlib", root=tmp_path)
        assert module.foo(2) == 42  # type: ignore[attr-defined]

        # mode='importlib' fails if no spec is found to load the module
        import importlib.util

        monkeypatch.setattr(
            importlib.util, "spec_from_file_location", lambda *args: None
        )
        with pytest.raises(ImportError):
            import_path(simple_module, mode="importlib", root=tmp_path)


def test_resolve_package_path(tmp_path: Path) -> None:
    pkg = tmp_path / "pkg1"
    pkg.mkdir()
    (pkg / "__init__.py").touch()
    (pkg / "subdir").mkdir()
    (pkg / "subdir/__init__.py").touch()
    assert resolve_package_path(pkg) == pkg
    assert resolve_package_path(pkg.joinpath("subdir", "__init__.py")) == pkg


def test_package_unimportable(tmp_path: Path) -> None:
    pkg = tmp_path / "pkg1-1"
    pkg.mkdir()
    pkg.joinpath("__init__.py").touch()
    subdir = pkg.joinpath("subdir")
    subdir.mkdir()
    pkg.joinpath("subdir/__init__.py").touch()
    assert resolve_package_path(subdir) == subdir
    xyz = subdir.joinpath("xyz.py")
    xyz.touch()
    assert resolve_package_path(xyz) == subdir
    assert not resolve_package_path(pkg)


def test_access_denied_during_cleanup(tmp_path: Path, monkeypatch: MonkeyPatch) -> None:
    """Ensure that deleting a numbered dir does not fail because of OSErrors (#4262)."""
    path = tmp_path / "temp-1"
    path.mkdir()

    def renamed_failed(*args):
        raise OSError("access denied")

    monkeypatch.setattr(Path, "rename", renamed_failed)

    lock_path = get_lock_path(path)
    maybe_delete_a_numbered_dir(path)
    assert not lock_path.is_file()


def test_long_path_during_cleanup(tmp_path: Path) -> None:
    """Ensure that deleting long path works (particularly on Windows (#6775))."""
    path = (tmp_path / ("a" * 250)).resolve()
    if sys.platform == "win32":
        # make sure that the full path is > 260 characters without any
        # component being over 260 characters
        assert len(str(path)) > 260
        extended_path = "\\\\?\\" + str(path)
    else:
        extended_path = str(path)
    os.mkdir(extended_path)
    assert os.path.isdir(extended_path)
    maybe_delete_a_numbered_dir(path)
    assert not os.path.isdir(extended_path)


def test_get_extended_length_path_str() -> None:
    assert get_extended_length_path_str(r"c:\foo") == r"\\?\c:\foo"
    assert get_extended_length_path_str(r"\\share\foo") == r"\\?\UNC\share\foo"
    assert get_extended_length_path_str(r"\\?\UNC\share\foo") == r"\\?\UNC\share\foo"
    assert get_extended_length_path_str(r"\\?\c:\foo") == r"\\?\c:\foo"


def test_suppress_error_removing_lock(tmp_path: Path) -> None:
    """ensure_deletable should be resilient if lock file cannot be removed (#5456, #7491)"""
    path = tmp_path / "dir"
    path.mkdir()
    lock = get_lock_path(path)
    lock.touch()
    mtime = lock.stat().st_mtime

    with unittest.mock.patch.object(Path, "unlink", side_effect=OSError) as m:
        assert not ensure_deletable(
            path, consider_lock_dead_if_created_before=mtime + 30
        )
        assert m.call_count == 1
    assert lock.is_file()

    with unittest.mock.patch.object(Path, "is_file", side_effect=OSError) as m:
        assert not ensure_deletable(
            path, consider_lock_dead_if_created_before=mtime + 30
        )
        assert m.call_count == 1
    assert lock.is_file()

    # check now that we can remove the lock file in normal circumstances
    assert ensure_deletable(path, consider_lock_dead_if_created_before=mtime + 30)
    assert not lock.is_file()


def test_bestrelpath() -> None:
    curdir = Path("/foo/bar/baz/path")
    assert bestrelpath(curdir, curdir) == "."
    assert bestrelpath(curdir, curdir / "hello" / "world") == "hello" + os.sep + "world"
    assert bestrelpath(curdir, curdir.parent / "sister") == ".." + os.sep + "sister"
    assert bestrelpath(curdir, curdir.parent) == ".."
    assert bestrelpath(curdir, Path("hello")) == "hello"


def test_commonpath() -> None:
    path = Path("/foo/bar/baz/path")
    subpath = path / "sampledir"
    assert commonpath(path, subpath) == path
    assert commonpath(subpath, path) == path
    assert commonpath(Path(str(path) + "suffix"), path) == path.parent
    assert commonpath(path, path.parent.parent) == path.parent.parent


def test_visit_ignores_errors(tmp_path: Path) -> None:
    symlink_or_skip("recursive", tmp_path / "recursive")
    tmp_path.joinpath("foo").write_bytes(b"")
    tmp_path.joinpath("bar").write_bytes(b"")

    assert [
        entry.name for entry in visit(str(tmp_path), recurse=lambda entry: False)
    ] == ["bar", "foo"]


@pytest.mark.skipif(not sys.platform.startswith("win"), reason="Windows only")
def test_samefile_false_negatives(tmp_path: Path, monkeypatch: MonkeyPatch) -> None:
    """
    import_file() should not raise ImportPathMismatchError if the paths are exactly
    equal on Windows. It seems directories mounted as UNC paths make os.path.samefile
    return False, even when they are clearly equal.
    """
    module_path = tmp_path.joinpath("my_module.py")
    module_path.write_text("def foo(): return 42")
    monkeypatch.syspath_prepend(tmp_path)

    with monkeypatch.context() as mp:
        # Forcibly make os.path.samefile() return False here to ensure we are comparing
        # the paths too. Using a context to narrow the patch as much as possible given
        # this is an important system function.
        mp.setattr(os.path, "samefile", lambda x, y: False)
        module = import_path(module_path, root=tmp_path)
    assert getattr(module, "foo")() == 42


class TestImportLibMode:
    @pytest.mark.skipif(sys.version_info < (3, 7), reason="Dataclasses in Python3.7+")
    def test_importmode_importlib_with_dataclass(self, tmp_path: Path) -> None:
        """Ensure that importlib mode works with a module containing dataclasses (#7856)."""
        fn = tmp_path.joinpath("_src/tests/test_dataclass.py")
        fn.parent.mkdir(parents=True)
        fn.write_text(
            dedent(
                """
                from dataclasses import dataclass

                @dataclass
                class Data:
                    value: str
                """
            )
        )

        module = import_path(fn, mode="importlib", root=tmp_path)
        Data: Any = getattr(module, "Data")
        data = Data(value="foo")
        assert data.value == "foo"
        assert data.__module__ == "_src.tests.test_dataclass"

    def test_importmode_importlib_with_pickle(self, tmp_path: Path) -> None:
        """Ensure that importlib mode works with pickle (#7859)."""
        fn = tmp_path.joinpath("_src/tests/test_pickle.py")
        fn.parent.mkdir(parents=True)
        fn.write_text(
            dedent(
                """
                import pickle

                def _action():
                    return 42

                def round_trip():
                    s = pickle.dumps(_action)
                    return pickle.loads(s)
                """
            )
        )

        module = import_path(fn, mode="importlib", root=tmp_path)
        round_trip = getattr(module, "round_trip")
        action = round_trip()
        assert action() == 42

    def test_importmode_importlib_with_pickle_separate_modules(
        self, tmp_path: Path
    ) -> None:
        """
        Ensure that importlib mode works can load pickles that look similar but are
        defined in separate modules.
        """
        fn1 = tmp_path.joinpath("_src/m1/tests/test.py")
        fn1.parent.mkdir(parents=True)
        fn1.write_text(
            dedent(
                """
                import attr
                import pickle

                @attr.s(auto_attribs=True)
                class Data:
                    x: int = 42
                """
            )
        )

        fn2 = tmp_path.joinpath("_src/m2/tests/test.py")
        fn2.parent.mkdir(parents=True)
        fn2.write_text(
            dedent(
                """
                import attr
                import pickle

                @attr.s(auto_attribs=True)
                class Data:
                    x: str = ""
                """
            )
        )

        import pickle

        def round_trip(obj):
            s = pickle.dumps(obj)
            return pickle.loads(s)

        module = import_path(fn1, mode="importlib", root=tmp_path)
        Data1 = getattr(module, "Data")

        module = import_path(fn2, mode="importlib", root=tmp_path)
        Data2 = getattr(module, "Data")

        assert round_trip(Data1(20)) == Data1(20)
        assert round_trip(Data2("hello")) == Data2("hello")
        assert Data1.__module__ == "_src.m1.tests.test"
        assert Data2.__module__ == "_src.m2.tests.test"

    def test_module_name_from_path(self, tmp_path: Path) -> None:
        result = module_name_from_path(tmp_path / "src/tests/test_foo.py", tmp_path)
        assert result == "src.tests.test_foo"

        # Path is not relative to root dir: use the full path to obtain the module name.
        result = module_name_from_path(Path("/home/foo/test_foo.py"), Path("/bar"))
        assert result == "home.foo.test_foo"

    def test_insert_missing_modules(self) -> None:
        modules = {"src.tests.foo": ModuleType("src.tests.foo")}
        insert_missing_modules(modules, "src.tests.foo")
        assert sorted(modules) == ["src", "src.tests", "src.tests.foo"]

        mod = ModuleType("mod", doc="My Module")
        modules = {"src": mod}
        insert_missing_modules(modules, "src")
        assert modules == {"src": mod}

        modules = {}
        insert_missing_modules(modules, "")
        assert modules == {}
