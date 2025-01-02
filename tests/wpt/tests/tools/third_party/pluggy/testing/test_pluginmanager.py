"""
``PluginManager`` unit and public API testing.
"""

import importlib.metadata
from typing import Any
from typing import List

import pytest

from pluggy import HookCallError
from pluggy import HookimplMarker
from pluggy import HookspecMarker
from pluggy import PluginManager
from pluggy import PluginValidationError


hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def test_plugin_double_register(pm: PluginManager) -> None:
    """Registering the same plugin more then once isn't allowed"""
    pm.register(42, name="abc")
    with pytest.raises(ValueError):
        pm.register(42, name="abc")
    with pytest.raises(ValueError):
        pm.register(42, name="def")


def test_pm(pm: PluginManager) -> None:
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

    out2 = pm.list_name_plugin()
    assert len(out2) == 1
    assert out2 == [("hello", a2)]


def test_has_plugin(pm: PluginManager) -> None:
    class A:
        pass

    a1 = A()
    pm.register(a1, "hello")
    assert pm.is_registered(a1)
    assert pm.has_plugin("hello")


def test_register_dynamic_attr(he_pm: PluginManager) -> None:
    class A:
        def __getattr__(self, name):
            if name[0] != "_":
                return 42
            raise AttributeError()

    a = A()
    he_pm.register(a)
    assert not he_pm.get_hookcallers(a)


def test_pm_name(pm: PluginManager) -> None:
    class A:
        pass

    a1 = A()
    name = pm.register(a1, name="hello")
    assert name == "hello"
    pm.unregister(a1)
    assert pm.get_plugin("hello") is None
    assert not pm.is_registered(a1)
    assert not pm.get_plugins()
    name2 = pm.register(a1, name="hello")
    assert name2 == name
    pm.unregister(name="hello")
    assert pm.get_plugin("hello") is None
    assert not pm.is_registered(a1)
    assert not pm.get_plugins()


def test_set_blocked(pm: PluginManager) -> None:
    class A:
        pass

    a1 = A()
    name = pm.register(a1)
    assert name is not None
    assert pm.is_registered(a1)
    assert not pm.is_blocked(name)
    assert pm.get_plugins() == {a1}

    pm.set_blocked(name)
    assert pm.is_blocked(name)
    assert not pm.is_registered(a1)
    assert pm.get_plugins() == set()

    pm.set_blocked("somename")
    assert pm.is_blocked("somename")
    assert not pm.register(A(), "somename")
    pm.unregister(name="somename")
    assert pm.is_blocked("somename")
    assert pm.get_plugins() == set()

    # Unblock.
    assert not pm.unblock("someothername")
    assert pm.unblock("somename")
    assert not pm.is_blocked("somename")
    assert not pm.unblock("somename")
    assert pm.register(A(), "somename")


def test_register_mismatch_method(he_pm: PluginManager) -> None:
    class hello:
        @hookimpl
        def he_method_notexists(self):
            pass

    plugin = hello()

    he_pm.register(plugin)
    with pytest.raises(PluginValidationError) as excinfo:
        he_pm.check_pending()
    assert excinfo.value.plugin is plugin


def test_register_mismatch_arg(he_pm: PluginManager) -> None:
    class hello:
        @hookimpl
        def he_method1(self, qlwkje):
            pass

    plugin = hello()

    with pytest.raises(PluginValidationError) as excinfo:
        he_pm.register(plugin)
    assert excinfo.value.plugin is plugin


def test_register_hookwrapper_not_a_generator_function(he_pm: PluginManager) -> None:
    class hello:
        @hookimpl(hookwrapper=True)
        def he_method1(self):
            pass  # pragma: no cover

    plugin = hello()

    with pytest.raises(PluginValidationError, match="generator function") as excinfo:
        he_pm.register(plugin)
    assert excinfo.value.plugin is plugin


