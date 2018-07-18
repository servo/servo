from __future__ import absolute_import, division, print_function
import pytest
import sys

from _pytest.skipping import MarkEvaluator, folded_skips, pytest_runtest_setup
from _pytest.runner import runtestprotocol


class TestEvaluator(object):

    def test_no_marker(self, testdir):
        item = testdir.getitem("def test_func(): pass")
        evalskipif = MarkEvaluator(item, "skipif")
        assert not evalskipif
        assert not evalskipif.istrue()

    def test_marked_no_args(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.xyz
            def test_func():
                pass
        """
        )
        ev = MarkEvaluator(item, "xyz")
        assert ev
        assert ev.istrue()
        expl = ev.getexplanation()
        assert expl == ""
        assert not ev.get("run", False)

    def test_marked_one_arg(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.xyz("hasattr(os, 'sep')")
            def test_func():
                pass
        """
        )
        ev = MarkEvaluator(item, "xyz")
        assert ev
        assert ev.istrue()
        expl = ev.getexplanation()
        assert expl == "condition: hasattr(os, 'sep')"

    @pytest.mark.skipif("sys.version_info[0] >= 3")
    def test_marked_one_arg_unicode(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.xyz(u"hasattr(os, 'sep')")
            def test_func():
                pass
        """
        )
        ev = MarkEvaluator(item, "xyz")
        assert ev
        assert ev.istrue()
        expl = ev.getexplanation()
        assert expl == "condition: hasattr(os, 'sep')"

    def test_marked_one_arg_with_reason(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.xyz("hasattr(os, 'sep')", attr=2, reason="hello world")
            def test_func():
                pass
        """
        )
        ev = MarkEvaluator(item, "xyz")
        assert ev
        assert ev.istrue()
        expl = ev.getexplanation()
        assert expl == "hello world"
        assert ev.get("attr") == 2

    def test_marked_one_arg_twice(self, testdir):
        lines = [
            """@pytest.mark.skipif("not hasattr(os, 'murks')")""",
            """@pytest.mark.skipif("hasattr(os, 'murks')")""",
        ]
        for i in range(0, 2):
            item = testdir.getitem(
                """
                import pytest
                %s
                %s
                def test_func():
                    pass
            """
                % (lines[i], lines[(i + 1) % 2])
            )
            ev = MarkEvaluator(item, "skipif")
            assert ev
            assert ev.istrue()
            expl = ev.getexplanation()
            assert expl == "condition: not hasattr(os, 'murks')"

    def test_marked_one_arg_twice2(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.skipif("hasattr(os, 'murks')")
            @pytest.mark.skipif("not hasattr(os, 'murks')")
            def test_func():
                pass
        """
        )
        ev = MarkEvaluator(item, "skipif")
        assert ev
        assert ev.istrue()
        expl = ev.getexplanation()
        assert expl == "condition: not hasattr(os, 'murks')"

    def test_marked_skip_with_not_string(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.skipif(False)
            def test_func():
                pass
        """
        )
        ev = MarkEvaluator(item, "skipif")
        exc = pytest.raises(pytest.fail.Exception, ev.istrue)
        assert """Failed: you need to specify reason=STRING when using booleans as conditions.""" in exc.value.msg

    def test_skipif_class(self, testdir):
        item, = testdir.getitems(
            """
            import pytest
            class TestClass(object):
                pytestmark = pytest.mark.skipif("config._hackxyz")
                def test_func(self):
                    pass
        """
        )
        item.config._hackxyz = 3
        ev = MarkEvaluator(item, "skipif")
        assert ev.istrue()
        expl = ev.getexplanation()
        assert expl == "condition: config._hackxyz"


