from pluggy import _formatdef, varnames


def test_varnames():
    def f(x):
        i = 3  # noqa

    class A(object):
        def f(self, y):
            pass

    class B(object):
        def __call__(self, z):
            pass

    assert varnames(f) == (("x",), ())
    assert varnames(A().f) == (('y',), ())
    assert varnames(B()) == (('z',), ())


def test_varnames_default():
    def f(x, y=3):
        pass

    assert varnames(f) == (("x",), ("y",))


def test_varnames_class():
    class C(object):
        def __init__(self, x):
            pass

    class D(object):
        pass

    class E(object):
        def __init__(self, x):
            pass

    class F(object):
        pass

    assert varnames(C) == (("x",), ())
    assert varnames(D) == ((), ())
    assert varnames(E) == (("x",), ())
    assert varnames(F) == ((), ())


def test_formatdef():
    def function1():
        pass

    assert _formatdef(function1) == 'function1()'

    def function2(arg1):
        pass

    assert _formatdef(function2) == "function2(arg1)"

    def function3(arg1, arg2="qwe"):
        pass

    assert _formatdef(function3) == "function3(arg1, arg2='qwe')"

    def function4(arg1, *args, **kwargs):
        pass

    assert _formatdef(function4) == "function4(arg1, *args, **kwargs)"
