import warnings
import py
import pytest
from _pytest.recwarn import WarningsRecorder


def test_recwarn_functional(testdir):
    reprec = testdir.inline_runsource("""
        import warnings
        oldwarn = warnings.showwarning
        def test_method(recwarn):
            assert warnings.showwarning != oldwarn
            warnings.warn("hello")
            warn = recwarn.pop()
            assert isinstance(warn.message, UserWarning)
        def test_finalized():
            assert warnings.showwarning == oldwarn
    """)
    res = reprec.countoutcomes()
    assert tuple(res) == (2, 0, 0), res


class TestWarningsRecorderChecker(object):
    def test_recording(self, recwarn):
        showwarning = py.std.warnings.showwarning
        rec = WarningsRecorder()
        with rec:
            assert py.std.warnings.showwarning != showwarning
            assert not rec.list
            py.std.warnings.warn_explicit("hello", UserWarning, "xyz", 13)
            assert len(rec.list) == 1
            py.std.warnings.warn(DeprecationWarning("hello"))
            assert len(rec.list) == 2
            warn = rec.pop()
            assert str(warn.message) == "hello"
            l = rec.list
            rec.clear()
            assert len(rec.list) == 0
            assert l is rec.list
            pytest.raises(AssertionError, "rec.pop()")

        assert showwarning == py.std.warnings.showwarning

    def test_typechecking(self):
        from _pytest.recwarn import WarningsChecker
        with pytest.raises(TypeError):
            WarningsChecker(5)
        with pytest.raises(TypeError):
            WarningsChecker(('hi', RuntimeWarning))
        with pytest.raises(TypeError):
            WarningsChecker([DeprecationWarning, RuntimeWarning])

    def test_invalid_enter_exit(self):
        # wrap this test in WarningsRecorder to ensure warning state gets reset
        with WarningsRecorder():
            with pytest.raises(RuntimeError):
                rec = WarningsRecorder()
                rec.__exit__(None, None, None)  # can't exit before entering

            with pytest.raises(RuntimeError):
                rec = WarningsRecorder()
                with rec:
                    with rec:
                        pass  # can't enter twice


