from typing import Callable
from typing import List
from typing import Mapping
from typing import Sequence
from typing import Type
from typing import Union

import pytest

from pluggy import HookCallError
from pluggy import HookimplMarker
from pluggy import HookspecMarker
from pluggy._callers import _multicall
from pluggy._hooks import HookImpl


hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def MC(
    methods: Sequence[Callable[..., object]],
    kwargs: Mapping[str, object],
    firstresult: bool = False,
) -> Union[object, List[object]]:
    caller = _multicall
    hookfuncs = []
    for method in methods:
        f = HookImpl(None, "<temp>", method, method.example_impl)  # type: ignore[attr-defined]
        hookfuncs.append(f)
    return caller("foo", hookfuncs, kwargs, firstresult)


def test_keyword_args() -> None:
    @hookimpl
    def f(x):
        return x + 1

    class A:
        @hookimpl
        def f(self, x, y):
            return x + y

    reslist = MC([f, A().f], dict(x=23, y=24))
    assert reslist == [24 + 23, 24]


def test_keyword_args_with_defaultargs() -> None:
    @hookimpl
    def f(x, z=1):
        return x + z

    reslist = MC([f], dict(x=23, y=24))
    assert reslist == [24]


def test_tags_call_error() -> None:
    @hookimpl
    def f(x):
        return x

    with pytest.raises(HookCallError):
        MC([f], {})


def test_call_none_is_no_result() -> None:
    @hookimpl
    def m1():
        return 1

    @hookimpl
    def m2():
        return None

    res = MC([m1, m2], {}, firstresult=True)
    assert res == 1
    res = MC([m1, m2], {}, firstresult=False)
    assert res == [1]


def test_hookwrapper() -> None:
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


def test_hookwrapper_two_yields() -> None:
    @hookimpl(hookwrapper=True)
    def m():
        yield
        yield

    with pytest.raises(RuntimeError, match="has second yield"):
        MC([m], {})


def test_wrapper() -> None:
    out = []

    @hookimpl(wrapper=True)
    def m1():
        out.append("m1 init")
        result = yield
        out.append("m1 finish")
        return result * 2

    @hookimpl
    def m2():
        out.append("m2")
        return 2

    res = MC([m2, m1], {})
    assert res == [2, 2]
    assert out == ["m1 init", "m2", "m1 finish"]
    out[:] = []
    res = MC([m2, m1], {}, firstresult=True)
    assert res == 4
    assert out == ["m1 init", "m2", "m1 finish"]


def test_wrapper_two_yields() -> None:
    @hookimpl(wrapper=True)
    def m():
        yield
        yield

    with pytest.raises(RuntimeError, match="has second yield"):
        MC([m], {})


def test_hookwrapper_order() -> None:
    out = []

    @hookimpl(hookwrapper=True)
    def m1():
        out.append("m1 init")
        yield 1
        out.append("m1 finish")

    @hookimpl(wrapper=True)
    def m2():
        out.append("m2 init")
        result = yield 2
        out.append("m2 finish")
        return result

    @hookimpl(hookwrapper=True)
    def m3():
        out.append("m3 init")
        yield 3
        out.append("m3 finish")

    @hookimpl(hookwrapper=True)
    def m4():
        out.append("m4 init")
        yield 4
        out.append("m4 finish")

    res = MC([m4, m3, m2, m1], {})
    assert res == []
    assert out == [
        "m1 init",
        "m2 init",
        "m3 init",
        "m4 init",
        "m4 finish",
        "m3 finish",
        "m2 finish",
        "m1 finish",
    ]


def test_hookwrapper_not_yield() -> None:
    @hookimpl(hookwrapper=True)
    def m1():
        pass

    with pytest.raises(TypeError):
        MC([m1], {})


def test_hookwrapper_yield_not_executed() -> None:
    @hookimpl(hookwrapper=True)
    def m1():
        if False:
            yield  # type: ignore[unreachable]

    with pytest.raises(RuntimeError, match="did not yield"):
        MC([m1], {})


def test_hookwrapper_too_many_yield() -> None:
    @hookimpl(hookwrapper=True)
    def m1():
        yield 1
        yield 2

    with pytest.raises(RuntimeError) as ex:
        MC([m1], {})
    assert "m1" in str(ex.value)
    assert (__file__ + ":") in str(ex.value)


def test_wrapper_yield_not_executed() -> None:
    @hookimpl(wrapper=True)
    def m1():
        if False:
            yield  # type: ignore[unreachable]

    with pytest.raises(RuntimeError, match="did not yield"):
        MC([m1], {})


def test_wrapper_too_many_yield() -> None:
    out = []

    @hookimpl(wrapper=True)
    def m1():
        try:
            yield 1
            yield 2
        finally:
            out.append("cleanup")

    with pytest.raises(RuntimeError) as ex:
        try:
            MC([m1], {})
        finally:
            out.append("finally")
    assert "m1" in str(ex.value)
    assert (__file__ + ":") in str(ex.value)
    assert out == ["cleanup", "finally"]


