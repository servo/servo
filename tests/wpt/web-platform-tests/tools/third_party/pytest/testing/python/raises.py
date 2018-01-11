import pytest
import sys


class TestRaises(object):
    def test_raises(self):
        source = "int('qwe')"
        excinfo = pytest.raises(ValueError, source)
        code = excinfo.traceback[-1].frame.code
        s = str(code.fullsource)
        assert s == source

    def test_raises_exec(self):
        pytest.raises(ValueError, "a,x = []")

    def test_raises_syntax_error(self):
        pytest.raises(SyntaxError, "qwe qwe qwe")

    def test_raises_function(self):
        pytest.raises(ValueError, int, 'hello')

    def test_raises_callable_no_exception(self):
        class A(object):
            def __call__(self):
                pass
        try:
            pytest.raises(ValueError, A())
        except pytest.raises.Exception:
            pass

    def test_raises_as_contextmanager(self, testdir):
        testdir.makepyfile("""
            from __future__ import with_statement
            import py, pytest
            import _pytest._code

            def test_simple():
                with pytest.raises(ZeroDivisionError) as excinfo:
                    assert isinstance(excinfo, _pytest._code.ExceptionInfo)
                    1/0
                print (excinfo)
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
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            '*3 passed*',
        ])

    def test_noclass(self):
        with pytest.raises(TypeError):
            pytest.raises('wrong', lambda: None)

    def test_tuple(self):
        with pytest.raises((KeyError, ValueError)):
            raise KeyError('oops')

    def test_no_raise_message(self):
        try:
            pytest.raises(ValueError, int, '0')
        except pytest.raises.Exception as e:
            assert e.msg == "DID NOT RAISE {0}".format(repr(ValueError))
        else:
            assert False, "Expected pytest.raises.Exception"

        try:
            with pytest.raises(ValueError):
                pass
        except pytest.raises.Exception as e:
            assert e.msg == "DID NOT RAISE {0}".format(repr(ValueError))
        else:
            assert False, "Expected pytest.raises.Exception"

    def test_custom_raise_message(self):
        message = "TEST_MESSAGE"
        try:
            with pytest.raises(ValueError, message=message):
                pass
        except pytest.raises.Exception as e:
            assert e.msg == message
        else:
            assert False, "Expected pytest.raises.Exception"

    @pytest.mark.parametrize('method', ['function', 'with'])
    def test_raises_cyclic_reference(self, method):
        """
        Ensure pytest.raises does not leave a reference cycle (#1965).
        """
        import gc

        class T(object):
            def __call__(self):
                raise ValueError

        t = T()
        if method == 'function':
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
            int('asdf')

        msg = "with base 10"
        with pytest.raises(ValueError, match=msg):
            int('asdf')

        msg = "with base 16"
        expr = r"Pattern '{0}' not found in 'invalid literal for int\(\) with base 10: 'asdf''".format(msg)
        with pytest.raises(AssertionError, match=expr):
            with pytest.raises(ValueError, match=msg):
                int('asdf', base=10)
