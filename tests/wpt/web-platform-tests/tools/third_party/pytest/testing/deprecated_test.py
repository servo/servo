import warnings
from unittest import mock

import pytest
from _pytest import deprecated
from _pytest.pytester import Testdir


@pytest.mark.parametrize("attribute", pytest.collect.__all__)  # type: ignore
# false positive due to dynamic attribute
def test_pytest_collect_module_deprecated(attribute):
    with pytest.warns(DeprecationWarning, match=attribute):
        getattr(pytest.collect, attribute)


@pytest.mark.parametrize("plugin", sorted(deprecated.DEPRECATED_EXTERNAL_PLUGINS))
@pytest.mark.filterwarnings("default")
def test_external_plugins_integrated(testdir, plugin):
    testdir.syspathinsert()
    testdir.makepyfile(**{plugin: ""})

    with pytest.warns(pytest.PytestConfigWarning):
        testdir.parseconfig("-p", plugin)


def test_fillfuncargs_is_deprecated() -> None:
    with pytest.warns(
        pytest.PytestDeprecationWarning,
        match="The `_fillfuncargs` function is deprecated",
    ):
        pytest._fillfuncargs(mock.Mock())


def test_minus_k_dash_is_deprecated(testdir) -> None:
    threepass = testdir.makepyfile(
        test_threepass="""
        def test_one(): assert 1
        def test_two(): assert 1
        def test_three(): assert 1
    """
    )
    result = testdir.runpytest("-k=-test_two", threepass)
    result.stdout.fnmatch_lines(["*The `-k '-expr'` syntax*deprecated*"])


def test_minus_k_colon_is_deprecated(testdir) -> None:
    threepass = testdir.makepyfile(
        test_threepass="""
        def test_one(): assert 1
        def test_two(): assert 1
        def test_three(): assert 1
    """
    )
    result = testdir.runpytest("-k", "test_two:", threepass)
    result.stdout.fnmatch_lines(["*The `-k 'expr:'` syntax*deprecated*"])


def test_fscollector_gethookproxy_isinitpath(testdir: Testdir) -> None:
    module = testdir.getmodulecol(
        """
        def test_foo(): pass
        """,
        withinit=True,
    )
    assert isinstance(module, pytest.Module)
    package = module.parent
    assert isinstance(package, pytest.Package)

    with pytest.warns(pytest.PytestDeprecationWarning, match="gethookproxy"):
        package.gethookproxy(testdir.tmpdir)

    with pytest.warns(pytest.PytestDeprecationWarning, match="isinitpath"):
        package.isinitpath(testdir.tmpdir)

    # The methods on Session are *not* deprecated.
    session = module.session
    with warnings.catch_warnings(record=True) as rec:
        session.gethookproxy(testdir.tmpdir)
        session.isinitpath(testdir.tmpdir)
    assert len(rec) == 0