def test_register_both_wrapper_and_hookwrapper(he_pm: PluginManager) -> None:
    class hello:
        @hookimpl(wrapper=True, hookwrapper=True)
        def he_method1(self):
            yield  # pragma: no cover

    plugin = hello()

    with pytest.raises(
        PluginValidationError, match="wrapper.*hookwrapper.*mutually exclusive"
    ) as excinfo:
        he_pm.register(plugin)
    assert excinfo.value.plugin is plugin


def test_register(pm: PluginManager) -> None:
    class MyPlugin:
        @hookimpl
        def he_method1(self): ...

    my = MyPlugin()
    pm.register(my)
    assert pm.get_plugins() == {my}
    my2 = MyPlugin()
    pm.register(my2)
    assert pm.get_plugins() == {my, my2}

    assert pm.is_registered(my)
    assert pm.is_registered(my2)
    pm.unregister(my)
    assert not pm.is_registered(my)
    assert pm.get_plugins() == {my2}

    with pytest.raises(AssertionError, match=r"not registered"):
        pm.unregister(my)


def test_register_unknown_hooks(pm: PluginManager) -> None:
    class Plugin1:
        @hookimpl
        def he_method1(self, arg):
            return arg + 1

    pname = pm.register(Plugin1())
    assert pname is not None

    class Hooks:
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)
    # assert not pm._unverified_hooks
    assert pm.hook.he_method1(arg=1) == [2]
    hookcallers = pm.get_hookcallers(pm.get_plugin(pname))
    assert hookcallers is not None
    assert len(hookcallers) == 1


def test_register_historic(pm: PluginManager) -> None:
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


def test_historic_with_subset_hook_caller(pm: PluginManager) -> None:
    class Hooks:
        @hookspec(historic=True)
        def he_method1(self, arg): ...

    pm.add_hookspecs(Hooks)

    out = []

    class Plugin:
        @hookimpl
        def he_method1(self, arg):
            out.append(arg)

    plugin = Plugin()
    pm.register(plugin)

    class Plugin2:
        @hookimpl
        def he_method1(self, arg):
            out.append(arg * 10)

    shc = pm.subset_hook_caller("he_method1", remove_plugins=[plugin])
    shc.call_historic(kwargs=dict(arg=1))

    pm.register(Plugin2())
    assert out == [10]

    pm.register(Plugin())
    assert out == [10, 1]


@pytest.mark.parametrize("result_callback", [True, False])
def test_with_result_memorized(pm: PluginManager, result_callback: bool) -> None:
    """Verify that ``HookCaller._maybe_apply_history()`
    correctly applies the ``result_callback`` function, when provided,
    to the result from calling each newly registered hook.
    """
    out = []
    if not result_callback:
        callback = None
    else:

        def callback(res) -> None:
            out.append(res)

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


def test_with_callbacks_immediately_executed(pm: PluginManager) -> None:
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


def test_register_historic_incompat_hookwrapper(pm: PluginManager) -> None:
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


def test_register_historic_incompat_wrapper(pm: PluginManager) -> None:
    class Hooks:
        @hookspec(historic=True)
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    class Plugin:
        @hookimpl(wrapper=True)
        def he_method1(self, arg):
            yield

    with pytest.raises(PluginValidationError):
        pm.register(Plugin())


def test_call_extra(pm: PluginManager) -> None:
    class Hooks:
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    def he_method1(arg):
        return arg * 10

    out = pm.hook.he_method1.call_extra([he_method1], dict(arg=1))
    assert out == [10]


def test_call_with_too_few_args(pm: PluginManager) -> None:
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


def test_subset_hook_caller(pm: PluginManager) -> None:
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

    assert repr(hc) == "<_SubsetHookCaller 'he_method1'>"


def test_get_hookimpls(pm: PluginManager) -> None:
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


