from functools import wraps
from typing import Any
from typing import Callable
from typing import cast
from typing import TypeVar

from pluggy._hooks import varnames
from pluggy._manager import _formatdef


def test_varnames() -> None:
    def f(x) -> None:
        i = 3  # noqa

    class A:
        def f(self, y) -> None:
            pass

    class B:
        def __call__(self, z) -> None:
            pass

    assert varnames(f) == (("x",), ())
    assert varnames(A().f) == (("y",), ())
    assert varnames(B()) == (("z",), ())


def test_varnames_default() -> None:
    def f(x, y=3) -> None:
        pass

    assert varnames(f) == (("x",), ("y",))


def test_varnames_class() -> None:
    class C:
        def __init__(self, x) -> None:
            pass

    class D:
        pass

    class E:
        def __init__(self, x) -> None:
            pass

    class F:
        pass

    assert varnames(C) == (("x",), ())
    assert varnames(D) == ((), ())
    assert varnames(E) == (("x",), ())
    assert varnames(F) == ((), ())


def test_varnames_keyword_only() -> None:
    def f1(x, *, y) -> None:
        pass

    def f2(x, *, y=3) -> None:
        pass

    def f3(x=1, *, y=3) -> None:
        pass

    assert varnames(f1) == (("x",), ())
    assert varnames(f2) == (("x",), ())
    assert varnames(f3) == ((), ("x",))


def test_formatdef() -> None:
    def function1():
        pass

    assert _formatdef(function1) == "function1()"

    def function2(arg1):
        pass

    assert _formatdef(function2) == "function2(arg1)"

    def function3(arg1, arg2="qwe"):
        pass

    assert _formatdef(function3) == "function3(arg1, arg2='qwe')"

    def function4(arg1, *args, **kwargs):
        pass

    assert _formatdef(function4) == "function4(arg1, *args, **kwargs)"


def test_varnames_decorator() -> None:
    F = TypeVar("F", bound=Callable[..., Any])

    def my_decorator(func: F) -> F:
        @wraps(func)
        def wrapper(*args, **kwargs):
            return func(*args, **kwargs)

        return cast(F, wrapper)

    @my_decorator
    def example(a, b=123) -> None:
        pass

    class Example:
        @my_decorator
        def example_method(self, x, y=1) -> None:
            pass

    ex_inst = Example()

    assert varnames(example) == (("a",), ("b",))
    assert varnames(Example.example_method) == (("x",), ("y",))
    assert varnames(ex_inst.example_method) == (("x",), ("y",))
