from _pytest.main import EXIT_NOTESTSCOLLECTED
import pytest

def test_simple_unittest(testdir):
    testpath = testdir.makepyfile("""
        import unittest
        class MyTestCase(unittest.TestCase):
            def testpassing(self):
                self.assertEquals('foo', 'foo')
            def test_failing(self):
                self.assertEquals('foo', 'bar')
    """)
    reprec = testdir.inline_run(testpath)
    assert reprec.matchreport("testpassing").passed
    assert reprec.matchreport("test_failing").failed

def test_runTest_method(testdir):
    testdir.makepyfile("""
        import unittest
        class MyTestCaseWithRunTest(unittest.TestCase):
            def runTest(self):
                self.assertEquals('foo', 'foo')
        class MyTestCaseWithoutRunTest(unittest.TestCase):
            def runTest(self):
                self.assertEquals('foo', 'foo')
            def test_something(self):
                pass
        """)
    result = testdir.runpytest("-v")
    result.stdout.fnmatch_lines("""
        *MyTestCaseWithRunTest::runTest*
        *MyTestCaseWithoutRunTest::test_something*
        *2 passed*
    """)

def test_isclasscheck_issue53(testdir):
    testpath = testdir.makepyfile("""
        import unittest
        class _E(object):
            def __getattr__(self, tag):
                pass
        E = _E()
    """)
    result = testdir.runpytest(testpath)
    assert result.ret == EXIT_NOTESTSCOLLECTED

def test_setup(testdir):
    testpath = testdir.makepyfile("""
        import unittest
        class MyTestCase(unittest.TestCase):
            def setUp(self):
                self.foo = 1
            def setup_method(self, method):
                self.foo2 = 1
            def test_both(self):
                self.assertEquals(1, self.foo)
                assert self.foo2 == 1
            def teardown_method(self, method):
                assert 0, "42"

    """)
    reprec = testdir.inline_run("-s", testpath)
    assert reprec.matchreport("test_both", when="call").passed
    rep = reprec.matchreport("test_both", when="teardown")
    assert rep.failed and '42' in str(rep.longrepr)

def test_setUpModule(testdir):
    testpath = testdir.makepyfile("""
        l = []

        def setUpModule():
            l.append(1)

        def tearDownModule():
            del l[0]

        def test_hello():
            assert l == [1]

        def test_world():
            assert l == [1]
        """)
    result = testdir.runpytest(testpath)
    result.stdout.fnmatch_lines([
        "*2 passed*",
    ])

def test_setUpModule_failing_no_teardown(testdir):
    testpath = testdir.makepyfile("""
        l = []

        def setUpModule():
            0/0

        def tearDownModule():
            l.append(1)

        def test_hello():
            pass
    """)
    reprec = testdir.inline_run(testpath)
    reprec.assertoutcome(passed=0, failed=1)
    call = reprec.getcalls("pytest_runtest_setup")[0]
    assert not call.item.module.l

def test_new_instances(testdir):
    testpath = testdir.makepyfile("""
        import unittest
        class MyTestCase(unittest.TestCase):
            def test_func1(self):
                self.x = 2
            def test_func2(self):
                assert not hasattr(self, 'x')
    """)
    reprec = testdir.inline_run(testpath)
    reprec.assertoutcome(passed=2)

def test_teardown(testdir):
    testpath = testdir.makepyfile("""
        import unittest
        class MyTestCase(unittest.TestCase):
            l = []
            def test_one(self):
                pass
            def tearDown(self):
                self.l.append(None)
        class Second(unittest.TestCase):
            def test_check(self):
                self.assertEquals(MyTestCase.l, [None])
    """)
    reprec = testdir.inline_run(testpath)
    passed, skipped, failed = reprec.countoutcomes()
    assert failed == 0, failed
    assert passed == 2
    assert passed + skipped + failed == 2

@pytest.mark.skipif("sys.version_info < (2,7)")
def test_unittest_skip_issue148(testdir):
    testpath = testdir.makepyfile("""
        import unittest

        @unittest.skip("hello")
        class MyTestCase(unittest.TestCase):
            @classmethod
            def setUpClass(self):
                xxx
            def test_one(self):
                pass
            @classmethod
            def tearDownClass(self):
                xxx
    """)
    reprec = testdir.inline_run(testpath)
    reprec.assertoutcome(skipped=1)

