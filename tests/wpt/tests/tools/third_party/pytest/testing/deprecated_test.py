# mypy: allow-untyped-defs
from pathlib import Path
import re
import sys

from _pytest import deprecated
from _pytest.compat import legacy_path
from _pytest.pytester import Pytester
import pytest
from pytest import PytestDeprecationWarning


@pytest.mark.parametrize("plugin", sorted(deprecated.DEPRECATED_EXTERNAL_PLUGINS))
@pytest.mark.filterwarnings("default")
def test_external_plugins_integrated(pytester: Pytester, plugin) -> None:
    pytester.syspathinsert()
    pytester.makepyfile(**{plugin: ""})

    with pytest.warns(pytest.PytestConfigWarning):
        pytester.parseconfig("-p", plugin)


def test_hookspec_via_function_attributes_are_deprecated():
    from _pytest.config import PytestPluginManager

    pm = PytestPluginManager()

    class DeprecatedHookMarkerSpec:
        def pytest_bad_hook(self):
            pass

        pytest_bad_hook.historic = False  # type: ignore[attr-defined]

    with pytest.warns(
        PytestDeprecationWarning,
        match=r"Please use the pytest\.hookspec\(historic=False\) decorator",
    ) as recorder:
        pm.add_hookspecs(DeprecatedHookMarkerSpec)
    (record,) = recorder
    assert (
        record.lineno
        == DeprecatedHookMarkerSpec.pytest_bad_hook.__code__.co_firstlineno
    )
    assert record.filename == __file__


def test_hookimpl_via_function_attributes_are_deprecated():
    from _pytest.config import PytestPluginManager

    pm = PytestPluginManager()

    class DeprecatedMarkImplPlugin:
        def pytest_runtest_call(self):
            pass

        pytest_runtest_call.tryfirst = True  # type: ignore[attr-defined]

    with pytest.warns(
        PytestDeprecationWarning,
        match=r"Please use the pytest.hookimpl\(tryfirst=True\)",
    ) as recorder:
        pm.register(DeprecatedMarkImplPlugin())
    (record,) = recorder
    assert (
        record.lineno
        == DeprecatedMarkImplPlugin.pytest_runtest_call.__code__.co_firstlineno
    )
    assert record.filename == __file__


def test_yield_fixture_is_deprecated() -> None:
    with pytest.warns(DeprecationWarning, match=r"yield_fixture is deprecated"):

        @pytest.yield_fixture
        def fix():
            assert False


def test_private_is_deprecated() -> None:
    class PrivateInit:
        def __init__(self, foo: int, *, _ispytest: bool = False) -> None:
            deprecated.check_ispytest(_ispytest)

    with pytest.warns(
        pytest.PytestDeprecationWarning, match="private pytest class or function"
    ):
        PrivateInit(10)

    # Doesn't warn.
    PrivateInit(10, _ispytest=True)


@pytest.mark.parametrize("hooktype", ["hook", "ihook"])
def test_hookproxy_warnings_for_pathlib(tmp_path, hooktype, request):
    path = legacy_path(tmp_path)

    PATH_WARN_MATCH = r".*path: py\.path\.local\) argument is deprecated, please use \(collection_path: pathlib\.Path.*"
    if hooktype == "ihook":
        hooks = request.node.ihook
    else:
        hooks = request.config.hook

    with pytest.warns(PytestDeprecationWarning, match=PATH_WARN_MATCH) as r:
        l1 = sys._getframe().f_lineno
        hooks.pytest_ignore_collect(
            config=request.config, path=path, collection_path=tmp_path
        )
        l2 = sys._getframe().f_lineno

    (record,) = r
    assert record.filename == __file__
    assert l1 < record.lineno < l2

    hooks.pytest_ignore_collect(config=request.config, collection_path=tmp_path)

    # Passing entirely *different* paths is an outright error.
    with pytest.raises(ValueError, match=r"path.*fspath.*need to be equal"):
        with pytest.warns(PytestDeprecationWarning, match=PATH_WARN_MATCH) as r:
            hooks.pytest_ignore_collect(
                config=request.config, path=path, collection_path=Path("/bla/bla")
            )


def test_hookimpl_warnings_for_pathlib() -> None:
    class Plugin:
        def pytest_ignore_collect(self, path: object) -> None:
            raise NotImplementedError()

        def pytest_collect_file(self, path: object) -> None:
            raise NotImplementedError()

        def pytest_pycollect_makemodule(self, path: object) -> None:
            raise NotImplementedError()

        def pytest_report_header(self, startdir: object) -> str:
            raise NotImplementedError()

        def pytest_report_collectionfinish(self, startdir: object) -> str:
            raise NotImplementedError()

    pm = pytest.PytestPluginManager()
    with pytest.warns(
        pytest.PytestRemovedIn9Warning,
        match=r"py\.path\.local.* argument is deprecated",
    ) as wc:
        pm.register(Plugin())
    assert len(wc.list) == 5


def test_node_ctor_fspath_argument_is_deprecated(pytester: Pytester) -> None:
    mod = pytester.getmodulecol("")

    class MyFile(pytest.File):
        def collect(self):
            raise NotImplementedError()

    with pytest.warns(
        pytest.PytestDeprecationWarning,
        match=re.escape(
            "The (fspath: py.path.local) argument to MyFile is deprecated."
        ),
    ):
        MyFile.from_parent(
            parent=mod.parent,
            fspath=legacy_path("bla"),
        )


def test_fixture_disallow_on_marked_functions():
    """Test that applying @pytest.fixture to a marked function warns (#3364)."""
    with pytest.warns(
        pytest.PytestRemovedIn9Warning,
        match=r"Marks applied to fixtures have no effect",
    ) as record:

        @pytest.fixture
        @pytest.mark.parametrize("example", ["hello"])
        @pytest.mark.usefixtures("tmp_path")
        def foo():
            raise NotImplementedError()

    # it's only possible to get one warning here because you're already prevented
    # from applying @fixture twice
    # ValueError("fixture is being applied more than once to the same function")
    assert len(record) == 1


def test_fixture_disallow_marks_on_fixtures():
    """Test that applying a mark to a fixture warns (#3364)."""
    with pytest.warns(
        pytest.PytestRemovedIn9Warning,
        match=r"Marks applied to fixtures have no effect",
    ) as record:

        @pytest.mark.parametrize("example", ["hello"])
        @pytest.mark.usefixtures("tmp_path")
        @pytest.fixture
        def foo():
            raise NotImplementedError()

    assert len(record) == 2  # one for each mark decorator
    # should point to this file
    assert all(rec.filename == __file__ for rec in record)


def test_fixture_disallowed_between_marks():
    """Test that applying a mark to a fixture warns (#3364)."""
    with pytest.warns(
        pytest.PytestRemovedIn9Warning,
        match=r"Marks applied to fixtures have no effect",
    ) as record:

        @pytest.mark.parametrize("example", ["hello"])
        @pytest.fixture
        @pytest.mark.usefixtures("tmp_path")
        def foo():
            raise NotImplementedError()

    assert len(record) == 2  # one for each mark decorator
