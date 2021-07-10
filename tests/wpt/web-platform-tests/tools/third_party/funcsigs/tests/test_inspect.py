# Copyright 2001-2013 Python Software Foundation; All Rights Reserved
from __future__ import absolute_import, division, print_function
import collections
import functools
import sys

import unittest2 as unittest

import funcsigs as inspect


class TestSignatureObject(unittest.TestCase):
    @staticmethod
    def signature(func):
        sig = inspect.signature(func)
        return (tuple((param.name,
                       (Ellipsis if param.default is param.empty else param.default),
                       (Ellipsis if param.annotation is param.empty
                                                        else param.annotation),
                       str(param.kind).lower())
                                    for param in sig.parameters.values()),
                (Ellipsis if sig.return_annotation is sig.empty
                                            else sig.return_annotation))

    if sys.version_info[0] > 2:
        exec("""
def test_signature_object(self):
    S = inspect.Signature
    P = inspect.Parameter

    self.assertEqual(str(S()), '()')

    def test(po, pk, *args, ko, **kwargs):
        pass
    sig = inspect.signature(test)
    po = sig.parameters['po'].replace(kind=P.POSITIONAL_ONLY)
    pk = sig.parameters['pk']
    args = sig.parameters['args']
    ko = sig.parameters['ko']
    kwargs = sig.parameters['kwargs']

    S((po, pk, args, ko, kwargs))

    with self.assertRaisesRegex(ValueError, 'wrong parameter order'):
        S((pk, po, args, ko, kwargs))

    with self.assertRaisesRegex(ValueError, 'wrong parameter order'):
        S((po, args, pk, ko, kwargs))

    with self.assertRaisesRegex(ValueError, 'wrong parameter order'):
        S((args, po, pk, ko, kwargs))

    with self.assertRaisesRegex(ValueError, 'wrong parameter order'):
        S((po, pk, args, kwargs, ko))

    kwargs2 = kwargs.replace(name='args')
    with self.assertRaisesRegex(ValueError, 'duplicate parameter name'):
        S((po, pk, args, kwargs2, ko))
""")

    def test_signature_immutability(self):
        def test(a):
            pass
        sig = inspect.signature(test)

        with self.assertRaises(AttributeError):
            sig.foo = 'bar'

        # Python2 does not have MappingProxyType class
        if sys.version_info[:2] < (3, 3):
            return

        with self.assertRaises(TypeError):
            sig.parameters['a'] = None

    def test_signature_on_noarg(self):
        def test():
            pass
        self.assertEqual(self.signature(test), ((), Ellipsis))

    if sys.version_info[0] > 2:
        exec("""
def test_signature_on_wargs(self):
    def test(a, b:'foo') -> 123:
        pass
    self.assertEqual(self.signature(test),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('b', Ellipsis, 'foo', "positional_or_keyword")),
                      123))
""")

    if sys.version_info[0] > 2:
        exec("""
def test_signature_on_wkwonly(self):
    def test(*, a:float, b:str) -> int:
        pass
    self.assertEqual(self.signature(test),
                     ((('a', Ellipsis, float, "keyword_only"),
                       ('b', Ellipsis, str, "keyword_only")),
                       int))
""")

    if sys.version_info[0] > 2:
        exec("""
def test_signature_on_complex_args(self):
    def test(a, b:'foo'=10, *args:'bar', spam:'baz', ham=123, **kwargs:int):
        pass
    self.assertEqual(self.signature(test),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('b', 10, 'foo', "positional_or_keyword"),
                       ('args', Ellipsis, 'bar', "var_positional"),
                       ('spam', Ellipsis, 'baz', "keyword_only"),
                       ('ham', 123, Ellipsis, "keyword_only"),
                       ('kwargs', Ellipsis, int, "var_keyword")),
                      Ellipsis))
""")

    def test_signature_on_builtin_function(self):
        with self.assertRaisesRegex(ValueError, 'not supported by signature'):
            inspect.signature(type)
        with self.assertRaisesRegex(ValueError, 'not supported by signature'):
            # support for 'wrapper_descriptor'
            inspect.signature(type.__call__)
            if hasattr(sys, 'pypy_version_info'):
                raise ValueError('not supported by signature')
        with self.assertRaisesRegex(ValueError, 'not supported by signature'):
            # support for 'method-wrapper'
            inspect.signature(min.__call__)
            if hasattr(sys, 'pypy_version_info'):
                raise ValueError('not supported by signature')
        with self.assertRaisesRegex(ValueError,
                                     'no signature found for builtin function'):
            # support for 'method-wrapper'
            inspect.signature(min)

    def test_signature_on_non_function(self):
        with self.assertRaisesRegex(TypeError, 'is not a callable object'):
            inspect.signature(42)

        with self.assertRaisesRegex(TypeError, 'is not a Python function'):
            inspect.Signature.from_function(42)

    if sys.version_info[0] > 2:
        exec("""
def test_signature_on_method(self):
    class Test:
        def foo(self, arg1, arg2=1) -> int:
            pass

    meth = Test().foo

    self.assertEqual(self.signature(meth),
                     ((('arg1', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('arg2', 1, Ellipsis, "positional_or_keyword")),
                      int))
""")

    if sys.version_info[0] > 2:
        exec("""
def test_signature_on_classmethod(self):
    class Test:
        @classmethod
        def foo(cls, arg1, *, arg2=1):
            pass

    meth = Test().foo
    self.assertEqual(self.signature(meth),
                     ((('arg1', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('arg2', 1, Ellipsis, "keyword_only")),
                      Ellipsis))

    meth = Test.foo
    self.assertEqual(self.signature(meth),
                     ((('arg1', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('arg2', 1, Ellipsis, "keyword_only")),
                      Ellipsis))
""")

    if sys.version_info[0] > 2:
        exec("""
def test_signature_on_staticmethod(self):
    class Test:
        @staticmethod
        def foo(cls, *, arg):
            pass

    meth = Test().foo
    self.assertEqual(self.signature(meth),
                     ((('cls', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('arg', Ellipsis, Ellipsis, "keyword_only")),
                      Ellipsis))

    meth = Test.foo
    self.assertEqual(self.signature(meth),
                     ((('cls', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('arg', Ellipsis, Ellipsis, "keyword_only")),
                      Ellipsis))
""")

    if sys.version_info[0] > 2:
        exec("""
def test_signature_on_partial(self):
    from functools import partial

    def test():
        pass

    self.assertEqual(self.signature(partial(test)), ((), Ellipsis))

    with self.assertRaisesRegex(ValueError, "has incorrect arguments"):
        inspect.signature(partial(test, 1))

    with self.assertRaisesRegex(ValueError, "has incorrect arguments"):
        inspect.signature(partial(test, a=1))

    def test(a, b, *, c, d):
        pass

    self.assertEqual(self.signature(partial(test)),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('b', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('c', Ellipsis, Ellipsis, "keyword_only"),
                       ('d', Ellipsis, Ellipsis, "keyword_only")),
                      Ellipsis))

    self.assertEqual(self.signature(partial(test, 1)),
                     ((('b', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('c', Ellipsis, Ellipsis, "keyword_only"),
                       ('d', Ellipsis, Ellipsis, "keyword_only")),
                      Ellipsis))

    self.assertEqual(self.signature(partial(test, 1, c=2)),
                     ((('b', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('c', 2, Ellipsis, "keyword_only"),
                       ('d', Ellipsis, Ellipsis, "keyword_only")),
                      Ellipsis))

    self.assertEqual(self.signature(partial(test, b=1, c=2)),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('b', 1, Ellipsis, "positional_or_keyword"),
                       ('c', 2, Ellipsis, "keyword_only"),
                       ('d', Ellipsis, Ellipsis, "keyword_only")),
                      Ellipsis))

    self.assertEqual(self.signature(partial(test, 0, b=1, c=2)),
                     ((('b', 1, Ellipsis, "positional_or_keyword"),
                       ('c', 2, Ellipsis, "keyword_only"),
                       ('d', Ellipsis, Ellipsis, "keyword_only"),),
                      Ellipsis))

    def test(a, *args, b, **kwargs):
        pass

    self.assertEqual(self.signature(partial(test, 1)),
                     ((('args', Ellipsis, Ellipsis, "var_positional"),
                       ('b', Ellipsis, Ellipsis, "keyword_only"),
                       ('kwargs', Ellipsis, Ellipsis, "var_keyword")),
                      Ellipsis))

    self.assertEqual(self.signature(partial(test, 1, 2, 3)),
                     ((('args', Ellipsis, Ellipsis, "var_positional"),
                       ('b', Ellipsis, Ellipsis, "keyword_only"),
                       ('kwargs', Ellipsis, Ellipsis, "var_keyword")),
                      Ellipsis))


    self.assertEqual(self.signature(partial(test, 1, 2, 3, test=True)),
                     ((('args', Ellipsis, Ellipsis, "var_positional"),
                       ('b', Ellipsis, Ellipsis, "keyword_only"),
                       ('kwargs', Ellipsis, Ellipsis, "var_keyword")),
                      Ellipsis))

    self.assertEqual(self.signature(partial(test, 1, 2, 3, test=1, b=0)),
                     ((('args', Ellipsis, Ellipsis, "var_positional"),
                       ('b', 0, Ellipsis, "keyword_only"),
                       ('kwargs', Ellipsis, Ellipsis, "var_keyword")),
                      Ellipsis))

    self.assertEqual(self.signature(partial(test, b=0)),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('args', Ellipsis, Ellipsis, "var_positional"),
                       ('b', 0, Ellipsis, "keyword_only"),
                       ('kwargs', Ellipsis, Ellipsis, "var_keyword")),
                      Ellipsis))

    self.assertEqual(self.signature(partial(test, b=0, test=1)),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('args', Ellipsis, Ellipsis, "var_positional"),
                       ('b', 0, Ellipsis, "keyword_only"),
                       ('kwargs', Ellipsis, Ellipsis, "var_keyword")),
                      Ellipsis))

    def test(a, b, c:int) -> 42:
        pass

    sig = test.__signature__ = inspect.signature(test)

    self.assertEqual(self.signature(partial(partial(test, 1))),
                     ((('b', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('c', Ellipsis, int, "positional_or_keyword")),
                      42))

    self.assertEqual(self.signature(partial(partial(test, 1), 2)),
                     ((('c', Ellipsis, int, "positional_or_keyword"),),
                      42))

    psig = inspect.signature(partial(partial(test, 1), 2))

    def foo(a):
        return a
    _foo = partial(partial(foo, a=10), a=20)
    self.assertEqual(self.signature(_foo),
                     ((('a', 20, Ellipsis, "positional_or_keyword"),),
                      Ellipsis))
    # check that we don't have any side-effects in signature(),
    # and the partial object is still functioning
    self.assertEqual(_foo(), 20)

    def foo(a, b, c):
        return a, b, c
    _foo = partial(partial(foo, 1, b=20), b=30)
    self.assertEqual(self.signature(_foo),
                     ((('b', 30, Ellipsis, "positional_or_keyword"),
                       ('c', Ellipsis, Ellipsis, "positional_or_keyword")),
                      Ellipsis))
    self.assertEqual(_foo(c=10), (1, 30, 10))
    _foo = partial(_foo, 2) # now 'b' has two values -
                            # positional and keyword
    with self.assertRaisesRegex(ValueError, "has incorrect arguments"):
        inspect.signature(_foo)

    def foo(a, b, c, *, d):
        return a, b, c, d
    _foo = partial(partial(foo, d=20, c=20), b=10, d=30)
    self.assertEqual(self.signature(_foo),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('b', 10, Ellipsis, "positional_or_keyword"),
                       ('c', 20, Ellipsis, "positional_or_keyword"),
                       ('d', 30, Ellipsis, "keyword_only")),
                      Ellipsis))
    ba = inspect.signature(_foo).bind(a=200, b=11)
    self.assertEqual(_foo(*ba.args, **ba.kwargs), (200, 11, 20, 30))

    def foo(a=1, b=2, c=3):
        return a, b, c
    _foo = partial(foo, a=10, c=13)
    ba = inspect.signature(_foo).bind(11)
    self.assertEqual(_foo(*ba.args, **ba.kwargs), (11, 2, 13))
    ba = inspect.signature(_foo).bind(11, 12)
    self.assertEqual(_foo(*ba.args, **ba.kwargs), (11, 12, 13))
    ba = inspect.signature(_foo).bind(11, b=12)
    self.assertEqual(_foo(*ba.args, **ba.kwargs), (11, 12, 13))
    ba = inspect.signature(_foo).bind(b=12)
    self.assertEqual(_foo(*ba.args, **ba.kwargs), (10, 12, 13))
    _foo = partial(_foo, b=10)
    ba = inspect.signature(_foo).bind(12, 14)
    self.assertEqual(_foo(*ba.args, **ba.kwargs), (12, 14, 13))
""")

    if sys.version_info[0] > 2:
        exec("""
def test_signature_on_decorated(self):
    import functools

    def decorator(func):
        @functools.wraps(func)
        def wrapper(*args, **kwargs) -> int:
            return func(*args, **kwargs)
        return wrapper

    class Foo:
        @decorator
        def bar(self, a, b):
            pass

    self.assertEqual(self.signature(Foo.bar),
                     ((('self', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('b', Ellipsis, Ellipsis, "positional_or_keyword")),
                      Ellipsis))

    self.assertEqual(self.signature(Foo().bar),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('b', Ellipsis, Ellipsis, "positional_or_keyword")),
                      Ellipsis))

    # Test that we handle method wrappers correctly
    def decorator(func):
        @functools.wraps(func)
        def wrapper(*args, **kwargs) -> int:
            return func(42, *args, **kwargs)
        sig = inspect.signature(func)
        new_params = tuple(sig.parameters.values())[1:]
        wrapper.__signature__ = sig.replace(parameters=new_params)
        return wrapper

    class Foo:
        @decorator
        def __call__(self, a, b):
            pass

    self.assertEqual(self.signature(Foo.__call__),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('b', Ellipsis, Ellipsis, "positional_or_keyword")),
                      Ellipsis))

    self.assertEqual(self.signature(Foo().__call__),
                     ((('b', Ellipsis, Ellipsis, "positional_or_keyword"),),
                      Ellipsis))
""")

    if sys.version_info[0] > 2:
        exec("""
def test_signature_on_class(self):
    class C:
        def __init__(self, a):
            pass

    self.assertEqual(self.signature(C),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),),
                      Ellipsis))

    class CM(type):
        def __call__(cls, a):
            pass
    class C(metaclass=CM):
        def __init__(self, b):
            pass

    self.assertEqual(self.signature(C),
                     ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),),
                      Ellipsis))

    class CM(type):
        def __new__(mcls, name, bases, dct, *, foo=1):
            return super().__new__(mcls, name, bases, dct)
    class C(metaclass=CM):
        def __init__(self, b):
            pass

    self.assertEqual(self.signature(C),
                     ((('b', Ellipsis, Ellipsis, "positional_or_keyword"),),
                      Ellipsis))

    self.assertEqual(self.signature(CM),
                     ((('name', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('bases', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('dct', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('foo', 1, Ellipsis, "keyword_only")),
                      Ellipsis))

    class CMM(type):
        def __new__(mcls, name, bases, dct, *, foo=1):
            return super().__new__(mcls, name, bases, dct)
        def __call__(cls, nm, bs, dt):
            return type(nm, bs, dt)
    class CM(type, metaclass=CMM):
        def __new__(mcls, name, bases, dct, *, bar=2):
            return super().__new__(mcls, name, bases, dct)
    class C(metaclass=CM):
        def __init__(self, b):
            pass

    self.assertEqual(self.signature(CMM),
                     ((('name', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('bases', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('dct', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('foo', 1, Ellipsis, "keyword_only")),
                      Ellipsis))

    self.assertEqual(self.signature(CM),
                     ((('nm', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('bs', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('dt', Ellipsis, Ellipsis, "positional_or_keyword")),
                      Ellipsis))

    self.assertEqual(self.signature(C),
                     ((('b', Ellipsis, Ellipsis, "positional_or_keyword"),),
                      Ellipsis))

    class CM(type):
        def __init__(cls, name, bases, dct, *, bar=2):
            return super().__init__(name, bases, dct)
    class C(metaclass=CM):
        def __init__(self, b):
            pass

    self.assertEqual(self.signature(CM),
                     ((('name', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('bases', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('dct', Ellipsis, Ellipsis, "positional_or_keyword"),
                       ('bar', 2, Ellipsis, "keyword_only")),
                      Ellipsis))
""")

    def test_signature_on_callable_objects(self):
        class Foo(object):
            def __call__(self, a):
                pass

        self.assertEqual(self.signature(Foo()),
                         ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),),
                          Ellipsis))

        class Spam(object):
            pass
        with self.assertRaisesRegex(TypeError, "is not a callable object"):
            inspect.signature(Spam())

        class Bar(Spam, Foo):
            pass

        self.assertEqual(self.signature(Bar()),
                         ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),),
                          Ellipsis))

        class ToFail(object):
            __call__ = type
        with self.assertRaisesRegex(ValueError, "not supported by signature"):
            inspect.signature(ToFail())

        if sys.version_info[0] < 3:
            return

        class Wrapped(object):
            pass
        Wrapped.__wrapped__ = lambda a: None
        self.assertEqual(self.signature(Wrapped),
                         ((('a', Ellipsis, Ellipsis, "positional_or_keyword"),),
                          Ellipsis))

    def test_signature_on_lambdas(self):
        self.assertEqual(self.signature((lambda a=10: a)),
                         ((('a', 10, Ellipsis, "positional_or_keyword"),),
                          Ellipsis))

    if sys.version_info[0] > 2:
        exec("""
def test_signature_equality(self):
    def foo(a, *, b:int) -> float: pass
    self.assertNotEqual(inspect.signature(foo), 42)

    def bar(a, *, b:int) -> float: pass
    self.assertEqual(inspect.signature(foo), inspect.signature(bar))

    def bar(a, *, b:int) -> int: pass
    self.assertNotEqual(inspect.signature(foo), inspect.signature(bar))

    def bar(a, *, b:int): pass
    self.assertNotEqual(inspect.signature(foo), inspect.signature(bar))

    def bar(a, *, b:int=42) -> float: pass
    self.assertNotEqual(inspect.signature(foo), inspect.signature(bar))

    def bar(a, *, c) -> float: pass
    self.assertNotEqual(inspect.signature(foo), inspect.signature(bar))

    def bar(a, b:int) -> float: pass
    self.assertNotEqual(inspect.signature(foo), inspect.signature(bar))
    def spam(b:int, a) -> float: pass
    self.assertNotEqual(inspect.signature(spam), inspect.signature(bar))

    def foo(*, a, b, c): pass
    def bar(*, c, b, a): pass
    self.assertEqual(inspect.signature(foo), inspect.signature(bar))

    def foo(*, a=1, b, c): pass
    def bar(*, c, b, a=1): pass
    self.assertEqual(inspect.signature(foo), inspect.signature(bar))

    def foo(pos, *, a=1, b, c): pass
    def bar(pos, *, c, b, a=1): pass
    self.assertEqual(inspect.signature(foo), inspect.signature(bar))

    def foo(pos, *, a, b, c): pass
    def bar(pos, *, c, b, a=1): pass
    self.assertNotEqual(inspect.signature(foo), inspect.signature(bar))

    def foo(pos, *args, a=42, b, c, **kwargs:int): pass
    def bar(pos, *args, c, b, a=42, **kwargs:int): pass
    self.assertEqual(inspect.signature(foo), inspect.signature(bar))
""")

    def test_signature_unhashable(self):
        def foo(a): pass
        sig = inspect.signature(foo)
        with self.assertRaisesRegex(TypeError, 'unhashable type'):
            hash(sig)


    if sys.version_info[0] > 2:
        exec("""
def test_signature_str(self):
    def foo(a:int=1, *, b, c=None, **kwargs) -> 42:
        pass
    self.assertEqual(str(inspect.signature(foo)),
                     '(a:int=1, *, b, c=None, **kwargs) -> 42')

    def foo(a:int=1, *args, b, c=None, **kwargs) -> 42:
        pass
    self.assertEqual(str(inspect.signature(foo)),
                     '(a:int=1, *args, b, c=None, **kwargs) -> 42')

    def foo():
        pass
    self.assertEqual(str(inspect.signature(foo)), '()')
""")

    if sys.version_info[0] > 2:
        exec("""
def test_signature_str_positional_only(self):
    P = inspect.Parameter

    def test(a_po, *, b, **kwargs):
        return a_po, kwargs

    sig = inspect.signature(test)
    new_params = list(sig.parameters.values())
    new_params[0] = new_params[0].replace(kind=P.POSITIONAL_ONLY)
    test.__signature__ = sig.replace(parameters=new_params)

    self.assertEqual(str(inspect.signature(test)),
                     '(<a_po>, *, b, **kwargs)')

    sig = inspect.signature(test)
    new_params = list(sig.parameters.values())
    new_params[0] = new_params[0].replace(name=None)
    test.__signature__ = sig.replace(parameters=new_params)
    self.assertEqual(str(inspect.signature(test)),
                     '(<0>, *, b, **kwargs)')
""")

    if sys.version_info[0] > 2:
        exec("""
def test_signature_replace_anno(self):
    def test() -> 42:
        pass

    sig = inspect.signature(test)
    sig = sig.replace(return_annotation=None)
    self.assertIs(sig.return_annotation, None)
    sig = sig.replace(return_annotation=sig.empty)
    self.assertIs(sig.return_annotation, sig.empty)
    sig = sig.replace(return_annotation=42)
    self.assertEqual(sig.return_annotation, 42)
    self.assertEqual(sig, inspect.signature(test))
""")


