import os
import shutil
import sys
import types
from typing import List

import pytest
from _pytest.config import Config
from _pytest.config import ExitCode
from _pytest.config import PytestPluginManager
from _pytest.config.exceptions import UsageError
from _pytest.main import Session
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pathlib import import_path
from _pytest.pytester import Pytester


@pytest.fixture
def pytestpm() -> PytestPluginManager:
    return PytestPluginManager()


class TestPytestPluginInteractions:
    def test_addhooks_conftestplugin(
        self, pytester: Pytester, _config_for_test: Config
    ) -> None:
        pytester.makepyfile(
            newhooks="""
            def pytest_myhook(xyz):
                "new hook"
        """
        )
        conf = pytester.makeconftest(
            """
            import newhooks
            def pytest_addhooks(pluginmanager):
                pluginmanager.add_hookspecs(newhooks)
            def pytest_myhook(xyz):
                return xyz + 1
        """
        )
        config = _config_for_test
        pm = config.pluginmanager
        pm.hook.pytest_addhooks.call_historic(
            kwargs=dict(pluginmanager=config.pluginmanager)
        )
        config.pluginmanager._importconftest(
            conf, importmode="prepend", rootpath=pytester.path
        )
        # print(config.pluginmanager.get_plugins())
        res = config.hook.pytest_myhook(xyz=10)
        assert res == [11]

    def test_addhooks_nohooks(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import sys
            def pytest_addhooks(pluginmanager):
                pluginmanager.add_hookspecs(sys)
        """
        )
        res = pytester.runpytest()
        assert res.ret != 0
        res.stderr.fnmatch_lines(["*did not find*sys*"])

    def test_do_option_postinitialize(self, pytester: Pytester) -> None:
        config = pytester.parseconfigure()
        assert not hasattr(config.option, "test123")
        p = pytester.makepyfile(
            """
            def pytest_addoption(parser):
                parser.addoption('--test123', action="store_true",
                    default=True)
        """
        )
        config.pluginmanager._importconftest(
            p, importmode="prepend", rootpath=pytester.path
        )
        assert config.option.test123

    def test_configure(self, pytester: Pytester) -> None:
        config = pytester.parseconfig()
        values = []

        class A:
            def pytest_configure(self):
                values.append(self)

        config.pluginmanager.register(A())
        assert len(values) == 0
        config._do_configure()
        assert len(values) == 1
        config.pluginmanager.register(A())  # leads to a configured() plugin
        assert len(values) == 2
        assert values[0] != values[1]

        config._ensure_unconfigure()
        config.pluginmanager.register(A())
        assert len(values) == 2

    def test_hook_tracing(self, _config_for_test: Config) -> None:
        pytestpm = _config_for_test.pluginmanager  # fully initialized with plugins
        saveindent = []

        class api1:
            def pytest_plugin_registered(self):
                saveindent.append(pytestpm.trace.root.indent)

        class api2:
            def pytest_plugin_registered(self):
                saveindent.append(pytestpm.trace.root.indent)
                raise ValueError()

        values: List[str] = []
        pytestpm.trace.root.setwriter(values.append)
        undo = pytestpm.enable_tracing()
        try:
            indent = pytestpm.trace.root.indent
            p = api1()
            pytestpm.register(p)
            assert pytestpm.trace.root.indent == indent
            assert len(values) >= 2
            assert "pytest_plugin_registered" in values[0]
            assert "finish" in values[1]

            values[:] = []
            with pytest.raises(ValueError):
                pytestpm.register(api2())
            assert pytestpm.trace.root.indent == indent
            assert saveindent[0] > indent
        finally:
            undo()

    def test_hook_proxy(self, pytester: Pytester) -> None:
        """Test the gethookproxy function(#2016)"""
        config = pytester.parseconfig()
        session = Session.from_config(config)
        pytester.makepyfile(**{"tests/conftest.py": "", "tests/subdir/conftest.py": ""})

        conftest1 = pytester.path.joinpath("tests/conftest.py")
        conftest2 = pytester.path.joinpath("tests/subdir/conftest.py")

        config.pluginmanager._importconftest(
            conftest1, importmode="prepend", rootpath=pytester.path
        )
        ihook_a = session.gethookproxy(pytester.path / "tests")
        assert ihook_a is not None
        config.pluginmanager._importconftest(
            conftest2, importmode="prepend", rootpath=pytester.path
        )
        ihook_b = session.gethookproxy(pytester.path / "tests")
        assert ihook_a is not ihook_b

    def test_hook_with_addoption(self, pytester: Pytester) -> None:
        """Test that hooks can be used in a call to pytest_addoption"""
        pytester.makepyfile(
            newhooks="""
            import pytest
            @pytest.hookspec(firstresult=True)
            def pytest_default_value():
                pass
        """
        )
        pytester.makepyfile(
            myplugin="""
            import newhooks
            def pytest_addhooks(pluginmanager):
                pluginmanager.add_hookspecs(newhooks)
            def pytest_addoption(parser, pluginmanager):
                default_value = pluginmanager.hook.pytest_default_value()
                parser.addoption("--config", help="Config, defaults to %(default)s", default=default_value)
        """
        )
        pytester.makeconftest(
            """
            pytest_plugins=("myplugin",)
            def pytest_default_value():
                return "default_value"
        """
        )
        res = pytester.runpytest("--help")
        res.stdout.fnmatch_lines(["*--config=CONFIG*default_value*"])


def test_default_markers(pytester: Pytester) -> None:
    result = pytester.runpytest("--markers")
    result.stdout.fnmatch_lines(["*tryfirst*first*", "*trylast*last*"])


def test_importplugin_error_message(
    pytester: Pytester, pytestpm: PytestPluginManager
) -> None:
    """Don't hide import errors when importing plugins and provide
    an easy to debug message.

    See #375 and #1998.
    """
    pytester.syspathinsert(pytester.path)
    pytester.makepyfile(
        qwe="""\
        def test_traceback():
            raise ImportError('Not possible to import: ☺')
        test_traceback()
        """
    )
    with pytest.raises(ImportError) as excinfo:
        pytestpm.import_plugin("qwe")

    assert str(excinfo.value).endswith(
        'Error importing plugin "qwe": Not possible to import: ☺'
    )
    assert "in test_traceback" in str(excinfo.traceback[-1])


class TestPytestPluginManager:
    def test_register_imported_modules(self) -> None:
        pm = PytestPluginManager()
        mod = types.ModuleType("x.y.pytest_hello")
        pm.register(mod)
        assert pm.is_registered(mod)
        values = pm.get_plugins()
        assert mod in values
        pytest.raises(ValueError, pm.register, mod)
        pytest.raises(ValueError, lambda: pm.register(mod))
        # assert not pm.is_registered(mod2)
        assert pm.get_plugins() == values

    def test_canonical_import(self, monkeypatch):
        mod = types.ModuleType("pytest_xyz")
        monkeypatch.setitem(sys.modules, "pytest_xyz", mod)
        pm = PytestPluginManager()
        pm.import_plugin("pytest_xyz")
        assert pm.get_plugin("pytest_xyz") == mod
        assert pm.is_registered(mod)

    def test_consider_module(
        self, pytester: Pytester, pytestpm: PytestPluginManager
    ) -> None:
        pytester.syspathinsert()
        pytester.makepyfile(pytest_p1="#")
        pytester.makepyfile(pytest_p2="#")
        mod = types.ModuleType("temp")
        mod.__dict__["pytest_plugins"] = ["pytest_p1", "pytest_p2"]
        pytestpm.consider_module(mod)
        assert pytestpm.get_plugin("pytest_p1").__name__ == "pytest_p1"
        assert pytestpm.get_plugin("pytest_p2").__name__ == "pytest_p2"

    def test_consider_module_import_module(
        self, pytester: Pytester, _config_for_test: Config
    ) -> None:
        pytestpm = _config_for_test.pluginmanager
        mod = types.ModuleType("x")
        mod.__dict__["pytest_plugins"] = "pytest_a"
        aplugin = pytester.makepyfile(pytest_a="#")
        reprec = pytester.make_hook_recorder(pytestpm)
        pytester.syspathinsert(aplugin.parent)
        pytestpm.consider_module(mod)
        call = reprec.getcall(pytestpm.hook.pytest_plugin_registered.name)
        assert call.plugin.__name__ == "pytest_a"

        # check that it is not registered twice
        pytestpm.consider_module(mod)
        values = reprec.getcalls("pytest_plugin_registered")
        assert len(values) == 1

    def test_consider_env_fails_to_import(
        self, monkeypatch: MonkeyPatch, pytestpm: PytestPluginManager
    ) -> None:
        monkeypatch.setenv("PYTEST_PLUGINS", "nonexisting", prepend=",")
        with pytest.raises(ImportError):
            pytestpm.consider_env()

    @pytest.mark.filterwarnings("always")
    def test_plugin_skip(self, pytester: Pytester, monkeypatch: MonkeyPatch) -> None:
        p = pytester.makepyfile(
            skipping1="""
            import pytest
            pytest.skip("hello", allow_module_level=True)
        """
        )
        shutil.copy(p, p.with_name("skipping2.py"))
        monkeypatch.setenv("PYTEST_PLUGINS", "skipping2")
        result = pytester.runpytest("-p", "skipping1", syspathinsert=True)
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.fnmatch_lines(
            ["*skipped plugin*skipping1*hello*", "*skipped plugin*skipping2*hello*"]
        )

    def test_consider_env_plugin_instantiation(
        self,
        pytester: Pytester,
        monkeypatch: MonkeyPatch,
        pytestpm: PytestPluginManager,
    ) -> None:
        pytester.syspathinsert()
        pytester.makepyfile(xy123="#")
        monkeypatch.setitem(os.environ, "PYTEST_PLUGINS", "xy123")
        l1 = len(pytestpm.get_plugins())
        pytestpm.consider_env()
        l2 = len(pytestpm.get_plugins())
        assert l2 == l1 + 1
        assert pytestpm.get_plugin("xy123")
        pytestpm.consider_env()
        l3 = len(pytestpm.get_plugins())
        assert l2 == l3

    def test_pluginmanager_ENV_startup(
        self, pytester: Pytester, monkeypatch: MonkeyPatch
    ) -> None:
        pytester.makepyfile(pytest_x500="#")
        p = pytester.makepyfile(
            """
            import pytest
            def test_hello(pytestconfig):
                plugin = pytestconfig.pluginmanager.get_plugin('pytest_x500')
                assert plugin is not None
        """
        )
        monkeypatch.setenv("PYTEST_PLUGINS", "pytest_x500", prepend=",")
        result = pytester.runpytest(p, syspathinsert=True)
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_import_plugin_importname(
        self, pytester: Pytester, pytestpm: PytestPluginManager
    ) -> None:
        pytest.raises(ImportError, pytestpm.import_plugin, "qweqwex.y")
        pytest.raises(ImportError, pytestpm.import_plugin, "pytest_qweqwx.y")

        pytester.syspathinsert()
        pluginname = "pytest_hello"
        pytester.makepyfile(**{pluginname: ""})
        pytestpm.import_plugin("pytest_hello")
        len1 = len(pytestpm.get_plugins())
        pytestpm.import_plugin("pytest_hello")
        len2 = len(pytestpm.get_plugins())
        assert len1 == len2
        plugin1 = pytestpm.get_plugin("pytest_hello")
        assert plugin1.__name__.endswith("pytest_hello")
        plugin2 = pytestpm.get_plugin("pytest_hello")
        assert plugin2 is plugin1

    def test_import_plugin_dotted_name(
        self, pytester: Pytester, pytestpm: PytestPluginManager
    ) -> None:
        pytest.raises(ImportError, pytestpm.import_plugin, "qweqwex.y")
        pytest.raises(ImportError, pytestpm.import_plugin, "pytest_qweqwex.y")

        pytester.syspathinsert()
        pytester.mkpydir("pkg").joinpath("plug.py").write_text("x=3")
        pluginname = "pkg.plug"
        pytestpm.import_plugin(pluginname)
        mod = pytestpm.get_plugin("pkg.plug")
        assert mod.x == 3

    def test_consider_conftest_deps(
        self,
        pytester: Pytester,
        pytestpm: PytestPluginManager,
    ) -> None:
        mod = import_path(
            pytester.makepyfile("pytest_plugins='xyz'"), root=pytester.path
        )
        with pytest.raises(ImportError):
            pytestpm.consider_conftest(mod)


class TestPytestPluginManagerBootstrapming:
    def test_preparse_args(self, pytestpm: PytestPluginManager) -> None:
        pytest.raises(
            ImportError, lambda: pytestpm.consider_preparse(["xyz", "-p", "hello123"])
        )

        # Handles -p without space (#3532).
        with pytest.raises(ImportError) as excinfo:
            pytestpm.consider_preparse(["-phello123"])
        assert '"hello123"' in excinfo.value.args[0]
        pytestpm.consider_preparse(["-pno:hello123"])

        # Handles -p without following arg (when used without argparse).
        pytestpm.consider_preparse(["-p"])

        with pytest.raises(UsageError, match="^plugin main cannot be disabled$"):
            pytestpm.consider_preparse(["-p", "no:main"])

    def test_plugin_prevent_register(self, pytestpm: PytestPluginManager) -> None:
        pytestpm.consider_preparse(["xyz", "-p", "no:abc"])
        l1 = pytestpm.get_plugins()
        pytestpm.register(42, name="abc")
        l2 = pytestpm.get_plugins()
        assert len(l2) == len(l1)
        assert 42 not in l2

    def test_plugin_prevent_register_unregistered_alredy_registered(
        self, pytestpm: PytestPluginManager
    ) -> None:
        pytestpm.register(42, name="abc")
        l1 = pytestpm.get_plugins()
        assert 42 in l1
        pytestpm.consider_preparse(["xyz", "-p", "no:abc"])
        l2 = pytestpm.get_plugins()
        assert 42 not in l2

    def test_plugin_prevent_register_stepwise_on_cacheprovider_unregister(
        self, pytestpm: PytestPluginManager
    ) -> None:
        """From PR #4304: The only way to unregister a module is documented at
        the end of https://docs.pytest.org/en/stable/how-to/plugins.html.

        When unregister cacheprovider, then unregister stepwise too.
        """
        pytestpm.register(42, name="cacheprovider")
        pytestpm.register(43, name="stepwise")
        l1 = pytestpm.get_plugins()
        assert 42 in l1
        assert 43 in l1
        pytestpm.consider_preparse(["xyz", "-p", "no:cacheprovider"])
        l2 = pytestpm.get_plugins()
        assert 42 not in l2
        assert 43 not in l2

    def test_blocked_plugin_can_be_used(self, pytestpm: PytestPluginManager) -> None:
        pytestpm.consider_preparse(["xyz", "-p", "no:abc", "-p", "abc"])

        assert pytestpm.has_plugin("abc")
        assert not pytestpm.is_blocked("abc")
        assert not pytestpm.is_blocked("pytest_abc")
