# mypy: allow-untyped-defs
import sys
from typing import List
from typing import Optional
from typing import Type
from typing import Union
import warnings

import pytest
from pytest import ExitCode
from pytest import Pytester
from pytest import WarningsRecorder


def test_recwarn_stacklevel(recwarn: WarningsRecorder) -> None:
    warnings.warn("hello")
    warn = recwarn.pop()
    assert warn.filename == __file__


def test_recwarn_functional(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import warnings
        def test_method(recwarn):
            warnings.warn("hello")
            warn = recwarn.pop()
            assert isinstance(warn.message, UserWarning)
    """
    )
    reprec = pytester.inline_run()
    reprec.assertoutcome(passed=1)


@pytest.mark.filterwarnings("")
def test_recwarn_captures_deprecation_warning(recwarn: WarningsRecorder) -> None:
    """
    Check that recwarn can capture DeprecationWarning by default
    without custom filterwarnings (see #8666).
    """
    warnings.warn(DeprecationWarning("some deprecation"))
    assert len(recwarn) == 1
    assert recwarn.pop(DeprecationWarning)


class TestSubclassWarningPop:
    class ParentWarning(Warning):
        pass

    class ChildWarning(ParentWarning):
        pass

    class ChildOfChildWarning(ChildWarning):
        pass

    @staticmethod
    def raise_warnings_from_list(_warnings: List[Type[Warning]]):
        for warn in _warnings:
            warnings.warn(f"Warning {warn().__repr__()}", warn)

    def test_pop_finds_exact_match(self):
        with pytest.warns((self.ParentWarning, self.ChildWarning)) as record:
            self.raise_warnings_from_list(
                [self.ChildWarning, self.ParentWarning, self.ChildOfChildWarning]
            )

        assert len(record) == 3
        _warn = record.pop(self.ParentWarning)
        assert _warn.category is self.ParentWarning

    def test_pop_raises_if_no_match(self):
        with pytest.raises(AssertionError):
            with pytest.warns(self.ParentWarning) as record:
                self.raise_warnings_from_list([self.ParentWarning])
            record.pop(self.ChildOfChildWarning)

    def test_pop_finds_best_inexact_match(self):
        with pytest.warns(self.ParentWarning) as record:
            self.raise_warnings_from_list(
                [self.ChildOfChildWarning, self.ChildWarning, self.ChildOfChildWarning]
            )

        _warn = record.pop(self.ParentWarning)
        assert _warn.category is self.ChildWarning


class TestWarningsRecorderChecker:
    def test_recording(self) -> None:
        rec = WarningsRecorder(_ispytest=True)
        with rec:
            assert not rec.list
            warnings.warn_explicit("hello", UserWarning, "xyz", 13)
            assert len(rec.list) == 1
            warnings.warn(DeprecationWarning("hello"))
            assert len(rec.list) == 2
            warn = rec.pop()
            assert str(warn.message) == "hello"
            values = rec.list
            rec.clear()
            assert len(rec.list) == 0
            assert values is rec.list
            pytest.raises(AssertionError, rec.pop)

    def test_warn_stacklevel(self) -> None:
        """#4243"""
        rec = WarningsRecorder(_ispytest=True)
        with rec:
            warnings.warn("test", DeprecationWarning, 2)

    def test_typechecking(self) -> None:
        from _pytest.recwarn import WarningsChecker

        with pytest.raises(TypeError):
            WarningsChecker(5, _ispytest=True)  # type: ignore[arg-type]
        with pytest.raises(TypeError):
            WarningsChecker(("hi", RuntimeWarning), _ispytest=True)  # type: ignore[arg-type]
        with pytest.raises(TypeError):
            WarningsChecker([DeprecationWarning, RuntimeWarning], _ispytest=True)  # type: ignore[arg-type]

    def test_invalid_enter_exit(self) -> None:
        # wrap this test in WarningsRecorder to ensure warning state gets reset
        with WarningsRecorder(_ispytest=True):
            with pytest.raises(RuntimeError):
                rec = WarningsRecorder(_ispytest=True)
                rec.__exit__(None, None, None)  # can't exit before entering

            with pytest.raises(RuntimeError):
                rec = WarningsRecorder(_ispytest=True)
                with rec:
                    with rec:
                        pass  # can't enter twice


class TestDeprecatedCall:
    """test pytest.deprecated_call()"""

    def dep(self, i: int, j: Optional[int] = None) -> int:
        if i == 0:
            warnings.warn("is deprecated", DeprecationWarning, stacklevel=1)
        return 42

    def dep_explicit(self, i: int) -> None:
        if i == 0:
            warnings.warn_explicit(
                "dep_explicit", category=DeprecationWarning, filename="hello", lineno=3
            )

    def test_deprecated_call_raises(self) -> None:
        with pytest.raises(pytest.fail.Exception, match="No warnings of type"):
            pytest.deprecated_call(self.dep, 3, 5)

    def test_deprecated_call(self) -> None:
        pytest.deprecated_call(self.dep, 0, 5)

    def test_deprecated_call_ret(self) -> None:
        ret = pytest.deprecated_call(self.dep, 0)
        assert ret == 42

    def test_deprecated_call_preserves(self) -> None:
        # Type ignored because `onceregistry` and `filters` are not
        # documented API.
        onceregistry = warnings.onceregistry.copy()  # type: ignore
        filters = warnings.filters[:]
        warn = warnings.warn
        warn_explicit = warnings.warn_explicit
        self.test_deprecated_call_raises()
        self.test_deprecated_call()
        assert onceregistry == warnings.onceregistry  # type: ignore
        assert filters == warnings.filters
        assert warn is warnings.warn
        assert warn_explicit is warnings.warn_explicit

    def test_deprecated_explicit_call_raises(self) -> None:
        with pytest.raises(pytest.fail.Exception):
            pytest.deprecated_call(self.dep_explicit, 3)

    def test_deprecated_explicit_call(self) -> None:
        pytest.deprecated_call(self.dep_explicit, 0)
        pytest.deprecated_call(self.dep_explicit, 0)

    @pytest.mark.parametrize("mode", ["context_manager", "call"])
    def test_deprecated_call_no_warning(self, mode) -> None:
        """Ensure deprecated_call() raises the expected failure when its block/function does
        not raise a deprecation warning.
        """

        def f():
            pass

        msg = "No warnings of type (.*DeprecationWarning.*, .*PendingDeprecationWarning.*)"
        with pytest.raises(pytest.fail.Exception, match=msg):
            if mode == "call":
                pytest.deprecated_call(f)
            else:
                with pytest.deprecated_call():
                    f()

    @pytest.mark.parametrize(
        "warning_type", [PendingDeprecationWarning, DeprecationWarning, FutureWarning]
    )
    @pytest.mark.parametrize("mode", ["context_manager", "call"])
    @pytest.mark.parametrize("call_f_first", [True, False])
    @pytest.mark.filterwarnings("ignore")
    def test_deprecated_call_modes(self, warning_type, mode, call_f_first) -> None:
        """Ensure deprecated_call() captures a deprecation warning as expected inside its
        block/function.
        """

        def f():
            warnings.warn(warning_type("hi"))
            return 10

        # ensure deprecated_call() can capture the warning even if it has already been triggered
        if call_f_first:
            assert f() == 10
        if mode == "call":
            assert pytest.deprecated_call(f) == 10
        else:
            with pytest.deprecated_call():
                assert f() == 10

    def test_deprecated_call_specificity(self) -> None:
        other_warnings = [
            Warning,
            UserWarning,
            SyntaxWarning,
            RuntimeWarning,
            ImportWarning,
            UnicodeWarning,
        ]
        for warning in other_warnings:

            def f():
                warnings.warn(warning("hi"))  # noqa: B023

            with pytest.warns(warning):
                with pytest.raises(pytest.fail.Exception):
                    pytest.deprecated_call(f)
                with pytest.raises(pytest.fail.Exception):
                    with pytest.deprecated_call():
                        f()

    def test_deprecated_call_supports_match(self) -> None:
        with pytest.deprecated_call(match=r"must be \d+$"):
            warnings.warn("value must be 42", DeprecationWarning)

        with pytest.deprecated_call():
            with pytest.raises(pytest.fail.Exception, match="DID NOT WARN"):
                with pytest.deprecated_call(match=r"must be \d+$"):
                    warnings.warn("this is not here", DeprecationWarning)


class TestWarns:
    def test_check_callable(self) -> None:
        source = "warnings.warn('w1', RuntimeWarning)"
        with pytest.raises(TypeError, match=r".* must be callable"):
            pytest.warns(RuntimeWarning, source)  # type: ignore

    def test_several_messages(self) -> None:
        # different messages, b/c Python suppresses multiple identical warnings
        pytest.warns(RuntimeWarning, lambda: warnings.warn("w1", RuntimeWarning))
        with pytest.warns(RuntimeWarning):
            with pytest.raises(pytest.fail.Exception):
                pytest.warns(UserWarning, lambda: warnings.warn("w2", RuntimeWarning))
        pytest.warns(RuntimeWarning, lambda: warnings.warn("w3", RuntimeWarning))

    def test_function(self) -> None:
        pytest.warns(
            SyntaxWarning, lambda msg: warnings.warn(msg, SyntaxWarning), "syntax"
        )

    def test_warning_tuple(self) -> None:
        pytest.warns(
            (RuntimeWarning, SyntaxWarning), lambda: warnings.warn("w1", RuntimeWarning)
        )
        pytest.warns(
            (RuntimeWarning, SyntaxWarning), lambda: warnings.warn("w2", SyntaxWarning)
        )
        with pytest.warns():
            pytest.raises(
                pytest.fail.Exception,
                lambda: pytest.warns(
                    (RuntimeWarning, SyntaxWarning),
                    lambda: warnings.warn("w3", UserWarning),
                ),
            )

    def test_as_contextmanager(self) -> None:
        with pytest.warns(RuntimeWarning):
            warnings.warn("runtime", RuntimeWarning)

        with pytest.warns(UserWarning):
            warnings.warn("user", UserWarning)

        with pytest.warns():
            with pytest.raises(pytest.fail.Exception) as excinfo:
                with pytest.warns(RuntimeWarning):
                    warnings.warn("user", UserWarning)
        excinfo.match(
            r"DID NOT WARN. No warnings of type \(.+RuntimeWarning.+,\) were emitted.\n"
            r" Emitted warnings: \[UserWarning\('user',?\)\]."
        )

        with pytest.warns():
            with pytest.raises(pytest.fail.Exception) as excinfo:
                with pytest.warns(UserWarning):
                    warnings.warn("runtime", RuntimeWarning)
        excinfo.match(
            r"DID NOT WARN. No warnings of type \(.+UserWarning.+,\) were emitted.\n"
            r" Emitted warnings: \[RuntimeWarning\('runtime',?\)]."
        )

        with pytest.raises(pytest.fail.Exception) as excinfo:
            with pytest.warns(UserWarning):
                pass
        excinfo.match(
            r"DID NOT WARN. No warnings of type \(.+UserWarning.+,\) were emitted.\n"
            r" Emitted warnings: \[\]."
        )

        warning_classes = (UserWarning, FutureWarning)
        with pytest.warns():
            with pytest.raises(pytest.fail.Exception) as excinfo:
                with pytest.warns(warning_classes) as warninfo:
                    warnings.warn("runtime", RuntimeWarning)
                    warnings.warn("import", ImportWarning)

        messages = [each.message for each in warninfo]
        expected_str = (
            f"DID NOT WARN. No warnings of type {warning_classes} were emitted.\n"
            f" Emitted warnings: {messages}."
        )

        assert str(excinfo.value) == expected_str

    def test_record(self) -> None:
        with pytest.warns(UserWarning) as record:
            warnings.warn("user", UserWarning)

        assert len(record) == 1
        assert str(record[0].message) == "user"

    def test_record_only(self) -> None:
        with pytest.warns() as record:
            warnings.warn("user", UserWarning)
            warnings.warn("runtime", RuntimeWarning)

        assert len(record) == 2
        assert str(record[0].message) == "user"
        assert str(record[1].message) == "runtime"

    def test_record_only_none_type_error(self) -> None:
        with pytest.raises(TypeError):
            pytest.warns(None)  # type: ignore[call-overload]

    def test_record_by_subclass(self) -> None:
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

    def test_double_test(self, pytester: Pytester) -> None:
        """If a test is run again, the warning should still be raised"""
        pytester.makepyfile(
            """
            import pytest
            import warnings

            @pytest.mark.parametrize('run', [1, 2])
            def test(run):
                with pytest.warns(RuntimeWarning):
                    warnings.warn("runtime", RuntimeWarning)
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*2 passed in*"])

    def test_match_regex(self) -> None:
        with pytest.warns(UserWarning, match=r"must be \d+$"):
            warnings.warn("value must be 42", UserWarning)

        with pytest.warns():
            with pytest.raises(pytest.fail.Exception):
                with pytest.warns(UserWarning, match=r"must be \d+$"):
                    warnings.warn("this is not here", UserWarning)

        with pytest.warns():
            with pytest.raises(pytest.fail.Exception):
                with pytest.warns(FutureWarning, match=r"must be \d+$"):
                    warnings.warn("value must be 42", UserWarning)

    def test_one_from_multiple_warns(self) -> None:
        with pytest.warns():
            with pytest.raises(pytest.fail.Exception, match="DID NOT WARN"):
                with pytest.warns(UserWarning, match=r"aaa"):
                    with pytest.warns(UserWarning, match=r"aaa"):
                        warnings.warn("cccccccccc", UserWarning)
                        warnings.warn("bbbbbbbbbb", UserWarning)
                        warnings.warn("aaaaaaaaaa", UserWarning)

    def test_none_of_multiple_warns(self) -> None:
        with pytest.warns():
            with pytest.raises(pytest.fail.Exception, match="DID NOT WARN"):
                with pytest.warns(UserWarning, match=r"aaa"):
                    warnings.warn("bbbbbbbbbb", UserWarning)
                    warnings.warn("cccccccccc", UserWarning)

    @pytest.mark.filterwarnings("ignore")
    def test_can_capture_previously_warned(self) -> None:
        def f() -> int:
            warnings.warn(UserWarning("ohai"))
            return 10

        assert f() == 10
        assert pytest.warns(UserWarning, f) == 10
        assert pytest.warns(UserWarning, f) == 10
        assert pytest.warns(UserWarning, f) != "10"  # type: ignore[comparison-overlap]

    def test_warns_context_manager_with_kwargs(self) -> None:
        with pytest.raises(TypeError) as excinfo:
            with pytest.warns(UserWarning, foo="bar"):  # type: ignore
                pass
        assert "Unexpected keyword arguments" in str(excinfo.value)

    def test_re_emit_single(self) -> None:
        with pytest.warns(DeprecationWarning):
            with pytest.warns(UserWarning):
                warnings.warn("user warning", UserWarning)
                warnings.warn("some deprecation warning", DeprecationWarning)

    def test_re_emit_multiple(self) -> None:
        with pytest.warns(UserWarning):
            warnings.warn("first warning", UserWarning)
            warnings.warn("second warning", UserWarning)

    def test_re_emit_match_single(self) -> None:
        with pytest.warns(DeprecationWarning):
            with pytest.warns(UserWarning, match="user warning"):
                warnings.warn("user warning", UserWarning)
                warnings.warn("some deprecation warning", DeprecationWarning)

    def test_re_emit_match_multiple(self) -> None:
        with warnings.catch_warnings():
            warnings.simplefilter("error")  # if anything is re-emitted
            with pytest.warns(UserWarning, match="user warning"):
                warnings.warn("first user warning", UserWarning)
                warnings.warn("second user warning", UserWarning)

    def test_re_emit_non_match_single(self) -> None:
        with pytest.warns(UserWarning, match="v2 warning"):
            with pytest.warns(UserWarning, match="v1 warning"):
                warnings.warn("v1 warning", UserWarning)
                warnings.warn("non-matching v2 warning", UserWarning)

    def test_catch_warning_within_raise(self) -> None:
        # warns-in-raises works since https://github.com/pytest-dev/pytest/pull/11129
        with pytest.raises(ValueError, match="some exception"):
            with pytest.warns(FutureWarning, match="some warning"):
                warnings.warn("some warning", category=FutureWarning)
                raise ValueError("some exception")
        # and raises-in-warns has always worked but we'll check for symmetry.
        with pytest.warns(FutureWarning, match="some warning"):
            with pytest.raises(ValueError, match="some exception"):
                warnings.warn("some warning", category=FutureWarning)
                raise ValueError("some exception")

    def test_skip_within_warns(self, pytester: Pytester) -> None:
        """Regression test for #11907."""
        pytester.makepyfile(
            """
            import pytest

            def test_it():
                with pytest.warns(Warning):
                    pytest.skip("this is OK")
            """,
        )

        result = pytester.runpytest()
        assert result.ret == ExitCode.OK
        result.assert_outcomes(skipped=1)

    def test_fail_within_warns(self, pytester: Pytester) -> None:
        """Regression test for #11907."""
        pytester.makepyfile(
            """
            import pytest

            def test_it():
                with pytest.warns(Warning):
                    pytest.fail("BOOM")
            """,
        )

        result = pytester.runpytest()
        assert result.ret == ExitCode.TESTS_FAILED
        result.assert_outcomes(failed=1)
        assert "DID NOT WARN" not in str(result.stdout)

    def test_exit_within_warns(self, pytester: Pytester) -> None:
        """Regression test for #11907."""
        pytester.makepyfile(
            """
            import pytest

            def test_it():
                with pytest.warns(Warning):
                    pytest.exit()
            """,
        )

        result = pytester.runpytest()
        assert result.ret == ExitCode.INTERRUPTED
        result.assert_outcomes()

    def test_keyboard_interrupt_within_warns(self, pytester: Pytester) -> None:
        """Regression test for #11907."""
        pytester.makepyfile(
            """
            import pytest

            def test_it():
                with pytest.warns(Warning):
                    raise KeyboardInterrupt()
            """,
        )

        result = pytester.runpytest_subprocess()
        assert result.ret == ExitCode.INTERRUPTED
        result.assert_outcomes()


def test_raise_type_error_on_invalid_warning() -> None:
    """Check pytest.warns validates warning messages are strings (#10865) or
    Warning instances (#11959)."""
    with pytest.raises(TypeError, match="Warning must be str or Warning"):
        with pytest.warns(UserWarning):
            warnings.warn(1)  # type: ignore


@pytest.mark.parametrize(
    "message",
    [
        pytest.param("Warning", id="str"),
        pytest.param(UserWarning(), id="UserWarning"),
        pytest.param(Warning(), id="Warning"),
    ],
)
def test_no_raise_type_error_on_valid_warning(message: Union[str, Warning]) -> None:
    """Check pytest.warns validates warning messages are strings (#10865) or
    Warning instances (#11959)."""
    with pytest.warns(Warning):
        warnings.warn(message)


@pytest.mark.skipif(
    hasattr(sys, "pypy_version_info"),
    reason="Not for pypy",
)
def test_raise_type_error_on_invalid_warning_message_cpython() -> None:
    # Check that we get the same behavior with the stdlib, at least if filtering
    # (see https://github.com/python/cpython/issues/103577 for details)
    with pytest.raises(TypeError):
        with warnings.catch_warnings():
            warnings.filterwarnings("ignore", "test")
            warnings.warn(1)  # type: ignore


def test_multiple_arg_custom_warning() -> None:
    """Test for issue #11906."""

    class CustomWarning(UserWarning):
        def __init__(self, a, b):
            pass

    with pytest.warns(CustomWarning):
        with pytest.raises(pytest.fail.Exception, match="DID NOT WARN"):
            with pytest.warns(CustomWarning, match="not gonna match"):
                a, b = 1, 2
                warnings.warn(CustomWarning(a, b))
