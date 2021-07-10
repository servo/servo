import os.path
import sys
import unittest.mock
from textwrap import dedent

import py

import pytest
from _pytest.pathlib import bestrelpath
from _pytest.pathlib import commonpath
from _pytest.pathlib import ensure_deletable
from _pytest.pathlib import fnmatch_ex
from _pytest.pathlib import get_extended_length_path_str
from _pytest.pathlib import get_lock_path
from _pytest.pathlib import import_path
from _pytest.pathlib import ImportPathMismatchError
from _pytest.pathlib import maybe_delete_a_numbered_dir
from _pytest.pathlib import Path
from _pytest.pathlib import resolve_package_path


class TestFNMatcherPort:
    """Test that our port of py.common.FNMatcher (fnmatch_ex) produces the
    same results as the original py.path.local.fnmatch method."""

    @pytest.fixture(params=["pathlib", "py.path"])
    def match(self, request):
        if request.param == "py.path":

            def match_(pattern, path):
                return py.path.local(path).fnmatch(pattern)

        else:
            assert request.param == "pathlib"

            def match_(pattern, path):
                return fnmatch_ex(pattern, path)

        return match_

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
            (drv1 + "/*.py", drv1 + "/foo.py"),
            (drv1 + "/foo/*.py", drv1 + "/foo/foo.py"),
            ("tests/**/test*.py", "tests/foo/test_foo.py"),
            ("tests/**/doc/test*.py", "tests/foo/bar/doc/test_foo.py"),
            ("tests/**/doc/**/test*.py", "tests/foo/doc/bar/test_foo.py"),
        ],
    )
    def test_matching(self, match, pattern, path):
        assert match(pattern, path)

    def test_matching_abspath(self, match):
        abspath = os.path.abspath(os.path.join("tests/foo.py"))
        assert match("tests/foo.py", abspath)

    @pytest.mark.parametrize(
        "pattern, path",
        [
            ("*.py", "foo.pyc"),
            ("*.py", "foo/foo.pyc"),
            ("tests/*.py", "foo/foo.py"),
            (drv1 + "/*.py", drv2 + "/foo.py"),
            (drv1 + "/foo/*.py", drv2 + "/foo/foo.py"),
            ("tests/**/test*.py", "tests/foo.py"),
            ("tests/**/test*.py", "foo/test_foo.py"),
            ("tests/**/doc/test*.py", "tests/foo/bar/doc/foo.py"),
            ("tests/**/doc/test*.py", "tests/foo/bar/test_foo.py"),
        ],
    )
    def test_not_matching(self, match, pattern, path):
        assert not match(pattern, path)


