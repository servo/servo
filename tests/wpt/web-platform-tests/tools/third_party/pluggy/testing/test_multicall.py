import pytest

from pluggy import _multicall, _legacymulticall, HookImpl, HookCallError
from pluggy.callers import _LegacyMultiCall
from pluggy import HookspecMarker, HookimplMarker


hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def test_uses_copy_of_methods():
    out = [lambda: 42]
    mc = _LegacyMultiCall(out, {})
    repr(mc)
    out[:] = []
    res = mc.execute()
    return res == 42


def MC(methods, kwargs, firstresult=False):
    caller = _multicall
    hookfuncs = []
    for method in methods:
        f = HookImpl(None, "<temp>", method, method.example_impl)
        hookfuncs.append(f)
        if '__multicall__' in f.argnames:
            caller = _legacymulticall
    return caller(hookfuncs, kwargs, firstresult=firstresult)


def test_call_passing():
    class P1(object):
        @hookimpl
        def m(self, __multicall__, x):
            assert len(__multicall__.results) == 1
            assert not __multicall__.hook_impls
            return 17

    class P2(object):
        @hookimpl
        def m(self, __multicall__, x):
            assert __multicall__.results == []
            assert __multicall__.hook_impls
            return 23

    p1 = P1()
    p2 = P2()
    reslist = MC([p1.m, p2.m], {"x": 23})
    assert len(reslist) == 2
    # ensure reversed order
    assert reslist == [23, 17]


def test_keyword_args():
    @hookimpl
    def f(x):
        return x + 1

    class A(object):
        @hookimpl
        def f(self, x, y):
            return x + y

    reslist = MC([f, A().f], dict(x=23, y=24))
    assert reslist == [24 + 23, 24]


def test_keyword_args_with_defaultargs():
    @hookimpl
    def f(x, z=1):
        return x + z
    reslist = MC([f], dict(x=23, y=24))
    assert reslist == [24]


def test_tags_call_error():
    @hookimpl
    def f(x):
        return x
    with pytest.raises(HookCallError):
        MC([f], {})


def test_call_subexecute():
    @hookimpl
    def m(__multicall__):
        subresult = __multicall__.execute()
        return subresult + 1

    @hookimpl
    def n():
        return 1

    res = MC([n, m], {}, firstresult=True)
    assert res == 2


def test_call_none_is_no_result():
    @hookimpl
    def m1():
        return 1

    @hookimpl
    def m2():
        return None

    res = MC([m1, m2], {}, firstresult=True)
    assert res == 1
    res = MC([m1, m2], {}, {})
    assert res == [1]


def test_hookwrapper():
    out = []

    @hookimpl(hookwrapper=True)
    def m1():
        out.append("m1 init")
        yield None
        out.append("m1 finish")

    @hookimpl
    def m2():
        out.append("m2")
        return 2

    res = MC([m2, m1], {})
    assert res == [2]
    assert out == ["m1 init", "m2", "m1 finish"]
    out[:] = []
    res = MC([m2, m1], {}, firstresult=True)
    assert res == 2
    assert out == ["m1 init", "m2", "m1 finish"]


def test_hookwrapper_order():
    out = []

    @hookimpl(hookwrapper=True)
    def m1():
        out.append("m1 init")
        yield 1
        out.append("m1 finish")

    @hookimpl(hookwrapper=True)
    def m2():
        out.append("m2 init")
        yield 2
        out.append("m2 finish")

    res = MC([m2, m1], {})
    assert res == []
    assert out == ["m1 init", "m2 init", "m2 finish", "m1 finish"]


def test_hookwrapper_not_yield():
    @hookimpl(hookwrapper=True)
    def m1():
        pass

    with pytest.raises(TypeError):
        MC([m1], {})


def test_hookwrapper_too_many_yield():
    @hookimpl(hookwrapper=True)
    def m1():
        yield 1
        yield 2

    with pytest.raises(RuntimeError) as ex:
        MC([m1], {})
    assert "m1" in str(ex.value)
    assert (__file__ + ':') in str(ex.value)


@pytest.mark.parametrize("exc", [ValueError, SystemExit])
def test_hookwrapper_exception(exc):
    out = []

    @hookimpl(hookwrapper=True)
    def m1():
        out.append("m1 init")
        yield None
        out.append("m1 finish")

    @hookimpl
    def m2():
        raise exc

    with pytest.raises(exc):
        MC([m2, m1], {})
    assert out == ["m1 init", "m1 finish"]
