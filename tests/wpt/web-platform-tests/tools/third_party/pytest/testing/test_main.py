import argparse
import os
import re
import sys
from pathlib import Path
from typing import Optional

import pytest
from _pytest.config import ExitCode
from _pytest.config import UsageError
from _pytest.main import resolve_collection_argument
from _pytest.main import validate_basetemp
from _pytest.pytester import Pytester


@pytest.mark.parametrize(
    "ret_exc",
    (
        pytest.param((None, ValueError)),
        pytest.param((42, SystemExit)),
        pytest.param((False, SystemExit)),
    ),
)
def test_wrap_session_notify_exception(ret_exc, pytester: Pytester) -> None:
    returncode, exc = ret_exc
    c1 = pytester.makeconftest(
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
    result = pytester.runpytest()
    if returncode:
        assert result.ret == returncode
    else:
        assert result.ret == ExitCode.INTERNAL_ERROR
    assert result.stdout.lines[0] == "INTERNALERROR> Traceback (most recent call last):"

    end_lines = (
        result.stdout.lines[-4:]
        if sys.version_info >= (3, 11)
        else result.stdout.lines[-3:]
    )

    if exc == SystemExit:
        assert end_lines == [
            f'INTERNALERROR>   File "{c1}", line 4, in pytest_sessionstart',
            'INTERNALERROR>     raise SystemExit("boom")',
            *(
                ("INTERNALERROR>     ^^^^^^^^^^^^^^^^^^^^^^^^",)
                if sys.version_info >= (3, 11)
                else ()
            ),
            "INTERNALERROR> SystemExit: boom",
        ]
    else:
        assert end_lines == [
            f'INTERNALERROR>   File "{c1}", line 4, in pytest_sessionstart',
            'INTERNALERROR>     raise ValueError("boom")',
            *(
                ("INTERNALERROR>     ^^^^^^^^^^^^^^^^^^^^^^^^",)
                if sys.version_info >= (3, 11)
                else ()
            ),
            "INTERNALERROR> ValueError: boom",
        ]
    if returncode is False:
        assert result.stderr.lines == ["mainloop: caught unexpected SystemExit!"]
    else:
        assert result.stderr.lines == [f"Exit: exiting after {exc.__name__}..."]


@pytest.mark.parametrize("returncode", (None, 42))
def test_wrap_session_exit_sessionfinish(
    returncode: Optional[int], pytester: Pytester
) -> None:
    pytester.makeconftest(
        """
        import pytest
        def pytest_sessionfinish():
            pytest.exit(reason="exit_pytest_sessionfinish", returncode={returncode})
    """.format(
            returncode=returncode
        )
    )
    result = pytester.runpytest()
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


def test_validate_basetemp_integration(pytester: Pytester) -> None:
    result = pytester.runpytest("--basetemp=.")
    result.stderr.fnmatch_lines("*basetemp must not be*")


class TestResolveCollectionArgument:
    @pytest.fixture
    def invocation_path(self, pytester: Pytester) -> Path:
        pytester.syspathinsert(pytester.path / "src")
        pytester.chdir()

        pkg = pytester.path.joinpath("src/pkg")
        pkg.mkdir(parents=True)
        pkg.joinpath("__init__.py").touch()
        pkg.joinpath("test.py").touch()
        return pytester.path

    def test_file(self, invocation_path: Path) -> None:
        """File and parts."""
        assert resolve_collection_argument(invocation_path, "src/pkg/test.py") == (
            invocation_path / "src/pkg/test.py",
            [],
        )
        assert resolve_collection_argument(invocation_path, "src/pkg/test.py::") == (
            invocation_path / "src/pkg/test.py",
            [""],
        )
        assert resolve_collection_argument(
            invocation_path, "src/pkg/test.py::foo::bar"
        ) == (invocation_path / "src/pkg/test.py", ["foo", "bar"])
        assert resolve_collection_argument(
            invocation_path, "src/pkg/test.py::foo::bar::"
        ) == (invocation_path / "src/pkg/test.py", ["foo", "bar", ""])

    def test_dir(self, invocation_path: Path) -> None:
        """Directory and parts."""
        assert resolve_collection_argument(invocation_path, "src/pkg") == (
            invocation_path / "src/pkg",
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

    def test_pypath(self, invocation_path: Path) -> None:
        """Dotted name and parts."""
        assert resolve_collection_argument(
            invocation_path, "pkg.test", as_pypath=True
        ) == (invocation_path / "src/pkg/test.py", [])
        assert resolve_collection_argument(
            invocation_path, "pkg.test::foo::bar", as_pypath=True
        ) == (invocation_path / "src/pkg/test.py", ["foo", "bar"])
        assert resolve_collection_argument(invocation_path, "pkg", as_pypath=True) == (
            invocation_path / "src/pkg",
            [],
        )

        with pytest.raises(
            UsageError, match=r"package argument cannot contain :: selection parts"
        ):
            resolve_collection_argument(
                invocation_path, "pkg::foo::bar", as_pypath=True
            )

    def test_parametrized_name_with_colons(self, invocation_path: Path) -> None:
        ret = resolve_collection_argument(
            invocation_path, "src/pkg/test.py::test[a::b]"
        )
        assert ret == (invocation_path / "src/pkg/test.py", ["test[a::b]"])

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

    def test_absolute_paths_are_resolved_correctly(self, invocation_path: Path) -> None:
        """Absolute paths resolve back to absolute paths."""
        full_path = str(invocation_path / "src")
        assert resolve_collection_argument(invocation_path, full_path) == (
            Path(os.path.abspath("src")),
            [],
        )

        # ensure full paths given in the command-line without the drive letter resolve
        # to the full path correctly (#7628)
        drive, full_path_without_drive = os.path.splitdrive(full_path)
        assert resolve_collection_argument(
            invocation_path, full_path_without_drive
        ) == (Path(os.path.abspath("src")), [])


def test_module_full_path_without_drive(pytester: Pytester) -> None:
    """Collect and run test using full path except for the drive letter (#7628).

    Passing a full path without a drive letter would trigger a bug in legacy_path
    where it would keep the full path without the drive letter around, instead of resolving
    to the full path, resulting in fixtures node ids not matching against test node ids correctly.
    """
    pytester.makepyfile(
        **{
            "project/conftest.py": """
                import pytest
                @pytest.fixture
                def fix(): return 1
            """,
        }
    )

    pytester.makepyfile(
        **{
            "project/tests/dummy_test.py": """
                def test(fix):
                    assert fix == 1
            """
        }
    )
    fn = pytester.path.joinpath("project/tests/dummy_test.py")
    assert fn.is_file()

    drive, path = os.path.splitdrive(str(fn))

    result = pytester.runpytest(path, "-v")
    result.stdout.fnmatch_lines(
        [
            os.path.join("project", "tests", "dummy_test.py") + "::test PASSED *",
            "* 1 passed in *",
        ]
    )
