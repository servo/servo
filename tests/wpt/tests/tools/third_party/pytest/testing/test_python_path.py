# mypy: allow-untyped-defs
import sys
from textwrap import dedent
from typing import Generator
from typing import List
from typing import Optional

from _pytest.pytester import Pytester
import pytest


@pytest.fixture()
def file_structure(pytester: Pytester) -> None:
    pytester.makepyfile(
        test_foo="""
        from foo import foo

        def test_foo():
            assert foo() == 1
        """
    )

    pytester.makepyfile(
        test_bar="""
        from bar import bar

        def test_bar():
            assert bar() == 2
        """
    )

    foo_py = pytester.mkdir("sub") / "foo.py"
    content = dedent(
        """
        def foo():
            return 1
        """
    )
    foo_py.write_text(content, encoding="utf-8")

    bar_py = pytester.mkdir("sub2") / "bar.py"
    content = dedent(
        """
        def bar():
            return 2
        """
    )
    bar_py.write_text(content, encoding="utf-8")


def test_one_dir(pytester: Pytester, file_structure) -> None:
    pytester.makefile(".ini", pytest="[pytest]\npythonpath=sub\n")
    result = pytester.runpytest("test_foo.py")
    assert result.ret == 0
    result.assert_outcomes(passed=1)


def test_two_dirs(pytester: Pytester, file_structure) -> None:
    pytester.makefile(".ini", pytest="[pytest]\npythonpath=sub sub2\n")
    result = pytester.runpytest("test_foo.py", "test_bar.py")
    assert result.ret == 0
    result.assert_outcomes(passed=2)


def test_module_not_found(pytester: Pytester, file_structure) -> None:
    """Without the pythonpath setting, the module should not be found."""
    pytester.makefile(".ini", pytest="[pytest]\n")
    result = pytester.runpytest("test_foo.py")
    assert result.ret == pytest.ExitCode.INTERRUPTED
    result.assert_outcomes(errors=1)
    expected_error = "E   ModuleNotFoundError: No module named 'foo'"
    result.stdout.fnmatch_lines([expected_error])


def test_no_ini(pytester: Pytester, file_structure) -> None:
    """If no ini file, test should error."""
    result = pytester.runpytest("test_foo.py")
    assert result.ret == pytest.ExitCode.INTERRUPTED
    result.assert_outcomes(errors=1)
    expected_error = "E   ModuleNotFoundError: No module named 'foo'"
    result.stdout.fnmatch_lines([expected_error])


def test_clean_up(pytester: Pytester) -> None:
    """Test that the plugin cleans up after itself."""
    # This is tough to test behaviorally because the cleanup really runs last.
    # So the test make several implementation assumptions:
    # - Cleanup is done in pytest_unconfigure().
    # - Not a hook wrapper.
    # So we can add a hook wrapper ourselves to test what it does.
    pytester.makefile(".ini", pytest="[pytest]\npythonpath=I_SHALL_BE_REMOVED\n")
    pytester.makepyfile(test_foo="""def test_foo(): pass""")

    before: Optional[List[str]] = None
    after: Optional[List[str]] = None

    class Plugin:
        @pytest.hookimpl(wrapper=True, tryfirst=True)
        def pytest_unconfigure(self) -> Generator[None, None, None]:
            nonlocal before, after
            before = sys.path.copy()
            try:
                return (yield)
            finally:
                after = sys.path.copy()

    result = pytester.runpytest_inprocess(plugins=[Plugin()])
    assert result.ret == 0

    assert before is not None
    assert after is not None
    assert any("I_SHALL_BE_REMOVED" in entry for entry in before)
    assert not any("I_SHALL_BE_REMOVED" in entry for entry in after)