def test_method_and_teardown_failing_reporting(testdir):
    testdir.makepyfile("""
        import unittest, pytest
        class TC(unittest.TestCase):
            def tearDown(self):
                assert 0, "down1"
            def test_method(self):
                assert False, "down2"
    """)
    result = testdir.runpytest("-s")
    assert result.ret == 1
    result.stdout.fnmatch_lines([
        "*tearDown*",
        "*assert 0*",
        "*test_method*",
        "*assert False*",
        "*1 failed*1 error*",
    ])

def test_setup_failure_is_shown(testdir):
    testdir.makepyfile("""
        import unittest
        import pytest
        class TC(unittest.TestCase):
            def setUp(self):
                assert 0, "down1"
            def test_method(self):
                print ("never42")
                xyz
    """)
    result = testdir.runpytest("-s")
    assert result.ret == 1
    result.stdout.fnmatch_lines([
        "*setUp*",
        "*assert 0*down1*",
        "*1 failed*",
    ])
    assert 'never42' not in result.stdout.str()

def test_setup_setUpClass(testdir):
    testpath = testdir.makepyfile("""
        import unittest
        import pytest
        class MyTestCase(unittest.TestCase):
            x = 0
            @classmethod
            def setUpClass(cls):
                cls.x += 1
            def test_func1(self):
                assert self.x == 1
            def test_func2(self):
                assert self.x == 1
            @classmethod
            def tearDownClass(cls):
                cls.x -= 1
        def test_teareddown():
            assert MyTestCase.x == 0
    """)
    reprec = testdir.inline_run(testpath)
    reprec.assertoutcome(passed=3)

def test_setup_class(testdir):
    testpath = testdir.makepyfile("""
        import unittest
        import pytest
        class MyTestCase(unittest.TestCase):
            x = 0
            def setup_class(cls):
                cls.x += 1
            def test_func1(self):
                assert self.x == 1
            def test_func2(self):
                assert self.x == 1
            def teardown_class(cls):
                cls.x -= 1
        def test_teareddown():
            assert MyTestCase.x == 0
    """)
    reprec = testdir.inline_run(testpath)
    reprec.assertoutcome(passed=3)


@pytest.mark.parametrize("type", ['Error', 'Failure'])
def test_testcase_adderrorandfailure_defers(testdir, type):
    testdir.makepyfile("""
        from unittest import TestCase
        import pytest
        class MyTestCase(TestCase):
            def run(self, result):
                excinfo = pytest.raises(ZeroDivisionError, lambda: 0/0)
                try:
                    result.add%s(self, excinfo._excinfo)
                except KeyboardInterrupt:
                    raise
                except:
                    pytest.fail("add%s should not raise")
            def test_hello(self):
                pass
    """ % (type, type))
    result = testdir.runpytest()
    assert 'should not raise' not in result.stdout.str()

@pytest.mark.parametrize("type", ['Error', 'Failure'])
def test_testcase_custom_exception_info(testdir, type):
    testdir.makepyfile("""
        from unittest import TestCase
        import py, pytest
        import _pytest._code
        class MyTestCase(TestCase):
            def run(self, result):
                excinfo = pytest.raises(ZeroDivisionError, lambda: 0/0)
                # we fake an incompatible exception info
                from _pytest.monkeypatch import monkeypatch
                mp = monkeypatch()
                def t(*args):
                    mp.undo()
                    raise TypeError()
                mp.setattr(_pytest._code, 'ExceptionInfo', t)
                try:
                    excinfo = excinfo._excinfo
                    result.add%(type)s(self, excinfo)
                finally:
                    mp.undo()
            def test_hello(self):
                pass
    """ % locals())
    result = testdir.runpytest()
    result.stdout.fnmatch_lines([
        "NOTE: Incompatible Exception Representation*",
        "*ZeroDivisionError*",
        "*1 failed*",
    ])

