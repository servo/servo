from pluggy._hooks import varnames
from pluggy._manager import _formatdef


def test_varnames():
    def f(x):
        i = 3  # noqa

    class A:
        def f(self, y):
            pass

    class B:
        def __call__(self, z):
            pass

    assert varnames(f) == (("x",), ())
    assert varnames(A().f) == (("y",), ())
    assert varnames(B()) == (("z",), ())


def test_varnames_default():
    def f(x, y=3):
        pass

    assert varnames(f) == (("x",), ("y",))


def test_varnames_class():
    class C:
        def __init__(self, x):
            pass

    class D:
        pass

    class E:
        def __init__(self, x):
            pass

    class F:
        pass

    assert varnames(C) == (("x",), ())
    assert varnames(D) == ((), ())
    assert varnames(E) == (("x",), ())
    assert varnames(F) == ((), ())


def test_varnames_keyword_only():
    def f1(x, *, y):
        pass

    def f2(x, *, y=3):
        pass

    def f3(x=1, *, y=3):
        pass

    assert varnames(f1) == (("x",), ())
    assert varnames(f2) == (("x",), ())
    assert varnames(f3) == ((), ("x",))


def test_formatdef():
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
