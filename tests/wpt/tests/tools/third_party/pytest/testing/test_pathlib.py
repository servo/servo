# mypy: allow-untyped-defs
import errno
import importlib.abc
import importlib.machinery
import os.path
from pathlib import Path
import pickle
import shutil
import sys
from textwrap import dedent
from types import ModuleType
from typing import Any
from typing import Generator
from typing import Iterator
from typing import Optional
from typing import Sequence
from typing import Tuple
import unittest.mock

from _pytest.monkeypatch import MonkeyPatch
from _pytest.pathlib import bestrelpath
from _pytest.pathlib import commonpath
from _pytest.pathlib import compute_module_name
from _pytest.pathlib import CouldNotResolvePathError
from _pytest.pathlib import ensure_deletable
from _pytest.pathlib import fnmatch_ex
from _pytest.pathlib import get_extended_length_path_str
from _pytest.pathlib import get_lock_path
from _pytest.pathlib import import_path
from _pytest.pathlib import ImportMode
from _pytest.pathlib import ImportPathMismatchError
from _pytest.pathlib import insert_missing_modules
from _pytest.pathlib import is_importable
from _pytest.pathlib import maybe_delete_a_numbered_dir
from _pytest.pathlib import module_name_from_path
from _pytest.pathlib import resolve_package_path
from _pytest.pathlib import resolve_pkg_root_and_module_name
from _pytest.pathlib import safe_exists
from _pytest.pathlib import symlink_or_skip
from _pytest.pathlib import visit
from _pytest.pytester import Pytester
from _pytest.pytester import RunResult
from _pytest.tmpdir import TempPathFactory
import pytest


@pytest.fixture(autouse=True)
def autouse_pytester(pytester: Pytester) -> None:
    """
    Fixture to make pytester() being autouse for all tests in this module.

    pytester makes sure to restore sys.path to its previous state, and many tests in this module
    import modules and change sys.path because of that, so common module names such as "test" or "test.conftest"
    end up leaking to tests in other modules.

    Note: we might consider extracting the sys.path restoration aspect into its own fixture, and apply it
    to the entire test suite always.
    """


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


