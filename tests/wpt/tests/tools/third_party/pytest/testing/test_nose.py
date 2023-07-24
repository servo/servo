import pytest
from _pytest.pytester import Pytester


def setup_module(mod):
    mod.nose = pytest.importorskip("nose")


def test_nose_setup(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        values = []
        from nose.tools import with_setup

        @with_setup(lambda: values.append(1), lambda: values.append(2))
        def test_hello():
            assert values == [1]

        def test_world():
            assert values == [1,2]

        test_hello.setup = lambda: values.append(1)
        test_hello.teardown = lambda: values.append(2)
    """
    )
    result = pytester.runpytest(p, "-p", "nose")
    result.assert_outcomes(passed=2)


def test_setup_func_with_setup_decorator() -> None:
    from _pytest.nose import call_optional

    values = []

    class A:
        @pytest.fixture(autouse=True)
        def f(self):
            values.append(1)

    call_optional(A(), "f")
    assert not values


def test_setup_func_not_callable() -> None:
    from _pytest.nose import call_optional

    class A:
        f = 1

    call_optional(A(), "f")


def test_nose_setup_func(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        from nose.tools import with_setup

        values = []

        def my_setup():
            a = 1
            values.append(a)

        def my_teardown():
            b = 2
            values.append(b)

        @with_setup(my_setup, my_teardown)
        def test_hello():
            print(values)
            assert values == [1]

        def test_world():
            print(values)
            assert values == [1,2]

    """
    )
    result = pytester.runpytest(p, "-p", "nose")
    result.assert_outcomes(passed=2)


def test_nose_setup_func_failure(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        from nose.tools import with_setup

        values = []
        my_setup = lambda x: 1
        my_teardown = lambda x: 2

        @with_setup(my_setup, my_teardown)
        def test_hello():
            print(values)
            assert values == [1]

        def test_world():
            print(values)
            assert values == [1,2]

    """
    )
    result = pytester.runpytest(p, "-p", "nose")
    result.stdout.fnmatch_lines(["*TypeError: <lambda>()*"])


def test_nose_setup_func_failure_2(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        values = []

        my_setup = 1
        my_teardown = 2

        def test_hello():
            assert values == []

        test_hello.setup = my_setup
        test_hello.teardown = my_teardown
    """
    )
    reprec = pytester.inline_run()
    reprec.assertoutcome(passed=1)


def test_nose_setup_partial(pytester: Pytester) -> None:
    pytest.importorskip("functools")
    p = pytester.makepyfile(
        """
        from functools import partial

        values = []

        def my_setup(x):
            a = x
            values.append(a)

        def my_teardown(x):
            b = x
            values.append(b)

        my_setup_partial = partial(my_setup, 1)
        my_teardown_partial = partial(my_teardown, 2)

        def test_hello():
            print(values)
            assert values == [1]

        def test_world():
            print(values)
            assert values == [1,2]

        test_hello.setup = my_setup_partial
        test_hello.teardown = my_teardown_partial
    """
    )
    result = pytester.runpytest(p, "-p", "nose")
    result.stdout.fnmatch_lines(["*2 passed*"])


def test_module_level_setup(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        from nose.tools import with_setup
        items = {}

        def setup():
            items.setdefault("setup", []).append("up")

        def teardown():
            items.setdefault("setup", []).append("down")

        def setup2():
            items.setdefault("setup2", []).append("up")

        def teardown2():
            items.setdefault("setup2", []).append("down")

        def test_setup_module_setup():
            assert items["setup"] == ["up"]

        def test_setup_module_setup_again():
            assert items["setup"] == ["up"]

        @with_setup(setup2, teardown2)
        def test_local_setup():
            assert items["setup"] == ["up"]
            assert items["setup2"] == ["up"]

        @with_setup(setup2, teardown2)
        def test_local_setup_again():
            assert items["setup"] == ["up"]
            assert items["setup2"] == ["up", "down", "up"]
    """
    )
    result = pytester.runpytest("-p", "nose")
    result.stdout.fnmatch_lines(["*4 passed*"])


def test_nose_style_setup_teardown(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        values = []

        def setup_module():
            values.append(1)

        def teardown_module():
            del values[0]

        def test_hello():
            assert values == [1]

        def test_world():
            assert values == [1]
        """
    )
    result = pytester.runpytest("-p", "nose")
    result.stdout.fnmatch_lines(["*2 passed*"])