class TestImportPath:
    """

    Most of the tests here were copied from py lib's tests for "py.local.path.pyimport".

    Having our own pyimport-like function is inline with removing py.path dependency in the future.
    """

    @pytest.fixture(scope="session")
    def path1(self, tmpdir_factory):
        path = tmpdir_factory.mktemp("path")
        self.setuptestfs(path)
        yield path
        assert path.join("samplefile").check()

    def setuptestfs(self, path):
        # print "setting up test fs for", repr(path)
        samplefile = path.ensure("samplefile")
        samplefile.write("samplefile\n")

        execfile = path.ensure("execfile")
        execfile.write("x=42")

        execfilepy = path.ensure("execfile.py")
        execfilepy.write("x=42")

        d = {1: 2, "hello": "world", "answer": 42}
        path.ensure("samplepickle").dump(d)

        sampledir = path.ensure("sampledir", dir=1)
        sampledir.ensure("otherfile")

        otherdir = path.ensure("otherdir", dir=1)
        otherdir.ensure("__init__.py")

        module_a = otherdir.ensure("a.py")
        module_a.write("from .b import stuff as result\n")
        module_b = otherdir.ensure("b.py")
        module_b.write('stuff="got it"\n')
        module_c = otherdir.ensure("c.py")
        module_c.write(
            dedent(
                """
            import py;
            import otherdir.a
            value = otherdir.a.result
        """
            )
        )
        module_d = otherdir.ensure("d.py")
        module_d.write(
            dedent(
                """
            import py;
            from otherdir import a
            value2 = a.result
        """
            )
        )

    def test_smoke_test(self, path1):
        obj = import_path(path1.join("execfile.py"))
        assert obj.x == 42  # type: ignore[attr-defined]
        assert obj.__name__ == "execfile"

    def test_renamed_dir_creates_mismatch(self, tmpdir, monkeypatch):
        p = tmpdir.ensure("a", "test_x123.py")
        import_path(p)
        tmpdir.join("a").move(tmpdir.join("b"))
        with pytest.raises(ImportPathMismatchError):
            import_path(tmpdir.join("b", "test_x123.py"))

        # Errors can be ignored.
        monkeypatch.setenv("PY_IGNORE_IMPORTMISMATCH", "1")
        import_path(tmpdir.join("b", "test_x123.py"))

        # PY_IGNORE_IMPORTMISMATCH=0 does not ignore error.
        monkeypatch.setenv("PY_IGNORE_IMPORTMISMATCH", "0")
        with pytest.raises(ImportPathMismatchError):
            import_path(tmpdir.join("b", "test_x123.py"))

    def test_messy_name(self, tmpdir):
        # http://bitbucket.org/hpk42/py-trunk/issue/129
        path = tmpdir.ensure("foo__init__.py")
        module = import_path(path)
        assert module.__name__ == "foo__init__"

    def test_dir(self, tmpdir):
        p = tmpdir.join("hello_123")
        p_init = p.ensure("__init__.py")
        m = import_path(p)
        assert m.__name__ == "hello_123"
        m = import_path(p_init)
        assert m.__name__ == "hello_123"

    def test_a(self, path1):
        otherdir = path1.join("otherdir")
        mod = import_path(otherdir.join("a.py"))
        assert mod.result == "got it"  # type: ignore[attr-defined]
        assert mod.__name__ == "otherdir.a"

    def test_b(self, path1):
        otherdir = path1.join("otherdir")
        mod = import_path(otherdir.join("b.py"))
        assert mod.stuff == "got it"  # type: ignore[attr-defined]
        assert mod.__name__ == "otherdir.b"

    def test_c(self, path1):
        otherdir = path1.join("otherdir")
        mod = import_path(otherdir.join("c.py"))
        assert mod.value == "got it"  # type: ignore[attr-defined]

    def test_d(self, path1):
        otherdir = path1.join("otherdir")
        mod = import_path(otherdir.join("d.py"))
        assert mod.value2 == "got it"  # type: ignore[attr-defined]

    def test_import_after(self, tmpdir):
        tmpdir.ensure("xxxpackage", "__init__.py")
        mod1path = tmpdir.ensure("xxxpackage", "module1.py")
        mod1 = import_path(mod1path)
        assert mod1.__name__ == "xxxpackage.module1"
        from xxxpackage import module1

        assert module1 is mod1

    def test_check_filepath_consistency(self, monkeypatch, tmpdir):
        name = "pointsback123"
        ModuleType = type(os)
        p = tmpdir.ensure(name + ".py")
        for ending in (".pyc", ".pyo"):
            mod = ModuleType(name)
            pseudopath = tmpdir.ensure(name + ending)
            mod.__file__ = str(pseudopath)
            monkeypatch.setitem(sys.modules, name, mod)
            newmod = import_path(p)
            assert mod == newmod
        monkeypatch.undo()
        mod = ModuleType(name)
        pseudopath = tmpdir.ensure(name + "123.py")
        mod.__file__ = str(pseudopath)
        monkeypatch.setitem(sys.modules, name, mod)
        with pytest.raises(ImportPathMismatchError) as excinfo:
            import_path(p)
        modname, modfile, orig = excinfo.value.args
        assert modname == name
        assert modfile == pseudopath
        assert orig == p
        assert issubclass(ImportPathMismatchError, ImportError)

    def test_issue131_on__init__(self, tmpdir):
        # __init__.py files may be namespace packages, and thus the
        # __file__ of an imported module may not be ourselves
        # see issue
        p1 = tmpdir.ensure("proja", "__init__.py")
        p2 = tmpdir.ensure("sub", "proja", "__init__.py")
        m1 = import_path(p1)
        m2 = import_path(p2)
        assert m1 == m2

    def test_ensuresyspath_append(self, tmpdir):
        root1 = tmpdir.mkdir("root1")
        file1 = root1.ensure("x123.py")
        assert str(root1) not in sys.path
        import_path(file1, mode="append")
        assert str(root1) == sys.path[-1]
        assert str(root1) not in sys.path[:-1]

    def test_invalid_path(self, tmpdir):
        with pytest.raises(ImportError):
            import_path(tmpdir.join("invalid.py"))

    @pytest.fixture
    def simple_module(self, tmpdir):
        fn = tmpdir.join("mymod.py")
        fn.write(
            dedent(
                """
            def foo(x): return 40 + x
            """
            )
        )
        return fn

    def test_importmode_importlib(self, simple_module):
        """`importlib` mode does not change sys.path."""
        module = import_path(simple_module, mode="importlib")
        assert module.foo(2) == 42  # type: ignore[attr-defined]
        assert simple_module.dirname not in sys.path

    def test_importmode_twice_is_different_module(self, simple_module):
        """`importlib` mode always returns a new module."""
        module1 = import_path(simple_module, mode="importlib")
        module2 = import_path(simple_module, mode="importlib")
        assert module1 is not module2

    def test_no_meta_path_found(self, simple_module, monkeypatch):
        """Even without any meta_path should still import module."""
        monkeypatch.setattr(sys, "meta_path", [])
        module = import_path(simple_module, mode="importlib")
        assert module.foo(2) == 42  # type: ignore[attr-defined]

        # mode='importlib' fails if no spec is found to load the module
        import importlib.util

        monkeypatch.setattr(
            importlib.util, "spec_from_file_location", lambda *args: None
        )
        with pytest.raises(ImportError):
            import_path(simple_module, mode="importlib")


def test_resolve_package_path(tmp_path):
    pkg = tmp_path / "pkg1"
    pkg.mkdir()
    (pkg / "__init__.py").touch()
    (pkg / "subdir").mkdir()
    (pkg / "subdir/__init__.py").touch()
    assert resolve_package_path(pkg) == pkg
    assert resolve_package_path(pkg.joinpath("subdir", "__init__.py")) == pkg


def test_package_unimportable(tmp_path):
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


def test_access_denied_during_cleanup(tmp_path, monkeypatch):
    """Ensure that deleting a numbered dir does not fail because of OSErrors (#4262)."""
    path = tmp_path / "temp-1"
    path.mkdir()

    def renamed_failed(*args):
        raise OSError("access denied")

    monkeypatch.setattr(Path, "rename", renamed_failed)

    lock_path = get_lock_path(path)
    maybe_delete_a_numbered_dir(path)
    assert not lock_path.is_file()


def test_long_path_during_cleanup(tmp_path):
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


def test_get_extended_length_path_str():
    assert get_extended_length_path_str(r"c:\foo") == r"\\?\c:\foo"
    assert get_extended_length_path_str(r"\\share\foo") == r"\\?\UNC\share\foo"
    assert get_extended_length_path_str(r"\\?\UNC\share\foo") == r"\\?\UNC\share\foo"
    assert get_extended_length_path_str(r"\\?\c:\foo") == r"\\?\c:\foo"


def test_suppress_error_removing_lock(tmp_path):
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
