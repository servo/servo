"""
``PluginManager`` unit and public API testing.
"""
import pytest

from pluggy import (
    PluginValidationError,
    HookCallError,
    HookimplMarker,
    HookspecMarker,
)
from pluggy._manager import importlib_metadata


hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def test_plugin_double_register(pm):
    """Registering the same plugin more then once isn't allowed"""
    pm.register(42, name="abc")
    with pytest.raises(ValueError):
        pm.register(42, name="abc")
    with pytest.raises(ValueError):
        pm.register(42, name="def")


def test_pm(pm):
    """Basic registration with objects"""

    class A:
        pass

    a1, a2 = A(), A()
    pm.register(a1)
    assert pm.is_registered(a1)
    pm.register(a2, "hello")
    assert pm.is_registered(a2)
    out = pm.get_plugins()
    assert a1 in out
    assert a2 in out
    assert pm.get_plugin("hello") == a2
    assert pm.unregister(a1) == a1
    assert not pm.is_registered(a1)

    out = pm.list_name_plugin()
    assert len(out) == 1
    assert out == [("hello", a2)]


def test_has_plugin(pm):
    class A:
        pass

    a1 = A()
    pm.register(a1, "hello")
    assert pm.is_registered(a1)
    assert pm.has_plugin("hello")


def test_register_dynamic_attr(he_pm):
    class A:
        def __getattr__(self, name):
            if name[0] != "_":
                return 42
            raise AttributeError()

    a = A()
    he_pm.register(a)
    assert not he_pm.get_hookcallers(a)


def test_pm_name(pm):
    class A:
        pass

    a1 = A()
    name = pm.register(a1, name="hello")
    assert name == "hello"
    pm.unregister(a1)
    assert pm.get_plugin(a1) is None
    assert not pm.is_registered(a1)
    assert not pm.get_plugins()
    name2 = pm.register(a1, name="hello")
    assert name2 == name
    pm.unregister(name="hello")
    assert pm.get_plugin(a1) is None
    assert not pm.is_registered(a1)
    assert not pm.get_plugins()


def test_set_blocked(pm):
    class A:
        pass

    a1 = A()
    name = pm.register(a1)
    assert pm.is_registered(a1)
    assert not pm.is_blocked(name)
    pm.set_blocked(name)
    assert pm.is_blocked(name)
    assert not pm.is_registered(a1)

    pm.set_blocked("somename")
    assert pm.is_blocked("somename")
    assert not pm.register(A(), "somename")
    pm.unregister(name="somename")
    assert pm.is_blocked("somename")


def test_register_mismatch_method(he_pm):
    class hello:
        @hookimpl
        def he_method_notexists(self):
            pass

    plugin = hello()

    he_pm.register(plugin)
    with pytest.raises(PluginValidationError) as excinfo:
        he_pm.check_pending()
    assert excinfo.value.plugin is plugin


def test_register_mismatch_arg(he_pm):
    class hello:
        @hookimpl
        def he_method1(self, qlwkje):
            pass

    plugin = hello()

    with pytest.raises(PluginValidationError) as excinfo:
        he_pm.register(plugin)
    assert excinfo.value.plugin is plugin


def test_register_hookwrapper_not_a_generator_function(he_pm):
    class hello:
        @hookimpl(hookwrapper=True)
        def he_method1(self):
            pass  # pragma: no cover

    plugin = hello()

    with pytest.raises(PluginValidationError, match="generator function") as excinfo:
        he_pm.register(plugin)
    assert excinfo.value.plugin is plugin


def test_register(pm):
    class MyPlugin:
        pass

    my = MyPlugin()
    pm.register(my)
    assert my in pm.get_plugins()
    my2 = MyPlugin()
    pm.register(my2)
    assert {my, my2}.issubset(pm.get_plugins())

    assert pm.is_registered(my)
    assert pm.is_registered(my2)
    pm.unregister(my)
    assert not pm.is_registered(my)
    assert my not in pm.get_plugins()


