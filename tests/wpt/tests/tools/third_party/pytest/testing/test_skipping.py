import sys
import textwrap

import pytest
from _pytest.pytester import Pytester
from _pytest.runner import runtestprotocol
from _pytest.skipping import evaluate_skip_marks
from _pytest.skipping import evaluate_xfail_marks
from _pytest.skipping import pytest_runtest_setup


class TestEvaluation:
    def test_no_marker(self, pytester: Pytester) -> None:
        item = pytester.getitem("def test_func(): pass")
        skipped = evaluate_skip_marks(item)
        assert not skipped

    def test_marked_xfail_no_args(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest
            @pytest.mark.xfail
            def test_func():
                pass
        """
        )
        xfailed = evaluate_xfail_marks(item)
        assert xfailed
        assert xfailed.reason == ""
        assert xfailed.run

    def test_marked_skipif_no_args(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest
            @pytest.mark.skipif
            def test_func():
                pass
        """
        )
        skipped = evaluate_skip_marks(item)
        assert skipped
        assert skipped.reason == ""

    def test_marked_one_arg(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest
            @pytest.mark.skipif("hasattr(os, 'sep')")
            def test_func():
                pass
        """
        )
        skipped = evaluate_skip_marks(item)
        assert skipped
        assert skipped.reason == "condition: hasattr(os, 'sep')"

    def test_marked_one_arg_with_reason(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest
            @pytest.mark.skipif("hasattr(os, 'sep')", attr=2, reason="hello world")
            def test_func():
                pass
        """
        )
        skipped = evaluate_skip_marks(item)
        assert skipped
        assert skipped.reason == "hello world"

    def test_marked_one_arg_twice(self, pytester: Pytester) -> None:
        lines = [
            """@pytest.mark.skipif("not hasattr(os, 'murks')")""",
            """@pytest.mark.skipif(condition="hasattr(os, 'murks')")""",
        ]
        for i in range(0, 2):
            item = pytester.getitem(
                """
                import pytest
                %s
                %s
                def test_func():
                    pass
            """
                % (lines[i], lines[(i + 1) % 2])
            )
            skipped = evaluate_skip_marks(item)
            assert skipped
            assert skipped.reason == "condition: not hasattr(os, 'murks')"

    def test_marked_one_arg_twice2(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest
            @pytest.mark.skipif("hasattr(os, 'murks')")
            @pytest.mark.skipif("not hasattr(os, 'murks')")
            def test_func():
                pass
        """
        )
        skipped = evaluate_skip_marks(item)
        assert skipped
        assert skipped.reason == "condition: not hasattr(os, 'murks')"

    def test_marked_skipif_with_boolean_without_reason(
        self, pytester: Pytester
    ) -> None:
        item = pytester.getitem(
            """
            import pytest
            @pytest.mark.skipif(False)
            def test_func():
                pass
        """
        )
        with pytest.raises(pytest.fail.Exception) as excinfo:
            evaluate_skip_marks(item)
        assert excinfo.value.msg is not None
        assert (
            """Error evaluating 'skipif': you need to specify reason=STRING when using booleans as conditions."""
            in excinfo.value.msg
        )

    def test_marked_skipif_with_invalid_boolean(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest

            class InvalidBool:
                def __bool__(self):
                    raise TypeError("INVALID")

            @pytest.mark.skipif(InvalidBool(), reason="xxx")
            def test_func():
                pass
        """
        )
        with pytest.raises(pytest.fail.Exception) as excinfo:
            evaluate_skip_marks(item)
        assert excinfo.value.msg is not None
        assert "Error evaluating 'skipif' condition as a boolean" in excinfo.value.msg
        assert "INVALID" in excinfo.value.msg

    def test_skipif_class(self, pytester: Pytester) -> None:
        (item,) = pytester.getitems(
            """
            import pytest
            class TestClass(object):
                pytestmark = pytest.mark.skipif("config._hackxyz")
                def test_func(self):
                    pass
        """
        )
        item.config._hackxyz = 3  # type: ignore[attr-defined]
        skipped = evaluate_skip_marks(item)
        assert skipped
        assert skipped.reason == "condition: config._hackxyz"

    def test_skipif_markeval_namespace(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest

            def pytest_markeval_namespace():
                return {"color": "green"}
            """
        )
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.skipif("color == 'green'")
            def test_1():
                assert True

            @pytest.mark.skipif("color == 'red'")
            def test_2():
                assert True
        """
        )
        res = pytester.runpytest(p)
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 skipped*"])
        res.stdout.fnmatch_lines(["*1 passed*"])

    def test_skipif_markeval_namespace_multiple(self, pytester: Pytester) -> None:
        """Keys defined by ``pytest_markeval_namespace()`` in nested plugins override top-level ones."""
        root = pytester.mkdir("root")
        root.joinpath("__init__.py").touch()
        root.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
            import pytest

            def pytest_markeval_namespace():
                return {"arg": "root"}
            """
            )
        )
        root.joinpath("test_root.py").write_text(
            textwrap.dedent(
                """\
            import pytest

            @pytest.mark.skipif("arg == 'root'")
            def test_root():
                assert False
            """
            )
        )
        foo = root.joinpath("foo")
        foo.mkdir()
        foo.joinpath("__init__.py").touch()
        foo.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
            import pytest

            def pytest_markeval_namespace():
                return {"arg": "foo"}
            """
            )
        )
        foo.joinpath("test_foo.py").write_text(
            textwrap.dedent(
                """\
            import pytest

            @pytest.mark.skipif("arg == 'foo'")
            def test_foo():
                assert False
            """
            )
        )
        bar = root.joinpath("bar")
        bar.mkdir()
        bar.joinpath("__init__.py").touch()
        bar.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
            import pytest

            def pytest_markeval_namespace():
                return {"arg": "bar"}
            """
            )
        )
        bar.joinpath("test_bar.py").write_text(
            textwrap.dedent(
                """\
            import pytest

            @pytest.mark.skipif("arg == 'bar'")
            def test_bar():
                assert False
            """
            )
        )

        reprec = pytester.inline_run("-vs", "--capture=no")
        reprec.assertoutcome(skipped=3)

    def test_skipif_markeval_namespace_ValueError(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest

            def pytest_markeval_namespace():
                return True
            """
        )
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.skipif("color == 'green'")
            def test_1():
                assert True
        """
        )
        res = pytester.runpytest(p)
        assert res.ret == 1
        res.stdout.fnmatch_lines(
            [
                "*ValueError: pytest_markeval_namespace() needs to return a dict, got True*"
            ]
        )


class TestXFail:
    @pytest.mark.parametrize("strict", [True, False])
    def test_xfail_simple(self, pytester: Pytester, strict: bool) -> None:
        item = pytester.getitem(
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

    def test_xfail_xpassed(self, pytester: Pytester) -> None:
        item = pytester.getitem(
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

    def test_xfail_using_platform(self, pytester: Pytester) -> None:
        """Verify that platform can be used with xfail statements."""
        item = pytester.getitem(
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

    def test_xfail_xpassed_strict(self, pytester: Pytester) -> None:
        item = pytester.getitem(
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
        assert str(callreport.longrepr) == "[XPASS(strict)] nope"
        assert not hasattr(callreport, "wasxfail")

    def test_xfail_run_anyway(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.xfail
            def test_func():
                assert 0
            def test_func2():
                pytest.xfail("hello")
        """
        )
        result = pytester.runpytest("--runxfail")
        result.stdout.fnmatch_lines(
            ["*def test_func():*", "*assert 0*", "*1 failed*1 pass*"]
        )

    @pytest.mark.parametrize(
        "test_input,expected",
        [
            (
                ["-rs"],
                ["SKIPPED [1] test_sample.py:2: unconditional skip", "*1 skipped*"],
            ),
            (
                ["-rs", "--runxfail"],
                ["SKIPPED [1] test_sample.py:2: unconditional skip", "*1 skipped*"],
            ),
        ],
    )
    def test_xfail_run_with_skip_mark(
        self, pytester: Pytester, test_input, expected
    ) -> None:
        pytester.makepyfile(
            test_sample="""
            import pytest
            @pytest.mark.skip
            def test_skip_location() -> None:
                assert 0
        """
        )
        result = pytester.runpytest(*test_input)
        result.stdout.fnmatch_lines(expected)

    def test_xfail_evalfalse_but_fails(self, pytester: Pytester) -> None:
        item = pytester.getitem(
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

    def test_xfail_not_report_default(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            test_one="""
            import pytest
            @pytest.mark.xfail
            def test_this():
                assert 0
        """
        )
        pytester.runpytest(p, "-v")
        # result.stdout.fnmatch_lines([
        #    "*HINT*use*-r*"
        # ])

    def test_xfail_not_run_xfail_reporting(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
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
        result = pytester.runpytest(p, "-rx")
        result.stdout.fnmatch_lines(
            [
                "*test_one*test_this*",
                "*NOTRUN*noway",
                "*test_one*test_this_true*",
                "*NOTRUN*condition:*True*",
                "*1 passed*",
            ]
        )

    def test_xfail_not_run_no_setup_run(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            test_one="""
            import pytest
            @pytest.mark.xfail(run=False, reason="hello")
            def test_this():
                assert 0
            def setup_module(mod):
                raise ValueError(42)
        """
        )
        result = pytester.runpytest(p, "-rx")
        result.stdout.fnmatch_lines(
            ["*test_one*test_this*", "*NOTRUN*hello", "*1 xfailed*"]
        )

    def test_xfail_xpass(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            test_one="""
            import pytest
            @pytest.mark.xfail
            def test_that():
                assert 1
        """
        )
        result = pytester.runpytest(p, "-rX")
        result.stdout.fnmatch_lines(["*XPASS*test_that*", "*1 xpassed*"])
        assert result.ret == 0

    def test_xfail_imperative(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest
            def test_this():
                pytest.xfail("hello")
        """
        )
        result = pytester.runpytest(p)
        result.stdout.fnmatch_lines(["*1 xfailed*"])
        result = pytester.runpytest(p, "-rx")
        result.stdout.fnmatch_lines(["*XFAIL*test_this*", "*reason:*hello*"])
        result = pytester.runpytest(p, "--runxfail")
        result.stdout.fnmatch_lines(["*1 pass*"])

    def test_xfail_imperative_in_setup_function(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest
            def setup_function(function):
                pytest.xfail("hello")

            def test_this():
                assert 0
        """
        )
        result = pytester.runpytest(p)
        result.stdout.fnmatch_lines(["*1 xfailed*"])
        result = pytester.runpytest(p, "-rx")
        result.stdout.fnmatch_lines(["*XFAIL*test_this*", "*reason:*hello*"])
        result = pytester.runpytest(p, "--runxfail")
        result.stdout.fnmatch_lines(
            """
            *def test_this*
            *1 fail*
        """
        )

    def xtest_dynamic_xfail_set_during_setup(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
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
        result = pytester.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines(["*XFAIL*test_this*", "*XPASS*test_that*"])

    def test_dynamic_xfail_no_run(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest
            @pytest.fixture
            def arg(request):
                request.applymarker(pytest.mark.xfail(run=False))
            def test_this(arg):
                assert 0
        """
        )
        result = pytester.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines(["*XFAIL*test_this*", "*NOTRUN*"])

    def test_dynamic_xfail_set_during_funcarg_setup(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest
            @pytest.fixture
            def arg(request):
                request.applymarker(pytest.mark.xfail)
            def test_this2(arg):
                assert 0
        """
        )
        result = pytester.runpytest(p)
        result.stdout.fnmatch_lines(["*1 xfailed*"])

    def test_dynamic_xfail_set_during_runtest_failed(self, pytester: Pytester) -> None:
        # Issue #7486.
        p = pytester.makepyfile(
            """
            import pytest
            def test_this(request):
                request.node.add_marker(pytest.mark.xfail(reason="xfail"))
                assert 0
        """
        )
        result = pytester.runpytest(p)
        result.assert_outcomes(xfailed=1)

    def test_dynamic_xfail_set_during_runtest_passed_strict(
        self, pytester: Pytester
    ) -> None:
        # Issue #7486.
        p = pytester.makepyfile(
            """
            import pytest
            def test_this(request):
                request.node.add_marker(pytest.mark.xfail(reason="xfail", strict=True))
        """
        )
        result = pytester.runpytest(p)
        result.assert_outcomes(failed=1)

    @pytest.mark.parametrize(
        "expected, actual, matchline",
        [
            ("TypeError", "TypeError", "*1 xfailed*"),
            ("(AttributeError, TypeError)", "TypeError", "*1 xfailed*"),
            ("TypeError", "IndexError", "*1 failed*"),
            ("(AttributeError, TypeError)", "IndexError", "*1 failed*"),
        ],
    )
    def test_xfail_raises(
        self, expected, actual, matchline, pytester: Pytester
    ) -> None:
        p = pytester.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(raises=%s)
            def test_raises():
                raise %s()
        """
            % (expected, actual)
        )
        result = pytester.runpytest(p)
        result.stdout.fnmatch_lines([matchline])

    def test_strict_sanity(self, pytester: Pytester) -> None:
        """Sanity check for xfail(strict=True): a failing test should behave
        exactly like a normal xfail."""
        p = pytester.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(reason='unsupported feature', strict=True)
            def test_foo():
                assert 0
        """
        )
        result = pytester.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines(["*XFAIL*", "*unsupported feature*"])
        assert result.ret == 0

    @pytest.mark.parametrize("strict", [True, False])
    def test_strict_xfail(self, pytester: Pytester, strict: bool) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.xfail(reason='unsupported feature', strict=%s)
            def test_foo():
                with open('foo_executed', 'w'): pass  # make sure test executes
        """
            % strict
        )
        result = pytester.runpytest(p, "-rxX")
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
        assert pytester.path.joinpath("foo_executed").exists()

    @pytest.mark.parametrize("strict", [True, False])
    def test_strict_xfail_condition(self, pytester: Pytester, strict: bool) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.xfail(False, reason='unsupported feature', strict=%s)
            def test_foo():
                pass
        """
            % strict
        )
        result = pytester.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines(["*1 passed*"])
        assert result.ret == 0

    @pytest.mark.parametrize("strict", [True, False])
    def test_xfail_condition_keyword(self, pytester: Pytester, strict: bool) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.xfail(condition=False, reason='unsupported feature', strict=%s)
            def test_foo():
                pass
        """
            % strict
        )
        result = pytester.runpytest(p, "-rxX")
        result.stdout.fnmatch_lines(["*1 passed*"])
        assert result.ret == 0

    @pytest.mark.parametrize("strict_val", ["true", "false"])
    def test_strict_xfail_default_from_file(
        self, pytester: Pytester, strict_val
    ) -> None:
        pytester.makeini(
            """
            [pytest]
            xfail_strict = %s
        """
            % strict_val
        )
        p = pytester.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(reason='unsupported feature')
            def test_foo():
                pass
        """
        )
        result = pytester.runpytest(p, "-rxX")
        strict = strict_val == "true"
        result.stdout.fnmatch_lines(["*1 failed*" if strict else "*1 xpassed*"])
        assert result.ret == (1 if strict else 0)

    def test_xfail_markeval_namespace(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest

            def pytest_markeval_namespace():
                return {"color": "green"}
            """
        )
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.xfail("color == 'green'")
            def test_1():
                assert False

            @pytest.mark.xfail("color == 'red'")
            def test_2():
                assert False
        """
        )
        res = pytester.runpytest(p)
        assert res.ret == 1
        res.stdout.fnmatch_lines(["*1 failed*"])
        res.stdout.fnmatch_lines(["*1 xfailed*"])


