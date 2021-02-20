import pytest
from _pytest.config import ExitCode


def test_version_verbose(testdir, pytestconfig):
    testdir.monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD")
    result = testdir.runpytest("--version", "--version")
    assert result.ret == 0
    result.stderr.fnmatch_lines(
        ["*pytest*{}*imported from*".format(pytest.__version__)]
    )
    if pytestconfig.pluginmanager.list_plugin_distinfo():
        result.stderr.fnmatch_lines(["*setuptools registered plugins:", "*at*"])


def test_version_less_verbose(testdir, pytestconfig):
    testdir.monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD")
    result = testdir.runpytest("--version")
    assert result.ret == 0
    # p = py.path.local(py.__file__).dirpath()
    result.stderr.fnmatch_lines(["pytest {}".format(pytest.__version__)])


def test_help(testdir):
    result = testdir.runpytest("--help")
    assert result.ret == 0
    result.stdout.fnmatch_lines(
        """
          -m MARKEXPR           only run tests matching given mark expression.
                                For example: -m 'mark1 and not mark2'.
        reporting:
          --durations=N *
        *setup.cfg*
        *minversion*
        *to see*markers*pytest --markers*
        *to see*fixtures*pytest --fixtures*
    """
    )


def test_none_help_param_raises_exception(testdir):
    """Test that a None help param raises a TypeError."""
    testdir.makeconftest(
        """
        def pytest_addoption(parser):
            parser.addini("test_ini", None, default=True, type="bool")
    """
    )
    result = testdir.runpytest("--help")
    result.stderr.fnmatch_lines(
        ["*TypeError: help argument cannot be None for test_ini*"]
    )


def test_empty_help_param(testdir):
    """Test that an empty help param is displayed correctly."""
    testdir.makeconftest(
        """
        def pytest_addoption(parser):
            parser.addini("test_ini", "", default=True, type="bool")
    """
    )
    result = testdir.runpytest("--help")
    assert result.ret == 0
    lines = [
        "  required_plugins (args):",
        "                        plugins that must be present for pytest to run*",
        "  test_ini (bool):*",
        "environment variables:",
    ]
    result.stdout.fnmatch_lines(lines, consecutive=True)


def test_hookvalidation_unknown(testdir):
    testdir.makeconftest(
        """
        def pytest_hello(xyz):
            pass
    """
    )
    result = testdir.runpytest()
    assert result.ret != 0
    result.stdout.fnmatch_lines(["*unknown hook*pytest_hello*"])


def test_hookvalidation_optional(testdir):
    testdir.makeconftest(
        """
        import pytest
        @pytest.hookimpl(optionalhook=True)
        def pytest_hello(xyz):
            pass
    """
    )
    result = testdir.runpytest()
    assert result.ret == ExitCode.NO_TESTS_COLLECTED


def test_traceconfig(testdir):
    result = testdir.runpytest("--traceconfig")
    result.stdout.fnmatch_lines(["*using*pytest*py*", "*active plugins*"])


def test_debug(testdir):
    result = testdir.runpytest_subprocess("--debug")
    assert result.ret == ExitCode.NO_TESTS_COLLECTED
    p = testdir.tmpdir.join("pytestdebug.log")
    assert "pytest_sessionstart" in p.read()


def test_PYTEST_DEBUG(testdir, monkeypatch):
    monkeypatch.setenv("PYTEST_DEBUG", "1")
    result = testdir.runpytest_subprocess()
    assert result.ret == ExitCode.NO_TESTS_COLLECTED
    result.stderr.fnmatch_lines(
        ["*pytest_plugin_registered*", "*manager*PluginManager*"]
    )
