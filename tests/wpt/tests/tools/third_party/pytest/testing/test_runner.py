# mypy: allow-untyped-defs
from functools import partial
import inspect
import os
from pathlib import Path
import sys
import types
from typing import Dict
from typing import List
from typing import Tuple
from typing import Type
import warnings

from _pytest import outcomes
from _pytest import reports
from _pytest import runner
from _pytest._code import ExceptionInfo
from _pytest._code.code import ExceptionChainRepr
from _pytest.config import ExitCode
from _pytest.monkeypatch import MonkeyPatch
from _pytest.outcomes import OutcomeException
from _pytest.pytester import Pytester
import pytest


if sys.version_info < (3, 11):
    from exceptiongroup import ExceptionGroup


class TestSetupState:
    def test_setup(self, pytester: Pytester) -> None:
        item = pytester.getitem("def test_func(): pass")
        ss = item.session._setupstate
        values = [1]
        ss.setup(item)
        ss.addfinalizer(values.pop, item)
        assert values
        ss.teardown_exact(None)
        assert not values

    def test_teardown_exact_stack_empty(self, pytester: Pytester) -> None:
        item = pytester.getitem("def test_func(): pass")
        ss = item.session._setupstate
        ss.setup(item)
        ss.teardown_exact(None)
        ss.teardown_exact(None)
        ss.teardown_exact(None)

    def test_setup_fails_and_failure_is_cached(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            def setup_module(mod):
                raise ValueError(42)
            def test_func(): pass
        """
        )
        ss = item.session._setupstate
        with pytest.raises(ValueError):
            ss.setup(item)
        with pytest.raises(ValueError):
            ss.setup(item)

    def test_teardown_multiple_one_fails(self, pytester: Pytester) -> None:
        r = []

        def fin1():
            r.append("fin1")

        def fin2():
            raise Exception("oops")

        def fin3():
            r.append("fin3")

        item = pytester.getitem("def test_func(): pass")
        ss = item.session._setupstate
        ss.setup(item)
        ss.addfinalizer(fin1, item)
        ss.addfinalizer(fin2, item)
        ss.addfinalizer(fin3, item)
        with pytest.raises(Exception) as err:
            ss.teardown_exact(None)
        assert err.value.args == ("oops",)
        assert r == ["fin3", "fin1"]

    def test_teardown_multiple_fail(self, pytester: Pytester) -> None:
        def fin1():
            raise Exception("oops1")

        def fin2():
            raise Exception("oops2")

        item = pytester.getitem("def test_func(): pass")
        ss = item.session._setupstate
        ss.setup(item)
        ss.addfinalizer(fin1, item)
        ss.addfinalizer(fin2, item)
        with pytest.raises(ExceptionGroup) as err:
            ss.teardown_exact(None)

        # Note that finalizers are run LIFO, but because FIFO is more intuitive for
        # users we reverse the order of messages, and see the error from fin1 first.
        err1, err2 = err.value.exceptions
        assert err1.args == ("oops1",)
        assert err2.args == ("oops2",)

    def test_teardown_multiple_scopes_one_fails(self, pytester: Pytester) -> None:
        module_teardown = []

        def fin_func():
            raise Exception("oops1")

        def fin_module():
            module_teardown.append("fin_module")

        item = pytester.getitem("def test_func(): pass")
        mod = item.listchain()[-2]
        ss = item.session._setupstate
        ss.setup(item)
        ss.addfinalizer(fin_module, mod)
        ss.addfinalizer(fin_func, item)
        with pytest.raises(Exception, match="oops1"):
            ss.teardown_exact(None)
        assert module_teardown == ["fin_module"]

    def test_teardown_multiple_scopes_several_fail(self, pytester) -> None:
        def raiser(exc):
            raise exc

        item = pytester.getitem("def test_func(): pass")
        mod = item.listchain()[-2]
        ss = item.session._setupstate
        ss.setup(item)
        ss.addfinalizer(partial(raiser, KeyError("from module scope")), mod)
        ss.addfinalizer(partial(raiser, TypeError("from function scope 1")), item)
        ss.addfinalizer(partial(raiser, ValueError("from function scope 2")), item)

        with pytest.raises(ExceptionGroup, match="errors during test teardown") as e:
            ss.teardown_exact(None)
        mod, func = e.value.exceptions
        assert isinstance(mod, KeyError)
        assert isinstance(func.exceptions[0], TypeError)  # type: ignore
        assert isinstance(func.exceptions[1], ValueError)  # type: ignore


class BaseFunctionalTests:
    def test_passfunction(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            def test_func():
                pass
        """
        )
        rep = reports[1]
        assert rep.passed
        assert not rep.failed
        assert rep.outcome == "passed"
        assert not rep.longrepr

    def test_failfunction(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            def test_func():
                assert 0
        """
        )
        rep = reports[1]
        assert not rep.passed
        assert not rep.skipped
        assert rep.failed
        assert rep.when == "call"
        assert rep.outcome == "failed"
        # assert isinstance(rep.longrepr, ReprExceptionInfo)

    def test_skipfunction(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            import pytest
            def test_func():
                pytest.skip("hello")
        """
        )
        rep = reports[1]
        assert not rep.failed
        assert not rep.passed
        assert rep.skipped
        assert rep.outcome == "skipped"
        # assert rep.skipped.when == "call"
        # assert rep.skipped.when == "call"
        # assert rep.skipped == "%sreason == "hello"
        # assert rep.skipped.location.lineno == 3
        # assert rep.skipped.location.path
        # assert not rep.skipped.failurerepr

    def test_skip_in_setup_function(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            import pytest
            def setup_function(func):
                pytest.skip("hello")
            def test_func():
                pass
        """
        )
        print(reports)
        rep = reports[0]
        assert not rep.failed
        assert not rep.passed
        assert rep.skipped
        # assert rep.skipped.reason == "hello"
        # assert rep.skipped.location.lineno == 3
        # assert rep.skipped.location.lineno == 3
        assert len(reports) == 2
        assert reports[1].passed  # teardown

    def test_failure_in_setup_function(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            import pytest
            def setup_function(func):
                raise ValueError(42)
            def test_func():
                pass
        """
        )
        rep = reports[0]
        assert not rep.skipped
        assert not rep.passed
        assert rep.failed
        assert rep.when == "setup"
        assert len(reports) == 2

    def test_failure_in_teardown_function(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            import pytest
            def teardown_function(func):
                raise ValueError(42)
            def test_func():
                pass
        """
        )
        print(reports)
        assert len(reports) == 3
        rep = reports[2]
        assert not rep.skipped
        assert not rep.passed
        assert rep.failed
        assert rep.when == "teardown"
        # assert rep.longrepr.reprcrash.lineno == 3
        # assert rep.longrepr.reprtraceback.reprentries

    def test_custom_failure_repr(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            conftest="""
            import pytest
            class Function(pytest.Function):
                def repr_failure(self, excinfo):
                    return "hello"
        """
        )
        reports = pytester.runitem(
            """
            import pytest
            def test_func():
                assert 0
        """
        )
        rep = reports[1]
        assert not rep.skipped
        assert not rep.passed
        assert rep.failed
        # assert rep.outcome.when == "call"
        # assert rep.failed.where.lineno == 3
        # assert rep.failed.where.path.basename == "test_func.py"
        # assert rep.failed.failurerepr == "hello"

    def test_teardown_final_returncode(self, pytester: Pytester) -> None:
        rec = pytester.inline_runsource(
            """
            def test_func():
                pass
            def teardown_function(func):
                raise ValueError(42)
        """
        )
        assert rec.ret == 1

    def test_logstart_logfinish_hooks(self, pytester: Pytester) -> None:
        rec = pytester.inline_runsource(
            """
            import pytest
            def test_func():
                pass
        """
        )
        reps = rec.getcalls("pytest_runtest_logstart pytest_runtest_logfinish")
        assert [x._name for x in reps] == [
            "pytest_runtest_logstart",
            "pytest_runtest_logfinish",
        ]
        for rep in reps:
            assert rep.nodeid == "test_logstart_logfinish_hooks.py::test_func"
            assert rep.location == ("test_logstart_logfinish_hooks.py", 1, "test_func")

    def test_exact_teardown_issue90(self, pytester: Pytester) -> None:
        rec = pytester.inline_runsource(
            """
            import pytest

            class TestClass(object):
                def test_method(self):
                    pass
                def teardown_class(cls):
                    raise Exception()

            def test_func():
                import sys
                # on python2 exc_info is kept till a function exits
                # so we would end up calling test functions while
                # sys.exc_info would return the indexerror
                # from guessing the lastitem
                excinfo = sys.exc_info()
                import traceback
                assert excinfo[0] is None, \
                       traceback.format_exception(*excinfo)
            def teardown_function(func):
                raise ValueError(42)
        """
        )
        reps = rec.getreports("pytest_runtest_logreport")
        print(reps)
        for i in range(2):
            assert reps[i].nodeid.endswith("test_method")
            assert reps[i].passed
        assert reps[2].when == "teardown"
        assert reps[2].failed
        assert len(reps) == 6
        for i in range(3, 5):
            assert reps[i].nodeid.endswith("test_func")
            assert reps[i].passed
        assert reps[5].when == "teardown"
        assert reps[5].nodeid.endswith("test_func")
        assert reps[5].failed

    def test_exact_teardown_issue1206(self, pytester: Pytester) -> None:
        """Issue shadowing error with wrong number of arguments on teardown_method."""
        rec = pytester.inline_runsource(
            """
            import pytest

            class TestClass(object):
                def teardown_method(self, x, y, z):
                    pass

                def test_method(self):
                    assert True
        """
        )
        reps = rec.getreports("pytest_runtest_logreport")
        print(reps)
        assert len(reps) == 3
        #
        assert reps[0].nodeid.endswith("test_method")
        assert reps[0].passed
        assert reps[0].when == "setup"
        #
        assert reps[1].nodeid.endswith("test_method")
        assert reps[1].passed
        assert reps[1].when == "call"
        #
        assert reps[2].nodeid.endswith("test_method")
        assert reps[2].failed
        assert reps[2].when == "teardown"
        longrepr = reps[2].longrepr
        assert isinstance(longrepr, ExceptionChainRepr)
        assert longrepr.reprcrash
        assert longrepr.reprcrash.message in (
            "TypeError: teardown_method() missing 2 required positional arguments: 'y' and 'z'",
            # Python >= 3.10
            "TypeError: TestClass.teardown_method() missing 2 required positional arguments: 'y' and 'z'",
        )

    def test_failure_in_setup_function_ignores_custom_repr(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            conftest="""
            import pytest
            class Function(pytest.Function):
                def repr_failure(self, excinfo):
                    assert 0
        """
        )
        reports = pytester.runitem(
            """
            def setup_function(func):
                raise ValueError(42)
            def test_func():
                pass
        """
        )
        assert len(reports) == 2
        rep = reports[0]
        print(rep)
        assert not rep.skipped
        assert not rep.passed
        assert rep.failed
        # assert rep.outcome.when == "setup"
        # assert rep.outcome.where.lineno == 3
        # assert rep.outcome.where.path.basename == "test_func.py"
        # assert isinstance(rep.failed.failurerepr, PythonFailureRepr)

    def test_systemexit_does_not_bail_out(self, pytester: Pytester) -> None:
        try:
            reports = pytester.runitem(
                """
                def test_func():
                    raise SystemExit(42)
            """
            )
        except SystemExit:
            assert False, "runner did not catch SystemExit"
        rep = reports[1]
        assert rep.failed
        assert rep.when == "call"

    def test_exit_propagates(self, pytester: Pytester) -> None:
        try:
            pytester.runitem(
                """
                import pytest
                def test_func():
                    raise pytest.exit.Exception()
            """
            )
        except pytest.exit.Exception:
            pass
        else:
            assert False, "did not raise"


class TestExecutionNonForked(BaseFunctionalTests):
    def getrunner(self):
        def f(item):
            return runner.runtestprotocol(item, log=False)

        return f

    def test_keyboardinterrupt_propagates(self, pytester: Pytester) -> None:
        try:
            pytester.runitem(
                """
                def test_func():
                    raise KeyboardInterrupt("fake")
            """
            )
        except KeyboardInterrupt:
            pass
        else:
            assert False, "did not raise"


class TestSessionReports:
    def test_collect_result(self, pytester: Pytester) -> None:
        col = pytester.getmodulecol(
            """
            def test_func1():
                pass
            class TestClass(object):
                pass
        """
        )
        rep = runner.collect_one_node(col)
        assert not rep.failed
        assert not rep.skipped
        assert rep.passed
        locinfo = rep.location
        assert locinfo is not None
        assert locinfo[0] == col.path.name
        assert not locinfo[1]
        assert locinfo[2] == col.path.name
        res = rep.result
        assert len(res) == 2
        assert res[0].name == "test_func1"
        assert res[1].name == "TestClass"


reporttypes: List[Type[reports.BaseReport]] = [
    reports.BaseReport,
    reports.TestReport,
    reports.CollectReport,
]


@pytest.mark.parametrize(
    "reporttype", reporttypes, ids=[x.__name__ for x in reporttypes]
)
def test_report_extra_parameters(reporttype: Type[reports.BaseReport]) -> None:
    args = list(inspect.signature(reporttype.__init__).parameters.keys())[1:]
    basekw: Dict[str, List[object]] = dict.fromkeys(args, [])
    report = reporttype(newthing=1, **basekw)
    assert report.newthing == 1


def test_callinfo() -> None:
    ci = runner.CallInfo.from_call(lambda: 0, "collect")
    assert ci.when == "collect"
    assert ci.result == 0
    assert "result" in repr(ci)
    assert repr(ci) == "<CallInfo when='collect' result: 0>"
    assert str(ci) == "<CallInfo when='collect' result: 0>"

    ci2 = runner.CallInfo.from_call(lambda: 0 / 0, "collect")
    assert ci2.when == "collect"
    assert not hasattr(ci2, "result")
    assert repr(ci2) == f"<CallInfo when='collect' excinfo={ci2.excinfo!r}>"
    assert str(ci2) == repr(ci2)
    assert ci2.excinfo

    # Newlines are escaped.
    def raise_assertion():
        assert 0, "assert_msg"

    ci3 = runner.CallInfo.from_call(raise_assertion, "call")
    assert repr(ci3) == f"<CallInfo when='call' excinfo={ci3.excinfo!r}>"
    assert "\n" not in repr(ci3)


# design question: do we want general hooks in python files?
# then something like the following functional tests makes sense


@pytest.mark.xfail
def test_runtest_in_module_ordering(pytester: Pytester) -> None:
    p1 = pytester.makepyfile(
        """
        import pytest
        def pytest_runtest_setup(item): # runs after class-level!
            item.function.mylist.append("module")
        class TestClass(object):
            def pytest_runtest_setup(self, item):
                assert not hasattr(item.function, 'mylist')
                item.function.mylist = ['class']
            @pytest.fixture
            def mylist(self, request):
                return request.function.mylist
            @pytest.hookimpl(wrapper=True)
            def pytest_runtest_call(self, item):
                try:
                    yield
                except ValueError:
                    pass
            def test_hello1(self, mylist):
                assert mylist == ['class', 'module'], mylist
                raise ValueError()
            def test_hello2(self, mylist):
                assert mylist == ['class', 'module'], mylist
        def pytest_runtest_teardown(item):
            del item.function.mylist
    """
    )
    result = pytester.runpytest(p1)
    result.stdout.fnmatch_lines(["*2 passed*"])


def test_outcomeexception_exceptionattributes() -> None:
    outcome = outcomes.OutcomeException("test")
    assert outcome.args[0] == outcome.msg


def test_outcomeexception_passes_except_Exception() -> None:
    with pytest.raises(outcomes.OutcomeException):
        try:
            raise outcomes.OutcomeException("test")
        except Exception as e:
            raise NotImplementedError from e


def test_pytest_exit() -> None:
    with pytest.raises(pytest.exit.Exception) as excinfo:
        pytest.exit("hello")
    assert excinfo.errisinstance(pytest.exit.Exception)


def test_pytest_fail() -> None:
    with pytest.raises(pytest.fail.Exception) as excinfo:
        pytest.fail("hello")
    s = excinfo.exconly(tryshort=True)
    assert s.startswith("Failed")


def test_pytest_exit_msg(pytester: Pytester) -> None:
    pytester.makeconftest(
        """
    import pytest

    def pytest_configure(config):
        pytest.exit('oh noes')
    """
    )
    result = pytester.runpytest()
    result.stderr.fnmatch_lines(["Exit: oh noes"])


def _strip_resource_warnings(lines):
    # Assert no output on stderr, except for unreliable ResourceWarnings.
    # (https://github.com/pytest-dev/pytest/issues/5088)
    return [
        x
        for x in lines
        if not x.startswith(("Exception ignored in:", "ResourceWarning"))
    ]


def test_pytest_exit_returncode(pytester: Pytester) -> None:
    pytester.makepyfile(
        """\
        import pytest
        def test_foo():
            pytest.exit("some exit msg", 99)
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*! *Exit: some exit msg !*"])

    assert _strip_resource_warnings(result.stderr.lines) == []
    assert result.ret == 99

    # It prints to stderr also in case of exit during pytest_sessionstart.
    pytester.makeconftest(
        """\
        import pytest

        def pytest_sessionstart():
            pytest.exit("during_sessionstart", 98)
        """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*! *Exit: during_sessionstart !*"])
    assert _strip_resource_warnings(result.stderr.lines) == [
        "Exit: during_sessionstart"
    ]
    assert result.ret == 98


def test_pytest_fail_notrace_runtest(pytester: Pytester) -> None:
    """Test pytest.fail(..., pytrace=False) does not show tracebacks during test run."""
    pytester.makepyfile(
        """
        import pytest
        def test_hello():
            pytest.fail("hello", pytrace=False)
        def teardown_function(function):
            pytest.fail("world", pytrace=False)
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["world", "hello"])
    result.stdout.no_fnmatch_line("*def teardown_function*")


def test_pytest_fail_notrace_collection(pytester: Pytester) -> None:
    """Test pytest.fail(..., pytrace=False) does not show tracebacks during collection."""
    pytester.makepyfile(
        """
        import pytest
        def some_internal_function():
            pytest.fail("hello", pytrace=False)
        some_internal_function()
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["hello"])
    result.stdout.no_fnmatch_line("*def some_internal_function()*")


def test_pytest_fail_notrace_non_ascii(pytester: Pytester) -> None:
    """Fix pytest.fail with pytrace=False with non-ascii characters (#1178).

    This tests with native and unicode strings containing non-ascii chars.
    """
    pytester.makepyfile(
        """\
        import pytest

        def test_hello():
            pytest.fail('oh oh: ☺', pytrace=False)
        """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*test_hello*", "oh oh: ☺"])
    result.stdout.no_fnmatch_line("*def test_hello*")


def test_pytest_no_tests_collected_exit_status(pytester: Pytester) -> None:
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*collected 0 items*"])
    assert result.ret == ExitCode.NO_TESTS_COLLECTED

    pytester.makepyfile(
        test_foo="""
        def test_foo():
            assert 1
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*collected 1 item*"])
    result.stdout.fnmatch_lines(["*1 passed*"])
    assert result.ret == ExitCode.OK

    result = pytester.runpytest("-k nonmatch")
    result.stdout.fnmatch_lines(["*collected 1 item*"])
    result.stdout.fnmatch_lines(["*1 deselected*"])
    assert result.ret == ExitCode.NO_TESTS_COLLECTED


def test_exception_printing_skip() -> None:
    assert pytest.skip.Exception == pytest.skip.Exception
    try:
        pytest.skip("hello")
    except pytest.skip.Exception:
        excinfo = ExceptionInfo.from_current()
        s = excinfo.exconly(tryshort=True)
        assert s.startswith("Skipped")


def test_importorskip(monkeypatch) -> None:
    importorskip = pytest.importorskip

    def f():
        importorskip("asdlkj")

    try:
        sysmod = importorskip("sys")
        assert sysmod is sys
        # path = pytest.importorskip("os.path")
        # assert path == os.path
        excinfo = pytest.raises(pytest.skip.Exception, f)
        assert excinfo is not None
        excrepr = excinfo.getrepr()
        assert excrepr is not None
        assert excrepr.reprcrash is not None
        path = Path(excrepr.reprcrash.path)
        # check that importorskip reports the actual call
        # in this test the test_runner.py file
        assert path.stem == "test_runner"
        pytest.raises(SyntaxError, pytest.importorskip, "x y z")
        pytest.raises(SyntaxError, pytest.importorskip, "x=y")
        mod = types.ModuleType("hello123")
        mod.__version__ = "1.3"  # type: ignore
        monkeypatch.setitem(sys.modules, "hello123", mod)
        with pytest.raises(pytest.skip.Exception):
            pytest.importorskip("hello123", minversion="1.3.1")
        mod2 = pytest.importorskip("hello123", minversion="1.3")
        assert mod2 == mod
    except pytest.skip.Exception:  # pragma: no cover
        assert False, f"spurious skip: {ExceptionInfo.from_current()}"


def test_importorskip_imports_last_module_part() -> None:
    ospath = pytest.importorskip("os.path")
    assert os.path == ospath


class TestImportOrSkipExcType:
    """Tests for #11523."""

    def test_no_warning(self) -> None:
        # An attempt on a module which does not exist will raise ModuleNotFoundError, so it will
        # be skipped normally and no warning will be issued.
        with warnings.catch_warnings(record=True) as captured:
            warnings.simplefilter("always")

            with pytest.raises(pytest.skip.Exception):
                pytest.importorskip("TestImportOrSkipExcType_test_no_warning")

        assert captured == []

    def test_import_error_with_warning(self, pytester: Pytester) -> None:
        # Create a module which exists and can be imported, however it raises
        # ImportError due to some other problem. In this case we will issue a warning
        # about the future behavior change.
        fn = pytester.makepyfile("raise ImportError('some specific problem')")
        pytester.syspathinsert()

        with warnings.catch_warnings(record=True) as captured:
            warnings.simplefilter("always")

            with pytest.raises(pytest.skip.Exception):
                pytest.importorskip(fn.stem)

        [warning] = captured
        assert warning.category is pytest.PytestDeprecationWarning

    def test_import_error_suppress_warning(self, pytester: Pytester) -> None:
        # Same as test_import_error_with_warning, but we can suppress the warning
        # by passing ImportError as exc_type.
        fn = pytester.makepyfile("raise ImportError('some specific problem')")
        pytester.syspathinsert()

        with warnings.catch_warnings(record=True) as captured:
            warnings.simplefilter("always")

            with pytest.raises(pytest.skip.Exception):
                pytest.importorskip(fn.stem, exc_type=ImportError)

        assert captured == []

    def test_warning_integration(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            def test_foo():
                pytest.importorskip("warning_integration_module")
            """
        )
        pytester.makepyfile(
            warning_integration_module="""
                raise ImportError("required library foobar not compiled properly")
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*Module 'warning_integration_module' was found, but when imported by pytest it raised:",
                "*      ImportError('required library foobar not compiled properly')",
                "*1 skipped, 1 warning*",
            ]
        )


def test_importorskip_dev_module(monkeypatch) -> None:
    try:
        mod = types.ModuleType("mockmodule")
        mod.__version__ = "0.13.0.dev-43290"  # type: ignore
        monkeypatch.setitem(sys.modules, "mockmodule", mod)
        mod2 = pytest.importorskip("mockmodule", minversion="0.12.0")
        assert mod2 == mod
        with pytest.raises(pytest.skip.Exception):
            pytest.importorskip("mockmodule1", minversion="0.14.0")
    except pytest.skip.Exception:  # pragma: no cover
        assert False, f"spurious skip: {ExceptionInfo.from_current()}"


def test_importorskip_module_level(pytester: Pytester) -> None:
    """`importorskip` must be able to skip entire modules when used at module level."""
    pytester.makepyfile(
        """
        import pytest
        foobarbaz = pytest.importorskip("foobarbaz")

        def test_foo():
            pass
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*collected 0 items / 1 skipped*"])


def test_importorskip_custom_reason(pytester: Pytester) -> None:
    """Make sure custom reasons are used."""
    pytester.makepyfile(
        """
        import pytest
        foobarbaz = pytest.importorskip("foobarbaz2", reason="just because")

        def test_foo():
            pass
    """
    )
    result = pytester.runpytest("-ra")
    result.stdout.fnmatch_lines(["*just because*"])
    result.stdout.fnmatch_lines(["*collected 0 items / 1 skipped*"])


def test_pytest_cmdline_main(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest
        def test_hello():
            assert 1
        if __name__ == '__main__':
           pytest.cmdline.main([__file__])
    """
    )
    import subprocess

    popen = subprocess.Popen([sys.executable, str(p)], stdout=subprocess.PIPE)
    popen.communicate()
    ret = popen.wait()
    assert ret == 0


def test_unicode_in_longrepr(pytester: Pytester) -> None:
    pytester.makeconftest(
        """\
        import pytest
        @pytest.hookimpl(wrapper=True)
        def pytest_runtest_makereport():
            rep = yield
            if rep.when == "call":
                rep.longrepr = 'ä'
            return rep
        """
    )
    pytester.makepyfile(
        """
        def test_out():
            assert 0
    """
    )
    result = pytester.runpytest()
    assert result.ret == 1
    assert "UnicodeEncodeError" not in result.stderr.str()


def test_failure_in_setup(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        def setup_module():
            0/0
        def test_func():
            pass
    """
    )
    result = pytester.runpytest("--tb=line")
    result.stdout.no_fnmatch_line("*def setup_module*")


def test_makereport_getsource(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        def test_foo():
            if False: pass
            else: assert False
    """
    )
    result = pytester.runpytest()
    result.stdout.no_fnmatch_line("*INTERNALERROR*")
    result.stdout.fnmatch_lines(["*else: assert False*"])


def test_makereport_getsource_dynamic_code(
    pytester: Pytester, monkeypatch: MonkeyPatch
) -> None:
    """Test that exception in dynamically generated code doesn't break getting the source line."""
    import inspect

    original_findsource = inspect.findsource

    def findsource(obj):
        # Can be triggered by dynamically created functions
        if obj.__name__ == "foo":
            raise IndexError()
        return original_findsource(obj)

    monkeypatch.setattr(inspect, "findsource", findsource)

    pytester.makepyfile(
        """
        import pytest

        @pytest.fixture
        def foo(missing):
            pass

        def test_fix(foo):
            assert False
    """
    )
    result = pytester.runpytest("-vv")
    result.stdout.no_fnmatch_line("*INTERNALERROR*")
    result.stdout.fnmatch_lines(["*test_fix*", "*fixture*'missing'*not found*"])


def test_store_except_info_on_error() -> None:
    """Test that upon test failure, the exception info is stored on
    sys.last_traceback and friends."""

    # Simulate item that might raise a specific exception, depending on `raise_error` class var
    class ItemMightRaise:
        nodeid = "item_that_raises"
        raise_error = True

        def runtest(self):
            if self.raise_error:
                raise IndexError("TEST")

    try:
        runner.pytest_runtest_call(ItemMightRaise())  # type: ignore[arg-type]
    except IndexError:
        pass
    # Check that exception info is stored on sys
    assert sys.last_type is IndexError
    assert isinstance(sys.last_value, IndexError)
    if sys.version_info >= (3, 12, 0):
        assert isinstance(sys.last_exc, IndexError)  # type: ignore[attr-defined]

    assert sys.last_value.args[0] == "TEST"
    assert sys.last_traceback

    # The next run should clear the exception info stored by the previous run
    ItemMightRaise.raise_error = False
    runner.pytest_runtest_call(ItemMightRaise())  # type: ignore[arg-type]
    assert not hasattr(sys, "last_type")
    assert not hasattr(sys, "last_value")
    if sys.version_info >= (3, 12, 0):
        assert not hasattr(sys, "last_exc")
    assert not hasattr(sys, "last_traceback")


def test_current_test_env_var(pytester: Pytester, monkeypatch: MonkeyPatch) -> None:
    pytest_current_test_vars: List[Tuple[str, str]] = []
    monkeypatch.setattr(
        sys, "pytest_current_test_vars", pytest_current_test_vars, raising=False
    )
    pytester.makepyfile(
        """
        import pytest
        import sys
        import os

        @pytest.fixture
        def fix():
            sys.pytest_current_test_vars.append(('setup', os.environ['PYTEST_CURRENT_TEST']))
            yield
            sys.pytest_current_test_vars.append(('teardown', os.environ['PYTEST_CURRENT_TEST']))

        def test(fix):
            sys.pytest_current_test_vars.append(('call', os.environ['PYTEST_CURRENT_TEST']))
    """
    )
    result = pytester.runpytest_inprocess()
    assert result.ret == 0
    test_id = "test_current_test_env_var.py::test"
    assert pytest_current_test_vars == [
        ("setup", test_id + " (setup)"),
        ("call", test_id + " (call)"),
        ("teardown", test_id + " (teardown)"),
    ]
    assert "PYTEST_CURRENT_TEST" not in os.environ


class TestReportContents:
    """Test user-level API of ``TestReport`` objects."""

    def getrunner(self):
        return lambda item: runner.runtestprotocol(item, log=False)

    def test_longreprtext_pass(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            def test_func():
                pass
        """
        )
        rep = reports[1]
        assert rep.longreprtext == ""

    def test_longreprtext_skip(self, pytester: Pytester) -> None:
        """TestReport.longreprtext can handle non-str ``longrepr`` attributes (#7559)"""
        reports = pytester.runitem(
            """
            import pytest
            def test_func():
                pytest.skip()
            """
        )
        _, call_rep, _ = reports
        assert isinstance(call_rep.longrepr, tuple)
        assert "Skipped" in call_rep.longreprtext

    def test_longreprtext_collect_skip(self, pytester: Pytester) -> None:
        """CollectReport.longreprtext can handle non-str ``longrepr`` attributes (#7559)"""
        pytester.makepyfile(
            """
            import pytest
            pytest.skip(allow_module_level=True)
            """
        )
        rec = pytester.inline_run()
        calls = rec.getcalls("pytest_collectreport")
        _, call, _ = calls
        assert isinstance(call.report.longrepr, tuple)
        assert "Skipped" in call.report.longreprtext

    def test_longreprtext_failure(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            def test_func():
                x = 1
                assert x == 4
        """
        )
        rep = reports[1]
        assert "assert 1 == 4" in rep.longreprtext

    def test_captured_text(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            import pytest
            import sys

            @pytest.fixture
            def fix():
                sys.stdout.write('setup: stdout\\n')
                sys.stderr.write('setup: stderr\\n')
                yield
                sys.stdout.write('teardown: stdout\\n')
                sys.stderr.write('teardown: stderr\\n')
                assert 0

            def test_func(fix):
                sys.stdout.write('call: stdout\\n')
                sys.stderr.write('call: stderr\\n')
                assert 0
        """
        )
        setup, call, teardown = reports
        assert setup.capstdout == "setup: stdout\n"
        assert call.capstdout == "setup: stdout\ncall: stdout\n"
        assert teardown.capstdout == "setup: stdout\ncall: stdout\nteardown: stdout\n"

        assert setup.capstderr == "setup: stderr\n"
        assert call.capstderr == "setup: stderr\ncall: stderr\n"
        assert teardown.capstderr == "setup: stderr\ncall: stderr\nteardown: stderr\n"

    def test_no_captured_text(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            def test_func():
                pass
        """
        )
        rep = reports[1]
        assert rep.capstdout == ""
        assert rep.capstderr == ""

    def test_longrepr_type(self, pytester: Pytester) -> None:
        reports = pytester.runitem(
            """
            import pytest
            def test_func():
                pytest.fail(pytrace=False)
        """
        )
        rep = reports[1]
        assert isinstance(rep.longrepr, ExceptionChainRepr)


def test_outcome_exception_bad_msg() -> None:
    """Check that OutcomeExceptions validate their input to prevent confusing errors (#5578)"""

    def func() -> None:
        raise NotImplementedError()

    expected = (
        "OutcomeException expected string as 'msg' parameter, got 'function' instead.\n"
        "Perhaps you meant to use a mark?"
    )
    with pytest.raises(TypeError) as excinfo:
        OutcomeException(func)  # type: ignore
    assert str(excinfo.value) == expected


def test_pytest_version_env_var(pytester: Pytester, monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setenv("PYTEST_VERSION", "old version")
    pytester.makepyfile(
        """
        import pytest
        import os


        def test():
            assert os.environ.get("PYTEST_VERSION") == pytest.__version__
    """
    )
    result = pytester.runpytest_inprocess()
    assert result.ret == ExitCode.OK
    assert os.environ["PYTEST_VERSION"] == "old version"