class TestParameterObject(unittest.TestCase):

    def test_signature_parameter_kinds(self):
        P = inspect.Parameter
        self.assertTrue(P.POSITIONAL_ONLY < P.POSITIONAL_OR_KEYWORD < \
                        P.VAR_POSITIONAL < P.KEYWORD_ONLY < P.VAR_KEYWORD)

        self.assertEqual(str(P.POSITIONAL_ONLY), 'POSITIONAL_ONLY')
        self.assertTrue('POSITIONAL_ONLY' in repr(P.POSITIONAL_ONLY))

    def test_signature_parameter_object(self):
        p = inspect.Parameter('foo', default=10,
                              kind=inspect.Parameter.POSITIONAL_ONLY)
        self.assertEqual(p.name, 'foo')
        self.assertEqual(p.default, 10)
        self.assertIs(p.annotation, p.empty)
        self.assertEqual(p.kind, inspect.Parameter.POSITIONAL_ONLY)

        with self.assertRaisesRegex(ValueError, 'invalid value'):
            inspect.Parameter('foo', default=10, kind='123')

        with self.assertRaisesRegex(ValueError, 'not a valid parameter name'):
            inspect.Parameter('1', kind=inspect.Parameter.VAR_KEYWORD)

        with self.assertRaisesRegex(ValueError,
                                     'non-positional-only parameter'):
            inspect.Parameter(None, kind=inspect.Parameter.VAR_KEYWORD)

        with self.assertRaisesRegex(ValueError, 'cannot have default values'):
            inspect.Parameter('a', default=42,
                              kind=inspect.Parameter.VAR_KEYWORD)

        with self.assertRaisesRegex(ValueError, 'cannot have default values'):
            inspect.Parameter('a', default=42,
                              kind=inspect.Parameter.VAR_POSITIONAL)

        p = inspect.Parameter('a', default=42,
                              kind=inspect.Parameter.POSITIONAL_OR_KEYWORD)
        with self.assertRaisesRegex(ValueError, 'cannot have default values'):
            p.replace(kind=inspect.Parameter.VAR_POSITIONAL)

        self.assertTrue(repr(p).startswith('<Parameter'))

    def test_signature_parameter_equality(self):
        P = inspect.Parameter
        p = P('foo', default=42, kind=inspect.Parameter.KEYWORD_ONLY)

        self.assertEqual(p, p)
        self.assertNotEqual(p, 42)

        self.assertEqual(p, P('foo', default=42,
                              kind=inspect.Parameter.KEYWORD_ONLY))

    def test_signature_parameter_unhashable(self):
        p = inspect.Parameter('foo', default=42,
                              kind=inspect.Parameter.KEYWORD_ONLY)

        with self.assertRaisesRegex(TypeError, 'unhashable type'):
            hash(p)

    def test_signature_parameter_replace(self):
        p = inspect.Parameter('foo', default=42,
                              kind=inspect.Parameter.KEYWORD_ONLY)

        self.assertIsNot(p, p.replace())
        self.assertEqual(p, p.replace())

        p2 = p.replace(annotation=1)
        self.assertEqual(p2.annotation, 1)
        p2 = p2.replace(annotation=p2.empty)
        self.assertEqual(p, p2)

        p2 = p2.replace(name='bar')
        self.assertEqual(p2.name, 'bar')
        self.assertNotEqual(p2, p)

        with self.assertRaisesRegex(ValueError, 'not a valid parameter name'):
            p2 = p2.replace(name=p2.empty)

        p2 = p2.replace(name='foo', default=None)
        self.assertIs(p2.default, None)
        self.assertNotEqual(p2, p)

        p2 = p2.replace(name='foo', default=p2.empty)
        self.assertIs(p2.default, p2.empty)


        p2 = p2.replace(default=42, kind=p2.POSITIONAL_OR_KEYWORD)
        self.assertEqual(p2.kind, p2.POSITIONAL_OR_KEYWORD)
        self.assertNotEqual(p2, p)

        with self.assertRaisesRegex(ValueError, 'invalid value for'):
            p2 = p2.replace(kind=p2.empty)

        p2 = p2.replace(kind=p2.KEYWORD_ONLY)
        self.assertEqual(p2, p)

    def test_signature_parameter_positional_only(self):
        p = inspect.Parameter(None, kind=inspect.Parameter.POSITIONAL_ONLY)
        self.assertEqual(str(p), '<>')

        p = p.replace(name='1')
        self.assertEqual(str(p), '<1>')

    def test_signature_parameter_immutability(self):
        p = inspect.Parameter(None, kind=inspect.Parameter.POSITIONAL_ONLY)

        with self.assertRaises(AttributeError):
            p.foo = 'bar'

        with self.assertRaises(AttributeError):
            p.kind = 123


