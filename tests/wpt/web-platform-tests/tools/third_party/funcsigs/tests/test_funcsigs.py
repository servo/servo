try:
    # python 2.x
    import unittest2 as unittest
except ImportError:
    # python 3.x
    import unittest

import doctest
import sys

import funcsigs as inspect


class TestFunctionSignatures(unittest.TestCase):

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

    def test_zero_arguments(self):
        def test():
            pass
        self.assertEqual(self.signature(test),
                ((), Ellipsis))

    def test_single_positional_argument(self):
        def test(a):
            pass
        self.assertEqual(self.signature(test),
                (((('a', Ellipsis, Ellipsis, "positional_or_keyword")),), Ellipsis))

    def test_single_keyword_argument(self):
        def test(a=None):
            pass
        self.assertEqual(self.signature(test),
                (((('a', None, Ellipsis, "positional_or_keyword")),), Ellipsis))

    def test_var_args(self):
        def test(*args):
            pass
        self.assertEqual(self.signature(test),
                (((('args', Ellipsis, Ellipsis, "var_positional")),), Ellipsis))

    def test_keywords_args(self):
        def test(**kwargs):
            pass
        self.assertEqual(self.signature(test),
                (((('kwargs', Ellipsis, Ellipsis, "var_keyword")),), Ellipsis))

    def test_multiple_arguments(self):
        def test(a, b=None, *args, **kwargs):
            pass
        self.assertEqual(self.signature(test), ((
            ('a', Ellipsis, Ellipsis, "positional_or_keyword"),
            ('b', None, Ellipsis, "positional_or_keyword"),
            ('args', Ellipsis, Ellipsis, "var_positional"),
            ('kwargs', Ellipsis, Ellipsis, "var_keyword"),
            ), Ellipsis))

    def test_has_version(self):
        self.assertTrue(inspect.__version__)

    def test_readme(self):
        doctest.testfile('../README.rst')

    def test_unbound_method(self):
        if sys.version_info < (3,):
            self_kind = "positional_only"
        else:
            self_kind = "positional_or_keyword"
        class Test(object):
            def method(self):
                pass
            def method_with_args(self, a):
                pass
        self.assertEqual(self.signature(Test.method),
                (((('self', Ellipsis, Ellipsis, self_kind)),), Ellipsis))
        self.assertEqual(self.signature(Test.method_with_args), ((
                ('self', Ellipsis, Ellipsis, self_kind),
                ('a', Ellipsis, Ellipsis, "positional_or_keyword"),
                ), Ellipsis))


if __name__ == "__main__":
    unittest.begin()
