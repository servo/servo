# encoding: UTF-8
from __future__ import absolute_import, division, print_function
import pytest
import os
import re
import sys
import types

from _pytest.config import get_config, PytestPluginManager
from _pytest.main import EXIT_NOTESTSCOLLECTED, Session


@pytest.fixture
def pytestpm():
    return PytestPluginManager()


class TestPytestPluginInteractions(object):

    def test_addhooks_conftestplugin(self, testdir):
        testdir.makepyfile(
            newhooks="""
            def pytest_myhook(xyz):
                "new hook"
        """
        )
        conf = testdir.makeconftest(
            """
            import sys ; sys.path.insert(0, '.')
            import newhooks
            def pytest_addhooks(pluginmanager):
                pluginmanager.addhooks(newhooks)
            def pytest_myhook(xyz):
                return xyz + 1
        """
        )
        config = get_config()
        pm = config.pluginmanager
        pm.hook.pytest_addhooks.call_historic(
            kwargs=dict(pluginmanager=config.pluginmanager)
        )
        config.pluginmanager._importconftest(conf)
        # print(config.pluginmanager.get_plugins())
        res = config.hook.pytest_myhook(xyz=10)
        assert res == [11]

    def test_addhooks_nohooks(self, testdir):
        testdir.makeconftest(
            """
            import sys
            def pytest_addhooks(pluginmanager):
                pluginmanager.addhooks(sys)
        """
        )
        res = testdir.runpytest()
        assert res.ret != 0
        res.stderr.fnmatch_lines(["*did not find*sys*"])

    def test_namespace_early_from_import(self, testdir):
        p = testdir.makepyfile(
            """
            from pytest import Item
            from pytest import Item as Item2
            assert Item is Item2
        """
        )
        result = testdir.runpython(p)
        assert result.ret == 0

    def test_do_ext_namespace(self, testdir):
        testdir.makeconftest(
            """
            def pytest_namespace():
                return {'hello': 'world'}
        """
        )
        p = testdir.makepyfile(
            """
            from pytest import hello
            import pytest
            def test_hello():
                assert hello == "world"
                assert 'hello' in pytest.__all__
        """
        )
        reprec = testdir.inline_run(p)
        reprec.assertoutcome(passed=1)

    def test_do_option_postinitialize(self, testdir):
        config = testdir.parseconfigure()
        assert not hasattr(config.option, "test123")
        p = testdir.makepyfile(
            """
            def pytest_addoption(parser):
                parser.addoption('--test123', action="store_true",
                    default=True)
        """
        )
        config.pluginmanager._importconftest(p)
        assert config.option.test123

    def test_configure(self, testdir):
        config = testdir.parseconfig()
        values = []

        class A(object):

            def pytest_configure(self, config):
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

    def test_hook_tracing(self):
        pytestpm = get_config().pluginmanager  # fully initialized with plugins
        saveindent = []

        class api1(object):

            def pytest_plugin_registered(self):
                saveindent.append(pytestpm.trace.root.indent)

        class api2(object):

            def pytest_plugin_registered(self):
                saveindent.append(pytestpm.trace.root.indent)
                raise ValueError()

        values = []
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

    def test_hook_proxy(self, testdir):
        """Test the gethookproxy function(#2016)"""
        config = testdir.parseconfig()
        session = Session(config)
        testdir.makepyfile(**{"tests/conftest.py": "", "tests/subdir/conftest.py": ""})

        conftest1 = testdir.tmpdir.join("tests/conftest.py")
        conftest2 = testdir.tmpdir.join("tests/subdir/conftest.py")

        config.pluginmanager._importconftest(conftest1)
        ihook_a = session.gethookproxy(testdir.tmpdir.join("tests"))
        assert ihook_a is not None
        config.pluginmanager._importconftest(conftest2)
        ihook_b = session.gethookproxy(testdir.tmpdir.join("tests"))
        assert ihook_a is not ihook_b

    def test_warn_on_deprecated_addhooks(self, pytestpm):
        warnings = []

        class get_warnings(object):

            def pytest_logwarning(self, code, fslocation, message, nodeid):
                warnings.append(message)

        class Plugin(object):

            def pytest_testhook():
                pass

        pytestpm.register(get_warnings())
        before = list(warnings)
        pytestpm.addhooks(Plugin())
        assert len(warnings) == len(before) + 1
        assert "deprecated" in warnings[-1]


