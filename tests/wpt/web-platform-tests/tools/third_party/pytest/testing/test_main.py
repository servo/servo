import argparse
import os
import re
from typing import Optional

import py.path

import pytest
from _pytest.config import ExitCode
from _pytest.config import UsageError
from _pytest.main import resolve_collection_argument
from _pytest.main import validate_basetemp
from _pytest.pathlib import Path
from _pytest.pytester import Testdir


@pytest.mark.parametrize(
    "ret_exc",
    (
        pytest.param((None, ValueError)),
        pytest.param((42, SystemExit)),
        pytest.param((False, SystemExit)),
    ),
)
def test_wrap_session_notify_exception(ret_exc, testdir):
    returncode, exc = ret_exc
    c1 = testdir.makeconftest(
        """
        import pytest

        def pytest_sessionstart():
            raise {exc}("boom")

        def pytest_internalerror(excrepr, excinfo):
            returncode = {returncode!r}
            if returncode is not False:
                pytest.exit("exiting after %s..." % excinfo.typename, returncode={returncode!r})
    """.format(
            returncode=returncode, exc=exc.__name__
        )
    )
    result = testdir.runpytest()
    if returncode:
        assert result.ret == returncode
    else:
        assert result.ret == ExitCode.INTERNAL_ERROR
    assert result.stdout.lines[0] == "INTERNALERROR> Traceback (most recent call last):"

    if exc == SystemExit:
        assert result.stdout.lines[-3:] == [
            'INTERNALERROR>   File "{}", line 4, in pytest_sessionstart'.format(c1),
            'INTERNALERROR>     raise SystemExit("boom")',
            "INTERNALERROR> SystemExit: boom",
        ]
    else:
        assert result.stdout.lines[-3:] == [
            'INTERNALERROR>   File "{}", line 4, in pytest_sessionstart'.format(c1),
            'INTERNALERROR>     raise ValueError("boom")',
            "INTERNALERROR> ValueError: boom",
        ]
    if returncode is False:
        assert result.stderr.lines == ["mainloop: caught unexpected SystemExit!"]
    else:
        assert result.stderr.lines == ["Exit: exiting after {}...".format(exc.__name__)]


@pytest.mark.parametrize("returncode", (None, 42))
def test_wrap_session_exit_sessionfinish(
    returncode: Optional[int], testdir: Testdir
) -> None:
    testdir.makeconftest(
        """
        import pytest
        def pytest_sessionfinish():
            pytest.exit(msg="exit_pytest_sessionfinish", returncode={returncode})
    """.format(
            returncode=returncode
        )
    )
    result = testdir.runpytest()
    if returncode:
        assert result.ret == returncode
    else:
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
    assert result.stdout.lines[-1] == "collected 0 items"
    assert result.stderr.lines == ["Exit: exit_pytest_sessionfinish"]


@pytest.mark.parametrize("basetemp", ["foo", "foo/bar"])
def test_validate_basetemp_ok(tmp_path, basetemp, monkeypatch):
    monkeypatch.chdir(str(tmp_path))
    validate_basetemp(tmp_path / basetemp)


@pytest.mark.parametrize("basetemp", ["", ".", ".."])
def test_validate_basetemp_fails(tmp_path, basetemp, monkeypatch):
    monkeypatch.chdir(str(tmp_path))
    msg = "basetemp must not be empty, the current working directory or any parent directory of it"
    with pytest.raises(argparse.ArgumentTypeError, match=msg):
        if basetemp:
            basetemp = tmp_path / basetemp
        validate_basetemp(basetemp)


def test_validate_basetemp_integration(testdir):
    result = testdir.runpytest("--basetemp=.")
    result.stderr.fnmatch_lines("*basetemp must not be*")


