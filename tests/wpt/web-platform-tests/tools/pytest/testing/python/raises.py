import pytest

class TestRaises:
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
        class A:
            def __call__(self):
                pass
        try:
            pytest.raises(ValueError, A())
        except pytest.raises.Exception:
            pass

    def test_raises_flip_builtin_AssertionError(self):
        # we replace AssertionError on python level
        # however c code might still raise the builtin one
        from _pytest.assertion.util import BuiltinAssertionError # noqa
        pytest.raises(AssertionError,"""
            raise BuiltinAssertionError
        """)

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