@pytest.fixture(params=[True, False])
def ns_param(request: pytest.FixtureRequest) -> bool:
    """
    Simple parametrized fixture for tests which call import_path() with consider_namespace_packages
    using True and False.
    """
    return bool(request.param)


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

    @pytest.fixture(autouse=True)
    def preserve_sys(self):
        with unittest.mock.patch.dict(sys.modules):
            with unittest.mock.patch.object(sys, "path", list(sys.path)):
                yield

    def setuptestfs(self, path: Path) -> None:
        # print "setting up test fs for", repr(path)
        samplefile = path / "samplefile"
        samplefile.write_text("samplefile\n", encoding="utf-8")

        execfile = path / "execfile"
        execfile.write_text("x=42", encoding="utf-8")

        execfilepy = path / "execfile.py"
        execfilepy.write_text("x=42", encoding="utf-8")

        d = {1: 2, "hello": "world", "answer": 42}
        path.joinpath("samplepickle").write_bytes(pickle.dumps(d, 1))

        sampledir = path / "sampledir"
        sampledir.mkdir()
        sampledir.joinpath("otherfile").touch()

        otherdir = path / "otherdir"
        otherdir.mkdir()
        otherdir.joinpath("__init__.py").touch()

        module_a = otherdir / "a.py"
        module_a.write_text("from .b import stuff as result\n", encoding="utf-8")
        module_b = otherdir / "b.py"
        module_b.write_text('stuff="got it"\n', encoding="utf-8")
        module_c = otherdir / "c.py"
        module_c.write_text(
            dedent(
                """
            import pluggy;
            import otherdir.a
            value = otherdir.a.result
        """
            ),
            encoding="utf-8",
        )
        module_d = otherdir / "d.py"
        module_d.write_text(
            dedent(
                """
            import pluggy;
            from otherdir import a
            value2 = a.result
        """
            ),
            encoding="utf-8",
        )

    def test_smoke_test(self, path1: Path, ns_param: bool) -> None:
        obj = import_path(
            path1 / "execfile.py", root=path1, consider_namespace_packages=ns_param
        )
        assert obj.x == 42
        assert obj.__name__ == "execfile"

    def test_import_path_missing_file(self, path1: Path, ns_param: bool) -> None:
        with pytest.raises(ImportPathMismatchError):
            import_path(
                path1 / "sampledir", root=path1, consider_namespace_packages=ns_param
            )

    def test_renamed_dir_creates_mismatch(
        self, tmp_path: Path, monkeypatch: MonkeyPatch, ns_param: bool
    ) -> None:
        tmp_path.joinpath("a").mkdir()
        p = tmp_path.joinpath("a", "test_x123.py")
        p.touch()
        import_path(p, root=tmp_path, consider_namespace_packages=ns_param)
        tmp_path.joinpath("a").rename(tmp_path.joinpath("b"))
        with pytest.raises(ImportPathMismatchError):
            import_path(
                tmp_path.joinpath("b", "test_x123.py"),
                root=tmp_path,
                consider_namespace_packages=ns_param,
            )

        # Errors can be ignored.
        monkeypatch.setenv("PY_IGNORE_IMPORTMISMATCH", "1")
        import_path(
            tmp_path.joinpath("b", "test_x123.py"),
            root=tmp_path,
            consider_namespace_packages=ns_param,
        )

        # PY_IGNORE_IMPORTMISMATCH=0 does not ignore error.
        monkeypatch.setenv("PY_IGNORE_IMPORTMISMATCH", "0")
        with pytest.raises(ImportPathMismatchError):
            import_path(
                tmp_path.joinpath("b", "test_x123.py"),
                root=tmp_path,
                consider_namespace_packages=ns_param,
            )

    def test_messy_name(self, tmp_path: Path, ns_param: bool) -> None:
        # https://bitbucket.org/hpk42/py-trunk/issue/129
        path = tmp_path / "foo__init__.py"
        path.touch()
        module = import_path(path, root=tmp_path, consider_namespace_packages=ns_param)
        assert module.__name__ == "foo__init__"

    def test_dir(self, tmp_path: Path, ns_param: bool) -> None:
        p = tmp_path / "hello_123"
        p.mkdir()
        p_init = p / "__init__.py"
        p_init.touch()
        m = import_path(p, root=tmp_path, consider_namespace_packages=ns_param)
        assert m.__name__ == "hello_123"
        m = import_path(p_init, root=tmp_path, consider_namespace_packages=ns_param)
        assert m.__name__ == "hello_123"

    def test_a(self, path1: Path, ns_param: bool) -> None:
        otherdir = path1 / "otherdir"
        mod = import_path(
            otherdir / "a.py", root=path1, consider_namespace_packages=ns_param
        )
        assert mod.result == "got it"
        assert mod.__name__ == "otherdir.a"

    def test_b(self, path1: Path, ns_param: bool) -> None:
        otherdir = path1 / "otherdir"
        mod = import_path(
            otherdir / "b.py", root=path1, consider_namespace_packages=ns_param
        )
        assert mod.stuff == "got it"
        assert mod.__name__ == "otherdir.b"

    def test_c(self, path1: Path, ns_param: bool) -> None:
        otherdir = path1 / "otherdir"
        mod = import_path(
            otherdir / "c.py", root=path1, consider_namespace_packages=ns_param
        )
        assert mod.value == "got it"

    def test_d(self, path1: Path, ns_param: bool) -> None:
        otherdir = path1 / "otherdir"
        mod = import_path(
            otherdir / "d.py", root=path1, consider_namespace_packages=ns_param
        )
        assert mod.value2 == "got it"

    def test_import_after(self, tmp_path: Path, ns_param: bool) -> None:
        tmp_path.joinpath("xxxpackage").mkdir()
        tmp_path.joinpath("xxxpackage", "__init__.py").touch()
        mod1path = tmp_path.joinpath("xxxpackage", "module1.py")
        mod1path.touch()
        mod1 = import_path(
            mod1path, root=tmp_path, consider_namespace_packages=ns_param
        )
        assert mod1.__name__ == "xxxpackage.module1"
        from xxxpackage import module1

        assert module1 is mod1

    def test_check_filepath_consistency(
        self, monkeypatch: MonkeyPatch, tmp_path: Path, ns_param: bool
    ) -> None:
        name = "pointsback123"
        p = tmp_path.joinpath(name + ".py")
        p.touch()
        with monkeypatch.context() as mp:
            for ending in (".pyc", ".pyo"):
                mod = ModuleType(name)
                pseudopath = tmp_path.joinpath(name + ending)
                pseudopath.touch()
                mod.__file__ = str(pseudopath)
                mp.setitem(sys.modules, name, mod)
                newmod = import_path(
                    p, root=tmp_path, consider_namespace_packages=ns_param
                )
                assert mod == newmod
        mod = ModuleType(name)
        pseudopath = tmp_path.joinpath(name + "123.py")
        pseudopath.touch()
        mod.__file__ = str(pseudopath)
        monkeypatch.setitem(sys.modules, name, mod)
        with pytest.raises(ImportPathMismatchError) as excinfo:
            import_path(p, root=tmp_path, consider_namespace_packages=ns_param)
        modname, modfile, orig = excinfo.value.args
        assert modname == name
        assert modfile == str(pseudopath)
        assert orig == p
        assert issubclass(ImportPathMismatchError, ImportError)

    def test_ensuresyspath_append(self, tmp_path: Path, ns_param: bool) -> None:
        root1 = tmp_path / "root1"
        root1.mkdir()
        file1 = root1 / "x123.py"
        file1.touch()
        assert str(root1) not in sys.path
        import_path(
            file1, mode="append", root=tmp_path, consider_namespace_packages=ns_param
        )
        assert str(root1) == sys.path[-1]
        assert str(root1) not in sys.path[:-1]

    def test_invalid_path(self, tmp_path: Path, ns_param: bool) -> None:
        with pytest.raises(ImportError):
            import_path(
                tmp_path / "invalid.py",
                root=tmp_path,
                consider_namespace_packages=ns_param,
            )

    @pytest.fixture
    def simple_module(
        self, tmp_path: Path, request: pytest.FixtureRequest
    ) -> Iterator[Path]:
        name = f"mymod_{request.node.name}"
        fn = tmp_path / f"_src/tests/{name}.py"
        fn.parent.mkdir(parents=True)
        fn.write_text("def foo(x): return 40 + x", encoding="utf-8")
        module_name = module_name_from_path(fn, root=tmp_path)
        yield fn
        sys.modules.pop(module_name, None)

    def test_importmode_importlib(
        self,
        simple_module: Path,
        tmp_path: Path,
        request: pytest.FixtureRequest,
        ns_param: bool,
    ) -> None:
        """`importlib` mode does not change sys.path."""
        module = import_path(
            simple_module,
            mode="importlib",
            root=tmp_path,
            consider_namespace_packages=ns_param,
        )
        assert module.foo(2) == 42
        assert str(simple_module.parent) not in sys.path
        assert module.__name__ in sys.modules
        assert module.__name__ == f"_src.tests.mymod_{request.node.name}"
        assert "_src" in sys.modules
        assert "_src.tests" in sys.modules

    def test_remembers_previous_imports(
        self, simple_module: Path, tmp_path: Path, ns_param: bool
    ) -> None:
        """`importlib` mode called remembers previous module (#10341, #10811)."""
        module1 = import_path(
            simple_module,
            mode="importlib",
            root=tmp_path,
            consider_namespace_packages=ns_param,
        )
        module2 = import_path(
            simple_module,
            mode="importlib",
            root=tmp_path,
            consider_namespace_packages=ns_param,
        )
        assert module1 is module2

    def test_no_meta_path_found(
        self,
        simple_module: Path,
        monkeypatch: MonkeyPatch,
        tmp_path: Path,
        ns_param: bool,
    ) -> None:
        """Even without any meta_path should still import module."""
        monkeypatch.setattr(sys, "meta_path", [])
        module = import_path(
            simple_module,
            mode="importlib",
            root=tmp_path,
            consider_namespace_packages=ns_param,
        )
        assert module.foo(2) == 42

        # mode='importlib' fails if no spec is found to load the module
        import importlib.util

        # Force module to be re-imported.
        del sys.modules[module.__name__]

        monkeypatch.setattr(
            importlib.util, "spec_from_file_location", lambda *args: None
        )
        with pytest.raises(ImportError):
            import_path(
                simple_module,
                mode="importlib",
                root=tmp_path,
                consider_namespace_packages=False,
            )


