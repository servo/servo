import pytest

from pluggy import HookimplMarker, HookspecMarker, PluginValidationError
from pluggy._hooks import HookImpl

hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


@pytest.fixture
def hc(pm):
    class Hooks:
        @hookspec
        def he_method1(self, arg):
            pass

    pm.add_hookspecs(Hooks)
    return pm.hook.he_method1


@pytest.fixture
def addmeth(hc):
    def addmeth(tryfirst=False, trylast=False, hookwrapper=False):
        def wrap(func):
            hookimpl(tryfirst=tryfirst, trylast=trylast, hookwrapper=hookwrapper)(func)
            hc._add_hookimpl(HookImpl(None, "<temp>", func, func.example_impl))
            return func

        return wrap

    return addmeth


def funcs(hookmethods):
    return [hookmethod.function for hookmethod in hookmethods]


def test_adding_nonwrappers(hc, addmeth):
    @addmeth()
    def he_method1():
        pass

    @addmeth()
    def he_method2():
        pass

    @addmeth()
    def he_method3():
        pass

    assert funcs(hc._nonwrappers) == [he_method1, he_method2, he_method3]


def test_adding_nonwrappers_trylast(hc, addmeth):
    @addmeth()
    def he_method1_middle():
        pass

    @addmeth(trylast=True)
    def he_method1():
        pass

    @addmeth()
    def he_method1_b():
        pass

    assert funcs(hc._nonwrappers) == [he_method1, he_method1_middle, he_method1_b]


def test_adding_nonwrappers_trylast3(hc, addmeth):
    @addmeth()
    def he_method1_a():
        pass

    @addmeth(trylast=True)
    def he_method1_b():
        pass

    @addmeth()
    def he_method1_c():
        pass

    @addmeth(trylast=True)
    def he_method1_d():
        pass

    assert funcs(hc._nonwrappers) == [
        he_method1_d,
        he_method1_b,
        he_method1_a,
        he_method1_c,
    ]


def test_adding_nonwrappers_trylast2(hc, addmeth):
    @addmeth()
    def he_method1_middle():
        pass

    @addmeth()
    def he_method1_b():
        pass

    @addmeth(trylast=True)
    def he_method1():
        pass

    assert funcs(hc._nonwrappers) == [he_method1, he_method1_middle, he_method1_b]


def test_adding_nonwrappers_tryfirst(hc, addmeth):
    @addmeth(tryfirst=True)
    def he_method1():
        pass

    @addmeth()
    def he_method1_middle():
        pass

    @addmeth()
    def he_method1_b():
        pass

    assert funcs(hc._nonwrappers) == [he_method1_middle, he_method1_b, he_method1]


def test_adding_wrappers_ordering(hc, addmeth):
    @addmeth(hookwrapper=True)
    def he_method1():
        pass

    @addmeth()
    def he_method1_middle():
        pass

    @addmeth(hookwrapper=True)
    def he_method3():
        pass

    assert funcs(hc._nonwrappers) == [he_method1_middle]
    assert funcs(hc._wrappers) == [he_method1, he_method3]


def test_adding_wrappers_ordering_tryfirst(hc, addmeth):
    @addmeth(hookwrapper=True, tryfirst=True)
    def he_method1():
        pass

    @addmeth(hookwrapper=True)
    def he_method2():
        pass

    assert hc._nonwrappers == []
    assert funcs(hc._wrappers) == [he_method2, he_method1]


def test_hookspec(pm):
    class HookSpec:
        @hookspec()
        def he_myhook1(arg1):
            pass

        @hookspec(firstresult=True)
        def he_myhook2(arg1):
            pass

        @hookspec(firstresult=False)
        def he_myhook3(arg1):
            pass

    pm.add_hookspecs(HookSpec)
    assert not pm.hook.he_myhook1.spec.opts["firstresult"]
    assert pm.hook.he_myhook2.spec.opts["firstresult"]
    assert not pm.hook.he_myhook3.spec.opts["firstresult"]


@pytest.mark.parametrize("name", ["hookwrapper", "optionalhook", "tryfirst", "trylast"])
@pytest.mark.parametrize("val", [True, False])
def test_hookimpl(name, val):
    @hookimpl(**{name: val})
    def he_myhook1(arg1):
        pass

    if val:
        assert he_myhook1.example_impl.get(name)
    else:
        assert not hasattr(he_myhook1, name)


def test_hookrelay_registry(pm):
    """Verify hook caller instances are registered by name onto the relay
    and can be likewise unregistered."""

    class Api:
        @hookspec
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)
    hook = pm.hook
    assert hasattr(hook, "hello")
    assert repr(hook.hello).find("hello") != -1

    class Plugin:
        @hookimpl
        def hello(self, arg):
            return arg + 1

    plugin = Plugin()
    pm.register(plugin)
    out = hook.hello(arg=3)
    assert out == [4]
    assert not hasattr(hook, "world")
    pm.unregister(plugin)
    assert hook.hello(arg=3) == []


def test_hookrelay_registration_by_specname(pm):
    """Verify hook caller instances may also be registered by specifying a
    specname option to the hookimpl"""

    class Api:
        @hookspec
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)
    hook = pm.hook
    assert hasattr(hook, "hello")
    assert len(pm.hook.hello.get_hookimpls()) == 0

    class Plugin:
        @hookimpl(specname="hello")
        def foo(self, arg):
            return arg + 1

    plugin = Plugin()
    pm.register(plugin)
    out = hook.hello(arg=3)
    assert out == [4]


def test_hookrelay_registration_by_specname_raises(pm):
    """Verify using specname still raises the types of errors during registration as it
    would have without using specname."""

    class Api:
        @hookspec
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)

    # make sure a bad signature still raises an error when using specname
    class Plugin:
        @hookimpl(specname="hello")
        def foo(self, arg, too, many, args):
            return arg + 1

    with pytest.raises(PluginValidationError):
        pm.register(Plugin())

    # make sure check_pending still fails if specname doesn't have a
    # corresponding spec.  EVEN if the function name matches one.
    class Plugin2:
        @hookimpl(specname="bar")
        def hello(self, arg):
            return arg + 1

    pm.register(Plugin2())
    with pytest.raises(PluginValidationError):
        pm.check_pending()
