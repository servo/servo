import warnings
import pytest
from pluggy import PluginManager, HookimplMarker, HookspecMarker

hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def test_parse_hookimpl_override():
    class MyPluginManager(PluginManager):
        def parse_hookimpl_opts(self, module_or_class, name):
            opts = PluginManager.parse_hookimpl_opts(self, module_or_class, name)
            if opts is None:
                if name.startswith("x1"):
                    opts = {}
            return opts

    class Plugin(object):
        def x1meth(self):
            pass

        @hookimpl(hookwrapper=True, tryfirst=True)
        def x1meth2(self):
            pass

    class Spec(object):
        @hookspec
        def x1meth(self):
            pass

        @hookspec
        def x1meth2(self):
            pass

    pm = MyPluginManager(hookspec.project_name)
    pm.register(Plugin())
    pm.add_hookspecs(Spec)
    assert not pm.hook.x1meth._nonwrappers[0].hookwrapper
    assert not pm.hook.x1meth._nonwrappers[0].tryfirst
    assert not pm.hook.x1meth._nonwrappers[0].trylast
    assert not pm.hook.x1meth._nonwrappers[0].optionalhook

    assert pm.hook.x1meth2._wrappers[0].tryfirst
    assert pm.hook.x1meth2._wrappers[0].hookwrapper


def test_warn_when_deprecated_specified(recwarn):
    warning = DeprecationWarning("foo is deprecated")

    class Spec(object):
        @hookspec(warn_on_impl=warning)
        def foo(self):
            pass

    class Plugin(object):
        @hookimpl
        def foo(self):
            pass

    pm = PluginManager(hookspec.project_name)
    pm.add_hookspecs(Spec)

    with pytest.warns(DeprecationWarning) as records:
        pm.register(Plugin())
    (record,) = records
    assert record.message is warning
    assert record.filename == Plugin.foo.__code__.co_filename
    assert record.lineno == Plugin.foo.__code__.co_firstlineno


def test_plugin_getattr_raises_errors():
    """Pluggy must be able to handle plugins which raise weird exceptions
    when getattr() gets called (#11).
    """

    class DontTouchMe(object):
        def __getattr__(self, x):
            raise Exception("cant touch me")

    class Module(object):
        pass

    module = Module()
    module.x = DontTouchMe()

    pm = PluginManager(hookspec.project_name)
    # register() would raise an error
    pm.register(module, "donttouch")
    assert pm.get_plugin("donttouch") is module


def test_warning_on_call_vs_hookspec_arg_mismatch():
    """Verify that is a hook is called with less arguments then defined in the
    spec that a warning is emitted.
    """

    class Spec:
        @hookspec
        def myhook(self, arg1, arg2):
            pass

    class Plugin:
        @hookimpl
        def myhook(self, arg1):
            pass

    pm = PluginManager(hookspec.project_name)
    pm.register(Plugin())
    pm.add_hookspecs(Spec())

    with warnings.catch_warnings(record=True) as warns:
        warnings.simplefilter("always")

        # calling should trigger a warning
        pm.hook.myhook(arg1=1)

        assert len(warns) == 1
        warning = warns[-1]
        assert issubclass(warning.category, Warning)
        assert "Argument(s) ('arg2',)" in str(warning.message)


def test_repr():
    class Plugin:
        @hookimpl
        def myhook():
            raise NotImplementedError()

    pm = PluginManager(hookspec.project_name)

    plugin = Plugin()
    pname = pm.register(plugin)
    assert repr(pm.hook.myhook._nonwrappers[0]) == (
        "<HookImpl plugin_name=%r, plugin=%r>" % (pname, plugin)
    )
