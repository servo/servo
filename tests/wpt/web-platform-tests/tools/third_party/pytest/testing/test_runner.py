# -*- coding: utf-8 -*-
from __future__ import absolute_import, division, print_function

import _pytest._code
import inspect
import os
import py
import pytest
import sys
import types
from _pytest import runner, main, outcomes


class TestSetupState(object):

    def test_setup(self, testdir):
        ss = runner.SetupState()
        item = testdir.getitem("def test_func(): pass")
        values = [1]
        ss.prepare(item)
        ss.addfinalizer(values.pop, colitem=item)
        assert values
        ss._pop_and_teardown()
        assert not values

    def test_teardown_exact_stack_empty(self, testdir):
        item = testdir.getitem("def test_func(): pass")
        ss = runner.SetupState()
        ss.teardown_exact(item, None)
        ss.teardown_exact(item, None)
        ss.teardown_exact(item, None)

    def test_setup_fails_and_failure_is_cached(self, testdir):
        item = testdir.getitem(
            """
            def setup_module(mod):
                raise ValueError(42)
            def test_func(): pass
        """
        )
        ss = runner.SetupState()
        pytest.raises(ValueError, lambda: ss.prepare(item))
        pytest.raises(ValueError, lambda: ss.prepare(item))

    def test_teardown_multiple_one_fails(self, testdir):
        r = []

        def fin1():
            r.append("fin1")

        def fin2():
            raise Exception("oops")

        def fin3():
            r.append("fin3")

        item = testdir.getitem("def test_func(): pass")
        ss = runner.SetupState()
        ss.addfinalizer(fin1, item)
        ss.addfinalizer(fin2, item)
        ss.addfinalizer(fin3, item)
        with pytest.raises(Exception) as err:
            ss._callfinalizers(item)
        assert err.value.args == ("oops",)
        assert r == ["fin3", "fin1"]

    def test_teardown_multiple_fail(self, testdir):
        # Ensure the first exception is the one which is re-raised.
        # Ideally both would be reported however.
        def fin1():
            raise Exception("oops1")

        def fin2():
            raise Exception("oops2")

        item = testdir.getitem("def test_func(): pass")
        ss = runner.SetupState()
        ss.addfinalizer(fin1, item)
        ss.addfinalizer(fin2, item)
        with pytest.raises(Exception) as err:
            ss._callfinalizers(item)
        assert err.value.args == ("oops2",)

    def test_teardown_multiple_scopes_one_fails(self, testdir):
        module_teardown = []

        def fin_func():
            raise Exception("oops1")

        def fin_module():
            module_teardown.append("fin_module")

        item = testdir.getitem("def test_func(): pass")
        ss = runner.SetupState()
        ss.addfinalizer(fin_module, item.listchain()[-2])
        ss.addfinalizer(fin_func, item)
        ss.prepare(item)
        with pytest.raises(Exception, match="oops1"):
            ss.teardown_exact(item, None)
        assert module_teardown


