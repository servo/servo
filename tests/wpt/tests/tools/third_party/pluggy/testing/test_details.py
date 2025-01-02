import pytest

from pluggy import HookimplMarker
from pluggy import HookspecMarker
from pluggy import PluginManager


hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def test_parse_hookimpl_override() -> None:
    class MyPluginManager(PluginManager):
        def parse_hookimpl_opts(self, module_or_class, name):
            opts = PluginManager.parse_hookimpl_opts(self, module_or_class, name)
            if opts is None:
                if name.startswith("x1"):
                    opts = {}  # type: ignore[assignment]
            return opts

    class Plugin:
        def x1meth(self):
            pass

        @hookimpl(hookwrapper=True, tryfirst=True)
        def x1meth2(self):
            yield  # pragma: no cover

        @hookimpl(wrapper=True, trylast=True)
        def x1meth3(self):
            return (yield)  # pragma: no cover

    class Spec:
        @hookspec
        def x1meth(self):
            pass

        @hookspec
        def x1meth2(self):
            pass

        @hookspec
        def x1meth3(self):
            pass

    pm = MyPluginManager(hookspec.project_name)
    pm.register(Plugin())
    pm.add_hookspecs(Spec)

    hookimpls = pm.hook.x1meth.get_hookimpls()
    assert len(hookimpls) == 1
    assert not hookimpls[0].hookwrapper
    assert not hookimpls[0].wrapper
    assert not hookimpls[0].tryfirst
    assert not hookimpls[0].trylast
    assert not hookimpls[0].optionalhook

    hookimpls = pm.hook.x1meth2.get_hookimpls()
    assert len(hookimpls) == 1
    assert hookimpls[0].hookwrapper
    assert not hookimpls[0].wrapper
    assert hookimpls[0].tryfirst

    hookimpls = pm.hook.x1meth3.get_hookimpls()
    assert len(hookimpls) == 1
    assert not hookimpls[0].hookwrapper
    assert hookimpls[0].wrapper
    assert not hookimpls[0].tryfirst
    assert hookimpls[0].trylast


def test_warn_when_deprecated_specified(recwarn) -> None:
    warning = DeprecationWarning("foo is deprecated")

    class Spec:
        @hookspec(warn_on_impl=warning)
        def foo(self):
            pass

    class Plugin:
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


def test_warn_when_deprecated_args_specified(recwarn) -> None:
    warning1 = DeprecationWarning("old1 is deprecated")
    warning2 = DeprecationWarning("old2 is deprecated")

    class Spec:
        @hookspec(
            warn_on_impl_args={
                "old1": warning1,
                "old2": warning2,
            },
        )
        def foo(self, old1, new, old2):
            raise NotImplementedError()

    class Plugin:
        @hookimpl
        def foo(self, old2, old1, new):
            raise NotImplementedError()

    pm = PluginManager(hookspec.project_name)
    pm.add_hookspecs(Spec)

    with pytest.warns(DeprecationWarning) as records:
        pm.register(Plugin())
    (record1, record2) = records
    assert record1.message is warning2
    assert record1.filename == Plugin.foo.__code__.co_filename
    assert record1.lineno == Plugin.foo.__code__.co_firstlineno
    assert record2.message is warning1
    assert record2.filename == Plugin.foo.__code__.co_filename
    assert record2.lineno == Plugin.foo.__code__.co_firstlineno


def test_plugin_getattr_raises_errors() -> None:
    """Pluggy must be able to handle plugins which raise weird exceptions
    when getattr() gets called (#11).
    """

    class DontTouchMe:
        def __getattr__(self, x):
            raise Exception("can't touch me")

    class Module:
        pass

    module = Module()
    module.x = DontTouchMe()  # type: ignore[attr-defined]

    pm = PluginManager(hookspec.project_name)
    # register() would raise an error
    pm.register(module, "donttouch")
    assert pm.get_plugin("donttouch") is module


def test_not_all_arguments_are_provided_issues_a_warning(pm: PluginManager) -> None:
    """Calling a hook without providing all arguments specified in
    the hook spec issues a warning."""

    class Spec:
        @hookspec
        def hello(self, arg1, arg2):
            pass

        @hookspec(historic=True)
        def herstory(self, arg1, arg2):
            pass

    pm.add_hookspecs(Spec)

    with pytest.warns(UserWarning, match=r"'arg1', 'arg2'.*cannot be found.*$"):
        pm.hook.hello()
    with pytest.warns(UserWarning, match=r"'arg2'.*cannot be found.*$"):
        pm.hook.hello(arg1=1)
    with pytest.warns(UserWarning, match=r"'arg1'.*cannot be found.*$"):
        pm.hook.hello(arg2=2)

    with pytest.warns(UserWarning, match=r"'arg1', 'arg2'.*cannot be found.*$"):
        pm.hook.hello.call_extra([], kwargs=dict())

    with pytest.warns(UserWarning, match=r"'arg1', 'arg2'.*cannot be found.*$"):
        pm.hook.herstory.call_historic(kwargs=dict())


def test_repr() -> None:
    class Plugin:
        @hookimpl
        def myhook(self):
            raise NotImplementedError()

    pm = PluginManager(hookspec.project_name)

    plugin = Plugin()
    pname = pm.register(plugin)
    assert repr(pm.hook.myhook.get_hookimpls()[0]) == (
        f"<HookImpl plugin_name={pname!r}, plugin={plugin!r}>"
    )
