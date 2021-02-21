import inspect

import _pytest.warning_types
import pytest


@pytest.mark.parametrize(
    "warning_class",
    [
        w
        for n, w in vars(_pytest.warning_types).items()
        if inspect.isclass(w) and issubclass(w, Warning)
    ],
)
def test_warning_types(warning_class):
    """Make sure all warnings declared in _pytest.warning_types are displayed as coming
    from 'pytest' instead of the internal module (#5452).
    """
    assert warning_class.__module__ == "pytest"


@pytest.mark.filterwarnings("error::pytest.PytestWarning")
def test_pytest_warnings_repr_integration_test(testdir):
    """Small integration test to ensure our small hack of setting the __module__ attribute
    of our warnings actually works (#5452).
    """
    testdir.makepyfile(
        """
        import pytest
        import warnings

        def test():
            warnings.warn(pytest.PytestWarning("some warning"))
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["E       pytest.PytestWarning: some warning"])
