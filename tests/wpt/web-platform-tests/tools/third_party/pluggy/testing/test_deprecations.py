"""
Deprecation warnings testing roundup.
"""
import pytest
from pluggy.callers import _Result
from pluggy import PluginManager, HookimplMarker, HookspecMarker

hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def test_result_deprecated():
    r = _Result(10, None)
    with pytest.deprecated_call():
        assert r.result == 10


def test_implprefix_deprecated():
    with pytest.deprecated_call():
        pm = PluginManager("blah", implprefix="blah_")

    class Plugin:
        def blah_myhook(self, arg1):
            return arg1

    with pytest.deprecated_call():
        pm.register(Plugin())


def test_callhistoric_proc_deprecated(pm):
    """``proc`` kwarg to `PluginMananger.call_historic()` is now officially
    deprecated.
    """

    class P1(object):
        @hookspec(historic=True)
        @hookimpl
        def m(self, x):
            pass

    p1 = P1()
    pm.add_hookspecs(p1)
    pm.register(p1)
    with pytest.deprecated_call():
        pm.hook.m.call_historic(kwargs=dict(x=10), proc=lambda res: res)


def test_multicall_deprecated(pm):
    class P1(object):
        @hookimpl
        def m(self, __multicall__, x):
            pass

    pytest.deprecated_call(pm.register, P1())