class TestXFail(object):

    @pytest.mark.parametrize("strict", [True, False])
    def test_xfail_simple(self, testdir, strict):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.xfail(strict=%s)
            def test_func():
                assert 0
        """
            % strict
        )
        reports = runtestprotocol(item, log=False)
        assert len(reports) == 3
        callreport = reports[1]
        assert callreport.skipped
        assert callreport.wasxfail == ""

    def test_xfail_xpassed(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.xfail(reason="this is an xfail")
            def test_func():
                assert 1
        """
        )
        reports = runtestprotocol(item, log=False)
        assert len(reports) == 3
        callreport = reports[1]
        assert callreport.passed
        assert callreport.wasxfail == "this is an xfail"

    def test_xfail_using_platform(self, testdir):
        """
        Verify that platform can be used with xfail statements.
        """
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.xfail("platform.platform() == platform.platform()")
            def test_func():
                assert 0
        """
        )
        reports = runtestprotocol(item, log=False)
        assert len(reports) == 3
        callreport = reports[1]
        assert callreport.wasxfail

    def test_xfail_xpassed_strict(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.xfail(strict=True, reason="nope")
            def test_func():
                assert 1
        """
        )
        reports = runtestprotocol(item, log=False)
        assert len(reports) == 3
        callreport = reports[1]
        assert callreport.failed
        assert callreport.longrepr == "[XPASS(strict)] nope"
        assert not hasattr(callreport, "wasxfail")

    def test_xfail_run_anyway(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail
            def test_func():
                assert 0
            def test_func2():
                pytest.xfail("hello")
        """
        )
        result = testdir.runpytest("--runxfail")
        result.stdout.fnmatch_lines(
            ["*def test_func():*", "*assert 0*", "*1 failed*1 pass*"]
        )

    def test_xfail_evalfalse_but_fails(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.xfail('False')
            def test_func():
                assert 0
        """
        )
        reports = runtestprotocol(item, log=False)
        callreport = reports[1]
        assert callreport.failed
        assert not hasattr(callreport, "wasxfail")
        assert "xfail" in callreport.keywords

    def test_xfail_not_report_default(self, testdir):
        p = testdir.makepyfile(
            test_one="""
            import pytest
            @pytest.mark.xfail
            def test_this():
                assert 0
        """
        )
        testdir.runpytest(p, "-v")
        # result.stdout.fnmatch_lines([
        #    "*HINT*use*-r*"
        # ])

    def test_xfail_not_run_xfail_reporting(self, testdir):
        p = testdir.makepyfile(
            test_one="""
            import pytest
            @pytest.mark.xfail(run=False, reason="noway")
            def test_this():
                assert 0
            @pytest.mark.xfail("True", run=False)
            def test_this_true():
                assert 0
            @pytest.mark.xfail("False", run=False, reason="huh")
            def test_this_false():
                assert 1
        """
        )
        result = testdir.runpytest(p, "-rx")
        result.stdout.fnmatch_lines(
            [
                "*test_one*test_this*",
                "*NOTRUN*noway",
                "*test_one*test_this_true*",
                "*NOTRUN*condition:*True*",
                "*1 passed*",
            ]
        )

    def test_xfail_not_run_no_setup_run(self, testdir):
        p = testdir.makepyfile(
            test_one="""
            import pytest
            @pytest.mark.xfail(run=False, reason="hello")
            def test_this():
                assert 0
            def setup_module(mod):
                raise ValueError(42)
        """
        )
        result = testdir.runpytest(p, "-rx")
        result.stdout.fnmatch_lines(
            ["*test_one*test_this*", "*NOTRUN*hello", "*1 xfailed*"]
        )

    def test_xfail_xpass(self, testdir):
        p = testdir.makepyfile(
            test_one="""
            import pytest
            @pytest.mark.xfail
            def test_that():
                assert 1
        """
        )
        result = testdir.runpytest(p, "-rX")
        result.stdout.fnmatch_lines(["*XPASS*test_that*", "*1 xpassed*"])
        assert result.ret == 0

    def test_xfail_imperative(self, testdir):
        p = testdir.makepyfile(
            """
            import pytest
            def test_this():
                pytest.xfail("hello")
        """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(["*1 xfailed*"])
        result = testdir.runpytest(p, "-rx")
        result.stdout.fnmatch_lines(["*XFAIL*test_this*", "*reason:*hello*"])
        result = testdir.runpytest(p, "--runxfail")
        result.stdout.fnmatch_lines("*1 pass*")

    def test_xfail_imperative_in_setup_function(self, testdir):
        p = testdir.makepyfile(
            """
            import pytest
            def setup_function(function):
                pytest.xfail("hello")

            def test_this():
                assert 0
        """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(["*1 xfailed*"])
        result = testdir.runpytest(p, "-rx")
        result.stdout.fnmatch_lines(["*XFAIL*test_this*", "*reason:*hello*"])
        result = testdir.runpytest(p, "--runxfail")
        result.stdout.fnmatch_lines(
            """
            *def test_this*
            *1 fail*
        """
        )

    def xtest_dynamic_xfail_set_during_setup(self, testdir):
        p = testdir.makepyfile(
            """
            import pytest
            def setup_function(function):
                pytest.mark.xfail(function)
            def test_this():
                assert 0
            def test_that():
                assert 1
        """
        )
        result = testdir.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines(["*XFAIL*test_this*", "*XPASS*test_that*"])

    def test_dynamic_xfail_no_run(self, testdir):
        p = testdir.makepyfile(
            """
            import pytest
            @pytest.fixture
            def arg(request):
                request.applymarker(pytest.mark.xfail(run=False))
            def test_this(arg):
                assert 0
        """
        )
        result = testdir.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines(["*XFAIL*test_this*", "*NOTRUN*"])

    def test_dynamic_xfail_set_during_funcarg_setup(self, testdir):
        p = testdir.makepyfile(
            """
            import pytest
            @pytest.fixture
            def arg(request):
                request.applymarker(pytest.mark.xfail)
            def test_this2(arg):
                assert 0
        """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(["*1 xfailed*"])

    @pytest.mark.parametrize(
        "expected, actual, matchline",
        [
            ("TypeError", "TypeError", "*1 xfailed*"),
            ("(AttributeError, TypeError)", "TypeError", "*1 xfailed*"),
            ("TypeError", "IndexError", "*1 failed*"),
            ("(AttributeError, TypeError)", "IndexError", "*1 failed*"),
        ],
    )
    def test_xfail_raises(self, expected, actual, matchline, testdir):
        p = testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(raises=%s)
            def test_raises():
                raise %s()
        """
            % (expected, actual)
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines([matchline])

    def test_strict_sanity(self, testdir):
        """sanity check for xfail(strict=True): a failing test should behave
        exactly like a normal xfail.
        """
        p = testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(reason='unsupported feature', strict=True)
            def test_foo():
                assert 0
        """
        )
        result = testdir.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines(["*XFAIL*", "*unsupported feature*"])
        assert result.ret == 0

    @pytest.mark.parametrize("strict", [True, False])
    def test_strict_xfail(self, testdir, strict):
        p = testdir.makepyfile(
            """
            import pytest

            @pytest.mark.xfail(reason='unsupported feature', strict=%s)
            def test_foo():
                with open('foo_executed', 'w'): pass  # make sure test executes
        """
            % strict
        )
        result = testdir.runpytest(p, "-rxX")
        if strict:
            result.stdout.fnmatch_lines(
                ["*test_foo*", "*XPASS(strict)*unsupported feature*"]
            )
        else:
            result.stdout.fnmatch_lines(
                [
                    "*test_strict_xfail*",
                    "XPASS test_strict_xfail.py::test_foo unsupported feature",
                ]
            )
        assert result.ret == (1 if strict else 0)
        assert testdir.tmpdir.join("foo_executed").isfile()

    @pytest.mark.parametrize("strict", [True, False])
    def test_strict_xfail_condition(self, testdir, strict):
        p = testdir.makepyfile(
            """
            import pytest

            @pytest.mark.xfail(False, reason='unsupported feature', strict=%s)
            def test_foo():
                pass
        """
            % strict
        )
        result = testdir.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines("*1 passed*")
        assert result.ret == 0

    @pytest.mark.parametrize("strict", [True, False])
    def test_xfail_condition_keyword(self, testdir, strict):
        p = testdir.makepyfile(
            """
            import pytest

            @pytest.mark.xfail(condition=False, reason='unsupported feature', strict=%s)
            def test_foo():
                pass
        """
            % strict
        )
        result = testdir.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines("*1 passed*")
        assert result.ret == 0

    @pytest.mark.parametrize("strict_val", ["true", "false"])
    def test_strict_xfail_default_from_file(self, testdir, strict_val):
        testdir.makeini(
            """
            [pytest]
            xfail_strict = %s
        """
            % strict_val
        )
        p = testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(reason='unsupported feature')
            def test_foo():
                pass
        """
        )
        result = testdir.runpytest(p, "-rxX")
        strict = strict_val == "true"
        result.stdout.fnmatch_lines("*1 failed*" if strict else "*1 xpassed*")
        assert result.ret == (1 if strict else 0)


class TestXFailwithSetupTeardown(object):

    def test_failing_setup_issue9(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            def setup_function(func):
                assert 0

            @pytest.mark.xfail
            def test_func():
                pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 xfail*"])

    def test_failing_teardown_issue9(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            def teardown_function(func):
                assert 0

            @pytest.mark.xfail
            def test_func():
                pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 xfail*"])


class TestSkip(object):

    def test_skip_class(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skip
            class TestSomething(object):
                def test_foo(self):
                    pass
                def test_bar(self):
                    pass

            def test_baz():
                pass
        """
        )
        rec = testdir.inline_run()
        rec.assertoutcome(skipped=2, passed=1)

    def test_skips_on_false_string(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skip('False')
            def test_foo():
                pass
        """
        )
        rec = testdir.inline_run()
        rec.assertoutcome(skipped=1)

    def test_arg_as_reason(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skip('testing stuff')
            def test_bar():
                pass
        """
        )
        result = testdir.runpytest("-rs")
        result.stdout.fnmatch_lines(["*testing stuff*", "*1 skipped*"])

    def test_skip_no_reason(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skip
            def test_foo():
                pass
        """
        )
        result = testdir.runpytest("-rs")
        result.stdout.fnmatch_lines(["*unconditional skip*", "*1 skipped*"])

    def test_skip_with_reason(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skip(reason="for lolz")
            def test_bar():
                pass
        """
        )
        result = testdir.runpytest("-rs")
        result.stdout.fnmatch_lines(["*for lolz*", "*1 skipped*"])

    def test_only_skips_marked_test(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skip
            def test_foo():
                pass
            @pytest.mark.skip(reason="nothing in particular")
            def test_bar():
                pass
            def test_baz():
                assert True
        """
        )
        result = testdir.runpytest("-rs")
        result.stdout.fnmatch_lines(["*nothing in particular*", "*1 passed*2 skipped*"])

    def test_strict_and_skip(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skip
            def test_hello():
                pass
        """
        )
        result = testdir.runpytest("-rs")
        result.stdout.fnmatch_lines(["*unconditional skip*", "*1 skipped*"])


class TestSkipif(object):

    def test_skipif_conditional(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.skipif("hasattr(os, 'sep')")
            def test_func():
                pass
        """
        )
        x = pytest.raises(pytest.skip.Exception, lambda: pytest_runtest_setup(item))
        assert x.value.msg == "condition: hasattr(os, 'sep')"

    @pytest.mark.parametrize(
        "params", ["\"hasattr(sys, 'platform')\"", 'True, reason="invalid platform"']
    )
    def test_skipif_reporting(self, testdir, params):
        p = testdir.makepyfile(
            test_foo="""
            import pytest
            @pytest.mark.skipif(%(params)s)
            def test_that():
                assert 0
        """
            % dict(params=params)
        )
        result = testdir.runpytest(p, "-s", "-rs")
        result.stdout.fnmatch_lines(["*SKIP*1*test_foo.py*platform*", "*1 skipped*"])
        assert result.ret == 0

    def test_skipif_using_platform(self, testdir):
        item = testdir.getitem(
            """
            import pytest
            @pytest.mark.skipif("platform.platform() == platform.platform()")
            def test_func():
                pass
        """
        )
        pytest.raises(pytest.skip.Exception, lambda: pytest_runtest_setup(item))

    @pytest.mark.parametrize(
        "marker, msg1, msg2",
        [("skipif", "SKIP", "skipped"), ("xfail", "XPASS", "xpassed")],
    )
    def test_skipif_reporting_multiple(self, testdir, marker, msg1, msg2):
        testdir.makepyfile(
            test_foo="""
            import pytest
            @pytest.mark.{marker}(False, reason='first_condition')
            @pytest.mark.{marker}(True, reason='second_condition')
            def test_foobar():
                assert 1
        """.format(
                marker=marker
            )
        )
        result = testdir.runpytest("-s", "-rsxX")
        result.stdout.fnmatch_lines(
            [
                "*{msg1}*test_foo.py*second_condition*".format(msg1=msg1),
                "*1 {msg2}*".format(msg2=msg2),
            ]
        )
        assert result.ret == 0


def test_skip_not_report_default(testdir):
    p = testdir.makepyfile(
        test_one="""
        import pytest
        def test_this():
            pytest.skip("hello")
    """
    )
    result = testdir.runpytest(p, "-v")
    result.stdout.fnmatch_lines(
        [
            # "*HINT*use*-r*",
            "*1 skipped*"
        ]
    )


def test_skipif_class(testdir):
    p = testdir.makepyfile(
        """
        import pytest

        class TestClass(object):
            pytestmark = pytest.mark.skipif("True")
            def test_that(self):
                assert 0
            def test_though(self):
                assert 0
    """
    )
    result = testdir.runpytest(p)
    result.stdout.fnmatch_lines(["*2 skipped*"])


def test_skip_reasons_folding():
    path = "xyz"
    lineno = 3
    message = "justso"
    longrepr = (path, lineno, message)

    class X(object):
        pass

    ev1 = X()
    ev1.when = "execute"
    ev1.skipped = True
    ev1.longrepr = longrepr

    ev2 = X()
    ev2.when = "execute"
    ev2.longrepr = longrepr
    ev2.skipped = True

    # ev3 might be a collection report
    ev3 = X()
    ev3.longrepr = longrepr
    ev3.skipped = True

    values = folded_skips([ev1, ev2, ev3])
    assert len(values) == 1
    num, fspath, lineno, reason = values[0]
    assert num == 3
    assert fspath == path
    assert lineno == lineno
    assert reason == message


def test_skipped_reasons_functional(testdir):
    testdir.makepyfile(
        test_one="""
            from conftest import doskip
            def setup_function(func):
                doskip()
            def test_func():
                pass
            class TestClass(object):
                def test_method(self):
                    doskip()
       """,
        conftest="""
            import pytest
            def doskip():
                pytest.skip('test')
        """,
    )
    result = testdir.runpytest("-rs")
    result.stdout.fnmatch_lines(["*SKIP*2*conftest.py:4: test"])
    assert result.ret == 0


def test_skipped_folding(testdir):
    testdir.makepyfile(
        test_one="""
            import pytest
            pytestmark = pytest.mark.skip("Folding")
            def setup_function(func):
                pass
            def test_func():
                pass
            class TestClass(object):
                def test_method(self):
                    pass
       """
    )
    result = testdir.runpytest("-rs")
    result.stdout.fnmatch_lines(["*SKIP*2*test_one.py: Folding"])
    assert result.ret == 0


def test_reportchars(testdir):
    testdir.makepyfile(
        """
        import pytest
        def test_1():
            assert 0
        @pytest.mark.xfail
        def test_2():
            assert 0
        @pytest.mark.xfail
        def test_3():
            pass
        def test_4():
            pytest.skip("four")
    """
    )
    result = testdir.runpytest("-rfxXs")
    result.stdout.fnmatch_lines(
        ["FAIL*test_1*", "XFAIL*test_2*", "XPASS*test_3*", "SKIP*four*"]
    )


def test_reportchars_error(testdir):
    testdir.makepyfile(
        conftest="""
        def pytest_runtest_teardown():
            assert 0
        """,
        test_simple="""
        def test_foo():
            pass
        """,
    )
    result = testdir.runpytest("-rE")
    result.stdout.fnmatch_lines(["ERROR*test_foo*"])


def test_reportchars_all(testdir):
    testdir.makepyfile(
        """
        import pytest
        def test_1():
            assert 0
        @pytest.mark.xfail
        def test_2():
            assert 0
        @pytest.mark.xfail
        def test_3():
            pass
        def test_4():
            pytest.skip("four")
    """
    )
    result = testdir.runpytest("-ra")
    result.stdout.fnmatch_lines(
        ["FAIL*test_1*", "SKIP*four*", "XFAIL*test_2*", "XPASS*test_3*"]
    )


def test_reportchars_all_error(testdir):
    testdir.makepyfile(
        conftest="""
        def pytest_runtest_teardown():
            assert 0
        """,
        test_simple="""
        def test_foo():
            pass
        """,
    )
    result = testdir.runpytest("-ra")
    result.stdout.fnmatch_lines(["ERROR*test_foo*"])


@pytest.mark.xfail("hasattr(sys, 'pypy_version_info')")
def test_errors_in_xfail_skip_expressions(testdir):
    testdir.makepyfile(
        """
        import pytest
        @pytest.mark.skipif("asd")
        def test_nameerror():
            pass
        @pytest.mark.xfail("syntax error")
        def test_syntax():
            pass

        def test_func():
            pass
    """
    )
    result = testdir.runpytest()
    markline = "                ^"
    if sys.platform.startswith("java"):
        # XXX report this to java
        markline = "*" + markline[8:]
    result.stdout.fnmatch_lines(
        [
            "*ERROR*test_nameerror*",
            "*evaluating*skipif*expression*",
            "*asd*",
            "*ERROR*test_syntax*",
            "*evaluating*xfail*expression*",
            "    syntax error",
            markline,
            "SyntaxError: invalid syntax",
            "*1 pass*2 error*",
        ]
    )


def test_xfail_skipif_with_globals(testdir):
    testdir.makepyfile(
        """
        import pytest
        x = 3
        @pytest.mark.skipif("x == 3")
        def test_skip1():
            pass
        @pytest.mark.xfail("x == 3")
        def test_boolean():
            assert 0
    """
    )
    result = testdir.runpytest("-rsx")
    result.stdout.fnmatch_lines(["*SKIP*x == 3*", "*XFAIL*test_boolean*", "*x == 3*"])


def test_direct_gives_error(testdir):
    testdir.makepyfile(
        """
        import pytest
        @pytest.mark.skipif(True)
        def test_skip1():
            pass
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*1 error*"])


def test_default_markers(testdir):
    result = testdir.runpytest("--markers")
    result.stdout.fnmatch_lines(
        [
            "*skipif(*condition)*skip*",
            "*xfail(*condition, reason=None, run=True, raises=None, strict=False)*expected failure*",
        ]
    )


def test_xfail_test_setup_exception(testdir):
    testdir.makeconftest(
        """
            def pytest_runtest_setup():
                0 / 0
        """
    )
    p = testdir.makepyfile(
        """
            import pytest
            @pytest.mark.xfail
            def test_func():
                assert 0
        """
    )
    result = testdir.runpytest(p)
    assert result.ret == 0
    assert "xfailed" in result.stdout.str()
    assert "xpassed" not in result.stdout.str()


def test_imperativeskip_on_xfail_test(testdir):
    testdir.makepyfile(
        """
        import pytest
        @pytest.mark.xfail
        def test_that_fails():
            assert 0

        @pytest.mark.skipif("True")
        def test_hello():
            pass
    """
    )
    testdir.makeconftest(
        """
        import pytest
        def pytest_runtest_setup(item):
            pytest.skip("abc")
    """
    )
    result = testdir.runpytest("-rsxX")
    result.stdout.fnmatch_lines_random(
        """
        *SKIP*abc*
        *SKIP*condition: True*
        *2 skipped*
    """
    )


class TestBooleanCondition(object):

    def test_skipif(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skipif(True, reason="True123")
            def test_func1():
                pass
            @pytest.mark.skipif(False, reason="True123")
            def test_func2():
                pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            """
            *1 passed*1 skipped*
        """
        )

    def test_skipif_noreason(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skipif(True)
            def test_func():
                pass
        """
        )
        result = testdir.runpytest("-rs")
        result.stdout.fnmatch_lines(
            """
            *1 error*
        """
        )

    def test_xfail(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(True, reason="True123")
            def test_func():
                assert 0
        """
        )
        result = testdir.runpytest("-rxs")
        result.stdout.fnmatch_lines(
            """
            *XFAIL*
            *True123*
            *1 xfail*
        """
        )


def test_xfail_item(testdir):
    # Ensure pytest.xfail works with non-Python Item
    testdir.makeconftest(
        """
        import pytest

        class MyItem(pytest.Item):
            nodeid = 'foo'
            def runtest(self):
                pytest.xfail("Expected Failure")

        def pytest_collect_file(path, parent):
            return MyItem("foo", parent)
    """
    )
    result = testdir.inline_run()
    passed, skipped, failed = result.listoutcomes()
    assert not failed
    xfailed = [r for r in skipped if hasattr(r, "wasxfail")]
    assert xfailed


def test_module_level_skip_error(testdir):
    """
    Verify that using pytest.skip at module level causes a collection error
    """
    testdir.makepyfile(
        """
        import pytest
        @pytest.skip
        def test_func():
            assert True
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines("*Using pytest.skip outside of a test is not allowed*")


def test_module_level_skip_with_allow_module_level(testdir):
    """
    Verify that using pytest.skip(allow_module_level=True) is allowed
    """
    testdir.makepyfile(
        """
        import pytest
        pytest.skip("skip_module_level", allow_module_level=True)

        def test_func():
            assert 0
    """
    )
    result = testdir.runpytest("-rxs")
    result.stdout.fnmatch_lines("*SKIP*skip_module_level")


def test_invalid_skip_keyword_parameter(testdir):
    """
    Verify that using pytest.skip() with unknown parameter raises an error
    """
    testdir.makepyfile(
        """
        import pytest
        pytest.skip("skip_module_level", unknown=1)

        def test_func():
            assert 0
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines("*TypeError:*['unknown']*")


def test_mark_xfail_item(testdir):
    # Ensure pytest.mark.xfail works with non-Python Item
    testdir.makeconftest(
        """
        import pytest

        class MyItem(pytest.Item):
            nodeid = 'foo'
            def setup(self):
                marker = pytest.mark.xfail(True, reason="Expected failure")
                self.add_marker(marker)
            def runtest(self):
                assert False

        def pytest_collect_file(path, parent):
            return MyItem("foo", parent)
    """
    )
    result = testdir.inline_run()
    passed, skipped, failed = result.listoutcomes()
    assert not failed
    xfailed = [r for r in skipped if hasattr(r, "wasxfail")]
    assert xfailed


def test_summary_list_after_errors(testdir):
    """Ensure the list of errors/fails/xfails/skips appears after tracebacks in terminal reporting."""
    testdir.makepyfile(
        """
        import pytest
        def test_fail():
            assert 0
    """
    )
    result = testdir.runpytest("-ra")
    result.stdout.fnmatch_lines(
        [
            "=* FAILURES *=",
            "*= short test summary info =*",
            "FAIL test_summary_list_after_errors.py::test_fail",
        ]
    )
