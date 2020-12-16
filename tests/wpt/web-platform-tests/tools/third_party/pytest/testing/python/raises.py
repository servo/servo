# -*- coding: utf-8 -*-
import sys

import six

import pytest
from _pytest.compat import dummy_context_manager
from _pytest.outcomes import Failed
from _pytest.warning_types import PytestDeprecationWarning


class TestRaises(object):
    def test_raises(self):
        source = "int('qwe')"
        with pytest.warns(PytestDeprecationWarning):
            excinfo = pytest.raises(ValueError, source)
        code = excinfo.traceback[-1].frame.code
        s = str(code.fullsource)
        assert s == source

    def test_raises_exec(self):
        with pytest.warns(PytestDeprecationWarning) as warninfo:
            pytest.raises(ValueError, "a,x = []")
        assert warninfo[0].filename == __file__

    def test_raises_exec_correct_filename(self):
        with pytest.warns(PytestDeprecationWarning):
            excinfo = pytest.raises(ValueError, 'int("s")')
            assert __file__ in excinfo.traceback[-1].path

    def test_raises_syntax_error(self):
        with pytest.warns(PytestDeprecationWarning) as warninfo:
            pytest.raises(SyntaxError, "qwe qwe qwe")
        assert warninfo[0].filename == __file__

    def test_raises_function(self):
        pytest.raises(ValueError, int, "hello")

    def test_raises_callable_no_exception(self):
        class A(object):
            def __call__(self):
                pass

        try:
            pytest.raises(ValueError, A())
        except pytest.raises.Exception:
            pass

    def test_raises_falsey_type_error(self):
        with pytest.raises(TypeError):
            with pytest.raises(AssertionError, match=0):
                raise AssertionError("ohai")

    def test_raises_repr_inflight(self):
        """Ensure repr() on an exception info inside a pytest.raises with block works (#4386)"""

        class E(Exception):
            pass

        with pytest.raises(E) as excinfo:
            # this test prints the inflight uninitialized object
            # using repr and str as well as pprint to demonstrate
            # it works
            print(str(excinfo))
            print(repr(excinfo))
            import pprint

            pprint.pprint(excinfo)
            raise E()

    def test_raises_as_contextmanager(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            import _pytest._code

            def test_simple():
                with pytest.raises(ZeroDivisionError) as excinfo:
                    assert isinstance(excinfo, _pytest._code.ExceptionInfo)
                    1/0
                print(excinfo)
                assert excinfo.type == ZeroDivisionError
                assert isinstance(excinfo.value, ZeroDivisionError)

            def test_noraise():
                with pytest.raises(pytest.raises.Exception):
                    with pytest.raises(ValueError):
                           int()

            def test_raise_wrong_exception_passes_by():
                with pytest.raises(ZeroDivisionError):
                    with pytest.raises(ValueError):
                           1/0
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*3 passed*"])

    def test_does_not_raise(self, testdir):
        testdir.makepyfile(
            """
            from contextlib import contextmanager
            import pytest

            @contextmanager
            def does_not_raise():
                yield

            @pytest.mark.parametrize('example_input,expectation', [
                (3, does_not_raise()),
                (2, does_not_raise()),
                (1, does_not_raise()),
                (0, pytest.raises(ZeroDivisionError)),
            ])
            def test_division(example_input, expectation):
                '''Test how much I know division.'''
                with expectation:
                    assert (6 / example_input) is not None
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*4 passed*"])

    def test_does_not_raise_does_raise(self, testdir):
        testdir.makepyfile(
            """
            from contextlib import contextmanager
            import pytest

            @contextmanager
            def does_not_raise():
                yield

            @pytest.mark.parametrize('example_input,expectation', [
                (0, does_not_raise()),
                (1, pytest.raises(ZeroDivisionError)),
            ])
            def test_division(example_input, expectation):
                '''Test how much I know division.'''
                with expectation:
                    assert (6 / example_input) is not None
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*2 failed*"])

    def test_noclass(self):
        with pytest.raises(TypeError):
            pytest.raises("wrong", lambda: None)

    def test_invalid_arguments_to_raises(self):
        with pytest.raises(TypeError, match="unknown"):
            with pytest.raises(TypeError, unknown="bogus"):
                raise ValueError()

    def test_tuple(self):
        with pytest.raises((KeyError, ValueError)):
            raise KeyError("oops")

    def test_no_raise_message(self):
        try:
            pytest.raises(ValueError, int, "0")
        except pytest.raises.Exception as e:
            assert e.msg == "DID NOT RAISE {}".format(repr(ValueError))
        else:
            assert False, "Expected pytest.raises.Exception"

        try:
            with pytest.raises(ValueError):
                pass
        except pytest.raises.Exception as e:
            assert e.msg == "DID NOT RAISE {}".format(repr(ValueError))
        else:
            assert False, "Expected pytest.raises.Exception"

    def test_custom_raise_message(self):
        message = "TEST_MESSAGE"
        try:
            with pytest.warns(PytestDeprecationWarning):
                with pytest.raises(ValueError, message=message):
                    pass
        except pytest.raises.Exception as e:
            assert e.msg == message
        else:
            assert False, "Expected pytest.raises.Exception"

    @pytest.mark.parametrize("method", ["function", "with"])
    def test_raises_cyclic_reference(self, method):
        """
        Ensure pytest.raises does not leave a reference cycle (#1965).
        """
        import gc

        class T(object):
            def __call__(self):
                raise ValueError

        t = T()
        if method == "function":
            pytest.raises(ValueError, t)
        else:
            with pytest.raises(ValueError):
                t()

        # ensure both forms of pytest.raises don't leave exceptions in sys.exc_info()
        assert sys.exc_info() == (None, None, None)

        del t

        # ensure the t instance is not stuck in a cyclic reference
        for o in gc.get_objects():
            assert type(o) is not T

    def test_raises_match(self):
        msg = r"with base \d+"
        with pytest.raises(ValueError, match=msg):
            int("asdf")

        msg = "with base 10"
        with pytest.raises(ValueError, match=msg):
            int("asdf")

        msg = "with base 16"
        expr = r"Pattern '{}' not found in \"invalid literal for int\(\) with base 10: 'asdf'\"".format(
            msg
        )
        with pytest.raises(AssertionError, match=expr):
            with pytest.raises(ValueError, match=msg):
                int("asdf", base=10)

    def test_raises_match_wrong_type(self):
        """Raising an exception with the wrong type and match= given.

        pytest should throw the unexpected exception - the pattern match is not
        really relevant if we got a different exception.
        """
        with pytest.raises(ValueError):
            with pytest.raises(IndexError, match="nomatch"):
                int("asdf")

    def test_raises_exception_looks_iterable(self):
        from six import add_metaclass

        class Meta(type(object)):
            def __getitem__(self, item):
                return 1 / 0

            def __len__(self):
                return 1

        @add_metaclass(Meta)
        class ClassLooksIterableException(Exception):
            pass

        with pytest.raises(
            Failed,
            match=r"DID NOT RAISE <class 'raises(\..*)*ClassLooksIterableException'>",
        ):
            pytest.raises(ClassLooksIterableException, lambda: None)

    def test_raises_with_raising_dunder_class(self):
        """Test current behavior with regard to exceptions via __class__ (#4284)."""

        class CrappyClass(Exception):
            @property
            def __class__(self):
                assert False, "via __class__"

        if six.PY2:
            with pytest.raises(pytest.fail.Exception) as excinfo:
                with pytest.raises(CrappyClass()):
                    pass
            assert "DID NOT RAISE" in excinfo.value.args[0]

            with pytest.raises(CrappyClass) as excinfo:
                raise CrappyClass()
        else:
            with pytest.raises(AssertionError) as excinfo:
                with pytest.raises(CrappyClass()):
                    pass
            assert "via __class__" in excinfo.value.args[0]


class TestUnicodeHandling:
    """Test various combinations of bytes and unicode with pytest.raises (#5478)

    https://github.com/pytest-dev/pytest/pull/5479#discussion_r298852433
    """

    success = dummy_context_manager
    py2_only = pytest.mark.skipif(
        not six.PY2, reason="bytes in raises only supported in Python 2"
    )

    @pytest.mark.parametrize(
        "message, match, expectation",
        [
            (u"\u2603", u"\u2603", success()),
            (u"\u2603", u"\u2603foo", pytest.raises(AssertionError)),
            pytest.param(b"hello", b"hello", success(), marks=py2_only),
            pytest.param(
                b"hello", b"world", pytest.raises(AssertionError), marks=py2_only
            ),
            pytest.param(u"hello", b"hello", success(), marks=py2_only),
            pytest.param(
                u"hello", b"world", pytest.raises(AssertionError), marks=py2_only
            ),
            pytest.param(
                u"😊".encode("UTF-8"),
                b"world",
                pytest.raises(AssertionError),
                marks=py2_only,
            ),
            pytest.param(
                u"world",
                u"😊".encode("UTF-8"),
                pytest.raises(AssertionError),
                marks=py2_only,
            ),
        ],
    )
    def test_handling(self, message, match, expectation):
        with expectation:
            with pytest.raises(RuntimeError, match=match):
                raise RuntimeError(message)
