from __future__ import absolute_import, division, print_function
import warnings
import re
import py

import pytest
from _pytest.recwarn import WarningsRecorder


def test_recwarn_functional(testdir):
    reprec = testdir.inline_runsource("""
        import warnings
        def test_method(recwarn):
            warnings.warn("hello")
            warn = recwarn.pop()
            assert isinstance(warn.message, UserWarning)
    """)
    res = reprec.countoutcomes()
    assert tuple(res) == (1, 0, 0), res


class TestWarningsRecorderChecker(object):
    def test_recording(self):
        rec = WarningsRecorder()
        with rec:
            assert not rec.list
            py.std.warnings.warn_explicit("hello", UserWarning, "xyz", 13)
            assert len(rec.list) == 1
            py.std.warnings.warn(DeprecationWarning("hello"))
            assert len(rec.list) == 2
            warn = rec.pop()
            assert str(warn.message) == "hello"
            values = rec.list
            rec.clear()
            assert len(rec.list) == 0
            assert values is rec.list
            pytest.raises(AssertionError, "rec.pop()")

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
        assert 'Did not produce' in str(excinfo)

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

    @pytest.mark.parametrize('mode', ['context_manager', 'call'])
    def test_deprecated_call_no_warning(self, mode):
        """Ensure deprecated_call() raises the expected failure when its block/function does
        not raise a deprecation warning.
        """
        def f():
            pass

        msg = 'Did not produce DeprecationWarning or PendingDeprecationWarning'
        with pytest.raises(AssertionError, matches=msg):
            if mode == 'call':
                pytest.deprecated_call(f)
            else:
                with pytest.deprecated_call():
                    f()

    @pytest.mark.parametrize('warning_type', [PendingDeprecationWarning, DeprecationWarning])
    @pytest.mark.parametrize('mode', ['context_manager', 'call'])
    @pytest.mark.parametrize('call_f_first', [True, False])
    @pytest.mark.filterwarnings('ignore')
    def test_deprecated_call_modes(self, warning_type, mode, call_f_first):
        """Ensure deprecated_call() captures a deprecation warning as expected inside its
        block/function.
        """
        def f():
            warnings.warn(warning_type("hi"))
            return 10

        # ensure deprecated_call() can capture the warning even if it has already been triggered
        if call_f_first:
            assert f() == 10
        if mode == 'call':
            assert pytest.deprecated_call(f) == 10
        else:
            with pytest.deprecated_call():
                assert f() == 10

    @pytest.mark.parametrize('mode', ['context_manager', 'call'])
    def test_deprecated_call_exception_is_raised(self, mode):
        """If the block of the code being tested by deprecated_call() raises an exception,
        it must raise the exception undisturbed.
        """
        def f():
            raise ValueError('some exception')

        with pytest.raises(ValueError, match='some exception'):
            if mode == 'call':
                pytest.deprecated_call(f)
            else:
                with pytest.deprecated_call():
                    f()

    def test_deprecated_call_specificity(self):
        other_warnings = [Warning, UserWarning, SyntaxWarning, RuntimeWarning,
                          FutureWarning, ImportWarning, UnicodeWarning]
        for warning in other_warnings:
            def f():
                warnings.warn(warning("hi"))

            with pytest.raises(AssertionError):
                pytest.deprecated_call(f)
            with pytest.raises(AssertionError):
                with pytest.deprecated_call():
                    f()


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

        with pytest.warns(UserWarning):
            warnings.warn("user", UserWarning)

        with pytest.raises(pytest.fail.Exception) as excinfo:
            with pytest.warns(RuntimeWarning):
                warnings.warn("user", UserWarning)
        excinfo.match(r"DID NOT WARN. No warnings of type \(.+RuntimeWarning.+,\) was emitted. "
                      r"The list of emitted warnings is: \[UserWarning\('user',\)\].")

        with pytest.raises(pytest.fail.Exception) as excinfo:
            with pytest.warns(UserWarning):
                warnings.warn("runtime", RuntimeWarning)
        excinfo.match(r"DID NOT WARN. No warnings of type \(.+UserWarning.+,\) was emitted. "
                      r"The list of emitted warnings is: \[RuntimeWarning\('runtime',\)\].")

        with pytest.raises(pytest.fail.Exception) as excinfo:
            with pytest.warns(UserWarning):
                pass
        excinfo.match(r"DID NOT WARN. No warnings of type \(.+UserWarning.+,\) was emitted. "
                      r"The list of emitted warnings is: \[\].")

        warning_classes = (UserWarning, FutureWarning)
        with pytest.raises(pytest.fail.Exception) as excinfo:
            with pytest.warns(warning_classes) as warninfo:
                warnings.warn("runtime", RuntimeWarning)
                warnings.warn("import", ImportWarning)

        message_template = ("DID NOT WARN. No warnings of type {0} was emitted. "
                            "The list of emitted warnings is: {1}.")
        excinfo.match(re.escape(message_template.format(warning_classes,
                                                        [each.message for each in warninfo])))

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

    def test_record_by_subclass(self):
        with pytest.warns(Warning) as record:
            warnings.warn("user", UserWarning)
            warnings.warn("runtime", RuntimeWarning)

        assert len(record) == 2
        assert str(record[0].message) == "user"
        assert str(record[1].message) == "runtime"

        class MyUserWarning(UserWarning):
            pass

        class MyRuntimeWarning(RuntimeWarning):
            pass

        with pytest.warns((UserWarning, RuntimeWarning)) as record:
            warnings.warn("user", MyUserWarning)
            warnings.warn("runtime", MyRuntimeWarning)

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

    def test_match_regex(self):
        with pytest.warns(UserWarning, match=r'must be \d+$'):
            warnings.warn("value must be 42", UserWarning)

        with pytest.raises(pytest.fail.Exception):
            with pytest.warns(UserWarning, match=r'must be \d+$'):
                warnings.warn("this is not here", UserWarning)

        with pytest.raises(pytest.fail.Exception):
            with pytest.warns(FutureWarning, match=r'must be \d+$'):
                warnings.warn("value must be 42", UserWarning)

    def test_one_from_multiple_warns(self):
        with pytest.warns(UserWarning, match=r'aaa'):
            warnings.warn("cccccccccc", UserWarning)
            warnings.warn("bbbbbbbbbb", UserWarning)
            warnings.warn("aaaaaaaaaa", UserWarning)

    def test_none_of_multiple_warns(self):
        with pytest.raises(pytest.fail.Exception):
            with pytest.warns(UserWarning, match=r'aaa'):
                warnings.warn("bbbbbbbbbb", UserWarning)
                warnings.warn("cccccccccc", UserWarning)