def test_resolve_package_path(tmp_path: Path) -> None:
    pkg = tmp_path / "pkg1"
    pkg.mkdir()
    (pkg / "__init__.py").touch()
    (pkg / "subdir").mkdir()
    (pkg / "subdir/__init__.py").touch()
    assert resolve_package_path(pkg) == pkg
    assert resolve_package_path(pkg / "subdir/__init__.py") == pkg


def test_package_unimportable(tmp_path: Path) -> None:
    pkg = tmp_path / "pkg1-1"
    pkg.mkdir()
    pkg.joinpath("__init__.py").touch()
    subdir = pkg / "subdir"
    subdir.mkdir()
    (pkg / "subdir/__init__.py").touch()
    assert resolve_package_path(subdir) == subdir
    xyz = subdir / "xyz.py"
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
    module_path.write_text("def foo(): return 42", encoding="utf-8")
    monkeypatch.syspath_prepend(tmp_path)

    with monkeypatch.context() as mp:
        # Forcibly make os.path.samefile() return False here to ensure we are comparing
        # the paths too. Using a context to narrow the patch as much as possible given
        # this is an important system function.
        mp.setattr(os.path, "samefile", lambda x, y: False)
        module = import_path(
            module_path, root=tmp_path, consider_namespace_packages=False
        )
    assert getattr(module, "foo")() == 42