def test_register_unknown_hooks(pm):
    class Plugin1:
        @hookimpl
        def he_method1(self, arg):
            return arg + 1

    pname = pm.register(Plugin1())

    class Hooks:
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)
    # assert not pm._unverified_hooks
    assert pm.hook.he_method1(arg=1) == [2]
    assert len(pm.get_hookcallers(pm.get_plugin(pname))) == 1


def test_register_historic(pm):
    class Hooks:
        @hookspec(historic=True)
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    pm.hook.he_method1.call_historic(kwargs=dict(arg=1))
    out = []

    class Plugin:
        @hookimpl
        def he_method1(self, arg):
            out.append(arg)

    pm.register(Plugin())
    assert out == [1]

    class Plugin2:
        @hookimpl
        def he_method1(self, arg):
            out.append(arg * 10)

    pm.register(Plugin2())
    assert out == [1, 10]
    pm.hook.he_method1.call_historic(kwargs=dict(arg=12))
    assert out == [1, 10, 120, 12]


@pytest.mark.parametrize("result_callback", [True, False])
def test_with_result_memorized(pm, result_callback):
    """Verify that ``_HookCaller._maybe_apply_history()`
    correctly applies the ``result_callback`` function, when provided,
    to the result from calling each newly registered hook.
    """
    out = []
    if result_callback:

        def callback(res):
            out.append(res)

    else:
        callback = None

    class Hooks:
        @hookspec(historic=True)
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    class Plugin1:
        @hookimpl
        def he_method1(self, arg):
            return arg * 10

    pm.register(Plugin1())

    he_method1 = pm.hook.he_method1
    he_method1.call_historic(result_callback=callback, kwargs=dict(arg=1))

    class Plugin2:
        @hookimpl
        def he_method1(self, arg):
            return arg * 10

    pm.register(Plugin2())
    if result_callback:
        assert out == [10, 10]
    else:
        assert out == []


def test_with_callbacks_immediately_executed(pm):
    class Hooks:
        @hookspec(historic=True)
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    class Plugin1:
        @hookimpl
        def he_method1(self, arg):
            return arg * 10

    class Plugin2:
        @hookimpl
        def he_method1(self, arg):
            return arg * 20

    class Plugin3:
        @hookimpl
        def he_method1(self, arg):
            return arg * 30

    out = []
    pm.register(Plugin1())
    pm.register(Plugin2())

    he_method1 = pm.hook.he_method1
    he_method1.call_historic(lambda res: out.append(res), dict(arg=1))
    assert out == [20, 10]
    pm.register(Plugin3())
    assert out == [20, 10, 30]


def test_register_historic_incompat_hookwrapper(pm):
    class Hooks:
        @hookspec(historic=True)
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    out = []

    class Plugin:
        @hookimpl(hookwrapper=True)
        def he_method1(self, arg):
            out.append(arg)

    with pytest.raises(PluginValidationError):
        pm.register(Plugin())


def test_call_extra(pm):
    class Hooks:
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    def he_method1(arg):
        return arg * 10

    out = pm.hook.he_method1.call_extra([he_method1], dict(arg=1))
    assert out == [10]


def test_call_with_too_few_args(pm):
    class Hooks:
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    class Plugin1:
        @hookimpl
        def he_method1(self, arg):
            0 / 0

    pm.register(Plugin1())
    with pytest.raises(HookCallError):
        with pytest.warns(UserWarning):
            pm.hook.he_method1()


def test_subset_hook_caller(pm):
    class Hooks:
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    out = []

    class Plugin1:
        @hookimpl
        def he_method1(self, arg):
            out.append(arg)

    class Plugin2:
        @hookimpl
        def he_method1(self, arg):
            out.append(arg * 10)

    class PluginNo:
        pass

    plugin1, plugin2, plugin3 = Plugin1(), Plugin2(), PluginNo()
    pm.register(plugin1)
    pm.register(plugin2)
    pm.register(plugin3)
    pm.hook.he_method1(arg=1)
    assert out == [10, 1]
    out[:] = []

    hc = pm.subset_hook_caller("he_method1", [plugin1])
    hc(arg=2)
    assert out == [20]
    out[:] = []

    hc = pm.subset_hook_caller("he_method1", [plugin2])
    hc(arg=2)
    assert out == [2]
    out[:] = []

    pm.unregister(plugin1)
    hc(arg=2)
    assert out == []
    out[:] = []

    pm.hook.he_method1(arg=1)
    assert out == [10]


