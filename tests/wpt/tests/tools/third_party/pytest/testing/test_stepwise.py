# mypy: allow-untyped-defs
from pathlib import Path

from _pytest.cacheprovider import Cache
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pytester import Pytester
from _pytest.stepwise import STEPWISE_CACHE_DIR
import pytest


@pytest.fixture
def stepwise_pytester(pytester: Pytester) -> Pytester:
    # Rather than having to modify our testfile between tests, we introduce
    # a flag for whether or not the second test should fail.
    pytester.makeconftest(
        """
def pytest_addoption(parser):
    group = parser.getgroup('general')
    group.addoption('--fail', action='store_true', dest='fail')
    group.addoption('--fail-last', action='store_true', dest='fail_last')
"""
    )

    # Create a simple test suite.
    pytester.makepyfile(
        test_a="""
def test_success_before_fail():
    assert 1

def test_fail_on_flag(request):
    assert not request.config.getvalue('fail')

def test_success_after_fail():
    assert 1

def test_fail_last_on_flag(request):
    assert not request.config.getvalue('fail_last')

def test_success_after_last_fail():
    assert 1
"""
    )

    pytester.makepyfile(
        test_b="""
def test_success():
    assert 1
"""
    )

    # customize cache directory so we don't use the tox's cache directory, which makes tests in this module flaky
    pytester.makeini(
        """
        [pytest]
        cache_dir = .cache
    """
    )

    return pytester


@pytest.fixture
def error_pytester(pytester: Pytester) -> Pytester:
    pytester.makepyfile(
        test_a="""
def test_error(nonexisting_fixture):
    assert 1

def test_success_after_fail():
    assert 1
"""
    )

    return pytester


@pytest.fixture
def broken_pytester(pytester: Pytester) -> Pytester:
    pytester.makepyfile(
        working_testfile="def test_proper(): assert 1", broken_testfile="foobar"
    )
    return pytester


def _strip_resource_warnings(lines):
    # Strip unreliable ResourceWarnings, so no-output assertions on stderr can work.
    # (https://github.com/pytest-dev/pytest/issues/5088)
    return [
        x
        for x in lines
        if not x.startswith(("Exception ignored in:", "ResourceWarning"))
    ]


def test_run_without_stepwise(stepwise_pytester: Pytester) -> None:
    result = stepwise_pytester.runpytest("-v", "--strict-markers", "--fail")
    result.stdout.fnmatch_lines(["*test_success_before_fail PASSED*"])
    result.stdout.fnmatch_lines(["*test_fail_on_flag FAILED*"])
    result.stdout.fnmatch_lines(["*test_success_after_fail PASSED*"])


