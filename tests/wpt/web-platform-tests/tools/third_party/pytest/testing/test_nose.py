# -*- coding: utf-8 -*-
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import pytest


def setup_module(mod):
    mod.nose = pytest.importorskip("nose")


def test_nose_setup(testdir):
    p = testdir.makepyfile(
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
    result = testdir.runpytest(p, "-p", "nose")
    result.assert_outcomes(passed=2)


def test_setup_func_with_setup_decorator():
    from _pytest.nose import call_optional

    values = []

    class A(object):
        @pytest.fixture(autouse=True)
        def f(self):
            values.append(1)

    call_optional(A(), "f")
    assert not values


def test_setup_func_not_callable():
    from _pytest.nose import call_optional

    class A(object):
        f = 1

    call_optional(A(), "f")


def test_nose_setup_func(testdir):
    p = testdir.makepyfile(
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
    result = testdir.runpytest(p, "-p", "nose")
    result.assert_outcomes(passed=2)


def test_nose_setup_func_failure(testdir):
    p = testdir.makepyfile(
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
    result = testdir.runpytest(p, "-p", "nose")
    result.stdout.fnmatch_lines(["*TypeError: <lambda>()*"])


def test_nose_setup_func_failure_2(testdir):
    testdir.makepyfile(
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
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


def test_nose_setup_partial(testdir):
    pytest.importorskip("functools")
    p = testdir.makepyfile(
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
    result = testdir.runpytest(p, "-p", "nose")
    result.stdout.fnmatch_lines(["*2 passed*"])


def test_module_level_setup(testdir):
    testdir.makepyfile(
        """
        from nose.tools import with_setup
        items = {}

        def setup():
            items[1]=1

        def teardown():
            del items[1]

        def setup2():
            items[2] = 2

        def teardown2():
            del items[2]

        def test_setup_module_setup():
            assert items[1] == 1

        @with_setup(setup2, teardown2)
        def test_local_setup():
            assert items[2] == 2
            assert 1 not in items
    """
    )
    result = testdir.runpytest("-p", "nose")
    result.stdout.fnmatch_lines(["*2 passed*"])


def test_nose_style_setup_teardown(testdir):
    testdir.makepyfile(
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
    result = testdir.runpytest("-p", "nose")
    result.stdout.fnmatch_lines(["*2 passed*"])


def test_nose_setup_ordering(testdir):
    testdir.makepyfile(
        """
        def setup_module(mod):
            mod.visited = True

        class TestClass(object):
            def setup(self):
                assert visited
            def test_first(self):
                pass
        """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_apiwrapper_problem_issue260(testdir):
    # this would end up trying a call an optional teardown on the class
    # for plain unittests we dont want nose behaviour
    testdir.makepyfile(
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
    result = testdir.runpytest()
    result.assert_outcomes(passed=1)


def test_setup_teardown_linking_issue265(testdir):
    # we accidentally didnt integrate nose setupstate with normal setupstate
    # this test ensures that won't happen again
    testdir.makepyfile(
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
    reprec = testdir.runpytest()
    reprec.assert_outcomes(passed=1, skipped=1)


def test_SkipTest_during_collection(testdir):
    p = testdir.makepyfile(
        """
        import nose
        raise nose.SkipTest("during collection")
        def test_failing():
            assert False
        """
    )
    result = testdir.runpytest(p)
    result.assert_outcomes(skipped=1)


def test_SkipTest_in_test(testdir):
    testdir.makepyfile(
        """
        import nose

        def test_skipping():
            raise nose.SkipTest("in test")
        """
    )
    reprec = testdir.inline_run()
    reprec.assertoutcome(skipped=1)


def test_istest_function_decorator(testdir):
    p = testdir.makepyfile(
        """
        import nose.tools
        @nose.tools.istest
        def not_test_prefix():
            pass
        """
    )
    result = testdir.runpytest(p)
    result.assert_outcomes(passed=1)


def test_nottest_function_decorator(testdir):
    testdir.makepyfile(
        """
        import nose.tools
        @nose.tools.nottest
        def test_prefix():
            pass
        """
    )
    reprec = testdir.inline_run()
    assert not reprec.getfailedcollections()
    calls = reprec.getreports("pytest_runtest_logreport")
    assert not calls


def test_istest_class_decorator(testdir):
    p = testdir.makepyfile(
        """
        import nose.tools
        @nose.tools.istest
        class NotTestPrefix(object):
            def test_method(self):
                pass
        """
    )
    result = testdir.runpytest(p)
    result.assert_outcomes(passed=1)


def test_nottest_class_decorator(testdir):
    testdir.makepyfile(
        """
        import nose.tools
        @nose.tools.nottest
        class TestPrefix(object):
            def test_method(self):
                pass
        """
    )
    reprec = testdir.inline_run()
    assert not reprec.getfailedcollections()
    calls = reprec.getreports("pytest_runtest_logreport")
    assert not calls


def test_skip_test_with_unicode(testdir):
    testdir.makepyfile(
        """
        # -*- coding: utf-8 -*-
        import unittest
        class TestClass():
            def test_io(self):
                raise unittest.SkipTest(u'😊')
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["* 1 skipped *"])