def test_namespace_has_default_and_env_plugins(testdir):
    p = testdir.makepyfile(
        """
        import pytest
        pytest.mark
    """
    )
    result = testdir.runpython(p)
    assert result.ret == 0


def test_default_markers(testdir):
    result = testdir.runpytest("--markers")
    result.stdout.fnmatch_lines(["*tryfirst*first*", "*trylast*last*"])


def test_importplugin_error_message(testdir, pytestpm):
    """Don't hide import errors when importing plugins and provide
    an easy to debug message.

    See #375 and #1998.
    """
    testdir.syspathinsert(testdir.tmpdir)
    testdir.makepyfile(
        qwe="""
        # encoding: UTF-8
        def test_traceback():
            raise ImportError(u'Not possible to import: ☺')
        test_traceback()
    """
    )
    with pytest.raises(ImportError) as excinfo:
        pytestpm.import_plugin("qwe")

    expected_message = '.*Error importing plugin "qwe": Not possible to import: .'
    expected_traceback = ".*in test_traceback"
    assert re.match(expected_message, str(excinfo.value))
    assert re.match(expected_traceback, str(excinfo.traceback[-1]))


class TestPytestPluginManager(object):

    def test_register_imported_modules(self):
        pm = PytestPluginManager()
        mod = types.ModuleType("x.y.pytest_hello")
        pm.register(mod)
        assert pm.is_registered(mod)
        values = pm.get_plugins()
        assert mod in values
        pytest.raises(ValueError, "pm.register(mod)")
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

    def test_consider_module(self, testdir, pytestpm):
        testdir.syspathinsert()
        testdir.makepyfile(pytest_p1="#")
        testdir.makepyfile(pytest_p2="#")
        mod = types.ModuleType("temp")
        mod.pytest_plugins = ["pytest_p1", "pytest_p2"]
        pytestpm.consider_module(mod)
        assert pytestpm.get_plugin("pytest_p1").__name__ == "pytest_p1"
        assert pytestpm.get_plugin("pytest_p2").__name__ == "pytest_p2"

    def test_consider_module_import_module(self, testdir):
        pytestpm = get_config().pluginmanager
        mod = types.ModuleType("x")
        mod.pytest_plugins = "pytest_a"
        aplugin = testdir.makepyfile(pytest_a="#")
        reprec = testdir.make_hook_recorder(pytestpm)
        # syspath.prepend(aplugin.dirpath())
        sys.path.insert(0, str(aplugin.dirpath()))
        pytestpm.consider_module(mod)
        call = reprec.getcall(pytestpm.hook.pytest_plugin_registered.name)
        assert call.plugin.__name__ == "pytest_a"

        # check that it is not registered twice
        pytestpm.consider_module(mod)
        values = reprec.getcalls("pytest_plugin_registered")
        assert len(values) == 1

    def test_consider_env_fails_to_import(self, monkeypatch, pytestpm):
        monkeypatch.setenv("PYTEST_PLUGINS", "nonexisting", prepend=",")
        with pytest.raises(ImportError):
            pytestpm.consider_env()

    def test_plugin_skip(self, testdir, monkeypatch):
        p = testdir.makepyfile(
            skipping1="""
            import pytest
            pytest.skip("hello")
        """
        )
        p.copy(p.dirpath("skipping2.py"))
        monkeypatch.setenv("PYTEST_PLUGINS", "skipping2")
        result = testdir.runpytest("-rw", "-p", "skipping1", syspathinsert=True)
        assert result.ret == EXIT_NOTESTSCOLLECTED
        result.stdout.fnmatch_lines(
            ["*skipped plugin*skipping1*hello*", "*skipped plugin*skipping2*hello*"]
        )

    def test_consider_env_plugin_instantiation(self, testdir, monkeypatch, pytestpm):
        testdir.syspathinsert()
        testdir.makepyfile(xy123="#")
        monkeypatch.setitem(os.environ, "PYTEST_PLUGINS", "xy123")
        l1 = len(pytestpm.get_plugins())
        pytestpm.consider_env()
        l2 = len(pytestpm.get_plugins())
        assert l2 == l1 + 1
        assert pytestpm.get_plugin("xy123")
        pytestpm.consider_env()
        l3 = len(pytestpm.get_plugins())
        assert l2 == l3

    def test_pluginmanager_ENV_startup(self, testdir, monkeypatch):
        testdir.makepyfile(pytest_x500="#")
        p = testdir.makepyfile(
            """
            import pytest
            def test_hello(pytestconfig):
                plugin = pytestconfig.pluginmanager.get_plugin('pytest_x500')
                assert plugin is not None
        """
        )
        monkeypatch.setenv("PYTEST_PLUGINS", "pytest_x500", prepend=",")
        result = testdir.runpytest(p, syspathinsert=True)
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_import_plugin_importname(self, testdir, pytestpm):
        pytest.raises(ImportError, 'pytestpm.import_plugin("qweqwex.y")')
        pytest.raises(ImportError, 'pytestpm.import_plugin("pytest_qweqwx.y")')

        testdir.syspathinsert()
        pluginname = "pytest_hello"
        testdir.makepyfile(**{pluginname: ""})
        pytestpm.import_plugin("pytest_hello")
        len1 = len(pytestpm.get_plugins())
        pytestpm.import_plugin("pytest_hello")
        len2 = len(pytestpm.get_plugins())
        assert len1 == len2
        plugin1 = pytestpm.get_plugin("pytest_hello")
        assert plugin1.__name__.endswith("pytest_hello")
        plugin2 = pytestpm.get_plugin("pytest_hello")
        assert plugin2 is plugin1

    def test_import_plugin_dotted_name(self, testdir, pytestpm):
        pytest.raises(ImportError, 'pytestpm.import_plugin("qweqwex.y")')
        pytest.raises(ImportError, 'pytestpm.import_plugin("pytest_qweqwex.y")')

        testdir.syspathinsert()
        testdir.mkpydir("pkg").join("plug.py").write("x=3")
        pluginname = "pkg.plug"
        pytestpm.import_plugin(pluginname)
        mod = pytestpm.get_plugin("pkg.plug")
        assert mod.x == 3

    def test_consider_conftest_deps(self, testdir, pytestpm):
        mod = testdir.makepyfile("pytest_plugins='xyz'").pyimport()
        with pytest.raises(ImportError):
            pytestpm.consider_conftest(mod)


class TestPytestPluginManagerBootstrapming(object):

    def test_preparse_args(self, pytestpm):
        pytest.raises(
            ImportError, lambda: pytestpm.consider_preparse(["xyz", "-p", "hello123"])
        )

    def test_plugin_prevent_register(self, pytestpm):
        pytestpm.consider_preparse(["xyz", "-p", "no:abc"])
        l1 = pytestpm.get_plugins()
        pytestpm.register(42, name="abc")
        l2 = pytestpm.get_plugins()
        assert len(l2) == len(l1)
        assert 42 not in l2

    def test_plugin_prevent_register_unregistered_alredy_registered(self, pytestpm):
        pytestpm.register(42, name="abc")
        l1 = pytestpm.get_plugins()
        assert 42 in l1
        pytestpm.consider_preparse(["xyz", "-p", "no:abc"])
        l2 = pytestpm.get_plugins()
        assert 42 not in l2