def test_stepwise_output_summary(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest
        @pytest.mark.parametrize("expected", [True, True, True, True, False])
        def test_data(expected):
            assert expected
        """
    )
    result = pytester.runpytest("-v", "--stepwise")
    result.stdout.fnmatch_lines(["stepwise: no previously failed tests, not skipping."])
    result = pytester.runpytest("-v", "--stepwise")
    result.stdout.fnmatch_lines(
        ["stepwise: skipping 4 already passed items.", "*1 failed, 4 deselected*"]
    )


def test_fail_and_continue_with_stepwise(stepwise_pytester: Pytester) -> None:
    # Run the tests with a failing second test.
    result = stepwise_pytester.runpytest(
        "-v", "--strict-markers", "--stepwise", "--fail"
    )
    assert _strip_resource_warnings(result.stderr.lines) == []

    stdout = result.stdout.str()
    # Make sure we stop after first failing test.
    assert "test_success_before_fail PASSED" in stdout
    assert "test_fail_on_flag FAILED" in stdout
    assert "test_success_after_fail" not in stdout

    # "Fix" the test that failed in the last run and run it again.
    result = stepwise_pytester.runpytest("-v", "--strict-markers", "--stepwise")
    assert _strip_resource_warnings(result.stderr.lines) == []

    stdout = result.stdout.str()
    # Make sure the latest failing test runs and then continues.
    assert "test_success_before_fail" not in stdout
    assert "test_fail_on_flag PASSED" in stdout
    assert "test_success_after_fail PASSED" in stdout


@pytest.mark.parametrize("stepwise_skip", ["--stepwise-skip", "--sw-skip"])
def test_run_with_skip_option(stepwise_pytester: Pytester, stepwise_skip: str) -> None:
    result = stepwise_pytester.runpytest(
        "-v",
        "--strict-markers",
        "--stepwise",
        stepwise_skip,
        "--fail",
        "--fail-last",
    )
    assert _strip_resource_warnings(result.stderr.lines) == []

    stdout = result.stdout.str()
    # Make sure first fail is ignore and second fail stops the test run.
    assert "test_fail_on_flag FAILED" in stdout
    assert "test_success_after_fail PASSED" in stdout
    assert "test_fail_last_on_flag FAILED" in stdout
    assert "test_success_after_last_fail" not in stdout


def test_fail_on_errors(error_pytester: Pytester) -> None:
    result = error_pytester.runpytest("-v", "--strict-markers", "--stepwise")

    assert _strip_resource_warnings(result.stderr.lines) == []
    stdout = result.stdout.str()

    assert "test_error ERROR" in stdout
    assert "test_success_after_fail" not in stdout


def test_change_testfile(stepwise_pytester: Pytester) -> None:
    result = stepwise_pytester.runpytest(
        "-v", "--strict-markers", "--stepwise", "--fail", "test_a.py"
    )
    assert _strip_resource_warnings(result.stderr.lines) == []

    stdout = result.stdout.str()
    assert "test_fail_on_flag FAILED" in stdout

    # Make sure the second test run starts from the beginning, since the
    # test to continue from does not exist in testfile_b.
    result = stepwise_pytester.runpytest(
        "-v", "--strict-markers", "--stepwise", "test_b.py"
    )
    assert _strip_resource_warnings(result.stderr.lines) == []

    stdout = result.stdout.str()
    assert "test_success PASSED" in stdout


@pytest.mark.parametrize("broken_first", [True, False])
def test_stop_on_collection_errors(
    broken_pytester: Pytester, broken_first: bool
) -> None:
    """Stop during collection errors. Broken test first or broken test last
    actually surfaced a bug (#5444), so we test both situations."""
    files = ["working_testfile.py", "broken_testfile.py"]
    if broken_first:
        files.reverse()
    result = broken_pytester.runpytest("-v", "--strict-markers", "--stepwise", *files)
    result.stdout.fnmatch_lines("*error during collection*")


def test_xfail_handling(pytester: Pytester, monkeypatch: MonkeyPatch) -> None:
    """Ensure normal xfail is ignored, and strict xfail interrupts the session in sw mode

    (#5547)
    """
    monkeypatch.setattr("sys.dont_write_bytecode", True)

    contents = """
        import pytest
        def test_a(): pass

        @pytest.mark.xfail(strict={strict})
        def test_b(): assert {assert_value}

        def test_c(): pass
        def test_d(): pass
    """
    pytester.makepyfile(contents.format(assert_value="0", strict="False"))
    result = pytester.runpytest("--sw", "-v")
    result.stdout.fnmatch_lines(
        [
            "*::test_a PASSED *",
            "*::test_b XFAIL *",
            "*::test_c PASSED *",
            "*::test_d PASSED *",
            "* 3 passed, 1 xfailed in *",
        ]
    )

    pytester.makepyfile(contents.format(assert_value="1", strict="True"))
    result = pytester.runpytest("--sw", "-v")
    result.stdout.fnmatch_lines(
        [
            "*::test_a PASSED *",
            "*::test_b FAILED *",
            "* Interrupted*",
            "* 1 failed, 1 passed in *",
        ]
    )

    pytester.makepyfile(contents.format(assert_value="0", strict="True"))
    result = pytester.runpytest("--sw", "-v")
    result.stdout.fnmatch_lines(
        [
            "*::test_b XFAIL *",
            "*::test_c PASSED *",
            "*::test_d PASSED *",
            "* 2 passed, 1 deselected, 1 xfailed in *",
        ]
    )


def test_stepwise_skip_is_independent(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        def test_one():
            assert False

        def test_two():
            assert False

        def test_three():
            assert False

        """
    )
    result = pytester.runpytest("--tb", "no", "--stepwise-skip")
    result.assert_outcomes(failed=2)
    result.stdout.fnmatch_lines(
        [
            "FAILED test_stepwise_skip_is_independent.py::test_one - assert False",
            "FAILED test_stepwise_skip_is_independent.py::test_two - assert False",
            "*Interrupted: Test failed, continuing from this test next run.*",
        ]
    )


def test_sw_skip_help(pytester: Pytester) -> None:
    result = pytester.runpytest("-h")
    result.stdout.fnmatch_lines("*Implicitly enables --stepwise.")


def test_stepwise_xdist_dont_store_lastfailed(pytester: Pytester) -> None:
    pytester.makefile(
        ext=".ini",
        pytest=f"[pytest]\ncache_dir = {pytester.path}\n",
    )

    pytester.makepyfile(
        conftest="""
import pytest

@pytest.hookimpl(tryfirst=True)
def pytest_configure(config) -> None:
    config.workerinput = True
"""
    )
    pytester.makepyfile(
        test_one="""
def test_one():
    assert False
"""
    )
    result = pytester.runpytest("--stepwise")
    assert result.ret == pytest.ExitCode.INTERRUPTED

    stepwise_cache_file = (
        pytester.path / Cache._CACHE_PREFIX_VALUES / STEPWISE_CACHE_DIR
    )
    assert not Path(stepwise_cache_file).exists()


def test_disabled_stepwise_xdist_dont_clear_cache(pytester: Pytester) -> None:
    pytester.makefile(
        ext=".ini",
        pytest=f"[pytest]\ncache_dir = {pytester.path}\n",
    )

    stepwise_cache_file = (
        pytester.path / Cache._CACHE_PREFIX_VALUES / STEPWISE_CACHE_DIR
    )
    stepwise_cache_dir = stepwise_cache_file.parent
    stepwise_cache_dir.mkdir(exist_ok=True, parents=True)

    stepwise_cache_file_relative = f"{Cache._CACHE_PREFIX_VALUES}/{STEPWISE_CACHE_DIR}"

    expected_value = '"test_one.py::test_one"'
    content = {f"{stepwise_cache_file_relative}": expected_value}

    pytester.makefile(ext="", **content)

    pytester.makepyfile(
        conftest="""
import pytest

@pytest.hookimpl(tryfirst=True)
def pytest_configure(config) -> None:
    config.workerinput = True
"""
    )
    pytester.makepyfile(
        test_one="""
def test_one():
    assert True
"""
    )
    result = pytester.runpytest()
    assert result.ret == 0

    assert Path(stepwise_cache_file).exists()
    with stepwise_cache_file.open(encoding="utf-8") as file_handle:
        observed_value = file_handle.readlines()
    assert [expected_value] == observed_value