class TestSignatureBind(unittest.TestCase):
    @staticmethod
    def call(func, *args, **kwargs):
        sig = inspect.signature(func)
        ba = sig.bind(*args, **kwargs)
        return func(*ba.args, **ba.kwargs)

    def test_signature_bind_empty(self):
        def test():
            return 42

        self.assertEqual(self.call(test), 42)
        with self.assertRaisesRegex(TypeError, 'too many positional arguments'):
            self.call(test, 1)
        with self.assertRaisesRegex(TypeError, 'too many positional arguments'):
            self.call(test, 1, spam=10)
        with self.assertRaisesRegex(TypeError, 'too many keyword arguments'):
            self.call(test, spam=1)

    def test_signature_bind_var(self):
        def test(*args, **kwargs):
            return args, kwargs

        self.assertEqual(self.call(test), ((), {}))
        self.assertEqual(self.call(test, 1), ((1,), {}))
        self.assertEqual(self.call(test, 1, 2), ((1, 2), {}))
        self.assertEqual(self.call(test, foo='bar'), ((), {'foo': 'bar'}))
        self.assertEqual(self.call(test, 1, foo='bar'), ((1,), {'foo': 'bar'}))
        self.assertEqual(self.call(test, args=10), ((), {'args': 10}))
        self.assertEqual(self.call(test, 1, 2, foo='bar'),
                         ((1, 2), {'foo': 'bar'}))

    def test_signature_bind_just_args(self):
        def test(a, b, c):
            return a, b, c

        self.assertEqual(self.call(test, 1, 2, 3), (1, 2, 3))

        with self.assertRaisesRegex(TypeError, 'too many positional arguments'):
            self.call(test, 1, 2, 3, 4)

        with self.assertRaisesRegex(TypeError, "'b' parameter lacking default"):
            self.call(test, 1)

        with self.assertRaisesRegex(TypeError, "'a' parameter lacking default"):
            self.call(test)

        def test(a, b, c=10):
            return a, b, c
        self.assertEqual(self.call(test, 1, 2, 3), (1, 2, 3))
        self.assertEqual(self.call(test, 1, 2), (1, 2, 10))

        def test(a=1, b=2, c=3):
            return a, b, c
        self.assertEqual(self.call(test, a=10, c=13), (10, 2, 13))
        self.assertEqual(self.call(test, a=10), (10, 2, 3))
        self.assertEqual(self.call(test, b=10), (1, 10, 3))

    def test_signature_bind_varargs_order(self):
        def test(*args):
            return args

        self.assertEqual(self.call(test), ())
        self.assertEqual(self.call(test, 1, 2, 3), (1, 2, 3))

    def test_signature_bind_args_and_varargs(self):
        def test(a, b, c=3, *args):
            return a, b, c, args

        self.assertEqual(self.call(test, 1, 2, 3, 4, 5), (1, 2, 3, (4, 5)))
        self.assertEqual(self.call(test, 1, 2), (1, 2, 3, ()))
        self.assertEqual(self.call(test, b=1, a=2), (2, 1, 3, ()))
        self.assertEqual(self.call(test, 1, b=2), (1, 2, 3, ()))

        with self.assertRaisesRegex(TypeError,
                                     "multiple values for argument 'c'"):
            self.call(test, 1, 2, 3, c=4)

    def test_signature_bind_just_kwargs(self):
        def test(**kwargs):
            return kwargs

        self.assertEqual(self.call(test), {})
        self.assertEqual(self.call(test, foo='bar', spam='ham'),
                         {'foo': 'bar', 'spam': 'ham'})

    def test_signature_bind_args_and_kwargs(self):
        def test(a, b, c=3, **kwargs):
            return a, b, c, kwargs

        self.assertEqual(self.call(test, 1, 2), (1, 2, 3, {}))
        self.assertEqual(self.call(test, 1, 2, foo='bar', spam='ham'),
                         (1, 2, 3, {'foo': 'bar', 'spam': 'ham'}))
        self.assertEqual(self.call(test, b=2, a=1, foo='bar', spam='ham'),
                         (1, 2, 3, {'foo': 'bar', 'spam': 'ham'}))
        self.assertEqual(self.call(test, a=1, b=2, foo='bar', spam='ham'),
                         (1, 2, 3, {'foo': 'bar', 'spam': 'ham'}))
        self.assertEqual(self.call(test, 1, b=2, foo='bar', spam='ham'),
                         (1, 2, 3, {'foo': 'bar', 'spam': 'ham'}))
        self.assertEqual(self.call(test, 1, b=2, c=4, foo='bar', spam='ham'),
                         (1, 2, 4, {'foo': 'bar', 'spam': 'ham'}))
        self.assertEqual(self.call(test, 1, 2, 4, foo='bar'),
                         (1, 2, 4, {'foo': 'bar'}))
        self.assertEqual(self.call(test, c=5, a=4, b=3),
                         (4, 3, 5, {}))

    if sys.version_info[0] > 2:
        exec("""
def test_signature_bind_kwonly(self):
    def test(*, foo):
        return foo
    with self.assertRaisesRegex(TypeError,
                                 'too many positional arguments'):
        self.call(test, 1)
    self.assertEqual(self.call(test, foo=1), 1)

    def test(a, *, foo=1, bar):
        return foo
    with self.assertRaisesRegex(TypeError,
                                 "'bar' parameter lacking default value"):
        self.call(test, 1)

    def test(foo, *, bar):
        return foo, bar
    self.assertEqual(self.call(test, 1, bar=2), (1, 2))
    self.assertEqual(self.call(test, bar=2, foo=1), (1, 2))

    with self.assertRaisesRegex(TypeError,
                                 'too many keyword arguments'):
        self.call(test, bar=2, foo=1, spam=10)

    with self.assertRaisesRegex(TypeError,
                                 'too many positional arguments'):
        self.call(test, 1, 2)

    with self.assertRaisesRegex(TypeError,
                                 'too many positional arguments'):
        self.call(test, 1, 2, bar=2)

    with self.assertRaisesRegex(TypeError,
                                 'too many keyword arguments'):
        self.call(test, 1, bar=2, spam='ham')

    with self.assertRaisesRegex(TypeError,
                                 "'bar' parameter lacking default value"):
        self.call(test, 1)

    def test(foo, *, bar, **bin):
        return foo, bar, bin
    self.assertEqual(self.call(test, 1, bar=2), (1, 2, {}))
    self.assertEqual(self.call(test, foo=1, bar=2), (1, 2, {}))
    self.assertEqual(self.call(test, 1, bar=2, spam='ham'),
                     (1, 2, {'spam': 'ham'}))
    self.assertEqual(self.call(test, spam='ham', foo=1, bar=2),
                     (1, 2, {'spam': 'ham'}))
    with self.assertRaisesRegex(TypeError,
                                 "'foo' parameter lacking default value"):
        self.call(test, spam='ham', bar=2)
    self.assertEqual(self.call(test, 1, bar=2, bin=1, spam=10),
                     (1, 2, {'bin': 1, 'spam': 10}))
""")
#
    if sys.version_info[0] > 2:
        exec("""
def test_signature_bind_arguments(self):
    def test(a, *args, b, z=100, **kwargs):
        pass
    sig = inspect.signature(test)
    ba = sig.bind(10, 20, b=30, c=40, args=50, kwargs=60)
    # we won't have 'z' argument in the bound arguments object, as we didn't
    # pass it to the 'bind'
    self.assertEqual(tuple(ba.arguments.items()),
                     (('a', 10), ('args', (20,)), ('b', 30),
                      ('kwargs', {'c': 40, 'args': 50, 'kwargs': 60})))
    self.assertEqual(ba.kwargs,
                     {'b': 30, 'c': 40, 'args': 50, 'kwargs': 60})
    self.assertEqual(ba.args, (10, 20))
""")
#
    if sys.version_info[0] > 2:
        exec("""
def test_signature_bind_positional_only(self):
    P = inspect.Parameter

    def test(a_po, b_po, c_po=3, foo=42, *, bar=50, **kwargs):
        return a_po, b_po, c_po, foo, bar, kwargs

    sig = inspect.signature(test)
    new_params = collections.OrderedDict(tuple(sig.parameters.items()))
    for name in ('a_po', 'b_po', 'c_po'):
        new_params[name] = new_params[name].replace(kind=P.POSITIONAL_ONLY)
    new_sig = sig.replace(parameters=new_params.values())
    test.__signature__ = new_sig

    self.assertEqual(self.call(test, 1, 2, 4, 5, bar=6),
                     (1, 2, 4, 5, 6, {}))

    with self.assertRaisesRegex(TypeError, "parameter is positional only"):
        self.call(test, 1, 2, c_po=4)

    with self.assertRaisesRegex(TypeError, "parameter is positional only"):
        self.call(test, a_po=1, b_po=2)
""")

    def test_bind_self(self):
        class F:
            def f(a, self):
                return a, self
        an_f = F()
        partial_f = functools.partial(F.f, an_f)
        ba = inspect.signature(partial_f).bind(self=10)
        self.assertEqual((an_f, 10), partial_f(*ba.args, **ba.kwargs))


class TestBoundArguments(unittest.TestCase):

    def test_signature_bound_arguments_unhashable(self):
        def foo(a): pass
        ba = inspect.signature(foo).bind(1)

        with self.assertRaisesRegex(TypeError, 'unhashable type'):
            hash(ba)

    def test_signature_bound_arguments_equality(self):
        def foo(a): pass
        ba = inspect.signature(foo).bind(1)
        self.assertEqual(ba, ba)

        ba2 = inspect.signature(foo).bind(1)
        self.assertEqual(ba, ba2)

        ba3 = inspect.signature(foo).bind(2)
        self.assertNotEqual(ba, ba3)
        ba3.arguments['a'] = 1
        self.assertEqual(ba, ba3)

        def bar(b): pass
        ba4 = inspect.signature(bar).bind(1)
        self.assertNotEqual(ba, ba4)