def test_get_hookcallers(pm: PluginManager) -> None:
    class Hooks:
        @hookspec
        def he_method1(self): ...

        @hookspec
        def he_method2(self): ...

    pm.add_hookspecs(Hooks)

    class Plugin1:
        @hookimpl
        def he_method1(self): ...

        @hookimpl
        def he_method2(self): ...

    class Plugin2:
        @hookimpl
        def he_method1(self): ...

    class Plugin3:
        @hookimpl
        def he_method2(self): ...

    plugin1 = Plugin1()
    pm.register(plugin1)
    plugin2 = Plugin2()
    pm.register(plugin2)
    plugin3 = Plugin3()
    pm.register(plugin3)

    hookcallers1 = pm.get_hookcallers(plugin1)
    assert hookcallers1 is not None
    assert len(hookcallers1) == 2
    hookcallers2 = pm.get_hookcallers(plugin2)
    assert hookcallers2 is not None
    assert len(hookcallers2) == 1
    hookcallers3 = pm.get_hookcallers(plugin3)
    assert hookcallers3 is not None
    assert len(hookcallers3) == 1
    assert hookcallers1 == hookcallers2 + hookcallers3

    assert pm.get_hookcallers(object()) is None


def test_add_hookspecs_nohooks(pm: PluginManager) -> None:
    class NoHooks:
        pass

    with pytest.raises(ValueError):
        pm.add_hookspecs(NoHooks)


def test_load_setuptools_instantiation(monkeypatch, pm: PluginManager) -> None:
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

    monkeypatch.setattr(importlib.metadata, "distributions", my_distributions)
    num = pm.load_setuptools_entrypoints("hello")
    assert num == 1
    plugin = pm.get_plugin("myname")
    assert plugin is not None
    assert plugin.x == 42
    ret = pm.list_plugin_distinfo()
    # poor man's `assert ret == [(plugin, mock.ANY)]`
    assert len(ret) == 1
    assert len(ret[0]) == 2
    assert ret[0][0] == plugin
    assert ret[0][1]._dist == dist  # type: ignore[comparison-overlap]
    num = pm.load_setuptools_entrypoints("hello")
    assert num == 0  # no plugin loaded by this call


def test_add_tracefuncs(he_pm: PluginManager) -> None:
    out: List[Any] = []

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


def test_hook_tracing(he_pm: PluginManager) -> None:
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
    out: List[Any] = []
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


@pytest.mark.parametrize("historic", [False, True])
def test_register_while_calling(
    pm: PluginManager,
    historic: bool,
) -> None:
    """Test that registering an impl of a hook while it is being called does
    not affect the call itself, only later calls.

    For historic hooks however, the hook is called immediately on registration.

    Regression test for #438.
    """
    hookspec = HookspecMarker("example")

    class Hooks:
        @hookspec(historic=historic)
        def configure(self) -> int:
            raise NotImplementedError()

    class Plugin1:
        @hookimpl
        def configure(self) -> int:
            return 1

    class Plugin2:
        def __init__(self) -> None:
            self.already_registered = False

        @hookimpl
        def configure(self) -> int:
            if not self.already_registered:
                pm.register(Plugin4())
                pm.register(Plugin5())
                pm.register(Plugin6())
                self.already_registered = True
            return 2

    class Plugin3:
        @hookimpl
        def configure(self) -> int:
            return 3

    class Plugin4:
        @hookimpl(tryfirst=True)
        def configure(self) -> int:
            return 4

    class Plugin5:
        @hookimpl()
        def configure(self) -> int:
            return 5

    class Plugin6:
        @hookimpl(trylast=True)
        def configure(self) -> int:
            return 6

    pm.add_hookspecs(Hooks)
    pm.register(Plugin1())
    pm.register(Plugin2())
    pm.register(Plugin3())

    if not historic:
        result = pm.hook.configure()
        assert result == [3, 2, 1]
        result = pm.hook.configure()
        assert result == [4, 5, 3, 2, 1, 6]
    else:
        result = []
        pm.hook.configure.call_historic(result.append)
        assert result == [4, 5, 6, 3, 2, 1]
        result = []
        pm.hook.configure.call_historic(result.append)
        assert result == [4, 5, 3, 2, 1, 6]