def test_testcase_totally_incompatible_exception_info(testdir):
    item, = testdir.getitems("""
        from unittest import TestCase
        class MyTestCase(TestCase):
            def test_hello(self):
                pass
    """)
    item.addError(None, 42)
    excinfo = item._excinfo.pop(0)
    assert 'ERROR: Unknown Incompatible' in str(excinfo.getrepr())

def test_module_level_pytestmark(testdir):
    testpath = testdir.makepyfile("""
        import unittest
        import pytest
        pytestmark = pytest.mark.xfail
        class MyTestCase(unittest.TestCase):
            def test_func1(self):
                assert 0
    """)
    reprec = testdir.inline_run(testpath, "-s")
    reprec.assertoutcome(skipped=1)


def test_trial_testcase_skip_property(testdir):
    pytest.importorskip('twisted.trial.unittest')
    testpath = testdir.makepyfile("""
        from twisted.trial import unittest
        class MyTestCase(unittest.TestCase):
            skip = 'dont run'
            def test_func(self):
                pass
        """)
    reprec = testdir.inline_run(testpath, "-s")
    reprec.assertoutcome(skipped=1)


def test_trial_testfunction_skip_property(testdir):
    pytest.importorskip('twisted.trial.unittest')
    testpath = testdir.makepyfile("""
        from twisted.trial import unittest
        class MyTestCase(unittest.TestCase):
            def test_func(self):
                pass
            test_func.skip = 'dont run'
        """)
    reprec = testdir.inline_run(testpath, "-s")
    reprec.assertoutcome(skipped=1)


def test_trial_testcase_todo_property(testdir):
    pytest.importorskip('twisted.trial.unittest')
    testpath = testdir.makepyfile("""
        from twisted.trial import unittest
        class MyTestCase(unittest.TestCase):
            todo = 'dont run'
            def test_func(self):
                assert 0
        """)
    reprec = testdir.inline_run(testpath, "-s")
    reprec.assertoutcome(skipped=1)


def test_trial_testfunction_todo_property(testdir):
    pytest.importorskip('twisted.trial.unittest')
    testpath = testdir.makepyfile("""
        from twisted.trial import unittest
        class MyTestCase(unittest.TestCase):
            def test_func(self):
                assert 0
            test_func.todo = 'dont run'
        """)
    reprec = testdir.inline_run(testpath, "-s")
    reprec.assertoutcome(skipped=1)