class TestImportLibMode:
    def test_importmode_importlib_with_dataclass(
        self, tmp_path: Path, ns_param: bool
    ) -> None:
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
            ),
            encoding="utf-8",
        )

        module = import_path(
            fn, mode="importlib", root=tmp_path, consider_namespace_packages=ns_param
        )
        Data: Any = getattr(module, "Data")
        data = Data(value="foo")
        assert data.value == "foo"
        assert data.__module__ == "_src.tests.test_dataclass"

        # Ensure we do not import the same module again (#11475).
        module2 = import_path(
            fn, mode="importlib", root=tmp_path, consider_namespace_packages=ns_param
        )
        assert module is module2

    def test_importmode_importlib_with_pickle(
        self, tmp_path: Path, ns_param: bool
    ) -> None:
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
            ),
            encoding="utf-8",
        )

        module = import_path(
            fn, mode="importlib", root=tmp_path, consider_namespace_packages=ns_param
        )
        round_trip = getattr(module, "round_trip")
        action = round_trip()
        assert action() == 42

        # Ensure we do not import the same module again (#11475).
        module2 = import_path(
            fn, mode="importlib", root=tmp_path, consider_namespace_packages=ns_param
        )
        assert module is module2

    def test_importmode_importlib_with_pickle_separate_modules(
        self, tmp_path: Path, ns_param: bool
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
                import dataclasses
                import pickle

                @dataclasses.dataclass
                class Data:
                    x: int = 42
                """
            ),
            encoding="utf-8",
        )

        fn2 = tmp_path.joinpath("_src/m2/tests/test.py")
        fn2.parent.mkdir(parents=True)
        fn2.write_text(
            dedent(
                """
                import dataclasses
                import pickle

                @dataclasses.dataclass
                class Data:
                    x: str = ""
                """
            ),
            encoding="utf-8",
        )

        import pickle

        def round_trip(obj):
            s = pickle.dumps(obj)
            return pickle.loads(s)

        module = import_path(
            fn1, mode="importlib", root=tmp_path, consider_namespace_packages=ns_param
        )
        Data1 = getattr(module, "Data")

        module = import_path(
            fn2, mode="importlib", root=tmp_path, consider_namespace_packages=ns_param
        )
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

        # Importing __init__.py files should return the package as module name.
        result = module_name_from_path(tmp_path / "src/app/__init__.py", tmp_path)
        assert result == "src.app"

        # Unless __init__.py file is at the root, in which case we cannot have an empty module name.
        result = module_name_from_path(tmp_path / "__init__.py", tmp_path)
        assert result == "__init__"

        # Modules which start with "." are considered relative and will not be imported
        # unless part of a package, so we replace it with a "_" when generating the fake module name.
        result = module_name_from_path(tmp_path / ".env/tests/test_foo.py", tmp_path)
        assert result == "_env.tests.test_foo"

        # We want to avoid generating extra intermediate modules if some directory just happens
        # to contain a "." in the name.
        result = module_name_from_path(
            tmp_path / ".env.310/tests/test_foo.py", tmp_path
        )
        assert result == "_env_310.tests.test_foo"

    def test_resolve_pkg_root_and_module_name(
        self, tmp_path: Path, monkeypatch: MonkeyPatch, pytester: Pytester
    ) -> None:
        # Create a directory structure first without __init__.py files.
        (tmp_path / "src/app/core").mkdir(parents=True)
        models_py = tmp_path / "src/app/core/models.py"
        models_py.touch()

        with pytest.raises(CouldNotResolvePathError):
            _ = resolve_pkg_root_and_module_name(models_py)

        # Create the __init__.py files, it should now resolve to a proper module name.
        (tmp_path / "src/app/__init__.py").touch()
        (tmp_path / "src/app/core/__init__.py").touch()
        assert resolve_pkg_root_and_module_name(
            models_py, consider_namespace_packages=True
        ) == (
            tmp_path / "src",
            "app.core.models",
        )

        # If we add tmp_path to sys.path, src becomes a namespace package.
        monkeypatch.syspath_prepend(tmp_path)
        validate_namespace_package(pytester, [tmp_path], ["src.app.core.models"])

        assert resolve_pkg_root_and_module_name(
            models_py, consider_namespace_packages=True
        ) == (
            tmp_path,
            "src.app.core.models",
        )
        assert resolve_pkg_root_and_module_name(
            models_py, consider_namespace_packages=False
        ) == (
            tmp_path / "src",
            "app.core.models",
        )

    def test_insert_missing_modules(
        self, monkeypatch: MonkeyPatch, tmp_path: Path
    ) -> None:
        monkeypatch.chdir(tmp_path)
        # Use 'xxx' and 'xxy' as parent names as they are unlikely to exist and
        # don't end up being imported.
        modules = {"xxx.tests.foo": ModuleType("xxx.tests.foo")}
        insert_missing_modules(modules, "xxx.tests.foo")
        assert sorted(modules) == ["xxx", "xxx.tests", "xxx.tests.foo"]

        mod = ModuleType("mod", doc="My Module")
        modules = {"xxy": mod}
        insert_missing_modules(modules, "xxy")
        assert modules == {"xxy": mod}

        modules = {}
        insert_missing_modules(modules, "")
        assert modules == {}

    def test_parent_contains_child_module_attribute(
        self, monkeypatch: MonkeyPatch, tmp_path: Path
    ):
        monkeypatch.chdir(tmp_path)
        # Use 'xxx' and 'xxy' as parent names as they are unlikely to exist and
        # don't end up being imported.
        modules = {"xxx.tests.foo": ModuleType("xxx.tests.foo")}
        insert_missing_modules(modules, "xxx.tests.foo")
        assert sorted(modules) == ["xxx", "xxx.tests", "xxx.tests.foo"]
        assert modules["xxx"].tests is modules["xxx.tests"]
        assert modules["xxx.tests"].foo is modules["xxx.tests.foo"]

    def test_importlib_package(
        self, monkeypatch: MonkeyPatch, tmp_path: Path, ns_param: bool
    ):
        """
        Importing a package using --importmode=importlib should not import the
        package's __init__.py file more than once (#11306).
        """
        monkeypatch.chdir(tmp_path)
        monkeypatch.syspath_prepend(tmp_path)

        package_name = "importlib_import_package"
        tmp_path.joinpath(package_name).mkdir()
        init = tmp_path.joinpath(f"{package_name}/__init__.py")
        init.write_text(
            dedent(
                """
                from .singleton import Singleton

                instance = Singleton()
                """
            ),
            encoding="ascii",
        )
        singleton = tmp_path.joinpath(f"{package_name}/singleton.py")
        singleton.write_text(
            dedent(
                """
                class Singleton:
                    INSTANCES = []

                    def __init__(self) -> None:
                        self.INSTANCES.append(self)
                        if len(self.INSTANCES) > 1:
                            raise RuntimeError("Already initialized")
                """
            ),
            encoding="ascii",
        )

        mod = import_path(
            init,
            root=tmp_path,
            mode=ImportMode.importlib,
            consider_namespace_packages=ns_param,
        )
        assert len(mod.instance.INSTANCES) == 1
        # Ensure we do not import the same module again (#11475).
        mod2 = import_path(
            init,
            root=tmp_path,
            mode=ImportMode.importlib,
            consider_namespace_packages=ns_param,
        )
        assert mod is mod2

    def test_importlib_root_is_package(self, pytester: Pytester) -> None:
        """
        Regression for importing a `__init__`.py file that is at the root
        (#11417).
        """
        pytester.makepyfile(__init__="")
        pytester.makepyfile(
            """
            def test_my_test():
                assert True
            """
        )

        result = pytester.runpytest("--import-mode=importlib")
        result.stdout.fnmatch_lines("* 1 passed *")

    def create_installed_doctests_and_tests_dir(
        self, path: Path, monkeypatch: MonkeyPatch
    ) -> Tuple[Path, Path, Path]:
        """
        Create a directory structure where the application code is installed in a virtual environment,
        and the tests are in an outside ".tests" directory.

        Return the paths to the core module (installed in the virtualenv), and the test modules.
        """
        app = path / "src/app"
        app.mkdir(parents=True)
        (app / "__init__.py").touch()
        core_py = app / "core.py"
        core_py.write_text(
            dedent(
                """
                def foo():
                    '''
                    >>> 1 + 1
                    2
                    '''
                """
            ),
            encoding="ascii",
        )

        # Install it into a site-packages directory, and add it to sys.path, mimicking what
        # happens when installing into a virtualenv.
        site_packages = path / ".env/lib/site-packages"
        site_packages.mkdir(parents=True)
        shutil.copytree(app, site_packages / "app")
        assert (site_packages / "app/core.py").is_file()

        monkeypatch.syspath_prepend(site_packages)

        # Create the tests files, outside 'src' and the virtualenv.
        # We use the same test name on purpose, but in different directories, to ensure
        # this works as advertised.
        conftest_path1 = path / ".tests/a/conftest.py"
        conftest_path1.parent.mkdir(parents=True)
        conftest_path1.write_text(
            dedent(
                """
                import pytest
                @pytest.fixture
                def a_fix(): return "a"
                """
            ),
            encoding="ascii",
        )
        test_path1 = path / ".tests/a/test_core.py"
        test_path1.write_text(
            dedent(
                """
                import app.core
                def test(a_fix):
                    assert a_fix == "a"
                """,
            ),
            encoding="ascii",
        )

        conftest_path2 = path / ".tests/b/conftest.py"
        conftest_path2.parent.mkdir(parents=True)
        conftest_path2.write_text(
            dedent(
                """
                import pytest
                @pytest.fixture
                def b_fix(): return "b"
                """
            ),
            encoding="ascii",
        )

        test_path2 = path / ".tests/b/test_core.py"
        test_path2.write_text(
            dedent(
                """
                import app.core
                def test(b_fix):
                    assert b_fix == "b"
                """,
            ),
            encoding="ascii",
        )
        return (site_packages / "app/core.py"), test_path1, test_path2

    def test_import_using_normal_mechanism_first(
        self, monkeypatch: MonkeyPatch, pytester: Pytester, ns_param: bool
    ) -> None:
        """
        Test import_path imports from the canonical location when possible first, only
        falling back to its normal flow when the module being imported is not reachable via sys.path (#11475).
        """
        core_py, test_path1, test_path2 = self.create_installed_doctests_and_tests_dir(
            pytester.path, monkeypatch
        )

        # core_py is reached from sys.path, so should be imported normally.
        mod = import_path(
            core_py,
            mode="importlib",
            root=pytester.path,
            consider_namespace_packages=ns_param,
        )
        assert mod.__name__ == "app.core"
        assert mod.__file__ and Path(mod.__file__) == core_py

        # Ensure we do not import the same module again (#11475).
        mod2 = import_path(
            core_py,
            mode="importlib",
            root=pytester.path,
            consider_namespace_packages=ns_param,
        )
        assert mod is mod2

        # tests are not reachable from sys.path, so they are imported as a standalone modules.
        # Instead of '.tests.a.test_core', we import as "_tests.a.test_core" because
        # importlib considers module names starting with '.' to be local imports.
        mod = import_path(
            test_path1,
            mode="importlib",
            root=pytester.path,
            consider_namespace_packages=ns_param,
        )
        assert mod.__name__ == "_tests.a.test_core"

        # Ensure we do not import the same module again (#11475).
        mod2 = import_path(
            test_path1,
            mode="importlib",
            root=pytester.path,
            consider_namespace_packages=ns_param,
        )
        assert mod is mod2

        mod = import_path(
            test_path2,
            mode="importlib",
            root=pytester.path,
            consider_namespace_packages=ns_param,
        )
        assert mod.__name__ == "_tests.b.test_core"

        # Ensure we do not import the same module again (#11475).
        mod2 = import_path(
            test_path2,
            mode="importlib",
            root=pytester.path,
            consider_namespace_packages=ns_param,
        )
        assert mod is mod2

    def test_import_using_normal_mechanism_first_integration(
        self, monkeypatch: MonkeyPatch, pytester: Pytester, ns_param: bool
    ) -> None:
        """
        Same test as above, but verify the behavior calling pytest.

        We should not make this call in the same test as above, as the modules have already
        been imported by separate import_path() calls.
        """
        core_py, test_path1, test_path2 = self.create_installed_doctests_and_tests_dir(
            pytester.path, monkeypatch
        )
        result = pytester.runpytest(
            "--import-mode=importlib",
            "-o",
            f"consider_namespace_packages={ns_param}",
            "--doctest-modules",
            "--pyargs",
            "app",
            "./.tests",
        )
        result.stdout.fnmatch_lines(
            [
                f"{core_py.relative_to(pytester.path)} . *",
                f"{test_path1.relative_to(pytester.path)} . *",
                f"{test_path2.relative_to(pytester.path)} . *",
                "* 3 passed*",
            ]
        )

    def test_import_path_imports_correct_file(
        self, pytester: Pytester, ns_param: bool
    ) -> None:
        """
        Import the module by the given path, even if other module with the same name
        is reachable from sys.path.
        """
        pytester.syspathinsert()
        # Create a 'x.py' module reachable from sys.path that raises AssertionError
        # if imported.
        x_at_root = pytester.path / "x.py"
        x_at_root.write_text("raise AssertionError('x at root')", encoding="ascii")

        # Create another x.py module, but in some subdirectories to ensure it is not
        # accessible from sys.path.
        x_in_sub_folder = pytester.path / "a/b/x.py"
        x_in_sub_folder.parent.mkdir(parents=True)
        x_in_sub_folder.write_text("X = 'a/b/x'", encoding="ascii")

        # Import our x.py module from the subdirectories.
        # The 'x.py' module from sys.path was not imported for sure because
        # otherwise we would get an AssertionError.
        mod = import_path(
            x_in_sub_folder,
            mode=ImportMode.importlib,
            root=pytester.path,
            consider_namespace_packages=ns_param,
        )
        assert mod.__file__ and Path(mod.__file__) == x_in_sub_folder
        assert mod.X == "a/b/x"

        mod2 = import_path(
            x_in_sub_folder,
            mode=ImportMode.importlib,
            root=pytester.path,
            consider_namespace_packages=ns_param,
        )
        assert mod is mod2

        # Attempt to import root 'x.py'.
        with pytest.raises(AssertionError, match="x at root"):
            _ = import_path(
                x_at_root,
                mode=ImportMode.importlib,
                root=pytester.path,
                consider_namespace_packages=ns_param,
            )


def test_safe_exists(tmp_path: Path) -> None:
    d = tmp_path.joinpath("some_dir")
    d.mkdir()
    assert safe_exists(d) is True

    f = tmp_path.joinpath("some_file")
    f.touch()
    assert safe_exists(f) is True

    # Use unittest.mock() as a context manager to have a very narrow
    # patch lifetime.
    p = tmp_path.joinpath("some long filename" * 100)
    with unittest.mock.patch.object(
        Path,
        "exists",
        autospec=True,
        side_effect=OSError(errno.ENAMETOOLONG, "name too long"),
    ):
        assert safe_exists(p) is False

    with unittest.mock.patch.object(
        Path,
        "exists",
        autospec=True,
        side_effect=ValueError("name too long"),
    ):
        assert safe_exists(p) is False


def test_import_sets_module_as_attribute(pytester: Pytester) -> None:
    """Unittest test for #12194."""
    pytester.path.joinpath("foo/bar/baz").mkdir(parents=True)
    pytester.path.joinpath("foo/__init__.py").touch()
    pytester.path.joinpath("foo/bar/__init__.py").touch()
    pytester.path.joinpath("foo/bar/baz/__init__.py").touch()
    pytester.syspathinsert()

    # Import foo.bar.baz and ensure parent modules also ended up imported.
    baz = import_path(
        pytester.path.joinpath("foo/bar/baz/__init__.py"),
        mode=ImportMode.importlib,
        root=pytester.path,
        consider_namespace_packages=False,
    )
    assert baz.__name__ == "foo.bar.baz"
    foo = sys.modules["foo"]
    assert foo.__name__ == "foo"
    bar = sys.modules["foo.bar"]
    assert bar.__name__ == "foo.bar"

    # Check parent modules have an attribute pointing to their children.
    assert bar.baz is baz
    assert foo.bar is bar

    # Ensure we returned the "foo.bar" module cached in sys.modules.
    bar_2 = import_path(
        pytester.path.joinpath("foo/bar/__init__.py"),
        mode=ImportMode.importlib,
        root=pytester.path,
        consider_namespace_packages=False,
    )
    assert bar_2 is bar


def test_import_sets_module_as_attribute_without_init_files(pytester: Pytester) -> None:
    """Similar to test_import_sets_module_as_attribute, but without __init__.py files."""
    pytester.path.joinpath("foo/bar").mkdir(parents=True)
    pytester.path.joinpath("foo/bar/baz.py").touch()
    pytester.syspathinsert()

    # Import foo.bar.baz and ensure parent modules also ended up imported.
    baz = import_path(
        pytester.path.joinpath("foo/bar/baz.py"),
        mode=ImportMode.importlib,
        root=pytester.path,
        consider_namespace_packages=False,
    )
    assert baz.__name__ == "foo.bar.baz"
    foo = sys.modules["foo"]
    assert foo.__name__ == "foo"
    bar = sys.modules["foo.bar"]
    assert bar.__name__ == "foo.bar"

    # Check parent modules have an attribute pointing to their children.
    assert bar.baz is baz
    assert foo.bar is bar

    # Ensure we returned the "foo.bar.baz" module cached in sys.modules.
    baz_2 = import_path(
        pytester.path.joinpath("foo/bar/baz.py"),
        mode=ImportMode.importlib,
        root=pytester.path,
        consider_namespace_packages=False,
    )
    assert baz_2 is baz


def test_import_sets_module_as_attribute_regression(pytester: Pytester) -> None:
    """Regression test for #12194."""
    pytester.path.joinpath("foo/bar/baz").mkdir(parents=True)
    pytester.path.joinpath("foo/__init__.py").touch()
    pytester.path.joinpath("foo/bar/__init__.py").touch()
    pytester.path.joinpath("foo/bar/baz/__init__.py").touch()
    f = pytester.makepyfile(
        """
        import foo
        from foo.bar import baz
        foo.bar.baz

        def test_foo() -> None:
            pass
        """
    )

    pytester.syspathinsert()
    result = pytester.runpython(f)
    assert result.ret == 0

    result = pytester.runpytest("--import-mode=importlib", "--doctest-modules")
    assert result.ret == 0


def test_import_submodule_not_namespace(pytester: Pytester) -> None:
    """
    Regression test for importing a submodule 'foo.bar' while there is a 'bar' directory
    reachable from sys.path -- ensuring the top-level module does not end up imported as a namespace
    package.

    #12194
    https://github.com/pytest-dev/pytest/pull/12208#issuecomment-2056458432
    """
    pytester.syspathinsert()
    # Create package 'foo' with a submodule 'bar'.
    pytester.path.joinpath("foo").mkdir()
    foo_path = pytester.path.joinpath("foo/__init__.py")
    foo_path.touch()
    bar_path = pytester.path.joinpath("foo/bar.py")
    bar_path.touch()
    # Create top-level directory in `sys.path` with the same name as that submodule.
    pytester.path.joinpath("bar").mkdir()

    # Import `foo`, then `foo.bar`, and check they were imported from the correct location.
    foo = import_path(
        foo_path,
        mode=ImportMode.importlib,
        root=pytester.path,
        consider_namespace_packages=False,
    )
    bar = import_path(
        bar_path,
        mode=ImportMode.importlib,
        root=pytester.path,
        consider_namespace_packages=False,
    )
    assert foo.__name__ == "foo"
    assert bar.__name__ == "foo.bar"
    assert foo.__file__ is not None
    assert bar.__file__ is not None
    assert Path(foo.__file__) == foo_path
    assert Path(bar.__file__) == bar_path


class TestNamespacePackages:
    """Test import_path support when importing from properly namespace packages."""

    @pytest.fixture(autouse=True)
    def setup_imports_tracking(self, monkeypatch: MonkeyPatch) -> None:
        monkeypatch.setattr(sys, "pytest_namespace_packages_test", [], raising=False)

    def setup_directories(
        self, tmp_path: Path, monkeypatch: Optional[MonkeyPatch], pytester: Pytester
    ) -> Tuple[Path, Path]:
        # Use a code to guard against modules being imported more than once.
        # This is a safeguard in case future changes break this invariant.
        code = dedent(
            """
            import sys
            imported = getattr(sys, "pytest_namespace_packages_test", [])
            assert __name__ not in imported, f"{__name__} already imported"
            imported.append(__name__)
            sys.pytest_namespace_packages_test = imported
            """
        )

        # Set up a namespace package "com.company", containing
        # two subpackages, "app" and "calc".
        (tmp_path / "src/dist1/com/company/app/core").mkdir(parents=True)
        (tmp_path / "src/dist1/com/company/app/__init__.py").write_text(
            code, encoding="UTF-8"
        )
        (tmp_path / "src/dist1/com/company/app/core/__init__.py").write_text(
            code, encoding="UTF-8"
        )
        models_py = tmp_path / "src/dist1/com/company/app/core/models.py"
        models_py.touch()

        (tmp_path / "src/dist2/com/company/calc/algo").mkdir(parents=True)
        (tmp_path / "src/dist2/com/company/calc/__init__.py").write_text(
            code, encoding="UTF-8"
        )
        (tmp_path / "src/dist2/com/company/calc/algo/__init__.py").write_text(
            code, encoding="UTF-8"
        )
        algorithms_py = tmp_path / "src/dist2/com/company/calc/algo/algorithms.py"
        algorithms_py.write_text(code, encoding="UTF-8")

        r = validate_namespace_package(
            pytester,
            [tmp_path / "src/dist1", tmp_path / "src/dist2"],
            ["com.company.app.core.models", "com.company.calc.algo.algorithms"],
        )
        assert r.ret == 0
        if monkeypatch is not None:
            monkeypatch.syspath_prepend(tmp_path / "src/dist1")
            monkeypatch.syspath_prepend(tmp_path / "src/dist2")
        return models_py, algorithms_py

    @pytest.mark.parametrize("import_mode", ["prepend", "append", "importlib"])
    def test_resolve_pkg_root_and_module_name_ns_multiple_levels(
        self,
        tmp_path: Path,
        monkeypatch: MonkeyPatch,
        pytester: Pytester,
        import_mode: str,
    ) -> None:
        models_py, algorithms_py = self.setup_directories(
            tmp_path, monkeypatch, pytester
        )

        pkg_root, module_name = resolve_pkg_root_and_module_name(
            models_py, consider_namespace_packages=True
        )
        assert (pkg_root, module_name) == (
            tmp_path / "src/dist1",
            "com.company.app.core.models",
        )

        mod = import_path(
            models_py, mode=import_mode, root=tmp_path, consider_namespace_packages=True
        )
        assert mod.__name__ == "com.company.app.core.models"
        assert mod.__file__ == str(models_py)

        # Ensure we do not import the same module again (#11475).
        mod2 = import_path(
            models_py, mode=import_mode, root=tmp_path, consider_namespace_packages=True
        )
        assert mod is mod2

        pkg_root, module_name = resolve_pkg_root_and_module_name(
            algorithms_py, consider_namespace_packages=True
        )
        assert (pkg_root, module_name) == (
            tmp_path / "src/dist2",
            "com.company.calc.algo.algorithms",
        )

        mod = import_path(
            algorithms_py,
            mode=import_mode,
            root=tmp_path,
            consider_namespace_packages=True,
        )
        assert mod.__name__ == "com.company.calc.algo.algorithms"
        assert mod.__file__ == str(algorithms_py)

        # Ensure we do not import the same module again (#11475).
        mod2 = import_path(
            algorithms_py,
            mode=import_mode,
            root=tmp_path,
            consider_namespace_packages=True,
        )
        assert mod is mod2

    @pytest.mark.parametrize("import_mode", ["prepend", "append", "importlib"])
    def test_incorrect_namespace_package(
        self,
        tmp_path: Path,
        monkeypatch: MonkeyPatch,
        pytester: Pytester,
        import_mode: str,
    ) -> None:
        models_py, algorithms_py = self.setup_directories(
            tmp_path, monkeypatch, pytester
        )
        # Namespace packages must not have an __init__.py at its top-level
        # directory; if it does, it is no longer a namespace package, and we fall back
        # to importing just the part of the package containing the __init__.py files.
        (tmp_path / "src/dist1/com/__init__.py").touch()

        # Because of the __init__ file, 'com' is no longer a namespace package:
        # 'com.company.app' is importable as a normal module.
        # 'com.company.calc' is no longer importable because 'com' is not a namespace package anymore.
        r = validate_namespace_package(
            pytester,
            [tmp_path / "src/dist1", tmp_path / "src/dist2"],
            ["com.company.app.core.models", "com.company.calc.algo.algorithms"],
        )
        assert r.ret == 1
        r.stderr.fnmatch_lines("*No module named 'com.company.calc*")

        pkg_root, module_name = resolve_pkg_root_and_module_name(
            models_py, consider_namespace_packages=True
        )
        assert (pkg_root, module_name) == (
            tmp_path / "src/dist1",
            "com.company.app.core.models",
        )

        # dist2/com/company will contain a normal Python package.
        pkg_root, module_name = resolve_pkg_root_and_module_name(
            algorithms_py, consider_namespace_packages=True
        )
        assert (pkg_root, module_name) == (
            tmp_path / "src/dist2/com/company",
            "calc.algo.algorithms",
        )

    def test_detect_meta_path(
        self,
        tmp_path: Path,
        monkeypatch: MonkeyPatch,
        pytester: Pytester,
    ) -> None:
        """
        resolve_pkg_root_and_module_name() considers sys.meta_path when importing namespace packages.

        Regression test for #12112.
        """

        class CustomImporter(importlib.abc.MetaPathFinder):
            """
            Imports the module name "com" as a namespace package.

            This ensures our namespace detection considers sys.meta_path, which is important
            to support all possible ways a module can be imported (for example editable installs).
            """

            def find_spec(
                self, name: str, path: Any = None, target: Any = None
            ) -> Optional[importlib.machinery.ModuleSpec]:
                if name == "com":
                    spec = importlib.machinery.ModuleSpec("com", loader=None)
                    spec.submodule_search_locations = [str(com_root_2), str(com_root_1)]
                    return spec
                return None

        # Setup directories without configuring sys.path.
        models_py, algorithms_py = self.setup_directories(
            tmp_path, monkeypatch=None, pytester=pytester
        )
        com_root_1 = tmp_path / "src/dist1/com"
        com_root_2 = tmp_path / "src/dist2/com"

        # Because the namespace package is not setup correctly, we cannot resolve it as a namespace package.
        pkg_root, module_name = resolve_pkg_root_and_module_name(
            models_py, consider_namespace_packages=True
        )
        assert (pkg_root, module_name) == (
            tmp_path / "src/dist1/com/company",
            "app.core.models",
        )

        # Insert our custom importer, which will recognize the "com" directory as a namespace package.
        new_meta_path = [CustomImporter(), *sys.meta_path]
        monkeypatch.setattr(sys, "meta_path", new_meta_path)

        # Now we should be able to resolve the path as namespace package.
        pkg_root, module_name = resolve_pkg_root_and_module_name(
            models_py, consider_namespace_packages=True
        )
        assert (pkg_root, module_name) == (
            tmp_path / "src/dist1",
            "com.company.app.core.models",
        )

    @pytest.mark.parametrize("insert", [True, False])
    def test_full_ns_packages_without_init_files(
        self, pytester: Pytester, tmp_path: Path, monkeypatch: MonkeyPatch, insert: bool
    ) -> None:
        (tmp_path / "src/dist1/ns/b/app/bar/test").mkdir(parents=True)
        (tmp_path / "src/dist1/ns/b/app/bar/m.py").touch()

        if insert:
            # The presence of this __init__.py is not a problem, ns.b.app is still part of the namespace package.
            (tmp_path / "src/dist1/ns/b/app/__init__.py").touch()

        (tmp_path / "src/dist2/ns/a/core/foo/test").mkdir(parents=True)
        (tmp_path / "src/dist2/ns/a/core/foo/m.py").touch()

        # Validate the namespace package by importing it in a Python subprocess.
        r = validate_namespace_package(
            pytester,
            [tmp_path / "src/dist1", tmp_path / "src/dist2"],
            ["ns.b.app.bar.m", "ns.a.core.foo.m"],
        )
        assert r.ret == 0
        monkeypatch.syspath_prepend(tmp_path / "src/dist1")
        monkeypatch.syspath_prepend(tmp_path / "src/dist2")

        assert resolve_pkg_root_and_module_name(
            tmp_path / "src/dist1/ns/b/app/bar/m.py", consider_namespace_packages=True
        ) == (tmp_path / "src/dist1", "ns.b.app.bar.m")
        assert resolve_pkg_root_and_module_name(
            tmp_path / "src/dist2/ns/a/core/foo/m.py", consider_namespace_packages=True
        ) == (tmp_path / "src/dist2", "ns.a.core.foo.m")


def test_is_importable(pytester: Pytester) -> None:
    pytester.syspathinsert()

    path = pytester.path / "bar/foo.py"
    path.parent.mkdir()
    path.touch()
    assert is_importable("bar.foo", path) is True

    # Ensure that the module that can be imported points to the path we expect.
    path = pytester.path / "some/other/path/bar/foo.py"
    path.mkdir(parents=True, exist_ok=True)
    assert is_importable("bar.foo", path) is False

    # Paths containing "." cannot be imported.
    path = pytester.path / "bar.x/__init__.py"
    path.parent.mkdir()
    path.touch()
    assert is_importable("bar.x", path) is False

    # Pass starting with "." denote relative imports and cannot be checked using is_importable.
    path = pytester.path / ".bar.x/__init__.py"
    path.parent.mkdir()
    path.touch()
    assert is_importable(".bar.x", path) is False


def test_compute_module_name(tmp_path: Path) -> None:
    assert compute_module_name(tmp_path, tmp_path) is None
    assert compute_module_name(Path(), Path()) is None

    assert compute_module_name(tmp_path, tmp_path / "mod.py") == "mod"
    assert compute_module_name(tmp_path, tmp_path / "src/app/bar") == "src.app.bar"
    assert compute_module_name(tmp_path, tmp_path / "src/app/bar.py") == "src.app.bar"
    assert (
        compute_module_name(tmp_path, tmp_path / "src/app/bar/__init__.py")
        == "src.app.bar"
    )


def validate_namespace_package(
    pytester: Pytester, paths: Sequence[Path], modules: Sequence[str]
) -> RunResult:
    """
    Validate that a Python namespace package is set up correctly.

    In a sub interpreter, add 'paths' to sys.path and attempt to import the given modules.

    In this module many tests configure a set of files as a namespace package, this function
    is used as sanity check that our files are configured correctly from the point of view of Python.
    """
    lines = [
        "import sys",
        # Configure sys.path.
        *[f"sys.path.append(r{str(x)!r})" for x in paths],
        # Imports.
        *[f"import {x}" for x in modules],
    ]
    return pytester.runpython_c("\n".join(lines))