def test_fixtures_nose_setup_issue8394(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        def setup_module():
            pass

        def teardown_module():
            pass

        def setup_function(func):
            pass

        def teardown_function(func):
            pass

        def test_world():
            pass

        class Test(object):
            def setup_class(cls):
                pass

            def teardown_class(cls):
                pass

            def setup_method(self, meth):
                pass

            def teardown_method(self, meth):
                pass

            def test_method(self): pass
        """
    )
    match = "*no docstring available*"
    result = pytester.runpytest("--fixtures")
    assert result.ret == 0
    result.stdout.no_fnmatch_line(match)

    result = pytester.runpytest("--fixtures", "-v")
    assert result.ret == 0
    result.stdout.fnmatch_lines([match, match, match, match])


def test_nose_setup_ordering(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        def setup_module(mod):
            mod.visited = True

        class TestClass(object):
            def setup(self):
                assert visited
                self.visited_cls = True
            def test_first(self):
                assert visited
                assert self.visited_cls
        """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_apiwrapper_problem_issue260(pytester: Pytester) -> None:
    # this would end up trying a call an optional teardown on the class
    # for plain unittests we don't want nose behaviour
    pytester.makepyfile(
        """
        import unittest
        class TestCase(unittest.TestCase):
            def setup(self):
                #should not be called in unittest testcases
                assert 0, 'setup'
            def teardown(self):
                #should not be called in unittest testcases
                assert 0, 'teardown'
            def setUp(self):
                print('setup')
            def tearDown(self):
                print('teardown')
            def test_fun(self):
                pass
        """
    )
    result = pytester.runpytest()
    result.assert_outcomes(passed=1)


def test_setup_teardown_linking_issue265(pytester: Pytester) -> None:
    # we accidentally didn't integrate nose setupstate with normal setupstate
    # this test ensures that won't happen again
    pytester.makepyfile(
        '''
        import pytest

        class TestGeneric(object):
            def test_nothing(self):
                """Tests the API of the implementation (for generic and specialized)."""

        @pytest.mark.skipif("True", reason=
                    "Skip tests to check if teardown is skipped as well.")
        class TestSkipTeardown(TestGeneric):

            def setup(self):
                """Sets up my specialized implementation for $COOL_PLATFORM."""
                raise Exception("should not call setup for skipped tests")

            def teardown(self):
                """Undoes the setup."""
                raise Exception("should not call teardown for skipped tests")
        '''
    )
    reprec = pytester.runpytest()
    reprec.assert_outcomes(passed=1, skipped=1)


def test_SkipTest_during_collection(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import nose
        raise nose.SkipTest("during collection")
        def test_failing():
            assert False
        """
    )
    result = pytester.runpytest(p)
    result.assert_outcomes(skipped=1, warnings=1)


def test_SkipTest_in_test(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import nose

        def test_skipping():
            raise nose.SkipTest("in test")
        """
    )
    reprec = pytester.inline_run()
    reprec.assertoutcome(skipped=1)


def test_istest_function_decorator(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import nose.tools
        @nose.tools.istest
        def not_test_prefix():
            pass
        """
    )
    result = pytester.runpytest(p)
    result.assert_outcomes(passed=1)


def test_nottest_function_decorator(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import nose.tools
        @nose.tools.nottest
        def test_prefix():
            pass
        """
    )
    reprec = pytester.inline_run()
    assert not reprec.getfailedcollections()
    calls = reprec.getreports("pytest_runtest_logreport")
    assert not calls


def test_istest_class_decorator(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import nose.tools
        @nose.tools.istest
        class NotTestPrefix(object):
            def test_method(self):
                pass
        """
    )
    result = pytester.runpytest(p)
    result.assert_outcomes(passed=1)


def test_nottest_class_decorator(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import nose.tools
        @nose.tools.nottest
        class TestPrefix(object):
            def test_method(self):
                pass
        """
    )
    reprec = pytester.inline_run()
    assert not reprec.getfailedcollections()
    calls = reprec.getreports("pytest_runtest_logreport")
    assert not calls


def test_skip_test_with_unicode(pytester: Pytester) -> None:
    pytester.makepyfile(
        """\
        import unittest
        class TestClass():
            def test_io(self):
                raise unittest.SkipTest('😊')
        """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["* 1 skipped *"])


def test_raises(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        from nose.tools import raises

        @raises(RuntimeError)
        def test_raises_runtimeerror():
            raise RuntimeError

        @raises(Exception)
        def test_raises_baseexception_not_caught():
            raise BaseException

        @raises(BaseException)
        def test_raises_baseexception_caught():
            raise BaseException
        """
    )
    result = pytester.runpytest("-vv")
    result.stdout.fnmatch_lines(
        [
            "test_raises.py::test_raises_runtimeerror PASSED*",
            "test_raises.py::test_raises_baseexception_not_caught FAILED*",
            "test_raises.py::test_raises_baseexception_caught PASSED*",
            "*= FAILURES =*",
            "*_ test_raises_baseexception_not_caught _*",
            "",
            "arg = (), kw = {}",
            "",
            "    def newfunc(*arg, **kw):",
            "        try:",
            ">           func(*arg, **kw)",
            "",
            "*/nose/*: ",
            "_ _ *",
            "",
            "    @raises(Exception)",
            "    def test_raises_baseexception_not_caught():",
            ">       raise BaseException",
            "E       BaseException",
            "",
            "test_raises.py:9: BaseException",
            "* 1 failed, 2 passed *",
        ]
    )


def test_nose_setup_skipped_if_non_callable(pytester: Pytester) -> None:
    """Regression test for #9391."""
    p = pytester.makepyfile(
        __init__="",
        setup="""
        """,
        teardown="""
        """,
        test_it="""
        from . import setup, teardown

        def test_it():
            pass
        """,
    )
    result = pytester.runpytest(p, "-p", "nose")
    assert result.ret == 0