class TestDeprecatedCall(object):
    """test pytest.deprecated_call()"""

    def dep(self, i, j=None):
        if i == 0:
            py.std.warnings.warn("is deprecated", DeprecationWarning,
                                 stacklevel=1)
        return 42

    def dep_explicit(self, i):
        if i == 0:
            py.std.warnings.warn_explicit("dep_explicit", category=DeprecationWarning,
                                          filename="hello", lineno=3)

    def test_deprecated_call_raises(self):
        with pytest.raises(AssertionError) as excinfo:
            pytest.deprecated_call(self.dep, 3, 5)
        assert str(excinfo).find("did not produce") != -1

    def test_deprecated_call(self):
        pytest.deprecated_call(self.dep, 0, 5)

    def test_deprecated_call_ret(self):
        ret = pytest.deprecated_call(self.dep, 0)
        assert ret == 42

    def test_deprecated_call_preserves(self):
        onceregistry = py.std.warnings.onceregistry.copy()
        filters = py.std.warnings.filters[:]
        warn = py.std.warnings.warn
        warn_explicit = py.std.warnings.warn_explicit
        self.test_deprecated_call_raises()
        self.test_deprecated_call()
        assert onceregistry == py.std.warnings.onceregistry
        assert filters == py.std.warnings.filters
        assert warn is py.std.warnings.warn
        assert warn_explicit is py.std.warnings.warn_explicit

    def test_deprecated_explicit_call_raises(self):
        with pytest.raises(AssertionError):
            pytest.deprecated_call(self.dep_explicit, 3)

    def test_deprecated_explicit_call(self):
        pytest.deprecated_call(self.dep_explicit, 0)
        pytest.deprecated_call(self.dep_explicit, 0)

    def test_deprecated_call_as_context_manager_no_warning(self):
        with pytest.raises(pytest.fail.Exception) as ex:
            with pytest.deprecated_call():
                self.dep(1)
        assert str(ex.value) == "DID NOT WARN"

    def test_deprecated_call_as_context_manager(self):
        with pytest.deprecated_call():
            self.dep(0)

    def test_deprecated_call_pending(self):
        def f():
            py.std.warnings.warn(PendingDeprecationWarning("hi"))
        pytest.deprecated_call(f)

    def test_deprecated_call_specificity(self):
        other_warnings = [Warning, UserWarning, SyntaxWarning, RuntimeWarning,
                          FutureWarning, ImportWarning, UnicodeWarning]
        for warning in other_warnings:
            def f():
                py.std.warnings.warn(warning("hi"))
            with pytest.raises(AssertionError):
                pytest.deprecated_call(f)

    def test_deprecated_function_already_called(self, testdir):
        """deprecated_call should be able to catch a call to a deprecated
        function even if that function has already been called in the same
        module. See #1190.
        """
        testdir.makepyfile("""
            import warnings
            import pytest

            def deprecated_function():
                warnings.warn("deprecated", DeprecationWarning)

            def test_one():
                deprecated_function()

            def test_two():
                pytest.deprecated_call(deprecated_function)
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines('*=== 2 passed in *===')


class TestWarns(object):
    def test_strings(self):
        # different messages, b/c Python suppresses multiple identical warnings
        source1 = "warnings.warn('w1', RuntimeWarning)"
        source2 = "warnings.warn('w2', RuntimeWarning)"
        source3 = "warnings.warn('w3', RuntimeWarning)"
        pytest.warns(RuntimeWarning, source1)
        pytest.raises(pytest.fail.Exception,
                      lambda: pytest.warns(UserWarning, source2))
        pytest.warns(RuntimeWarning, source3)

    def test_function(self):
        pytest.warns(SyntaxWarning,
                     lambda msg: warnings.warn(msg, SyntaxWarning), "syntax")

    def test_warning_tuple(self):
        pytest.warns((RuntimeWarning, SyntaxWarning),
                     lambda: warnings.warn('w1', RuntimeWarning))
        pytest.warns((RuntimeWarning, SyntaxWarning),
                     lambda: warnings.warn('w2', SyntaxWarning))
        pytest.raises(pytest.fail.Exception,
                      lambda: pytest.warns(
                          (RuntimeWarning, SyntaxWarning),
                          lambda: warnings.warn('w3', UserWarning)))

    def test_as_contextmanager(self):
        with pytest.warns(RuntimeWarning):
            warnings.warn("runtime", RuntimeWarning)

        with pytest.raises(pytest.fail.Exception):
            with pytest.warns(RuntimeWarning):
                warnings.warn("user", UserWarning)

        with pytest.raises(pytest.fail.Exception):
            with pytest.warns(UserWarning):
                warnings.warn("runtime", RuntimeWarning)

        with pytest.warns(UserWarning):
            warnings.warn("user", UserWarning)

    def test_record(self):
        with pytest.warns(UserWarning) as record:
            warnings.warn("user", UserWarning)

        assert len(record) == 1
        assert str(record[0].message) == "user"

    def test_record_only(self):
        with pytest.warns(None) as record:
            warnings.warn("user", UserWarning)
            warnings.warn("runtime", RuntimeWarning)

        assert len(record) == 2
        assert str(record[0].message) == "user"
        assert str(record[1].message) == "runtime"

    def test_double_test(self, testdir):
        """If a test is run again, the warning should still be raised"""
        testdir.makepyfile('''
            import pytest
            import warnings

            @pytest.mark.parametrize('run', [1, 2])
            def test(run):
                with pytest.warns(RuntimeWarning):
                    warnings.warn("runtime", RuntimeWarning)
        ''')
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(['*2 passed in*'])