class BaseFunctionalTests(object):

    def test_passfunction(self, testdir):
        reports = testdir.runitem(
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

    def test_failfunction(self, testdir):
        reports = testdir.runitem(
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

    def test_skipfunction(self, testdir):
        reports = testdir.runitem(
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

    def test_skip_in_setup_function(self, testdir):
        reports = testdir.runitem(
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

    def test_failure_in_setup_function(self, testdir):
        reports = testdir.runitem(
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

    def test_failure_in_teardown_function(self, testdir):
        reports = testdir.runitem(
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

    def test_custom_failure_repr(self, testdir):
        testdir.makepyfile(
            conftest="""
            import pytest
            class Function(pytest.Function):
                def repr_failure(self, excinfo):
                    return "hello"
        """
        )
        reports = testdir.runitem(
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

    def test_teardown_final_returncode(self, testdir):
        rec = testdir.inline_runsource(
            """
            def test_func():
                pass
            def teardown_function(func):
                raise ValueError(42)
        """
        )
        assert rec.ret == 1

    def test_logstart_logfinish_hooks(self, testdir):
        rec = testdir.inline_runsource(
            """
            import pytest
            def test_func():
                pass
        """
        )
        reps = rec.getcalls("pytest_runtest_logstart pytest_runtest_logfinish")
        assert (
            [x._name for x in reps]
            == ["pytest_runtest_logstart", "pytest_runtest_logfinish"]
        )
        for rep in reps:
            assert rep.nodeid == "test_logstart_logfinish_hooks.py::test_func"
            assert rep.location == ("test_logstart_logfinish_hooks.py", 1, "test_func")

    def test_exact_teardown_issue90(self, testdir):
        rec = testdir.inline_runsource(
            """
            import pytest

            class TestClass(object):
                def test_method(self):
                    pass
                def teardown_class(cls):
                    raise Exception()

            def test_func():
                import sys
                # on python2 exc_info is keept till a function exits
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

    def test_exact_teardown_issue1206(self, testdir):
        """issue shadowing error with wrong number of arguments on teardown_method."""
        rec = testdir.inline_runsource(
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
        assert reps[2].longrepr.reprcrash.message in (
            # python3 error
            "TypeError: teardown_method() missing 2 required positional arguments: 'y' and 'z'",
            # python2 error
            "TypeError: teardown_method() takes exactly 4 arguments (2 given)",
        )

    def test_failure_in_setup_function_ignores_custom_repr(self, testdir):
        testdir.makepyfile(
            conftest="""
            import pytest
            class Function(pytest.Function):
                def repr_failure(self, excinfo):
                    assert 0
        """
        )
        reports = testdir.runitem(
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
        # assert instanace(rep.failed.failurerepr, PythonFailureRepr)

    def test_systemexit_does_not_bail_out(self, testdir):
        try:
            reports = testdir.runitem(
                """
                def test_func():
                    raise SystemExit(42)
            """
            )
        except SystemExit:
            pytest.fail("runner did not catch SystemExit")
        rep = reports[1]
        assert rep.failed
        assert rep.when == "call"

    def test_exit_propagates(self, testdir):
        try:
            testdir.runitem(
                """
                import pytest
                def test_func():
                    raise pytest.exit.Exception()
            """
            )
        except pytest.exit.Exception:
            pass
        else:
            pytest.fail("did not raise")


class TestExecutionNonForked(BaseFunctionalTests):

    def getrunner(self):

        def f(item):
            return runner.runtestprotocol(item, log=False)

        return f

    def test_keyboardinterrupt_propagates(self, testdir):
        try:
            testdir.runitem(
                """
                def test_func():
                    raise KeyboardInterrupt("fake")
            """
            )
        except KeyboardInterrupt:
            pass
        else:
            pytest.fail("did not raise")


class TestExecutionForked(BaseFunctionalTests):
    pytestmark = pytest.mark.skipif("not hasattr(os, 'fork')")

    def getrunner(self):
        # XXX re-arrange this test to live in pytest-xdist
        boxed = pytest.importorskip("xdist.boxed")
        return boxed.forked_run_report

    def test_suicide(self, testdir):
        reports = testdir.runitem(
            """
            def test_func():
                import os
                os.kill(os.getpid(), 15)
        """
        )
        rep = reports[0]
        assert rep.failed
        assert rep.when == "???"


class TestSessionReports(object):

    def test_collect_result(self, testdir):
        col = testdir.getmodulecol(
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
        assert locinfo[0] == col.fspath.basename
        assert not locinfo[1]
        assert locinfo[2] == col.fspath.basename
        res = rep.result
        assert len(res) == 2
        assert res[0].name == "test_func1"
        assert res[1].name == "TestClass"


reporttypes = [
    runner.BaseReport,
    runner.TestReport,
    runner.TeardownErrorReport,
    runner.CollectReport,
]


@pytest.mark.parametrize(
    "reporttype", reporttypes, ids=[x.__name__ for x in reporttypes]
)
def test_report_extra_parameters(reporttype):
    if hasattr(inspect, "signature"):
        args = list(inspect.signature(reporttype.__init__).parameters.keys())[1:]
    else:
        args = inspect.getargspec(reporttype.__init__)[0][1:]
    basekw = dict.fromkeys(args, [])
    report = reporttype(newthing=1, **basekw)
    assert report.newthing == 1


def test_callinfo():
    ci = runner.CallInfo(lambda: 0, "123")
    assert ci.when == "123"
    assert ci.result == 0
    assert "result" in repr(ci)
    ci = runner.CallInfo(lambda: 0 / 0, "123")
    assert ci.when == "123"
    assert not hasattr(ci, "result")
    assert ci.excinfo
    assert "exc" in repr(ci)


# design question: do we want general hooks in python files?
# then something like the following functional tests makes sense


@pytest.mark.xfail
def test_runtest_in_module_ordering(testdir):
    p1 = testdir.makepyfile(
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
            def pytest_runtest_call(self, item, __multicall__):
                try:
                    __multicall__.execute()
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
    result = testdir.runpytest(p1)
    result.stdout.fnmatch_lines(["*2 passed*"])


def test_outcomeexception_exceptionattributes():
    outcome = outcomes.OutcomeException("test")
    assert outcome.args[0] == outcome.msg


def test_outcomeexception_passes_except_Exception():
    with pytest.raises(outcomes.OutcomeException):
        try:
            raise outcomes.OutcomeException("test")
        except Exception:
            pass


def test_pytest_exit():
    try:
        pytest.exit("hello")
    except pytest.exit.Exception:
        excinfo = _pytest._code.ExceptionInfo()
        assert excinfo.errisinstance(KeyboardInterrupt)


def test_pytest_fail():
    try:
        pytest.fail("hello")
    except pytest.fail.Exception:
        excinfo = _pytest._code.ExceptionInfo()
        s = excinfo.exconly(tryshort=True)
        assert s.startswith("Failed")


def test_pytest_exit_msg(testdir):
    testdir.makeconftest(
        """
    import pytest

    def pytest_configure(config):
        pytest.exit('oh noes')
    """
    )
    result = testdir.runpytest()
    result.stderr.fnmatch_lines(["Exit: oh noes"])


def test_pytest_fail_notrace(testdir):
    testdir.makepyfile(
        """
        import pytest
        def test_hello():
            pytest.fail("hello", pytrace=False)
        def teardown_function(function):
            pytest.fail("world", pytrace=False)
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["world", "hello"])
    assert "def teardown_function" not in result.stdout.str()


@pytest.mark.parametrize("str_prefix", ["u", ""])
def test_pytest_fail_notrace_non_ascii(testdir, str_prefix):
    """Fix pytest.fail with pytrace=False with non-ascii characters (#1178).

    This tests with native and unicode strings containing non-ascii chars.
    """
    testdir.makepyfile(
        u"""
        # coding: utf-8
        import pytest

        def test_hello():
            pytest.fail(%s'oh oh: ☺', pytrace=False)
    """
        % str_prefix
    )
    result = testdir.runpytest()
    if sys.version_info[0] >= 3:
        result.stdout.fnmatch_lines(["*test_hello*", "oh oh: ☺"])
    else:
        result.stdout.fnmatch_lines(["*test_hello*", "oh oh: *"])
    assert "def test_hello" not in result.stdout.str()


def test_pytest_no_tests_collected_exit_status(testdir):
    result = testdir.runpytest()
    result.stdout.fnmatch_lines("*collected 0 items*")
    assert result.ret == main.EXIT_NOTESTSCOLLECTED

    testdir.makepyfile(
        test_foo="""
        def test_foo():
            assert 1
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines("*collected 1 item*")
    result.stdout.fnmatch_lines("*1 passed*")
    assert result.ret == main.EXIT_OK

    result = testdir.runpytest("-k nonmatch")
    result.stdout.fnmatch_lines("*collected 1 item*")
    result.stdout.fnmatch_lines("*1 deselected*")
    assert result.ret == main.EXIT_NOTESTSCOLLECTED


def test_exception_printing_skip():
    try:
        pytest.skip("hello")
    except pytest.skip.Exception:
        excinfo = _pytest._code.ExceptionInfo()
        s = excinfo.exconly(tryshort=True)
        assert s.startswith("Skipped")


def test_importorskip(monkeypatch):
    importorskip = pytest.importorskip

    def f():
        importorskip("asdlkj")

    try:
        sysmod = importorskip("sys")
        assert sysmod is sys
        # path = pytest.importorskip("os.path")
        # assert path == os.path
        excinfo = pytest.raises(pytest.skip.Exception, f)
        path = py.path.local(excinfo.getrepr().reprcrash.path)
        # check that importorskip reports the actual call
        # in this test the test_runner.py file
        assert path.purebasename == "test_runner"
        pytest.raises(SyntaxError, "pytest.importorskip('x y z')")
        pytest.raises(SyntaxError, "pytest.importorskip('x=y')")
        mod = types.ModuleType("hello123")
        mod.__version__ = "1.3"
        monkeypatch.setitem(sys.modules, "hello123", mod)
        pytest.raises(
            pytest.skip.Exception,
            """
            pytest.importorskip("hello123", minversion="1.3.1")
        """,
        )
        mod2 = pytest.importorskip("hello123", minversion="1.3")
        assert mod2 == mod
    except pytest.skip.Exception:
        print(_pytest._code.ExceptionInfo())
        pytest.fail("spurious skip")


def test_importorskip_imports_last_module_part():
    ospath = pytest.importorskip("os.path")
    assert os.path == ospath


def test_importorskip_dev_module(monkeypatch):
    try:
        mod = types.ModuleType("mockmodule")
        mod.__version__ = "0.13.0.dev-43290"
        monkeypatch.setitem(sys.modules, "mockmodule", mod)
        mod2 = pytest.importorskip("mockmodule", minversion="0.12.0")
        assert mod2 == mod
        pytest.raises(
            pytest.skip.Exception,
            """
            pytest.importorskip('mockmodule1', minversion='0.14.0')""",
        )
    except pytest.skip.Exception:
        print(_pytest._code.ExceptionInfo())
        pytest.fail("spurious skip")


def test_importorskip_module_level(testdir):
    """importorskip must be able to skip entire modules when used at module level"""
    testdir.makepyfile(
        """
        import pytest
        foobarbaz = pytest.importorskip("foobarbaz")

        def test_foo():
            pass
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*collected 0 items / 1 skipped*"])


def test_pytest_cmdline_main(testdir):
    p = testdir.makepyfile(
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


def test_unicode_in_longrepr(testdir):
    testdir.makeconftest(
        """
        # -*- coding: utf-8 -*-
        import pytest
        @pytest.hookimpl(hookwrapper=True)
        def pytest_runtest_makereport():
            outcome = yield
            rep = outcome.get_result()
            if rep.when == "call":
                rep.longrepr = u'ä'
    """
    )
    testdir.makepyfile(
        """
        def test_out():
            assert 0
    """
    )
    result = testdir.runpytest()
    assert result.ret == 1
    assert "UnicodeEncodeError" not in result.stderr.str()


def test_failure_in_setup(testdir):
    testdir.makepyfile(
        """
        def setup_module():
            0/0
        def test_func():
            pass
    """
    )
    result = testdir.runpytest("--tb=line")
    assert "def setup_module" not in result.stdout.str()


def test_makereport_getsource(testdir):
    testdir.makepyfile(
        """
        def test_foo():
            if False: pass
            else: assert False
    """
    )
    result = testdir.runpytest()
    assert "INTERNALERROR" not in result.stdout.str()
    result.stdout.fnmatch_lines(["*else: assert False*"])


def test_makereport_getsource_dynamic_code(testdir, monkeypatch):
    """Test that exception in dynamically generated code doesn't break getting the source line."""
    import inspect

    original_findsource = inspect.findsource

    def findsource(obj, *args, **kwargs):
        # Can be triggered by dynamically created functions
        if obj.__name__ == "foo":
            raise IndexError()
        return original_findsource(obj, *args, **kwargs)

    monkeypatch.setattr(inspect, "findsource", findsource)

    testdir.makepyfile(
        """
        import pytest

        @pytest.fixture
        def foo(missing):
            pass

        def test_fix(foo):
            assert False
    """
    )
    result = testdir.runpytest("-vv")
    assert "INTERNALERROR" not in result.stdout.str()
    result.stdout.fnmatch_lines(["*test_fix*", "*fixture*'missing'*not found*"])


def test_store_except_info_on_error():
    """ Test that upon test failure, the exception info is stored on
    sys.last_traceback and friends.
    """
    # Simulate item that might raise a specific exception, depending on `raise_error` class var
    class ItemMightRaise(object):
        nodeid = "item_that_raises"
        raise_error = True

        def runtest(self):
            if self.raise_error:
                raise IndexError("TEST")

    try:
        runner.pytest_runtest_call(ItemMightRaise())
    except IndexError:
        pass
    # Check that exception info is stored on sys
    assert sys.last_type is IndexError
    assert sys.last_value.args[0] == "TEST"
    assert sys.last_traceback

    # The next run should clear the exception info stored by the previous run
    ItemMightRaise.raise_error = False
    runner.pytest_runtest_call(ItemMightRaise())
    assert sys.last_type is None
    assert sys.last_value is None
    assert sys.last_traceback is None


def test_current_test_env_var(testdir, monkeypatch):
    pytest_current_test_vars = []
    monkeypatch.setattr(
        sys, "pytest_current_test_vars", pytest_current_test_vars, raising=False
    )
    testdir.makepyfile(
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
    result = testdir.runpytest_inprocess()
    assert result.ret == 0
    test_id = "test_current_test_env_var.py::test"
    assert (
        pytest_current_test_vars
        == [
            ("setup", test_id + " (setup)"),
            ("call", test_id + " (call)"),
            ("teardown", test_id + " (teardown)"),
        ]
    )
    assert "PYTEST_CURRENT_TEST" not in os.environ


class TestReportContents(object):
    """
    Test user-level API of ``TestReport`` objects.
    """

    def getrunner(self):
        return lambda item: runner.runtestprotocol(item, log=False)

    def test_longreprtext_pass(self, testdir):
        reports = testdir.runitem(
            """
            def test_func():
                pass
        """
        )
        rep = reports[1]
        assert rep.longreprtext == ""

    def test_longreprtext_failure(self, testdir):
        reports = testdir.runitem(
            """
            def test_func():
                x = 1
                assert x == 4
        """
        )
        rep = reports[1]
        assert "assert 1 == 4" in rep.longreprtext

    def test_captured_text(self, testdir):
        reports = testdir.runitem(
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

    def test_no_captured_text(self, testdir):
        reports = testdir.runitem(
            """
            def test_func():
                pass
        """
        )
        rep = reports[1]
        assert rep.capstdout == ""
        assert rep.capstderr == ""