class TestXFailwithSetupTeardown:
    def test_failing_setup_issue9(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            def setup_function(func):
                assert 0

            @pytest.mark.xfail
            def test_func():
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 xfail*"])

    def test_failing_teardown_issue9(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            def teardown_function(func):
                assert 0

            @pytest.mark.xfail
            def test_func():
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 xfail*"])


class TestSkip:
    def test_skip_class(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        rec = pytester.inline_run()
        rec.assertoutcome(skipped=2, passed=1)

    def test_skips_on_false_string(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.skip('False')
            def test_foo():
                pass
        """
        )
        rec = pytester.inline_run()
        rec.assertoutcome(skipped=1)

    def test_arg_as_reason(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.skip('testing stuff')
            def test_bar():
                pass
        """
        )
        result = pytester.runpytest("-rs")
        result.stdout.fnmatch_lines(["*testing stuff*", "*1 skipped*"])

    def test_skip_no_reason(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.skip
            def test_foo():
                pass
        """
        )
        result = pytester.runpytest("-rs")
        result.stdout.fnmatch_lines(["*unconditional skip*", "*1 skipped*"])

    def test_skip_with_reason(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.skip(reason="for lolz")
            def test_bar():
                pass
        """
        )
        result = pytester.runpytest("-rs")
        result.stdout.fnmatch_lines(["*for lolz*", "*1 skipped*"])

    def test_only_skips_marked_test(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest("-rs")
        result.stdout.fnmatch_lines(["*nothing in particular*", "*1 passed*2 skipped*"])

    def test_strict_and_skip(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.skip
            def test_hello():
                pass
        """
        )
        result = pytester.runpytest("-rs", "--strict-markers")
        result.stdout.fnmatch_lines(["*unconditional skip*", "*1 skipped*"])

    def test_wrong_skip_usage(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.skip(False, reason="I thought this was skipif")
            def test_hello():
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*TypeError: *__init__() got multiple values for argument 'reason'"
                " - maybe you meant pytest.mark.skipif?"
            ]
        )


class TestSkipif:
    def test_skipif_conditional(self, pytester: Pytester) -> None:
        item = pytester.getitem(
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
    def test_skipif_reporting(self, pytester: Pytester, params) -> None:
        p = pytester.makepyfile(
            test_foo="""
            import pytest
            @pytest.mark.skipif(%(params)s)
            def test_that():
                assert 0
        """
            % dict(params=params)
        )
        result = pytester.runpytest(p, "-s", "-rs")
        result.stdout.fnmatch_lines(["*SKIP*1*test_foo.py*platform*", "*1 skipped*"])
        assert result.ret == 0

    def test_skipif_using_platform(self, pytester: Pytester) -> None:
        item = pytester.getitem(
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
    def test_skipif_reporting_multiple(
        self, pytester: Pytester, marker, msg1, msg2
    ) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest("-s", "-rsxX")
        result.stdout.fnmatch_lines(
            [f"*{msg1}*test_foo.py*second_condition*", f"*1 {msg2}*"]
        )
        assert result.ret == 0


def test_skip_not_report_default(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        test_one="""
        import pytest
        def test_this():
            pytest.skip("hello")
    """
    )
    result = pytester.runpytest(p, "-v")
    result.stdout.fnmatch_lines(
        [
            # "*HINT*use*-r*",
            "*1 skipped*"
        ]
    )


def test_skipif_class(pytester: Pytester) -> None:
    p = pytester.makepyfile(
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
    result = pytester.runpytest(p)
    result.stdout.fnmatch_lines(["*2 skipped*"])


def test_skipped_reasons_functional(pytester: Pytester) -> None:
    pytester.makepyfile(
        test_one="""
            import pytest
            from conftest import doskip

            def setup_function(func):
                doskip()

            def test_func():
                pass

            class TestClass(object):
                def test_method(self):
                    doskip()

                @pytest.mark.skip("via_decorator")
                def test_deco(self):
                    assert 0
        """,
        conftest="""
            import pytest, sys
            def doskip():
                assert sys._getframe().f_lineno == 3
                pytest.skip('test')
        """,
    )
    result = pytester.runpytest("-rs")
    result.stdout.fnmatch_lines_random(
        [
            "SKIPPED [[]2[]] conftest.py:4: test",
            "SKIPPED [[]1[]] test_one.py:14: via_decorator",
        ]
    )
    assert result.ret == 0


def test_skipped_folding(pytester: Pytester) -> None:
    pytester.makepyfile(
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
    result = pytester.runpytest("-rs")
    result.stdout.fnmatch_lines(["*SKIP*2*test_one.py: Folding"])
    assert result.ret == 0


def test_reportchars(pytester: Pytester) -> None:
    pytester.makepyfile(
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
    result = pytester.runpytest("-rfxXs")
    result.stdout.fnmatch_lines(
        ["FAIL*test_1*", "XFAIL*test_2*", "XPASS*test_3*", "SKIP*four*"]
    )


def test_reportchars_error(pytester: Pytester) -> None:
    pytester.makepyfile(
        conftest="""
        def pytest_runtest_teardown():
            assert 0
        """,
        test_simple="""
        def test_foo():
            pass
        """,
    )
    result = pytester.runpytest("-rE")
    result.stdout.fnmatch_lines(["ERROR*test_foo*"])


def test_reportchars_all(pytester: Pytester) -> None:
    pytester.makepyfile(
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
        @pytest.fixture
        def fail():
            assert 0
        def test_5(fail):
            pass
    """
    )
    result = pytester.runpytest("-ra")
    result.stdout.fnmatch_lines(
        [
            "SKIP*four*",
            "XFAIL*test_2*",
            "XPASS*test_3*",
            "ERROR*test_5*",
            "FAIL*test_1*",
        ]
    )


def test_reportchars_all_error(pytester: Pytester) -> None:
    pytester.makepyfile(
        conftest="""
        def pytest_runtest_teardown():
            assert 0
        """,
        test_simple="""
        def test_foo():
            pass
        """,
    )
    result = pytester.runpytest("-ra")
    result.stdout.fnmatch_lines(["ERROR*test_foo*"])


def test_errors_in_xfail_skip_expressions(pytester: Pytester) -> None:
    pytester.makepyfile(
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
    result = pytester.runpytest()
    markline = "                ^"
    pypy_version_info = getattr(sys, "pypy_version_info", None)
    if pypy_version_info is not None and pypy_version_info < (6,):
        markline = markline[5:]
    elif sys.version_info >= (3, 8) or hasattr(sys, "pypy_version_info"):
        markline = markline[4:]

    if sys.version_info[:2] >= (3, 10):
        expected = [
            "*ERROR*test_nameerror*",
            "*asd*",
            "",
            "During handling of the above exception, another exception occurred:",
        ]
    else:
        expected = [
            "*ERROR*test_nameerror*",
        ]

    expected += [
        "*evaluating*skipif*condition*",
        "*asd*",
        "*ERROR*test_syntax*",
        "*evaluating*xfail*condition*",
        "    syntax error",
        markline,
        "SyntaxError: invalid syntax",
        "*1 pass*2 errors*",
    ]
    result.stdout.fnmatch_lines(expected)


def test_xfail_skipif_with_globals(pytester: Pytester) -> None:
    pytester.makepyfile(
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
    result = pytester.runpytest("-rsx")
    result.stdout.fnmatch_lines(["*SKIP*x == 3*", "*XFAIL*test_boolean*", "*x == 3*"])


def test_default_markers(pytester: Pytester) -> None:
    result = pytester.runpytest("--markers")
    result.stdout.fnmatch_lines(
        [
            "*skipif(condition, ..., [*], reason=...)*skip*",
            "*xfail(condition, ..., [*], reason=..., run=True, raises=None, strict=xfail_strict)*expected failure*",
        ]
    )


def test_xfail_test_setup_exception(pytester: Pytester) -> None:
    pytester.makeconftest(
        """
            def pytest_runtest_setup():
                0 / 0
        """
    )
    p = pytester.makepyfile(
        """
            import pytest
            @pytest.mark.xfail
            def test_func():
                assert 0
        """
    )
    result = pytester.runpytest(p)
    assert result.ret == 0
    assert "xfailed" in result.stdout.str()
    result.stdout.no_fnmatch_line("*xpassed*")


def test_imperativeskip_on_xfail_test(pytester: Pytester) -> None:
    pytester.makepyfile(
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
    pytester.makeconftest(
        """
        import pytest
        def pytest_runtest_setup(item):
            pytest.skip("abc")
    """
    )
    result = pytester.runpytest("-rsxX")
    result.stdout.fnmatch_lines_random(
        """
        *SKIP*abc*
        *SKIP*condition: True*
        *2 skipped*
    """
    )


class TestBooleanCondition:
    def test_skipif(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            """
            *1 passed*1 skipped*
        """
        )

    def test_skipif_noreason(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.skipif(True)
            def test_func():
                pass
        """
        )
        result = pytester.runpytest("-rs")
        result.stdout.fnmatch_lines(
            """
            *1 error*
        """
        )

    def test_xfail(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(True, reason="True123")
            def test_func():
                assert 0
        """
        )
        result = pytester.runpytest("-rxs")
        result.stdout.fnmatch_lines(
            """
            *XFAIL*
            *True123*
            *1 xfail*
        """
        )


def test_xfail_item(pytester: Pytester) -> None:
    # Ensure pytest.xfail works with non-Python Item
    pytester.makeconftest(
        """
        import pytest

        class MyItem(pytest.Item):
            nodeid = 'foo'
            def runtest(self):
                pytest.xfail("Expected Failure")

        def pytest_collect_file(file_path, parent):
            return MyItem.from_parent(name="foo", parent=parent)
    """
    )
    result = pytester.inline_run()
    passed, skipped, failed = result.listoutcomes()
    assert not failed
    xfailed = [r for r in skipped if hasattr(r, "wasxfail")]
    assert xfailed


def test_module_level_skip_error(pytester: Pytester) -> None:
    """Verify that using pytest.skip at module level causes a collection error."""
    pytester.makepyfile(
        """
        import pytest
        pytest.skip("skip_module_level")

        def test_func():
            assert True
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(
        ["*Using pytest.skip outside of a test will skip the entire module*"]
    )


def test_module_level_skip_with_allow_module_level(pytester: Pytester) -> None:
    """Verify that using pytest.skip(allow_module_level=True) is allowed."""
    pytester.makepyfile(
        """
        import pytest
        pytest.skip("skip_module_level", allow_module_level=True)

        def test_func():
            assert 0
    """
    )
    result = pytester.runpytest("-rxs")
    result.stdout.fnmatch_lines(["*SKIP*skip_module_level"])


def test_invalid_skip_keyword_parameter(pytester: Pytester) -> None:
    """Verify that using pytest.skip() with unknown parameter raises an error."""
    pytester.makepyfile(
        """
        import pytest
        pytest.skip("skip_module_level", unknown=1)

        def test_func():
            assert 0
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*TypeError:*['unknown']*"])


def test_mark_xfail_item(pytester: Pytester) -> None:
    # Ensure pytest.mark.xfail works with non-Python Item
    pytester.makeconftest(
        """
        import pytest

        class MyItem(pytest.Item):
            nodeid = 'foo'
            def setup(self):
                marker = pytest.mark.xfail("1 == 2", reason="Expected failure - false")
                self.add_marker(marker)
                marker = pytest.mark.xfail(True, reason="Expected failure - true")
                self.add_marker(marker)
            def runtest(self):
                assert False

        def pytest_collect_file(file_path, parent):
            return MyItem.from_parent(name="foo", parent=parent)
    """
    )
    result = pytester.inline_run()
    passed, skipped, failed = result.listoutcomes()
    assert not failed
    xfailed = [r for r in skipped if hasattr(r, "wasxfail")]
    assert xfailed


def test_summary_list_after_errors(pytester: Pytester) -> None:
    """Ensure the list of errors/fails/xfails/skips appears after tracebacks in terminal reporting."""
    pytester.makepyfile(
        """
        import pytest
        def test_fail():
            assert 0
    """
    )
    result = pytester.runpytest("-ra")
    result.stdout.fnmatch_lines(
        [
            "=* FAILURES *=",
            "*= short test summary info =*",
            "FAILED test_summary_list_after_errors.py::test_fail - assert 0",
        ]
    )


def test_importorskip() -> None:
    with pytest.raises(
        pytest.skip.Exception,
        match="^could not import 'doesnotexist': No module named .*",
    ):
        pytest.importorskip("doesnotexist")


def test_relpath_rootdir(pytester: Pytester) -> None:
    pytester.makepyfile(
        **{
            "tests/test_1.py": """
        import pytest
        @pytest.mark.skip()
        def test_pass():
            pass
            """,
        }
    )
    result = pytester.runpytest("-rs", "tests/test_1.py", "--rootdir=tests")
    result.stdout.fnmatch_lines(
        ["SKIPPED [[]1[]] tests/test_1.py:2: unconditional skip"]
    )


def test_skip_using_reason_works_ok(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest

        def test_skipping_reason():
            pytest.skip(reason="skippedreason")
        """
    )
    result = pytester.runpytest(p)
    result.stdout.no_fnmatch_line("*PytestDeprecationWarning*")
    result.assert_outcomes(skipped=1)


def test_fail_using_reason_works_ok(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest

        def test_failing_reason():
            pytest.fail(reason="failedreason")
        """
    )
    result = pytester.runpytest(p)
    result.stdout.no_fnmatch_line("*PytestDeprecationWarning*")
    result.assert_outcomes(failed=1)


def test_fail_fails_with_msg_and_reason(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest

        def test_fail_both_arguments():
            pytest.fail(reason="foo", msg="bar")
        """
    )
    result = pytester.runpytest(p)
    result.stdout.fnmatch_lines(
        "*UsageError: Passing both ``reason`` and ``msg`` to pytest.fail(...) is not permitted.*"
    )
    result.assert_outcomes(failed=1)


def test_skip_fails_with_msg_and_reason(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest

        def test_skip_both_arguments():
            pytest.skip(reason="foo", msg="bar")
        """
    )
    result = pytester.runpytest(p)
    result.stdout.fnmatch_lines(
        "*UsageError: Passing both ``reason`` and ``msg`` to pytest.skip(...) is not permitted.*"
    )
    result.assert_outcomes(failed=1)


def test_exit_with_msg_and_reason_fails(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest

        def test_exit_both_arguments():
            pytest.exit(reason="foo", msg="bar")
        """
    )
    result = pytester.runpytest(p)
    result.stdout.fnmatch_lines(
        "*UsageError: cannot pass reason and msg to exit(), `msg` is deprecated, use `reason`.*"
    )
    result.assert_outcomes(failed=1)


def test_exit_with_reason_works_ok(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest

        def test_exit_reason_only():
            pytest.exit(reason="foo")
        """
    )
    result = pytester.runpytest(p)
    result.stdout.fnmatch_lines("*_pytest.outcomes.Exit: foo*")
