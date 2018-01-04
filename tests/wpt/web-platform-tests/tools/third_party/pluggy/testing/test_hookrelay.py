import pytest
from pluggy import PluginValidationError, HookimplMarker, HookspecMarker


hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def test_happypath(pm):
    class Api(object):
        @hookspec
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)
    hook = pm.hook
    assert hasattr(hook, 'hello')
    assert repr(hook.hello).find("hello") != -1

    class Plugin(object):
        @hookimpl
        def hello(self, arg):
            return arg + 1

    plugin = Plugin()
    pm.register(plugin)
    out = hook.hello(arg=3)
    assert out == [4]
    assert not hasattr(hook, 'world')
    pm.unregister(plugin)
    assert hook.hello(arg=3) == []


def test_argmismatch(pm):
    class Api(object):
        @hookspec
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)

    class Plugin(object):
        @hookimpl
        def hello(self, argwrong):
            pass

    with pytest.raises(PluginValidationError) as exc:
        pm.register(Plugin())

    assert "argwrong" in str(exc.value)


def test_only_kwargs(pm):
    class Api(object):
        @hookspec
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)
    with pytest.raises(TypeError) as exc:
        pm.hook.hello(3)

    comprehensible = "hook calling supports only keyword arguments"
    assert comprehensible in str(exc.value)


def test_call_order(pm):
    class Api(object):
        @hookspec
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)

    class Plugin1(object):
        @hookimpl
        def hello(self, arg):
            return 1

    class Plugin2(object):
        @hookimpl
        def hello(self, arg):
            return 2

    class Plugin3(object):
        @hookimpl
        def hello(self, arg):
            return 3

    class Plugin4(object):
        @hookimpl(hookwrapper=True)
        def hello(self, arg):
            assert arg == 0
            outcome = yield
            assert outcome.get_result() == [3, 2, 1]

    pm.register(Plugin1())
    pm.register(Plugin2())
    pm.register(Plugin3())
    pm.register(Plugin4())  # hookwrapper should get same list result
    res = pm.hook.hello(arg=0)
    assert res == [3, 2, 1]


def test_firstresult_definition(pm):
    class Api(object):
        @hookspec(firstresult=True)
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)

    class Plugin1(object):
        @hookimpl
        def hello(self, arg):
            return arg + 1

    class Plugin2(object):
        @hookimpl
        def hello(self, arg):
            return arg - 1

    class Plugin3(object):
        @hookimpl
        def hello(self, arg):
            return None

    class Plugin4(object):
        @hookimpl(hookwrapper=True)
        def hello(self, arg):
            assert arg == 3
            outcome = yield
            assert outcome.get_result() == 2

    pm.register(Plugin1())  # discarded - not the last registered plugin
    pm.register(Plugin2())  # used as result
    pm.register(Plugin3())  # None result is ignored
    pm.register(Plugin4())  # hookwrapper should get same non-list result
    res = pm.hook.hello(arg=3)
    assert res == 2


def test_firstresult_force_result(pm):
    """Verify forcing a result in a wrapper.
    """
    class Api(object):
        @hookspec(firstresult=True)
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)

    class Plugin1(object):
        @hookimpl
        def hello(self, arg):
            return arg + 1

    class Plugin2(object):
        @hookimpl(hookwrapper=True)
        def hello(self, arg):
            assert arg == 3
            outcome = yield
            assert outcome.get_result() == 4
            outcome.force_result(0)

    class Plugin3(object):
        @hookimpl
        def hello(self, arg):
            return None

    pm.register(Plugin1())
    pm.register(Plugin2())  # wrapper
    pm.register(Plugin3())  # ignored since returns None
    res = pm.hook.hello(arg=3)
    assert res == 0  # this result is forced and not a list


def test_firstresult_returns_none(pm):
    """If None results are returned by underlying implementations ensure
    the multi-call loop returns a None value.
    """
    class Api(object):
        @hookspec(firstresult=True)
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)

    class Plugin1(object):
        @hookimpl
        def hello(self, arg):
            return None

    pm.register(Plugin1())
    res = pm.hook.hello(arg=3)
    assert res is None


def test_firstresult_no_plugin(pm):
    """If no implementations/plugins have been registered for a firstresult
    hook the multi-call loop should return a None value.
    """
    class Api(object):
        @hookspec(firstresult=True)
        def hello(self, arg):
            "api hook 1"

    pm.add_hookspecs(Api)
    res = pm.hook.hello(arg=3)
    assert res is None