class TestResolveCollectionArgument:
    @pytest.fixture
    def invocation_dir(self, testdir: Testdir) -> py.path.local:
        testdir.syspathinsert(str(testdir.tmpdir / "src"))
        testdir.chdir()

        pkg = testdir.tmpdir.join("src/pkg").ensure_dir()
        pkg.join("__init__.py").ensure()
        pkg.join("test.py").ensure()
        return testdir.tmpdir

    @pytest.fixture
    def invocation_path(self, invocation_dir: py.path.local) -> Path:
        return Path(str(invocation_dir))

    def test_file(self, invocation_dir: py.path.local, invocation_path: Path) -> None:
        """File and parts."""
        assert resolve_collection_argument(invocation_path, "src/pkg/test.py") == (
            invocation_dir / "src/pkg/test.py",
            [],
        )
        assert resolve_collection_argument(invocation_path, "src/pkg/test.py::") == (
            invocation_dir / "src/pkg/test.py",
            [""],
        )
        assert resolve_collection_argument(
            invocation_path, "src/pkg/test.py::foo::bar"
        ) == (invocation_dir / "src/pkg/test.py", ["foo", "bar"])
        assert resolve_collection_argument(
            invocation_path, "src/pkg/test.py::foo::bar::"
        ) == (invocation_dir / "src/pkg/test.py", ["foo", "bar", ""])

    def test_dir(self, invocation_dir: py.path.local, invocation_path: Path) -> None:
        """Directory and parts."""
        assert resolve_collection_argument(invocation_path, "src/pkg") == (
            invocation_dir / "src/pkg",
            [],
        )

        with pytest.raises(
            UsageError, match=r"directory argument cannot contain :: selection parts"
        ):
            resolve_collection_argument(invocation_path, "src/pkg::")

        with pytest.raises(
            UsageError, match=r"directory argument cannot contain :: selection parts"
        ):
            resolve_collection_argument(invocation_path, "src/pkg::foo::bar")

    def test_pypath(self, invocation_dir: py.path.local, invocation_path: Path) -> None:
        """Dotted name and parts."""
        assert resolve_collection_argument(
            invocation_path, "pkg.test", as_pypath=True
        ) == (invocation_dir / "src/pkg/test.py", [])
        assert resolve_collection_argument(
            invocation_path, "pkg.test::foo::bar", as_pypath=True
        ) == (invocation_dir / "src/pkg/test.py", ["foo", "bar"])
        assert resolve_collection_argument(invocation_path, "pkg", as_pypath=True) == (
            invocation_dir / "src/pkg",
            [],
        )

        with pytest.raises(
            UsageError, match=r"package argument cannot contain :: selection parts"
        ):
            resolve_collection_argument(
                invocation_path, "pkg::foo::bar", as_pypath=True
            )

    def test_does_not_exist(self, invocation_path: Path) -> None:
        """Given a file/module that does not exist raises UsageError."""
        with pytest.raises(
            UsageError, match=re.escape("file or directory not found: foobar")
        ):
            resolve_collection_argument(invocation_path, "foobar")

        with pytest.raises(
            UsageError,
            match=re.escape(
                "module or package not found: foobar (missing __init__.py?)"
            ),
        ):
            resolve_collection_argument(invocation_path, "foobar", as_pypath=True)

    def test_absolute_paths_are_resolved_correctly(
        self, invocation_dir: py.path.local, invocation_path: Path
    ) -> None:
        """Absolute paths resolve back to absolute paths."""
        full_path = str(invocation_dir / "src")
        assert resolve_collection_argument(invocation_path, full_path) == (
            py.path.local(os.path.abspath("src")),
            [],
        )

        # ensure full paths given in the command-line without the drive letter resolve
        # to the full path correctly (#7628)
        drive, full_path_without_drive = os.path.splitdrive(full_path)
        assert resolve_collection_argument(
            invocation_path, full_path_without_drive
        ) == (py.path.local(os.path.abspath("src")), [])


def test_module_full_path_without_drive(testdir):
    """Collect and run test using full path except for the drive letter (#7628).

    Passing a full path without a drive letter would trigger a bug in py.path.local
    where it would keep the full path without the drive letter around, instead of resolving
    to the full path, resulting in fixtures node ids not matching against test node ids correctly.
    """
    testdir.makepyfile(
        **{
            "project/conftest.py": """
                import pytest
                @pytest.fixture
                def fix(): return 1
            """,
        }
    )

    testdir.makepyfile(
        **{
            "project/tests/dummy_test.py": """
                def test(fix):
                    assert fix == 1
            """
        }
    )
    fn = testdir.tmpdir.join("project/tests/dummy_test.py")
    assert fn.isfile()

    drive, path = os.path.splitdrive(str(fn))

    result = testdir.runpytest(path, "-v")
    result.stdout.fnmatch_lines(
        [
            os.path.join("project", "tests", "dummy_test.py") + "::test PASSED *",
            "* 1 passed in *",
        ]
    )
