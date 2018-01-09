import pytest
import types

from pluggy import (PluginValidationError,
                    HookCallError, HookimplMarker, HookspecMarker)


hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def test_plugin_double_register(pm):
    pm.register(42, name="abc")
    with pytest.raises(ValueError):
        pm.register(42, name="abc")
    with pytest.raises(ValueError):
        pm.register(42, name="def")


def test_pm(pm):
    class A(object):
        pass

    a1, a2 = A(), A()
    pm.register(a1)
    assert pm.is_registered(a1)
    pm.register(a2, "hello")
    assert pm.is_registered(a2)
    out = pm.get_plugins()
    assert a1 in out
    assert a2 in out
    assert pm.get_plugin('hello') == a2
    assert pm.unregister(a1) == a1
    assert not pm.is_registered(a1)

    out = pm.list_name_plugin()
    assert len(out) == 1
    assert out == [("hello", a2)]


def test_has_plugin(pm):
    class A(object):
        pass

    a1 = A()
    pm.register(a1, 'hello')
    assert pm.is_registered(a1)
    assert pm.has_plugin('hello')


def test_register_dynamic_attr(he_pm):
    class A(object):
        def __getattr__(self, name):
            if name[0] != "_":
                return 42
            raise AttributeError()

    a = A()
    he_pm.register(a)
    assert not he_pm.get_hookcallers(a)


def test_pm_name(pm):
    class A(object):
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
    class A(object):
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
    class hello(object):
        @hookimpl
        def he_method_notexists(self):
            pass

    he_pm.register(hello())
    with pytest.raises(PluginValidationError):
        he_pm.check_pending()


def test_register_mismatch_arg(he_pm):
    class hello(object):
        @hookimpl
        def he_method1(self, qlwkje):
            pass

    with pytest.raises(PluginValidationError):
        he_pm.register(hello())


def test_register(pm):
    class MyPlugin(object):
        pass
    my = MyPlugin()
    pm.register(my)
    assert my in pm.get_plugins()
    my2 = MyPlugin()
    pm.register(my2)
    assert set([my, my2]).issubset(pm.get_plugins())

    assert pm.is_registered(my)
    assert pm.is_registered(my2)
    pm.unregister(my)
    assert not pm.is_registered(my)
    assert my not in pm.get_plugins()


def test_register_unknown_hooks(pm):
    class Plugin1(object):
        @hookimpl
        def he_method1(self, arg):
            return arg + 1

    pname = pm.register(Plugin1())

    class Hooks(object):
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)
    # assert not pm._unverified_hooks
    assert pm.hook.he_method1(arg=1) == [2]
    assert len(pm.get_hookcallers(pm.get_plugin(pname))) == 1


def test_register_historic(pm):
    class Hooks(object):
        @hookspec(historic=True)
        def he_method1(self, arg):
            pass
    pm.add_hookspecs(Hooks)

    pm.hook.he_method1.call_historic(kwargs=dict(arg=1))
    out = []

    class Plugin(object):
        @hookimpl
        def he_method1(self, arg):
            out.append(arg)

    pm.register(Plugin())
    assert out == [1]

    class Plugin2(object):
        @hookimpl
        def he_method1(self, arg):
            out.append(arg * 10)

    pm.register(Plugin2())
    assert out == [1, 10]
    pm.hook.he_method1.call_historic(kwargs=dict(arg=12))
    assert out == [1, 10, 120, 12]


def test_with_result_memorized(pm):
    class Hooks(object):
        @hookspec(historic=True)
        def he_method1(self, arg):
            pass
    pm.add_hookspecs(Hooks)

    he_method1 = pm.hook.he_method1
    he_method1.call_historic(lambda res: out.append(res), dict(arg=1))
    out = []

    class Plugin(object):
        @hookimpl
        def he_method1(self, arg):
            return arg * 10

    pm.register(Plugin())
    assert out == [10]


def test_with_callbacks_immediately_executed(pm):
    class Hooks(object):
        @hookspec(historic=True)
        def he_method1(self, arg):
            pass
    pm.add_hookspecs(Hooks)

    class Plugin1(object):
        @hookimpl
        def he_method1(self, arg):
            return arg * 10

    class Plugin2(object):
        @hookimpl
        def he_method1(self, arg):
            return arg * 20

    class Plugin3(object):
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
    class Hooks(object):
        @hookspec(historic=True)
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    out = []

    class Plugin(object):
        @hookimpl(hookwrapper=True)
        def he_method1(self, arg):
            out.append(arg)

    with pytest.raises(PluginValidationError):
        pm.register(Plugin())


def test_call_extra(pm):
    class Hooks(object):
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    def he_method1(arg):
        return arg * 10

    out = pm.hook.he_method1.call_extra([he_method1], dict(arg=1))
    assert out == [10]


def test_call_with_too_few_args(pm):
    class Hooks(object):
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    class Plugin1(object):
        @hookimpl
        def he_method1(self, arg):
            0 / 0
    pm.register(Plugin1())
    with pytest.raises(HookCallError):
        with pytest.warns(UserWarning):
            pm.hook.he_method1()


def test_subset_hook_caller(pm):
    class Hooks(object):
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)

    out = []

    class Plugin1(object):
        @hookimpl
        def he_method1(self, arg):
            out.append(arg)

    class Plugin2(object):
        @hookimpl
        def he_method1(self, arg):
            out.append(arg * 10)

    class PluginNo(object):
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


def test_multicall_deprecated(pm):
    class P1(object):
        @hookimpl
        def m(self, __multicall__, x):
            pass

    pytest.deprecated_call(pm.register, P1())


def test_add_hookspecs_nohooks(pm):
    with pytest.raises(ValueError):
        pm.add_hookspecs(10)


def test_reject_prefixed_module(pm):
    """Verify that a module type attribute that contains the project
    prefix in its name (in this case `'example_*'` isn't collected
    when registering a module which imports it.
    """
    pm._implprefix = 'example'
    conftest = types.ModuleType("conftest")
    src = ("""
def example_hook():
    pass
""")
    exec(src, conftest.__dict__)
    conftest.example_blah = types.ModuleType("example_blah")
    name = pm.register(conftest)
    assert name == 'conftest'
    assert getattr(pm.hook, 'example_blah', None) is None
    assert getattr(pm.hook, 'example_hook', None)  # conftest.example_hook should be collected
    assert pm.parse_hookimpl_opts(conftest, 'example_blah') is None
    assert pm.parse_hookimpl_opts(conftest, 'example_hook') == {}