class TestTrialUnittest:
    def setup_class(cls):
        cls.ut = pytest.importorskip("twisted.trial.unittest")

    def test_trial_testcase_runtest_not_collected(self, testdir):
        testdir.makepyfile("""
            from twisted.trial.unittest import TestCase

            class TC(TestCase):
                def test_hello(self):
                    pass
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)
        testdir.makepyfile("""
            from twisted.trial.unittest import TestCase

            class TC(TestCase):
                def runTest(self):
                    pass
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_trial_exceptions_with_skips(self, testdir):
        testdir.makepyfile("""
            from twisted.trial import unittest
            import pytest
            class TC(unittest.TestCase):
                def test_hello(self):
                    pytest.skip("skip_in_method")
                @pytest.mark.skipif("sys.version_info != 1")
                def test_hello2(self):
                    pass
                @pytest.mark.xfail(reason="iwanto")
                def test_hello3(self):
                    assert 0
                def test_hello4(self):
                    pytest.xfail("i2wanto")
                def test_trial_skip(self):
                    pass
                test_trial_skip.skip = "trialselfskip"

                def test_trial_todo(self):
                    assert 0
                test_trial_todo.todo = "mytodo"

                def test_trial_todo_success(self):
                    pass
                test_trial_todo_success.todo = "mytodo"

            class TC2(unittest.TestCase):
                def setup_class(cls):
                    pytest.skip("skip_in_setup_class")
                def test_method(self):
                    pass
        """)
        result = testdir.runpytest("-rxs")
        assert result.ret == 0
        result.stdout.fnmatch_lines_random([
            "*XFAIL*test_trial_todo*",
            "*trialselfskip*",
            "*skip_in_setup_class*",
            "*iwanto*",
            "*i2wanto*",
            "*sys.version_info*",
            "*skip_in_method*",
            "*4 skipped*3 xfail*1 xpass*",
        ])

    def test_trial_error(self, testdir):
        testdir.makepyfile("""
            from twisted.trial.unittest import TestCase
            from twisted.internet.defer import Deferred
            from twisted.internet import reactor

            class TC(TestCase):
                def test_one(self):
                    crash

                def test_two(self):
                    def f(_):
                        crash

                    d = Deferred()
                    d.addCallback(f)
                    reactor.callLater(0.3, d.callback, None)
                    return d

                def test_three(self):
                    def f():
                        pass # will never get called
                    reactor.callLater(0.3, f)
                # will crash at teardown

                def test_four(self):
                    def f(_):
                        reactor.callLater(0.3, f)
                        crash

                    d = Deferred()
                    d.addCallback(f)
                    reactor.callLater(0.3, d.callback, None)
                    return d
                # will crash both at test time and at teardown
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*ERRORS*",
            "*DelayedCalls*",
            "*test_four*",
            "*NameError*crash*",
            "*test_one*",
            "*NameError*crash*",
            "*test_three*",
            "*DelayedCalls*",
            "*test_two*",
            "*crash*",
        ])

    def test_trial_pdb(self, testdir):
        p = testdir.makepyfile("""
            from twisted.trial import unittest
            import pytest
            class TC(unittest.TestCase):
                def test_hello(self):
                    assert 0, "hellopdb"
        """)
        child = testdir.spawn_pytest(p)
        child.expect("hellopdb")
        child.sendeof()

def test_djangolike_testcase(testdir):
    # contributed from Morten Breekevold
    testdir.makepyfile("""
        from unittest import TestCase, main

        class DjangoLikeTestCase(TestCase):

            def setUp(self):
                print ("setUp()")

            def test_presetup_has_been_run(self):
                print ("test_thing()")
                self.assertTrue(hasattr(self, 'was_presetup'))

            def tearDown(self):
                print ("tearDown()")

            def __call__(self, result=None):
                try:
                    self._pre_setup()
                except (KeyboardInterrupt, SystemExit):
                    raise
                except Exception:
                    import sys
                    result.addError(self, sys.exc_info())
                    return
                super(DjangoLikeTestCase, self).__call__(result)
                try:
                    self._post_teardown()
                except (KeyboardInterrupt, SystemExit):
                    raise
                except Exception:
                    import sys
                    result.addError(self, sys.exc_info())
                    return

            def _pre_setup(self):
                print ("_pre_setup()")
                self.was_presetup = True

            def _post_teardown(self):
                print ("_post_teardown()")
    """)
    result = testdir.runpytest("-s")
    assert result.ret == 0
    result.stdout.fnmatch_lines([
        "*_pre_setup()*",
        "*setUp()*",
        "*test_thing()*",
        "*tearDown()*",
        "*_post_teardown()*",
    ])


def test_unittest_not_shown_in_traceback(testdir):
    testdir.makepyfile("""
        import unittest
        class t(unittest.TestCase):
            def test_hello(self):
                x = 3
                self.assertEquals(x, 4)
    """)
    res = testdir.runpytest()
    assert "failUnlessEqual" not in res.stdout.str()

def test_unorderable_types(testdir):
    testdir.makepyfile("""
        import unittest
        class TestJoinEmpty(unittest.TestCase):
            pass

        def make_test():
            class Test(unittest.TestCase):
                pass
            Test.__name__ = "TestFoo"
            return Test
        TestFoo = make_test()
    """)
    result = testdir.runpytest()
    assert "TypeError" not in result.stdout.str()
    assert result.ret == EXIT_NOTESTSCOLLECTED

def test_unittest_typerror_traceback(testdir):
    testdir.makepyfile("""
        import unittest
        class TestJoinEmpty(unittest.TestCase):
            def test_hello(self, arg1):
                pass
    """)
    result = testdir.runpytest()
    assert "TypeError" in result.stdout.str()
    assert result.ret == 1

@pytest.mark.skipif("sys.version_info < (2,7)")
def test_unittest_unexpected_failure(testdir):
    testdir.makepyfile("""
        import unittest
        class MyTestCase(unittest.TestCase):
            @unittest.expectedFailure
            def test_func1(self):
                assert 0
            @unittest.expectedFailure
            def test_func2(self):
                assert 1
    """)
    result = testdir.runpytest("-rxX")
    result.stdout.fnmatch_lines([
        "*XFAIL*MyTestCase*test_func1*",
        "*XPASS*MyTestCase*test_func2*",
        "*1 xfailed*1 xpass*",
    ])


@pytest.mark.parametrize('fix_type, stmt', [
    ('fixture', 'return'),
    ('yield_fixture', 'yield'),
])
def test_unittest_setup_interaction(testdir, fix_type, stmt):
    testdir.makepyfile("""
        import unittest
        import pytest
        class MyTestCase(unittest.TestCase):
            @pytest.{fix_type}(scope="class", autouse=True)
            def perclass(self, request):
                request.cls.hello = "world"
                {stmt}
            @pytest.{fix_type}(scope="function", autouse=True)
            def perfunction(self, request):
                request.instance.funcname = request.function.__name__
                {stmt}

            def test_method1(self):
                assert self.funcname == "test_method1"
                assert self.hello == "world"

            def test_method2(self):
                assert self.funcname == "test_method2"

            def test_classattr(self):
                assert self.__class__.hello == "world"
    """.format(fix_type=fix_type, stmt=stmt))
    result = testdir.runpytest()
    result.stdout.fnmatch_lines("*3 passed*")


def test_non_unittest_no_setupclass_support(testdir):
    testpath = testdir.makepyfile("""
        class TestFoo:
            x = 0

            @classmethod
            def setUpClass(cls):
                cls.x = 1

            def test_method1(self):
                assert self.x == 0

            @classmethod
            def tearDownClass(cls):
                cls.x = 1

        def test_not_teareddown():
            assert TestFoo.x == 0

    """)
    reprec = testdir.inline_run(testpath)
    reprec.assertoutcome(passed=2)


def test_no_teardown_if_setupclass_failed(testdir):
    testpath = testdir.makepyfile("""
        import unittest

        class MyTestCase(unittest.TestCase):
            x = 0

            @classmethod
            def setUpClass(cls):
                cls.x = 1
                assert False

            def test_func1(self):
                cls.x = 10

            @classmethod
            def tearDownClass(cls):
                cls.x = 100

        def test_notTornDown():
            assert MyTestCase.x == 1
    """)
    reprec = testdir.inline_run(testpath)
    reprec.assertoutcome(passed=1, failed=1)


def test_issue333_result_clearing(testdir):
    testdir.makeconftest("""
        def pytest_runtest_call(__multicall__, item):
            __multicall__.execute()
            assert 0
    """)
    testdir.makepyfile("""
        import unittest
        class TestIt(unittest.TestCase):
            def test_func(self):
                0/0
    """)

    reprec = testdir.inline_run()
    reprec.assertoutcome(failed=1)

@pytest.mark.skipif("sys.version_info < (2,7)")
def test_unittest_raise_skip_issue748(testdir):
    testdir.makepyfile(test_foo="""
        import unittest

        class MyTestCase(unittest.TestCase):
            def test_one(self):
                raise unittest.SkipTest('skipping due to reasons')
    """)
    result = testdir.runpytest("-v", '-rs')
    result.stdout.fnmatch_lines("""
        *SKIP*[1]*test_foo.py*skipping due to reasons*
        *1 skipped*
    """)

@pytest.mark.skipif("sys.version_info < (2,7)")
def test_unittest_skip_issue1169(testdir):
    testdir.makepyfile(test_foo="""
        import unittest
        
        class MyTestCase(unittest.TestCase):
            @unittest.skip("skipping due to reasons")
            def test_skip(self):
                 self.fail()
        """)
    result = testdir.runpytest("-v", '-rs')
    result.stdout.fnmatch_lines("""
        *SKIP*[1]*skipping due to reasons*
        *1 skipped*
    """)