@pytest.mark.parametrize("exc", [ValueError, SystemExit])
def test_hookwrapper_exception(exc: "Type[BaseException]") -> None:
    out = []

    @hookimpl(hookwrapper=True)
    def m1():
        out.append("m1 init")
        result = yield
        assert isinstance(result.exception, exc)
        assert result.excinfo[0] == exc
        out.append("m1 finish")

    @hookimpl
    def m2():
        raise exc

    with pytest.raises(exc):
        MC([m2, m1], {})
    assert out == ["m1 init", "m1 finish"]


def test_hookwrapper_force_exception() -> None:
    out = []

    @hookimpl(hookwrapper=True)
    def m1():
        out.append("m1 init")
        result = yield
        try:
            result.get_result()
        except BaseException as exc:
            result.force_exception(exc)
        out.append("m1 finish")

    @hookimpl(hookwrapper=True)
    def m2():
        out.append("m2 init")
        result = yield
        try:
            result.get_result()
        except BaseException as exc:
            new_exc = OSError("m2")
            new_exc.__cause__ = exc
            result.force_exception(new_exc)
        out.append("m2 finish")

    @hookimpl(hookwrapper=True)
    def m3():
        out.append("m3 init")
        yield
        out.append("m3 finish")

    @hookimpl
    def m4():
        raise ValueError("m4")

    with pytest.raises(OSError, match="m2") as excinfo:
        MC([m4, m3, m2, m1], {})
    assert out == [
        "m1 init",
        "m2 init",
        "m3 init",
        "m3 finish",
        "m2 finish",
        "m1 finish",
    ]
    assert excinfo.value.__cause__ is not None
    assert str(excinfo.value.__cause__) == "m4"


@pytest.mark.parametrize("exc", [ValueError, SystemExit])
def test_wrapper_exception(exc: "Type[BaseException]") -> None:
    out = []

    @hookimpl(wrapper=True)
    def m1():
        out.append("m1 init")
        try:
            result = yield
        except BaseException as e:
            assert isinstance(e, exc)
            raise
        finally:
            out.append("m1 finish")
        return result

    @hookimpl
    def m2():
        out.append("m2 init")
        raise exc

    with pytest.raises(exc):
        MC([m2, m1], {})
    assert out == ["m1 init", "m2 init", "m1 finish"]


def test_wrapper_exception_chaining() -> None:
    @hookimpl
    def m1():
        raise Exception("m1")

    @hookimpl(wrapper=True)
    def m2():
        try:
            yield
        except Exception:
            raise Exception("m2")

    @hookimpl(wrapper=True)
    def m3():
        yield
        return 10

    @hookimpl(wrapper=True)
    def m4():
        try:
            yield
        except Exception as e:
            raise Exception("m4") from e

    with pytest.raises(Exception) as excinfo:
        MC([m1, m2, m3, m4], {})
    assert str(excinfo.value) == "m4"
    assert excinfo.value.__cause__ is not None
    assert str(excinfo.value.__cause__) == "m2"
    assert excinfo.value.__cause__.__context__ is not None
    assert str(excinfo.value.__cause__.__context__) == "m1"


def test_unwind_inner_wrapper_teardown_exc() -> None:
    out = []

    @hookimpl(wrapper=True)
    def m1():
        out.append("m1 init")
        try:
            yield
            out.append("m1 unreachable")
        except BaseException:
            out.append("m1 teardown")
            raise
        finally:
            out.append("m1 cleanup")

    @hookimpl(wrapper=True)
    def m2():
        out.append("m2 init")
        yield
        out.append("m2 raise")
        raise ValueError()

    with pytest.raises(ValueError):
        try:
            MC([m2, m1], {})
        finally:
            out.append("finally")

    assert out == [
        "m1 init",
        "m2 init",
        "m2 raise",
        "m1 teardown",
        "m1 cleanup",
        "finally",
    ]


def test_suppress_inner_wrapper_teardown_exc() -> None:
    out = []

    @hookimpl(wrapper=True)
    def m1():
        out.append("m1 init")
        result = yield
        out.append("m1 finish")
        return result

    @hookimpl(wrapper=True)
    def m2():
        out.append("m2 init")
        try:
            yield
            out.append("m2 unreachable")
        except ValueError:
            out.append("m2 suppress")
            return 22

    @hookimpl(wrapper=True)
    def m3():
        out.append("m3 init")
        yield
        out.append("m3 raise")
        raise ValueError()

    assert MC([m3, m2, m1], {}) == 22
    assert out == [
        "m1 init",
        "m2 init",
        "m3 init",
        "m3 raise",
        "m2 suppress",
        "m1 finish",
    ]