def test_get_hookimpls(pm):
    class Hooks:
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)
    assert pm.hook.he_method1.get_hookimpls() == []

    class Plugin1:
        @hookimpl
        def he_method1(self, arg):
            pass

    class Plugin2:
        @hookimpl
        def he_method1(self, arg):
            pass

    class PluginNo:
        pass

    plugin1, plugin2, plugin3 = Plugin1(), Plugin2(), PluginNo()
    pm.register(plugin1)
    pm.register(plugin2)
    pm.register(plugin3)

    hookimpls = pm.hook.he_method1.get_hookimpls()
    hook_plugins = [item.plugin for item in hookimpls]
    assert hook_plugins == [plugin1, plugin2]


def test_add_hookspecs_nohooks(pm):
    with pytest.raises(ValueError):
        pm.add_hookspecs(10)


def test_load_setuptools_instantiation(monkeypatch, pm):
    class EntryPoint:
        name = "myname"
        group = "hello"
        value = "myname:foo"

        def load(self):
            class PseudoPlugin:
                x = 42

            return PseudoPlugin()

    class Distribution:
        entry_points = (EntryPoint(),)

    dist = Distribution()

    def my_distributions():
        return (dist,)

    monkeypatch.setattr(importlib_metadata, "distributions", my_distributions)
    num = pm.load_setuptools_entrypoints("hello")
    assert num == 1
    plugin = pm.get_plugin("myname")
    assert plugin.x == 42
    ret = pm.list_plugin_distinfo()
    # poor man's `assert ret == [(plugin, mock.ANY)]`
    assert len(ret) == 1
    assert len(ret[0]) == 2
    assert ret[0][0] == plugin
    assert ret[0][1]._dist == dist
    num = pm.load_setuptools_entrypoints("hello")
    assert num == 0  # no plugin loaded by this call


def test_add_tracefuncs(he_pm):
    out = []

    class api1:
        @hookimpl
        def he_method1(self):
            out.append("he_method1-api1")

    class api2:
        @hookimpl
        def he_method1(self):
            out.append("he_method1-api2")

    he_pm.register(api1())
    he_pm.register(api2())

    def before(hook_name, hook_impls, kwargs):
        out.append((hook_name, list(hook_impls), kwargs))

    def after(outcome, hook_name, hook_impls, kwargs):
        out.append((outcome, hook_name, list(hook_impls), kwargs))

    undo = he_pm.add_hookcall_monitoring(before, after)

    he_pm.hook.he_method1(arg=1)
    assert len(out) == 4
    assert out[0][0] == "he_method1"
    assert len(out[0][1]) == 2
    assert isinstance(out[0][2], dict)
    assert out[1] == "he_method1-api2"
    assert out[2] == "he_method1-api1"
    assert len(out[3]) == 4
    assert out[3][1] == out[0][0]

    undo()
    he_pm.hook.he_method1(arg=1)
    assert len(out) == 4 + 2


def test_hook_tracing(he_pm):
    saveindent = []

    class api1:
        @hookimpl
        def he_method1(self):
            saveindent.append(he_pm.trace.root.indent)

    class api2:
        @hookimpl
        def he_method1(self):
            saveindent.append(he_pm.trace.root.indent)
            raise ValueError()

    he_pm.register(api1())
    out = []
    he_pm.trace.root.setwriter(out.append)
    undo = he_pm.enable_tracing()
    try:
        indent = he_pm.trace.root.indent
        he_pm.hook.he_method1(arg=1)
        assert indent == he_pm.trace.root.indent
        assert len(out) == 2
        assert "he_method1" in out[0]
        assert "finish" in out[1]

        out[:] = []
        he_pm.register(api2())

        with pytest.raises(ValueError):
            he_pm.hook.he_method1(arg=1)
        assert he_pm.trace.root.indent == indent
        assert saveindent[0] > indent
    finally:
        undo()
