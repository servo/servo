from textwrap import dedent

import _pytest._code
import pytest
from _pytest.pytester import get_public_names
from _pytest.fixtures import FixtureLookupError
from _pytest import fixtures


def test_getfuncargnames():
    def f():
        pass
    assert not fixtures.getfuncargnames(f)

    def g(arg):
        pass
    assert fixtures.getfuncargnames(g) == ('arg',)

    def h(arg1, arg2="hello"):
        pass
    assert fixtures.getfuncargnames(h) == ('arg1',)

    def h(arg1, arg2, arg3="hello"):
        pass
    assert fixtures.getfuncargnames(h) == ('arg1', 'arg2')

    class A(object):
        def f(self, arg1, arg2="hello"):
            pass

        @staticmethod
        def static(arg1, arg2):
            pass

    assert fixtures.getfuncargnames(A().f) == ('arg1',)
    assert fixtures.getfuncargnames(A.static, cls=A) == ('arg1', 'arg2')


class TestFillFixtures(object):
    def test_fillfuncargs_exposed(self):
        # used by oejskit, kept for compatibility
        assert pytest._fillfuncargs == fixtures.fillfixtures

    def test_funcarg_lookupfails(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def xyzsomething(request):
                return 42

            def test_func(some):
                pass
        """)
        result = testdir.runpytest()  # "--collect-only")
        assert result.ret != 0
        result.stdout.fnmatch_lines([
            "*def test_func(some)*",
            "*fixture*some*not found*",
            "*xyzsomething*",
        ])

    def test_funcarg_basic(self, testdir):
        item = testdir.getitem("""
            import pytest

            @pytest.fixture
            def some(request):
                return request.function.__name__
            @pytest.fixture
            def other(request):
                return 42
            def test_func(some, other):
                pass
        """)
        fixtures.fillfixtures(item)
        del item.funcargs["request"]
        assert len(get_public_names(item.funcargs)) == 2
        assert item.funcargs['some'] == "test_func"
        assert item.funcargs['other'] == 42

    def test_funcarg_lookup_modulelevel(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def something(request):
                return request.function.__name__

            class TestClass(object):
                def test_method(self, something):
                    assert something == "test_method"
            def test_func(something):
                assert something == "test_func"
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    def test_funcarg_lookup_classlevel(self, testdir):
        p = testdir.makepyfile("""
            import pytest
            class TestClass(object):

                @pytest.fixture
                def something(self, request):
                    return request.instance

                def test_method(self, something):
                    assert something is self
        """)
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines([
            "*1 passed*"
        ])

    def test_conftest_funcargs_only_available_in_subdir(self, testdir):
        sub1 = testdir.mkpydir("sub1")
        sub2 = testdir.mkpydir("sub2")
        sub1.join("conftest.py").write(_pytest._code.Source("""
            import pytest
            @pytest.fixture
            def arg1(request):
                pytest.raises(Exception, "request.getfixturevalue('arg2')")
        """))
        sub2.join("conftest.py").write(_pytest._code.Source("""
            import pytest
            @pytest.fixture
            def arg2(request):
                pytest.raises(Exception, "request.getfixturevalue('arg1')")
        """))

        sub1.join("test_in_sub1.py").write("def test_1(arg1): pass")
        sub2.join("test_in_sub2.py").write("def test_2(arg2): pass")
        result = testdir.runpytest("-v")
        result.assert_outcomes(passed=2)

    def test_extend_fixture_module_class(self, testdir):
        testfile = testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'

            class TestSpam(object):

                 @pytest.fixture
                 def spam(self, spam):
                     return spam * 2

                 def test_spam(self, spam):
                     assert spam == 'spamspam'
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = testdir.runpytest(testfile)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_extend_fixture_conftest_module(self, testdir):
        testdir.makeconftest("""
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'
        """)
        testfile = testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def spam(spam):
                return spam * 2

            def test_spam(spam):
                assert spam == 'spamspam'
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = testdir.runpytest(testfile)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_extend_fixture_conftest_conftest(self, testdir):
        testdir.makeconftest("""
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'
        """)
        pkg = testdir.mkpydir("pkg")
        pkg.join("conftest.py").write(_pytest._code.Source("""
            import pytest

            @pytest.fixture
            def spam(spam):
                return spam * 2
        """))
        testfile = pkg.join("test_spam.py")
        testfile.write(_pytest._code.Source("""
            def test_spam(spam):
                assert spam == "spamspam"
        """))
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = testdir.runpytest(testfile)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_extend_fixture_conftest_plugin(self, testdir):
        testdir.makepyfile(testplugin="""
            import pytest

            @pytest.fixture
            def foo():
                return 7
        """)
        testdir.syspathinsert()
        testdir.makeconftest("""
            import pytest

            pytest_plugins = 'testplugin'

            @pytest.fixture
            def foo(foo):
                return foo + 7
        """)
        testdir.makepyfile("""
            def test_foo(foo):
                assert foo == 14
        """)
        result = testdir.runpytest('-s')
        assert result.ret == 0

    def test_extend_fixture_plugin_plugin(self, testdir):
        # Two plugins should extend each order in loading order
        testdir.makepyfile(testplugin0="""
            import pytest

            @pytest.fixture
            def foo():
                return 7
        """)
        testdir.makepyfile(testplugin1="""
            import pytest

            @pytest.fixture
            def foo(foo):
                return foo + 7
        """)
        testdir.syspathinsert()
        testdir.makepyfile("""
            pytest_plugins = ['testplugin0', 'testplugin1']

            def test_foo(foo):
                assert foo == 14
        """)
        result = testdir.runpytest()
        assert result.ret == 0

    def test_override_parametrized_fixture_conftest_module(self, testdir):
        """Test override of the parametrized fixture with non-parametrized one on the test module level."""
        testdir.makeconftest("""
            import pytest

            @pytest.fixture(params=[1, 2, 3])
            def spam(request):
                return request.param
        """)
        testfile = testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'

            def test_spam(spam):
                assert spam == 'spam'
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = testdir.runpytest(testfile)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_override_parametrized_fixture_conftest_conftest(self, testdir):
        """Test override of the parametrized fixture with non-parametrized one on the conftest level."""
        testdir.makeconftest("""
            import pytest

            @pytest.fixture(params=[1, 2, 3])
            def spam(request):
                return request.param
        """)
        subdir = testdir.mkpydir('subdir')
        subdir.join("conftest.py").write(_pytest._code.Source("""
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'
        """))
        testfile = subdir.join("test_spam.py")
        testfile.write(_pytest._code.Source("""
            def test_spam(spam):
                assert spam == "spam"
        """))
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = testdir.runpytest(testfile)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_override_non_parametrized_fixture_conftest_module(self, testdir):
        """Test override of the non-parametrized fixture with parametrized one on the test module level."""
        testdir.makeconftest("""
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'
        """)
        testfile = testdir.makepyfile("""
            import pytest

            @pytest.fixture(params=[1, 2, 3])
            def spam(request):
                return request.param

            params = {'spam': 1}

            def test_spam(spam):
                assert spam == params['spam']
                params['spam'] += 1
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*3 passed*"])
        result = testdir.runpytest(testfile)
        result.stdout.fnmatch_lines(["*3 passed*"])

    def test_override_non_parametrized_fixture_conftest_conftest(self, testdir):
        """Test override of the non-parametrized fixture with parametrized one on the conftest level."""
        testdir.makeconftest("""
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'
        """)
        subdir = testdir.mkpydir('subdir')
        subdir.join("conftest.py").write(_pytest._code.Source("""
            import pytest

            @pytest.fixture(params=[1, 2, 3])
            def spam(request):
                return request.param
        """))
        testfile = subdir.join("test_spam.py")
        testfile.write(_pytest._code.Source("""
            params = {'spam': 1}

            def test_spam(spam):
                assert spam == params['spam']
                params['spam'] += 1
        """))
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*3 passed*"])
        result = testdir.runpytest(testfile)
        result.stdout.fnmatch_lines(["*3 passed*"])

    def test_override_autouse_fixture_with_parametrized_fixture_conftest_conftest(self, testdir):
        """Test override of the autouse fixture with parametrized one on the conftest level.
        This test covers the issue explained in issue 1601
        """
        testdir.makeconftest("""
            import pytest

            @pytest.fixture(autouse=True)
            def spam():
                return 'spam'
        """)
        subdir = testdir.mkpydir('subdir')
        subdir.join("conftest.py").write(_pytest._code.Source("""
            import pytest

            @pytest.fixture(params=[1, 2, 3])
            def spam(request):
                return request.param
        """))
        testfile = subdir.join("test_spam.py")
        testfile.write(_pytest._code.Source("""
            params = {'spam': 1}

            def test_spam(spam):
                assert spam == params['spam']
                params['spam'] += 1
        """))
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*3 passed*"])
        result = testdir.runpytest(testfile)
        result.stdout.fnmatch_lines(["*3 passed*"])

    def test_autouse_fixture_plugin(self, testdir):
        # A fixture from a plugin has no baseid set, which screwed up
        # the autouse fixture handling.
        testdir.makepyfile(testplugin="""
            import pytest

            @pytest.fixture(autouse=True)
            def foo(request):
                request.function.foo = 7
        """)
        testdir.syspathinsert()
        testdir.makepyfile("""
            pytest_plugins = 'testplugin'

            def test_foo(request):
                assert request.function.foo == 7
        """)
        result = testdir.runpytest()
        assert result.ret == 0

    def test_funcarg_lookup_error(self, testdir):
        testdir.makeconftest("""
            import pytest

            @pytest.fixture
            def a_fixture(): pass

            @pytest.fixture
            def b_fixture(): pass

            @pytest.fixture
            def c_fixture(): pass

            @pytest.fixture
            def d_fixture(): pass
        """)
        testdir.makepyfile("""
            def test_lookup_error(unknown):
                pass
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*ERROR at setup of test_lookup_error*",
            "  def test_lookup_error(unknown):*",
            "E       fixture 'unknown' not found",
            ">       available fixtures:*a_fixture,*b_fixture,*c_fixture,*d_fixture*monkeypatch,*",  # sorted
            ">       use 'py*test --fixtures *' for help on them.",
            "*1 error*",
        ])
        assert "INTERNAL" not in result.stdout.str()

    def test_fixture_excinfo_leak(self, testdir):
        # on python2 sys.excinfo would leak into fixture executions
        testdir.makepyfile("""
            import sys
            import traceback
            import pytest

            @pytest.fixture
            def leak():
                if sys.exc_info()[0]:  # python3 bug :)
                    traceback.print_exc()
                #fails
                assert sys.exc_info() == (None, None, None)

            def test_leak(leak):
                if sys.exc_info()[0]:  # python3 bug :)
                    traceback.print_exc()
                assert sys.exc_info() == (None, None, None)
        """)
        result = testdir.runpytest()
        assert result.ret == 0


class TestRequestBasic(object):
    def test_request_attributes(self, testdir):
        item = testdir.getitem("""
            import pytest

            @pytest.fixture
            def something(request): pass
            def test_func(something): pass
        """)
        req = fixtures.FixtureRequest(item)
        assert req.function == item.obj
        assert req.keywords == item.keywords
        assert hasattr(req.module, 'test_func')
        assert req.cls is None
        assert req.function.__name__ == "test_func"
        assert req.config == item.config
        assert repr(req).find(req.function.__name__) != -1

    def test_request_attributes_method(self, testdir):
        item, = testdir.getitems("""
            import pytest
            class TestB(object):

                @pytest.fixture
                def something(self, request):
                    return 1
                def test_func(self, something):
                    pass
        """)
        req = item._request
        assert req.cls.__name__ == "TestB"
        assert req.instance.__class__ == req.cls

    def test_request_contains_funcarg_arg2fixturedefs(self, testdir):
        modcol = testdir.getmodulecol("""
            import pytest
            @pytest.fixture
            def something(request):
                pass
            class TestClass(object):
                def test_method(self, something):
                    pass
        """)
        item1, = testdir.genitems([modcol])
        assert item1.name == "test_method"
        arg2fixturedefs = fixtures.FixtureRequest(item1)._arg2fixturedefs
        assert len(arg2fixturedefs) == 1
        assert arg2fixturedefs['something'][0].argname == "something"

    def test_getfixturevalue_recursive(self, testdir):
        testdir.makeconftest("""
            import pytest

            @pytest.fixture
            def something(request):
                return 1
        """)
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def something(request):
                return request.getfixturevalue("something") + 1
            def test_func(something):
                assert something == 2
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    @pytest.mark.parametrize(
        'getfixmethod', ('getfixturevalue', 'getfuncargvalue'))
    def test_getfixturevalue(self, testdir, getfixmethod):
        item = testdir.getitem("""
            import pytest
            values = [2]
            @pytest.fixture
            def something(request): return 1
            @pytest.fixture
            def other(request):
                return values.pop()
            def test_func(something): pass
        """)
        import contextlib
        if getfixmethod == 'getfuncargvalue':
            warning_expectation = pytest.warns(DeprecationWarning)
        else:
            # see #1830 for a cleaner way to accomplish this
            @contextlib.contextmanager
            def expecting_no_warning():
                yield

            warning_expectation = expecting_no_warning()

        req = item._request
        with warning_expectation:
            fixture_fetcher = getattr(req, getfixmethod)
            with pytest.raises(FixtureLookupError):
                fixture_fetcher("notexists")
            val = fixture_fetcher("something")
            assert val == 1
            val = fixture_fetcher("something")
            assert val == 1
            val2 = fixture_fetcher("other")
            assert val2 == 2
            val2 = fixture_fetcher("other")  # see about caching
            assert val2 == 2
            pytest._fillfuncargs(item)
            assert item.funcargs["something"] == 1
            assert len(get_public_names(item.funcargs)) == 2
            assert "request" in item.funcargs

    def test_request_addfinalizer(self, testdir):
        item = testdir.getitem("""
            import pytest
            teardownlist = []
            @pytest.fixture
            def something(request):
                request.addfinalizer(lambda: teardownlist.append(1))
            def test_func(something): pass
        """)
        item.session._setupstate.prepare(item)
        pytest._fillfuncargs(item)
        # successively check finalization calls
        teardownlist = item.getparent(pytest.Module).obj.teardownlist
        ss = item.session._setupstate
        assert not teardownlist
        ss.teardown_exact(item, None)
        print(ss.stack)
        assert teardownlist == [1]

    def test_mark_as_fixture_with_prefix_and_decorator_fails(self, testdir):
        testdir.makeconftest("""
            import pytest

            @pytest.fixture
            def pytest_funcarg__marked_with_prefix_and_decorator():
                pass
        """)
        result = testdir.runpytest_subprocess()
        assert result.ret != 0
        result.stdout.fnmatch_lines([
            "*AssertionError: fixtures cannot have*@pytest.fixture*",
            "*pytest_funcarg__marked_with_prefix_and_decorator*"
        ])

    def test_request_addfinalizer_failing_setup(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = [1]
            @pytest.fixture
            def myfix(request):
                request.addfinalizer(values.pop)
                assert 0
            def test_fix(myfix):
                pass
            def test_finalizer_ran():
                assert not values
        """)
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(failed=1, passed=1)

    def test_request_addfinalizer_failing_setup_module(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = [1, 2]
            @pytest.fixture(scope="module")
            def myfix(request):
                request.addfinalizer(values.pop)
                request.addfinalizer(values.pop)
                assert 0
            def test_fix(myfix):
                pass
        """)
        reprec = testdir.inline_run("-s")
        mod = reprec.getcalls("pytest_runtest_setup")[0].item.module
        assert not mod.values

    def test_request_addfinalizer_partial_setup_failure(self, testdir):
        p = testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture
            def something(request):
                request.addfinalizer(lambda: values.append(None))
            def test_func(something, missingarg):
                pass
            def test_second():
                assert len(values) == 1
        """)
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines([
            "*1 error*"  # XXX the whole module collection fails
        ])

    def test_request_subrequest_addfinalizer_exceptions(self, testdir):
        """
        Ensure exceptions raised during teardown by a finalizer are suppressed
        until all finalizers are called, re-raising the first exception (#2440)
        """
        testdir.makepyfile("""
            import pytest
            values = []
            def _excepts(where):
                raise Exception('Error in %s fixture' % where)
            @pytest.fixture
            def subrequest(request):
                return request
            @pytest.fixture
            def something(subrequest):
                subrequest.addfinalizer(lambda: values.append(1))
                subrequest.addfinalizer(lambda: values.append(2))
                subrequest.addfinalizer(lambda: _excepts('something'))
            @pytest.fixture
            def excepts(subrequest):
                subrequest.addfinalizer(lambda: _excepts('excepts'))
                subrequest.addfinalizer(lambda: values.append(3))
            def test_first(something, excepts):
                pass
            def test_second():
                assert values == [3, 2, 1]
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            '*Exception: Error in excepts fixture',
            '* 2 passed, 1 error in *',
        ])

    def test_request_getmodulepath(self, testdir):
        modcol = testdir.getmodulecol("def test_somefunc(): pass")
        item, = testdir.genitems([modcol])
        req = fixtures.FixtureRequest(item)
        assert req.fspath == modcol.fspath

    def test_request_fixturenames(self, testdir):
        testdir.makepyfile("""
            import pytest
            from _pytest.pytester import get_public_names
            @pytest.fixture()
            def arg1():
                pass
            @pytest.fixture()
            def farg(arg1):
                pass
            @pytest.fixture(autouse=True)
            def sarg(tmpdir):
                pass
            def test_function(request, farg):
                assert set(get_public_names(request.fixturenames)) == \
                       set(["tmpdir", "sarg", "arg1", "request", "farg",
                            "tmpdir_factory"])
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_funcargnames_compatattr(self, testdir):
        testdir.makepyfile("""
            import pytest
            def pytest_generate_tests(metafunc):
                assert metafunc.funcargnames == metafunc.fixturenames
            @pytest.fixture
            def fn(request):
                assert request._pyfuncitem.funcargnames == \
                       request._pyfuncitem.fixturenames
                return request.funcargnames, request.fixturenames

            def test_hello(fn):
                assert fn[0] == fn[1]
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_setupdecorator_and_xunit(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(scope='module', autouse=True)
            def setup_module():
                values.append("module")
            @pytest.fixture(autouse=True)
            def setup_function():
                values.append("function")

            def test_func():
                pass

            class TestClass(object):
                @pytest.fixture(scope="class", autouse=True)
                def setup_class(self):
                    values.append("class")
                @pytest.fixture(autouse=True)
                def setup_method(self):
                    values.append("method")
                def test_method(self):
                    pass
            def test_all():
                assert values == ["module", "function", "class",
                             "function", "method", "function"]
        """)
        reprec = testdir.inline_run("-v")
        reprec.assertoutcome(passed=3)

    def test_fixtures_sub_subdir_normalize_sep(self, testdir):
        # this tests that normalization of nodeids takes place
        b = testdir.mkdir("tests").mkdir("unit")
        b.join("conftest.py").write(_pytest._code.Source("""
            import pytest
            @pytest.fixture
            def arg1():
                pass
        """))
        p = b.join("test_module.py")
        p.write("def test_func(arg1): pass")
        result = testdir.runpytest(p, "--fixtures")
        assert result.ret == 0
        result.stdout.fnmatch_lines("""
            *fixtures defined*conftest*
            *arg1*
        """)

    def test_show_fixtures_color_yes(self, testdir):
        testdir.makepyfile("def test_this(): assert 1")
        result = testdir.runpytest('--color=yes', '--fixtures')
        assert '\x1b[32mtmpdir' in result.stdout.str()

    def test_newstyle_with_request(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture()
            def arg(request):
                pass
            def test_1(arg):
                pass
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_setupcontext_no_param(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(params=[1,2])
            def arg(request):
                return request.param

            @pytest.fixture(autouse=True)
            def mysetup(request, arg):
                assert not hasattr(request, "param")
            def test_1(arg):
                assert arg in (1,2)
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)


class TestRequestMarking(object):
    def test_applymarker(self, testdir):
        item1, item2 = testdir.getitems("""
            import pytest

            @pytest.fixture
            def something(request):
                pass
            class TestClass(object):
                def test_func1(self, something):
                    pass
                def test_func2(self, something):
                    pass
        """)
        req1 = fixtures.FixtureRequest(item1)
        assert 'xfail' not in item1.keywords
        req1.applymarker(pytest.mark.xfail)
        assert 'xfail' in item1.keywords
        assert 'skipif' not in item1.keywords
        req1.applymarker(pytest.mark.skipif)
        assert 'skipif' in item1.keywords
        pytest.raises(ValueError, "req1.applymarker(42)")

    def test_accesskeywords(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture()
            def keywords(request):
                return request.keywords
            @pytest.mark.XYZ
            def test_function(keywords):
                assert keywords["XYZ"]
                assert "abc" not in keywords
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_accessmarker_dynamic(self, testdir):
        testdir.makeconftest("""
            import pytest
            @pytest.fixture()
            def keywords(request):
                return request.keywords

            @pytest.fixture(scope="class", autouse=True)
            def marking(request):
                request.applymarker(pytest.mark.XYZ("hello"))
        """)
        testdir.makepyfile("""
            import pytest
            def test_fun1(keywords):
                assert keywords["XYZ"] is not None
                assert "abc" not in keywords
            def test_fun2(keywords):
                assert keywords["XYZ"] is not None
                assert "abc" not in keywords
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)


class TestRequestCachedSetup(object):
    def test_request_cachedsetup_defaultmodule(self, testdir):
        reprec = testdir.inline_runsource("""
            mysetup = ["hello",].pop

            import pytest

            @pytest.fixture
            def something(request):
                return request.cached_setup(mysetup, scope="module")

            def test_func1(something):
                assert something == "hello"
            class TestClass(object):
                def test_func1a(self, something):
                    assert something == "hello"
        """)
        reprec.assertoutcome(passed=2)

    def test_request_cachedsetup_class(self, testdir):
        reprec = testdir.inline_runsource("""
            mysetup = ["hello", "hello2", "hello3"].pop

            import pytest
            @pytest.fixture
            def something(request):
                return request.cached_setup(mysetup, scope="class")
            def test_func1(something):
                assert something == "hello3"
            def test_func2(something):
                assert something == "hello2"
            class TestClass(object):
                def test_func1a(self, something):
                    assert something == "hello"
                def test_func2b(self, something):
                    assert something == "hello"
        """)
        reprec.assertoutcome(passed=4)

    def test_request_cachedsetup_extrakey(self, testdir):
        item1 = testdir.getitem("def test_func(): pass")
        req1 = fixtures.FixtureRequest(item1)
        values = ["hello", "world"]

        def setup():
            return values.pop()

        ret1 = req1.cached_setup(setup, extrakey=1)
        ret2 = req1.cached_setup(setup, extrakey=2)
        assert ret2 == "hello"
        assert ret1 == "world"
        ret1b = req1.cached_setup(setup, extrakey=1)
        ret2b = req1.cached_setup(setup, extrakey=2)
        assert ret1 == ret1b
        assert ret2 == ret2b

    def test_request_cachedsetup_cache_deletion(self, testdir):
        item1 = testdir.getitem("def test_func(): pass")
        req1 = fixtures.FixtureRequest(item1)
        values = []

        def setup():
            values.append("setup")

        def teardown(val):
            values.append("teardown")

        req1.cached_setup(setup, teardown, scope="function")
        assert values == ['setup']
        # artificial call of finalizer
        setupstate = req1._pyfuncitem.session._setupstate
        setupstate._callfinalizers(item1)
        assert values == ["setup", "teardown"]
        req1.cached_setup(setup, teardown, scope="function")
        assert values == ["setup", "teardown", "setup"]
        setupstate._callfinalizers(item1)
        assert values == ["setup", "teardown", "setup", "teardown"]

    def test_request_cached_setup_two_args(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def arg1(request):
                return request.cached_setup(lambda: 42)
            @pytest.fixture
            def arg2(request):
                return request.cached_setup(lambda: 17)
            def test_two_different_setups(arg1, arg2):
                assert arg1 != arg2
        """)
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines([
            "*1 passed*"
        ])

    def test_request_cached_setup_getfixturevalue(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def arg1(request):
                arg1 = request.getfixturevalue("arg2")
                return request.cached_setup(lambda: arg1 + 1)
            @pytest.fixture
            def arg2(request):
                return request.cached_setup(lambda: 10)
            def test_two_funcarg(arg1):
                assert arg1 == 11
        """)
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines([
            "*1 passed*"
        ])

    def test_request_cached_setup_functional(self, testdir):
        testdir.makepyfile(test_0="""
            import pytest
            values = []
            @pytest.fixture
            def something(request):
                val = request.cached_setup(fsetup, fteardown)
                return val
            def fsetup(mycache=[1]):
                values.append(mycache.pop())
                return values
            def fteardown(something):
                values.remove(something[0])
                values.append(2)
            def test_list_once(something):
                assert something == [1]
            def test_list_twice(something):
                assert something == [1]
        """)
        testdir.makepyfile(test_1="""
            import test_0 # should have run already
            def test_check_test0_has_teardown_correct():
                assert test_0.values == [2]
        """)
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines([
            "*3 passed*"
        ])

    def test_issue117_sessionscopeteardown(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def app(request):
                app = request.cached_setup(
                    scope='session',
                    setup=lambda: 0,
                    teardown=lambda x: 3/x)
                return app
            def test_func(app):
                pass
        """)
        result = testdir.runpytest()
        assert result.ret != 0
        result.stdout.fnmatch_lines([
            "*3/x*",
            "*ZeroDivisionError*",
        ])


class TestFixtureUsages(object):
    def test_noargfixturedec(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture
            def arg1():
                return 1

            def test_func(arg1):
                assert arg1 == 1
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_receives_funcargs(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture()
            def arg1():
                return 1

            @pytest.fixture()
            def arg2(arg1):
                return arg1 + 1

            def test_add(arg2):
                assert arg2 == 2
            def test_all(arg1, arg2):
                assert arg1 == 1
                assert arg2 == 2
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    def test_receives_funcargs_scope_mismatch(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(scope="function")
            def arg1():
                return 1

            @pytest.fixture(scope="module")
            def arg2(arg1):
                return arg1 + 1

            def test_add(arg2):
                assert arg2 == 2
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*ScopeMismatch*involved factories*",
            "* def arg2*",
            "* def arg1*",
            "*1 error*"
        ])

    def test_receives_funcargs_scope_mismatch_issue660(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(scope="function")
            def arg1():
                return 1

            @pytest.fixture(scope="module")
            def arg2(arg1):
                return arg1 + 1

            def test_add(arg1, arg2):
                assert arg2 == 2
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*ScopeMismatch*involved factories*",
            "* def arg2*",
            "*1 error*"
        ])

    def test_invalid_scope(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(scope="functions")
            def badscope():
                pass

            def test_nothing(badscope):
                pass
        """)
        result = testdir.runpytest_inprocess()
        result.stdout.fnmatch_lines(
            ("*ValueError: fixture badscope from test_invalid_scope.py has an unsupported"
             " scope value 'functions'")
        )

    def test_funcarg_parametrized_and_used_twice(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(params=[1,2])
            def arg1(request):
                values.append(1)
                return request.param

            @pytest.fixture()
            def arg2(arg1):
                return arg1 + 1

            def test_add(arg1, arg2):
                assert arg2 == arg1 + 1
                assert len(values) == arg1
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*2 passed*"
        ])

    def test_factory_uses_unknown_funcarg_as_dependency_error(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture()
            def fail(missing):
                return

            @pytest.fixture()
            def call_fail(fail):
                return

            def test_missing(call_fail):
                pass
            """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines("""
            *pytest.fixture()*
            *def call_fail(fail)*
            *pytest.fixture()*
            *def fail*
            *fixture*'missing'*not found*
        """)

    def test_factory_setup_as_classes_fails(self, testdir):
        testdir.makepyfile("""
            import pytest
            class arg1(object):
                def __init__(self, request):
                    self.x = 1
            arg1 = pytest.fixture()(arg1)

        """)
        reprec = testdir.inline_run()
        values = reprec.getfailedcollections()
        assert len(values) == 1

    def test_request_can_be_overridden(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture()
            def request(request):
                request.a = 1
                return request
            def test_request(request):
                assert request.a == 1
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_usefixtures_marker(self, testdir):
        testdir.makepyfile("""
            import pytest

            values = []

            @pytest.fixture(scope="class")
            def myfix(request):
                request.cls.hello = "world"
                values.append(1)

            class TestClass(object):
                def test_one(self):
                    assert self.hello == "world"
                    assert len(values) == 1
                def test_two(self):
                    assert self.hello == "world"
                    assert len(values) == 1
            pytest.mark.usefixtures("myfix")(TestClass)
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    def test_usefixtures_ini(self, testdir):
        testdir.makeini("""
            [pytest]
            usefixtures = myfix
        """)
        testdir.makeconftest("""
            import pytest

            @pytest.fixture(scope="class")
            def myfix(request):
                request.cls.hello = "world"

        """)
        testdir.makepyfile("""
            class TestClass(object):
                def test_one(self):
                    assert self.hello == "world"
                def test_two(self):
                    assert self.hello == "world"
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    def test_usefixtures_seen_in_showmarkers(self, testdir):
        result = testdir.runpytest("--markers")
        result.stdout.fnmatch_lines("""
            *usefixtures(fixturename1*mark tests*fixtures*
        """)

    def test_request_instance_issue203(self, testdir):
        testdir.makepyfile("""
            import pytest

            class TestClass(object):
                @pytest.fixture
                def setup1(self, request):
                    assert self == request.instance
                    self.arg1 = 1
                def test_hello(self, setup1):
                    assert self.arg1 == 1
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_fixture_parametrized_with_iterator(self, testdir):
        testdir.makepyfile("""
            import pytest

            values = []
            def f():
                yield 1
                yield 2
            dec = pytest.fixture(scope="module", params=f())

            @dec
            def arg(request):
                return request.param
            @dec
            def arg2(request):
                return request.param

            def test_1(arg):
                values.append(arg)
            def test_2(arg2):
                values.append(arg2*10)
        """)
        reprec = testdir.inline_run("-v")
        reprec.assertoutcome(passed=4)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert values == [1, 2, 10, 20]


class TestFixtureManagerParseFactories(object):

    @pytest.fixture
    def testdir(self, request):
        testdir = request.getfixturevalue("testdir")
        testdir.makeconftest("""
            import pytest

            @pytest.fixture
            def hello(request):
                return "conftest"

            @pytest.fixture
            def fm(request):
                return request._fixturemanager

            @pytest.fixture
            def item(request):
                return request._pyfuncitem
        """)
        return testdir

    def test_parsefactories_evil_objects_issue214(self, testdir):
        testdir.makepyfile("""
            class A(object):
                def __call__(self):
                    pass
                def __getattr__(self, name):
                    raise RuntimeError()
            a = A()
            def test_hello():
                pass
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1, failed=0)

    def test_parsefactories_conftest(self, testdir):
        testdir.makepyfile("""
            def test_hello(item, fm):
                for name in ("fm", "hello", "item"):
                    faclist = fm.getfixturedefs(name, item.nodeid)
                    assert len(faclist) == 1
                    fac = faclist[0]
                    assert fac.func.__name__ == name
        """)
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(passed=1)

    def test_parsefactories_conftest_and_module_and_class(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def hello(request):
                return "module"
            class TestClass(object):
                @pytest.fixture
                def hello(self, request):
                    return "class"
                def test_hello(self, item, fm):
                    faclist = fm.getfixturedefs("hello", item.nodeid)
                    print (faclist)
                    assert len(faclist) == 3
                    assert faclist[0].func(item._request) == "conftest"
                    assert faclist[1].func(item._request) == "module"
                    assert faclist[2].func(item._request) == "class"
        """)
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(passed=1)

    def test_parsefactories_relative_node_ids(self, testdir):
        # example mostly taken from:
        # https://mail.python.org/pipermail/pytest-dev/2014-September/002617.html
        runner = testdir.mkdir("runner")
        package = testdir.mkdir("package")
        package.join("conftest.py").write(dedent("""\
            import pytest
            @pytest.fixture
            def one():
                return 1
        """))
        package.join("test_x.py").write(dedent("""\
            def test_x(one):
                assert one == 1
        """))
        sub = package.mkdir("sub")
        sub.join("__init__.py").ensure()
        sub.join("conftest.py").write(dedent("""\
            import pytest
            @pytest.fixture
            def one():
                return 2
        """))
        sub.join("test_y.py").write(dedent("""\
            def test_x(one):
                assert one == 2
        """))
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)
        with runner.as_cwd():
            reprec = testdir.inline_run("..")
            reprec.assertoutcome(passed=2)


class TestAutouseDiscovery(object):

    @pytest.fixture
    def testdir(self, testdir):
        testdir.makeconftest("""
            import pytest
            @pytest.fixture(autouse=True)
            def perfunction(request, tmpdir):
                pass

            @pytest.fixture()
            def arg1(tmpdir):
                pass
            @pytest.fixture(autouse=True)
            def perfunction2(arg1):
                pass

            @pytest.fixture
            def fm(request):
                return request._fixturemanager

            @pytest.fixture
            def item(request):
                return request._pyfuncitem
        """)
        return testdir

    def test_parsefactories_conftest(self, testdir):
        testdir.makepyfile("""
            from _pytest.pytester import get_public_names
            def test_check_setup(item, fm):
                autousenames = fm._getautousenames(item.nodeid)
                assert len(get_public_names(autousenames)) == 2
                assert "perfunction2" in autousenames
                assert "perfunction" in autousenames
        """)
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(passed=1)

    def test_two_classes_separated_autouse(self, testdir):
        testdir.makepyfile("""
            import pytest
            class TestA(object):
                values = []
                @pytest.fixture(autouse=True)
                def setup1(self):
                    self.values.append(1)
                def test_setup1(self):
                    assert self.values == [1]
            class TestB(object):
                values = []
                @pytest.fixture(autouse=True)
                def setup2(self):
                    self.values.append(1)
                def test_setup2(self):
                    assert self.values == [1]
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    def test_setup_at_classlevel(self, testdir):
        testdir.makepyfile("""
            import pytest
            class TestClass(object):
                @pytest.fixture(autouse=True)
                def permethod(self, request):
                    request.instance.funcname = request.function.__name__
                def test_method1(self):
                    assert self.funcname == "test_method1"
                def test_method2(self):
                    assert self.funcname == "test_method2"
        """)
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(passed=2)

    @pytest.mark.xfail(reason="'enabled' feature not implemented")
    def test_setup_enabled_functionnode(self, testdir):
        testdir.makepyfile("""
            import pytest

            def enabled(parentnode, markers):
                return "needsdb" in markers

            @pytest.fixture(params=[1,2])
            def db(request):
                return request.param

            @pytest.fixture(enabled=enabled, autouse=True)
            def createdb(db):
                pass

            def test_func1(request):
                assert "db" not in request.fixturenames

            @pytest.mark.needsdb
            def test_func2(request):
                assert "db" in request.fixturenames
        """)
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(passed=2)

    def test_callables_nocode(self, testdir):
        """
        a imported mock.call would break setup/factory discovery
        due to it being callable and __code__ not being a code object
        """
        testdir.makepyfile("""
           class _call(tuple):
               def __call__(self, *k, **kw):
                   pass
               def __getattr__(self, k):
                   return self

           call = _call()
        """)
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(failed=0, passed=0)

    def test_autouse_in_conftests(self, testdir):
        a = testdir.mkdir("a")
        b = testdir.mkdir("a1")
        conftest = testdir.makeconftest("""
            import pytest
            @pytest.fixture(autouse=True)
            def hello():
                xxx
        """)
        conftest.move(a.join(conftest.basename))
        a.join("test_something.py").write("def test_func(): pass")
        b.join("test_otherthing.py").write("def test_func(): pass")
        result = testdir.runpytest()
        result.stdout.fnmatch_lines("""
            *1 passed*1 error*
        """)

    def test_autouse_in_module_and_two_classes(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(autouse=True)
            def append1():
                values.append("module")
            def test_x():
                assert values == ["module"]

            class TestA(object):
                @pytest.fixture(autouse=True)
                def append2(self):
                    values.append("A")
                def test_hello(self):
                    assert values == ["module", "module", "A"], values
            class TestA2(object):
                def test_world(self):
                    assert values == ["module", "module", "A", "module"], values
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=3)


class TestAutouseManagement(object):
    def test_autouse_conftest_mid_directory(self, testdir):
        pkgdir = testdir.mkpydir("xyz123")
        pkgdir.join("conftest.py").write(_pytest._code.Source("""
            import pytest
            @pytest.fixture(autouse=True)
            def app():
                import sys
                sys._myapp = "hello"
        """))
        t = pkgdir.ensure("tests", "test_app.py")
        t.write(_pytest._code.Source("""
            import sys
            def test_app():
                assert sys._myapp == "hello"
        """))
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(passed=1)

    def test_autouse_honored_for_yield(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(autouse=True)
            def tst():
                global x
                x = 3
            def test_gen():
                def f(hello):
                    assert x == abs(hello)
                yield f, 3
                yield f, -3
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    def test_funcarg_and_setup(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(scope="module")
            def arg():
                values.append(1)
                return 0
            @pytest.fixture(scope="module", autouse=True)
            def something(arg):
                values.append(2)

            def test_hello(arg):
                assert len(values) == 2
                assert values == [1,2]
                assert arg == 0

            def test_hello2(arg):
                assert len(values) == 2
                assert values == [1,2]
                assert arg == 0
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    def test_uses_parametrized_resource(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(params=[1,2])
            def arg(request):
                return request.param

            @pytest.fixture(autouse=True)
            def something(arg):
                values.append(arg)

            def test_hello():
                if len(values) == 1:
                    assert values == [1]
                elif len(values) == 2:
                    assert values == [1, 2]
                else:
                    0/0

        """)
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(passed=2)

    def test_session_parametrized_function(self, testdir):
        testdir.makepyfile("""
            import pytest

            values = []

            @pytest.fixture(scope="session", params=[1,2])
            def arg(request):
               return request.param

            @pytest.fixture(scope="function", autouse=True)
            def append(request, arg):
                if request.function.__name__ == "test_some":
                    values.append(arg)

            def test_some():
                pass

            def test_result(arg):
                assert len(values) == arg
                assert values[:arg] == [1,2][:arg]
        """)
        reprec = testdir.inline_run("-v", "-s")
        reprec.assertoutcome(passed=4)

    def test_class_function_parametrization_finalization(self, testdir):
        p = testdir.makeconftest("""
            import pytest
            import pprint

            values = []

            @pytest.fixture(scope="function", params=[1,2])
            def farg(request):
                return request.param

            @pytest.fixture(scope="class", params=list("ab"))
            def carg(request):
                return request.param

            @pytest.fixture(scope="function", autouse=True)
            def append(request, farg, carg):
                def fin():
                    values.append("fin_%s%s" % (carg, farg))
                request.addfinalizer(fin)
        """)
        testdir.makepyfile("""
            import pytest

            class TestClass(object):
                def test_1(self):
                    pass
            class TestClass2(object):
                def test_2(self):
                    pass
        """)
        confcut = "--confcutdir={0}".format(testdir.tmpdir)
        reprec = testdir.inline_run("-v", "-s", confcut)
        reprec.assertoutcome(passed=8)
        config = reprec.getcalls("pytest_unconfigure")[0].config
        values = config.pluginmanager._getconftestmodules(p)[0].values
        assert values == ["fin_a1", "fin_a2", "fin_b1", "fin_b2"] * 2

    def test_scope_ordering(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(scope="function", autouse=True)
            def fappend2():
                values.append(2)
            @pytest.fixture(scope="class", autouse=True)
            def classappend3():
                values.append(3)
            @pytest.fixture(scope="module", autouse=True)
            def mappend():
                values.append(1)

            class TestHallo(object):
                def test_method(self):
                    assert values == [1,3,2]
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_parametrization_setup_teardown_ordering(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            def pytest_generate_tests(metafunc):
                if metafunc.cls is not None:
                    metafunc.parametrize("item", [1,2], scope="class")
            class TestClass(object):
                @pytest.fixture(scope="class", autouse=True)
                def addteardown(self, item, request):
                    values.append("setup-%d" % item)
                    request.addfinalizer(lambda: values.append("teardown-%d" % item))
                def test_step1(self, item):
                    values.append("step1-%d" % item)
                def test_step2(self, item):
                    values.append("step2-%d" % item)

            def test_finish():
                print (values)
                assert values == ["setup-1", "step1-1", "step2-1", "teardown-1",
                             "setup-2", "step1-2", "step2-2", "teardown-2",]
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=5)

    def test_ordering_autouse_before_explicit(self, testdir):
        testdir.makepyfile("""
            import pytest

            values = []
            @pytest.fixture(autouse=True)
            def fix1():
                values.append(1)
            @pytest.fixture()
            def arg1():
                values.append(2)
            def test_hello(arg1):
                assert values == [1,2]
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    @pytest.mark.issue226
    @pytest.mark.parametrize("param1", ["", "params=[1]"], ids=["p00", "p01"])
    @pytest.mark.parametrize("param2", ["", "params=[1]"], ids=["p10", "p11"])
    def test_ordering_dependencies_torndown_first(self, testdir, param1, param2):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(%(param1)s)
            def arg1(request):
                request.addfinalizer(lambda: values.append("fin1"))
                values.append("new1")
            @pytest.fixture(%(param2)s)
            def arg2(request, arg1):
                request.addfinalizer(lambda: values.append("fin2"))
                values.append("new2")

            def test_arg(arg2):
                pass
            def test_check():
                assert values == ["new1", "new2", "fin2", "fin1"]
        """ % locals())
        reprec = testdir.inline_run("-s")
        reprec.assertoutcome(passed=2)


class TestFixtureMarker(object):
    def test_parametrize(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(params=["a", "b", "c"])
            def arg(request):
                return request.param
            values = []
            def test_param(arg):
                values.append(arg)
            def test_result():
                assert values == list("abc")
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=4)

    def test_multiple_parametrization_issue_736(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture(params=[1,2,3])
            def foo(request):
                return request.param

            @pytest.mark.parametrize('foobar', [4,5,6])
            def test_issue(foo, foobar):
                assert foo in [1,2,3]
                assert foobar in [4,5,6]
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=9)

    @pytest.mark.parametrize('param_args', ["'fixt, val'", "'fixt,val'", "['fixt', 'val']", "('fixt', 'val')"])
    def test_override_parametrized_fixture_issue_979(self, testdir, param_args):
        """Make sure a parametrized argument can override a parametrized fixture.

        This was a regression introduced in the fix for #736.
        """
        testdir.makepyfile("""
            import pytest

            @pytest.fixture(params=[1, 2])
            def fixt(request):
                return request.param

            @pytest.mark.parametrize(%s, [(3, 'x'), (4, 'x')])
            def test_foo(fixt, val):
                pass
        """ % param_args)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    def test_scope_session(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(scope="module")
            def arg():
                values.append(1)
                return 1

            def test_1(arg):
                assert arg == 1
            def test_2(arg):
                assert arg == 1
                assert len(values) == 1
            class TestClass(object):
                def test3(self, arg):
                    assert arg == 1
                    assert len(values) == 1
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=3)

    def test_scope_session_exc(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(scope="session")
            def fix():
                values.append(1)
                pytest.skip('skipping')

            def test_1(fix):
                pass
            def test_2(fix):
                pass
            def test_last():
                assert values == [1]
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(skipped=2, passed=1)

    def test_scope_session_exc_two_fix(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            m = []
            @pytest.fixture(scope="session")
            def a():
                values.append(1)
                pytest.skip('skipping')
            @pytest.fixture(scope="session")
            def b(a):
                m.append(1)

            def test_1(b):
                pass
            def test_2(b):
                pass
            def test_last():
                assert values == [1]
                assert m == []
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(skipped=2, passed=1)

    def test_scope_exc(self, testdir):
        testdir.makepyfile(
            test_foo="""
                def test_foo(fix):
                    pass
            """,
            test_bar="""
                def test_bar(fix):
                    pass
            """,
            conftest="""
                import pytest
                reqs = []
                @pytest.fixture(scope="session")
                def fix(request):
                    reqs.append(1)
                    pytest.skip()
                @pytest.fixture
                def req_list():
                    return reqs
            """,
            test_real="""
                def test_last(req_list):
                    assert req_list == [1]
            """
        )
        reprec = testdir.inline_run()
        reprec.assertoutcome(skipped=2, passed=1)

    def test_scope_module_uses_session(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(scope="module")
            def arg():
                values.append(1)
                return 1

            def test_1(arg):
                assert arg == 1
            def test_2(arg):
                assert arg == 1
                assert len(values) == 1
            class TestClass(object):
                def test3(self, arg):
                    assert arg == 1
                    assert len(values) == 1
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=3)

    def test_scope_module_and_finalizer(self, testdir):
        testdir.makeconftest("""
            import pytest
            finalized_list = []
            created_list = []
            @pytest.fixture(scope="module")
            def arg(request):
                created_list.append(1)
                assert request.scope == "module"
                request.addfinalizer(lambda: finalized_list.append(1))
            @pytest.fixture
            def created(request):
                return len(created_list)
            @pytest.fixture
            def finalized(request):
                return len(finalized_list)
        """)
        testdir.makepyfile(
            test_mod1="""
                def test_1(arg, created, finalized):
                    assert created == 1
                    assert finalized == 0
                def test_2(arg, created, finalized):
                    assert created == 1
                    assert finalized == 0""",
            test_mod2="""
                def test_3(arg, created, finalized):
                    assert created == 2
                    assert finalized == 1""",
            test_mode3="""
                def test_4(arg, created, finalized):
                    assert created == 3
                    assert finalized == 2
            """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=4)

    @pytest.mark.parametrize("method", [
        'request.getfixturevalue("arg")',
        'request.cached_setup(lambda: None, scope="function")',
    ], ids=["getfixturevalue", "cached_setup"])
    def test_scope_mismatch_various(self, testdir, method):
        testdir.makeconftest("""
            import pytest
            finalized = []
            created = []
            @pytest.fixture(scope="function")
            def arg(request):
                pass
        """)
        testdir.makepyfile(
            test_mod1="""
                import pytest
                @pytest.fixture(scope="session")
                def arg(request):
                    %s
                def test_1(arg):
                    pass
            """ % method)
        result = testdir.runpytest()
        assert result.ret != 0
        result.stdout.fnmatch_lines([
            "*ScopeMismatch*You tried*function*session*request*",
        ])

    def test_register_only_with_mark(self, testdir):
        testdir.makeconftest("""
            import pytest
            @pytest.fixture()
            def arg():
                return 1
        """)
        testdir.makepyfile(
            test_mod1="""
                import pytest
                @pytest.fixture()
                def arg(arg):
                    return arg + 1
                def test_1(arg):
                    assert arg == 2
            """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_parametrize_and_scope(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(scope="module", params=["a", "b", "c"])
            def arg(request):
                return request.param
            values = []
            def test_param(arg):
                values.append(arg)
        """)
        reprec = testdir.inline_run("-v")
        reprec.assertoutcome(passed=3)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert len(values) == 3
        assert "a" in values
        assert "b" in values
        assert "c" in values

    def test_scope_mismatch(self, testdir):
        testdir.makeconftest("""
            import pytest
            @pytest.fixture(scope="function")
            def arg(request):
                pass
        """)
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(scope="session")
            def arg(arg):
                pass
            def test_mismatch(arg):
                pass
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*ScopeMismatch*",
            "*1 error*",
        ])

    def test_parametrize_separated_order(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture(scope="module", params=[1, 2])
            def arg(request):
                return request.param

            values = []
            def test_1(arg):
                values.append(arg)
            def test_2(arg):
                values.append(arg)
        """)
        reprec = testdir.inline_run("-v")
        reprec.assertoutcome(passed=4)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert values == [1, 1, 2, 2]

    def test_module_parametrized_ordering(self, testdir):
        testdir.makeini("""
            [pytest]
            console_output_style=classic
        """)
        testdir.makeconftest("""
            import pytest

            @pytest.fixture(scope="session", params="s1 s2".split())
            def sarg():
                pass
            @pytest.fixture(scope="module", params="m1 m2".split())
            def marg():
                pass
        """)
        testdir.makepyfile(test_mod1="""
            def test_func(sarg):
                pass
            def test_func1(marg):
                pass
        """, test_mod2="""
            def test_func2(sarg):
                pass
            def test_func3(sarg, marg):
                pass
            def test_func3b(sarg, marg):
                pass
            def test_func4(marg):
                pass
        """)
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines("""
            test_mod1.py::test_func[s1] PASSED
            test_mod2.py::test_func2[s1] PASSED
            test_mod2.py::test_func3[s1-m1] PASSED
            test_mod2.py::test_func3b[s1-m1] PASSED
            test_mod2.py::test_func3[s1-m2] PASSED
            test_mod2.py::test_func3b[s1-m2] PASSED
            test_mod1.py::test_func[s2] PASSED
            test_mod2.py::test_func2[s2] PASSED
            test_mod2.py::test_func3[s2-m1] PASSED
            test_mod2.py::test_func3b[s2-m1] PASSED
            test_mod2.py::test_func4[m1] PASSED
            test_mod2.py::test_func3[s2-m2] PASSED
            test_mod2.py::test_func3b[s2-m2] PASSED
            test_mod2.py::test_func4[m2] PASSED
            test_mod1.py::test_func1[m1] PASSED
            test_mod1.py::test_func1[m2] PASSED
        """)

    def test_class_ordering(self, testdir):
        testdir.makeini("""
            [pytest]
            console_output_style=classic
        """)
        testdir.makeconftest("""
            import pytest

            values = []

            @pytest.fixture(scope="function", params=[1,2])
            def farg(request):
                return request.param

            @pytest.fixture(scope="class", params=list("ab"))
            def carg(request):
                return request.param

            @pytest.fixture(scope="function", autouse=True)
            def append(request, farg, carg):
                def fin():
                    values.append("fin_%s%s" % (carg, farg))
                request.addfinalizer(fin)
        """)
        testdir.makepyfile("""
            import pytest

            class TestClass2(object):
                def test_1(self):
                    pass
                def test_2(self):
                    pass
            class TestClass(object):
                def test_3(self):
                    pass
        """)
        result = testdir.runpytest("-vs")
        result.stdout.fnmatch_lines("""
            test_class_ordering.py::TestClass2::test_1[1-a] PASSED
            test_class_ordering.py::TestClass2::test_1[2-a] PASSED
            test_class_ordering.py::TestClass2::test_2[1-a] PASSED
            test_class_ordering.py::TestClass2::test_2[2-a] PASSED
            test_class_ordering.py::TestClass2::test_1[1-b] PASSED
            test_class_ordering.py::TestClass2::test_1[2-b] PASSED
            test_class_ordering.py::TestClass2::test_2[1-b] PASSED
            test_class_ordering.py::TestClass2::test_2[2-b] PASSED
            test_class_ordering.py::TestClass::test_3[1-a] PASSED
            test_class_ordering.py::TestClass::test_3[2-a] PASSED
            test_class_ordering.py::TestClass::test_3[1-b] PASSED
            test_class_ordering.py::TestClass::test_3[2-b] PASSED
        """)

    def test_parametrize_separated_order_higher_scope_first(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture(scope="function", params=[1, 2])
            def arg(request):
                param = request.param
                request.addfinalizer(lambda: values.append("fin:%s" % param))
                values.append("create:%s" % param)
                return request.param

            @pytest.fixture(scope="module", params=["mod1", "mod2"])
            def modarg(request):
                param = request.param
                request.addfinalizer(lambda: values.append("fin:%s" % param))
                values.append("create:%s" % param)
                return request.param

            values = []
            def test_1(arg):
                values.append("test1")
            def test_2(modarg):
                values.append("test2")
            def test_3(arg, modarg):
                values.append("test3")
            def test_4(modarg, arg):
                values.append("test4")
        """)
        reprec = testdir.inline_run("-v")
        reprec.assertoutcome(passed=12)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        expected = [
            'create:1', 'test1', 'fin:1', 'create:2', 'test1',
            'fin:2', 'create:mod1', 'test2', 'create:1', 'test3',
            'fin:1', 'create:2', 'test3', 'fin:2', 'create:1',
            'test4', 'fin:1', 'create:2', 'test4', 'fin:2',
            'fin:mod1', 'create:mod2', 'test2', 'create:1', 'test3',
            'fin:1', 'create:2', 'test3', 'fin:2', 'create:1',
            'test4', 'fin:1', 'create:2', 'test4', 'fin:2',
            'fin:mod2']
        import pprint
        pprint.pprint(list(zip(values, expected)))
        assert values == expected

    def test_parametrized_fixture_teardown_order(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(params=[1,2], scope="class")
            def param1(request):
                return request.param

            values = []

            class TestClass(object):
                @classmethod
                @pytest.fixture(scope="class", autouse=True)
                def setup1(self, request, param1):
                    values.append(1)
                    request.addfinalizer(self.teardown1)
                @classmethod
                def teardown1(self):
                    assert values.pop() == 1
                @pytest.fixture(scope="class", autouse=True)
                def setup2(self, request, param1):
                    values.append(2)
                    request.addfinalizer(self.teardown2)
                @classmethod
                def teardown2(self):
                    assert values.pop() == 2
                def test(self):
                    pass

            def test_finish():
                assert not values
        """)
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines("""
            *3 passed*
        """)
        assert "error" not in result.stdout.str()

    def test_fixture_finalizer(self, testdir):
        testdir.makeconftest("""
            import pytest
            import sys

            @pytest.fixture
            def browser(request):

                def finalize():
                    sys.stdout.write('Finalized')
                request.addfinalizer(finalize)
                return {}
        """)
        b = testdir.mkdir("subdir")
        b.join("test_overridden_fixture_finalizer.py").write(dedent("""
            import pytest
            @pytest.fixture
            def browser(browser):
                browser['visited'] = True
                return browser

            def test_browser(browser):
                assert browser['visited'] is True
        """))
        reprec = testdir.runpytest("-s")
        for test in ['test_browser']:
            reprec.stdout.fnmatch_lines('*Finalized*')

    def test_class_scope_with_normal_tests(self, testdir):
        testpath = testdir.makepyfile("""
            import pytest

            class Box(object):
                value = 0

            @pytest.fixture(scope='class')
            def a(request):
                Box.value += 1
                return Box.value

            def test_a(a):
                assert a == 1

            class Test1(object):
                def test_b(self, a):
                    assert a == 2

            class Test2(object):
                def test_c(self, a):
                    assert a == 3""")
        reprec = testdir.inline_run(testpath)
        for test in ['test_a', 'test_b', 'test_c']:
            assert reprec.matchreport(test).passed

    def test_request_is_clean(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(params=[1, 2])
            def fix(request):
                request.addfinalizer(lambda: values.append(request.param))
            def test_fix(fix):
                pass
        """)
        reprec = testdir.inline_run("-s")
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert values == [1, 2]

    def test_parametrize_separated_lifecycle(self, testdir):
        testdir.makepyfile("""
            import pytest

            values = []
            @pytest.fixture(scope="module", params=[1, 2])
            def arg(request):
                x = request.param
                request.addfinalizer(lambda: values.append("fin%s" % x))
                return request.param
            def test_1(arg):
                values.append(arg)
            def test_2(arg):
                values.append(arg)
        """)
        reprec = testdir.inline_run("-vs")
        reprec.assertoutcome(passed=4)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        import pprint
        pprint.pprint(values)
        # assert len(values) == 6
        assert values[0] == values[1] == 1
        assert values[2] == "fin1"
        assert values[3] == values[4] == 2
        assert values[5] == "fin2"

    def test_parametrize_function_scoped_finalizers_called(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture(scope="function", params=[1, 2])
            def arg(request):
                x = request.param
                request.addfinalizer(lambda: values.append("fin%s" % x))
                return request.param

            values = []
            def test_1(arg):
                values.append(arg)
            def test_2(arg):
                values.append(arg)
            def test_3():
                assert len(values) == 8
                assert values == [1, "fin1", 2, "fin2", 1, "fin1", 2, "fin2"]
        """)
        reprec = testdir.inline_run("-v")
        reprec.assertoutcome(passed=5)

    @pytest.mark.issue246
    @pytest.mark.parametrize("scope", ["session", "function", "module"])
    def test_finalizer_order_on_parametrization(self, scope, testdir):
        testdir.makepyfile("""
            import pytest
            values = []

            @pytest.fixture(scope=%(scope)r, params=["1"])
            def fix1(request):
                return request.param

            @pytest.fixture(scope=%(scope)r)
            def fix2(request, base):
                def cleanup_fix2():
                    assert not values, "base should not have been finalized"
                request.addfinalizer(cleanup_fix2)

            @pytest.fixture(scope=%(scope)r)
            def base(request, fix1):
                def cleanup_base():
                    values.append("fin_base")
                    print ("finalizing base")
                request.addfinalizer(cleanup_base)

            def test_begin():
                pass
            def test_baz(base, fix2):
                pass
            def test_other():
                pass
        """ % {"scope": scope})
        reprec = testdir.inline_run("-lvs")
        reprec.assertoutcome(passed=3)

    @pytest.mark.issue396
    def test_class_scope_parametrization_ordering(self, testdir):
        testdir.makepyfile("""
            import pytest
            values = []
            @pytest.fixture(params=["John", "Doe"], scope="class")
            def human(request):
                request.addfinalizer(lambda: values.append("fin %s" % request.param))
                return request.param

            class TestGreetings(object):
                def test_hello(self, human):
                    values.append("test_hello")

            class TestMetrics(object):
                def test_name(self, human):
                    values.append("test_name")

                def test_population(self, human):
                    values.append("test_population")
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=6)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert values == ["test_hello", "fin John", "test_hello", "fin Doe",
                          "test_name", "test_population", "fin John",
                          "test_name", "test_population", "fin Doe"]

    def test_parametrize_setup_function(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture(scope="module", params=[1, 2])
            def arg(request):
                return request.param

            @pytest.fixture(scope="module", autouse=True)
            def mysetup(request, arg):
                request.addfinalizer(lambda: values.append("fin%s" % arg))
                values.append("setup%s" % arg)

            values = []
            def test_1(arg):
                values.append(arg)
            def test_2(arg):
                values.append(arg)
            def test_3():
                import pprint
                pprint.pprint(values)
                if arg == 1:
                    assert values == ["setup1", 1, 1, ]
                elif arg == 2:
                    assert values == ["setup1", 1, 1, "fin1",
                                 "setup2", 2, 2, ]

        """)
        reprec = testdir.inline_run("-v")
        reprec.assertoutcome(passed=6)

    def test_fixture_marked_function_not_collected_as_test(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture
            def test_app():
                return 1

            def test_something(test_app):
                assert test_app == 1
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_params_and_ids(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture(params=[object(), object()],
                            ids=['alpha', 'beta'])
            def fix(request):
                return request.param

            def test_foo(fix):
                assert 1
        """)
        res = testdir.runpytest('-v')
        res.stdout.fnmatch_lines([
            '*test_foo*alpha*',
            '*test_foo*beta*'])

    def test_params_and_ids_yieldfixture(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.yield_fixture(params=[object(), object()],
                                  ids=['alpha', 'beta'])
            def fix(request):
                 yield request.param

            def test_foo(fix):
                assert 1
        """)
        res = testdir.runpytest('-v')
        res.stdout.fnmatch_lines([
            '*test_foo*alpha*',
            '*test_foo*beta*'])

    @pytest.mark.issue920
    def test_deterministic_fixture_collection(self, testdir, monkeypatch):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture(scope="module",
                            params=["A",
                                    "B",
                                    "C"])
            def A(request):
                return request.param

            @pytest.fixture(scope="module",
                            params=["DDDDDDDDD", "EEEEEEEEEEEE", "FFFFFFFFFFF", "banansda"])
            def B(request, A):
                return request.param

            def test_foo(B):
                # Something funky is going on here.
                # Despite specified seeds, on what is collected,
                # sometimes we get unexpected passes. hashing B seems
                # to help?
                assert hash(B) or True
            """)
        monkeypatch.setenv("PYTHONHASHSEED", "1")
        out1 = testdir.runpytest_subprocess("-v")
        monkeypatch.setenv("PYTHONHASHSEED", "2")
        out2 = testdir.runpytest_subprocess("-v")
        out1 = [line for line in out1.outlines if line.startswith("test_deterministic_fixture_collection.py::test_foo")]
        out2 = [line for line in out2.outlines if line.startswith("test_deterministic_fixture_collection.py::test_foo")]
        assert len(out1) == 12
        assert out1 == out2


class TestRequestScopeAccess(object):
    pytestmark = pytest.mark.parametrize(("scope", "ok", "error"), [
        ["session", "", "fspath class function module"],
        ["module", "module fspath", "cls function"],
        ["class", "module fspath cls", "function"],
        ["function", "module fspath cls function", ""]
    ])

    def test_setup(self, testdir, scope, ok, error):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(scope=%r, autouse=True)
            def myscoped(request):
                for x in %r:
                    assert hasattr(request, x)
                for x in %r:
                    pytest.raises(AttributeError, lambda:
                        getattr(request, x))
                assert request.session
                assert request.config
            def test_func():
                pass
        """ % (scope, ok.split(), error.split()))
        reprec = testdir.inline_run("-l")
        reprec.assertoutcome(passed=1)

    def test_funcarg(self, testdir, scope, ok, error):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(scope=%r)
            def arg(request):
                for x in %r:
                    assert hasattr(request, x)
                for x in %r:
                    pytest.raises(AttributeError, lambda:
                        getattr(request, x))
                assert request.session
                assert request.config
            def test_func(arg):
                pass
        """ % (scope, ok.split(), error.split()))
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)


class TestErrors(object):
    def test_subfactory_missing_funcarg(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture()
            def gen(qwe123):
                return 1
            def test_something(gen):
                pass
        """)
        result = testdir.runpytest()
        assert result.ret != 0
        result.stdout.fnmatch_lines([
            "*def gen(qwe123):*",
            "*fixture*qwe123*not found*",
            "*1 error*",
        ])

    def test_issue498_fixture_finalizer_failing(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture
            def fix1(request):
                def f():
                    raise KeyError
                request.addfinalizer(f)
                return object()

            values = []
            def test_1(fix1):
                values.append(fix1)
            def test_2(fix1):
                values.append(fix1)
            def test_3():
                assert values[0] != values[1]
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines("""
            *ERROR*teardown*test_1*
            *KeyError*
            *ERROR*teardown*test_2*
            *KeyError*
            *3 pass*2 error*
        """)

    def test_setupfunc_missing_funcarg(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.fixture(autouse=True)
            def gen(qwe123):
                return 1
            def test_something():
                pass
        """)
        result = testdir.runpytest()
        assert result.ret != 0
        result.stdout.fnmatch_lines([
            "*def gen(qwe123):*",
            "*fixture*qwe123*not found*",
            "*1 error*",
        ])


class TestShowFixtures(object):
    def test_funcarg_compat(self, testdir):
        config = testdir.parseconfigure("--funcargs")
        assert config.option.showfixtures

    def test_show_fixtures(self, testdir):
        result = testdir.runpytest("--fixtures")
        result.stdout.fnmatch_lines([
            "*tmpdir*",
            "*temporary directory*",
        ])

    def test_show_fixtures_verbose(self, testdir):
        result = testdir.runpytest("--fixtures", "-v")
        result.stdout.fnmatch_lines([
            "*tmpdir*--*tmpdir.py*",
            "*temporary directory*",
        ])

    def test_show_fixtures_testmodule(self, testdir):
        p = testdir.makepyfile('''
            import pytest
            @pytest.fixture
            def _arg0():
                """ hidden """
            @pytest.fixture
            def arg1():
                """  hello world """
        ''')
        result = testdir.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines("""
            *tmpdir
            *fixtures defined from*
            *arg1*
            *hello world*
        """)
        assert "arg0" not in result.stdout.str()

    @pytest.mark.parametrize("testmod", [True, False])
    def test_show_fixtures_conftest(self, testdir, testmod):
        testdir.makeconftest('''
            import pytest
            @pytest.fixture
            def arg1():
                """  hello world """
        ''')
        if testmod:
            testdir.makepyfile("""
                def test_hello():
                    pass
            """)
        result = testdir.runpytest("--fixtures")
        result.stdout.fnmatch_lines("""
            *tmpdir*
            *fixtures defined from*conftest*
            *arg1*
            *hello world*
        """)

    def test_show_fixtures_trimmed_doc(self, testdir):
        p = testdir.makepyfile(dedent('''
            import pytest
            @pytest.fixture
            def arg1():
                """
                line1
                line2

                """
            @pytest.fixture
            def arg2():
                """
                line1
                line2

                """
        '''))
        result = testdir.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines(dedent("""
            * fixtures defined from test_show_fixtures_trimmed_doc *
            arg2
                line1
                line2
            arg1
                line1
                line2

        """))

    def test_show_fixtures_indented_doc(self, testdir):
        p = testdir.makepyfile(dedent('''
            import pytest
            @pytest.fixture
            def fixture1():
                """
                line1
                    indented line
                """
        '''))
        result = testdir.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines(dedent("""
            * fixtures defined from test_show_fixtures_indented_doc *
            fixture1
                line1
                    indented line
        """))

    def test_show_fixtures_indented_doc_first_line_unindented(self, testdir):
        p = testdir.makepyfile(dedent('''
            import pytest
            @pytest.fixture
            def fixture1():
                """line1
                line2
                    indented line
                """
        '''))
        result = testdir.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines(dedent("""
            * fixtures defined from test_show_fixtures_indented_doc_first_line_unindented *
            fixture1
                line1
                line2
                    indented line
        """))

    def test_show_fixtures_indented_in_class(self, testdir):
        p = testdir.makepyfile(dedent('''
            import pytest
            class TestClass:
                @pytest.fixture
                def fixture1(self):
                    """line1
                    line2
                        indented line
                    """
        '''))
        result = testdir.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines(dedent("""
            * fixtures defined from test_show_fixtures_indented_in_class *
            fixture1
                line1
                line2
                    indented line
        """))

    def test_show_fixtures_different_files(self, testdir):
        """
        #833: --fixtures only shows fixtures from first file
        """
        testdir.makepyfile(test_a='''
            import pytest

            @pytest.fixture
            def fix_a():
                """Fixture A"""
                pass

            def test_a(fix_a):
                pass
        ''')
        testdir.makepyfile(test_b='''
            import pytest

            @pytest.fixture
            def fix_b():
                """Fixture B"""
                pass

            def test_b(fix_b):
                pass
        ''')
        result = testdir.runpytest("--fixtures")
        result.stdout.fnmatch_lines("""
            * fixtures defined from test_a *
            fix_a
                Fixture A

            * fixtures defined from test_b *
            fix_b
                Fixture B
        """)

    def test_show_fixtures_with_same_name(self, testdir):
        testdir.makeconftest('''
            import pytest
            @pytest.fixture
            def arg1():
                """Hello World in conftest.py"""
                return "Hello World"
        ''')
        testdir.makepyfile('''
            def test_foo(arg1):
                assert arg1 == "Hello World"
        ''')
        testdir.makepyfile('''
            import pytest
            @pytest.fixture
            def arg1():
                """Hi from test module"""
                return "Hi"
            def test_bar(arg1):
                assert arg1 == "Hi"
        ''')
        result = testdir.runpytest("--fixtures")
        result.stdout.fnmatch_lines('''
            * fixtures defined from conftest *
            arg1
                Hello World in conftest.py

            * fixtures defined from test_show_fixtures_with_same_name *
            arg1
                Hi from test module
        ''')


@pytest.mark.parametrize('flavor', ['fixture', 'yield_fixture'])
class TestContextManagerFixtureFuncs(object):

    def test_simple(self, testdir, flavor):
        testdir.makepyfile("""
            import pytest
            @pytest.{flavor}
            def arg1():
                print ("setup")
                yield 1
                print ("teardown")
            def test_1(arg1):
                print ("test1 %s" % arg1)
            def test_2(arg1):
                print ("test2 %s" % arg1)
                assert 0
        """.format(flavor=flavor))
        result = testdir.runpytest("-s")
        result.stdout.fnmatch_lines("""
            *setup*
            *test1 1*
            *teardown*
            *setup*
            *test2 1*
            *teardown*
        """)

    def test_scoped(self, testdir, flavor):
        testdir.makepyfile("""
            import pytest
            @pytest.{flavor}(scope="module")
            def arg1():
                print ("setup")
                yield 1
                print ("teardown")
            def test_1(arg1):
                print ("test1 %s" % arg1)
            def test_2(arg1):
                print ("test2 %s" % arg1)
        """.format(flavor=flavor))
        result = testdir.runpytest("-s")
        result.stdout.fnmatch_lines("""
            *setup*
            *test1 1*
            *test2 1*
            *teardown*
        """)

    def test_setup_exception(self, testdir, flavor):
        testdir.makepyfile("""
            import pytest
            @pytest.{flavor}(scope="module")
            def arg1():
                pytest.fail("setup")
                yield 1
            def test_1(arg1):
                pass
        """.format(flavor=flavor))
        result = testdir.runpytest("-s")
        result.stdout.fnmatch_lines("""
            *pytest.fail*setup*
            *1 error*
        """)

    def test_teardown_exception(self, testdir, flavor):
        testdir.makepyfile("""
            import pytest
            @pytest.{flavor}(scope="module")
            def arg1():
                yield 1
                pytest.fail("teardown")
            def test_1(arg1):
                pass
        """.format(flavor=flavor))
        result = testdir.runpytest("-s")
        result.stdout.fnmatch_lines("""
            *pytest.fail*teardown*
            *1 passed*1 error*
        """)

    def test_yields_more_than_one(self, testdir, flavor):
        testdir.makepyfile("""
            import pytest
            @pytest.{flavor}(scope="module")
            def arg1():
                yield 1
                yield 2
            def test_1(arg1):
                pass
        """.format(flavor=flavor))
        result = testdir.runpytest("-s")
        result.stdout.fnmatch_lines("""
            *fixture function*
            *test_yields*:2*
        """)

    def test_custom_name(self, testdir, flavor):
        testdir.makepyfile("""
            import pytest
            @pytest.{flavor}(name='meow')
            def arg1():
                return 'mew'
            def test_1(meow):
                print(meow)
        """.format(flavor=flavor))
        result = testdir.runpytest("-s")
        result.stdout.fnmatch_lines("*mew*")


class TestParameterizedSubRequest(object):
    def test_call_from_fixture(self, testdir):
        testfile = testdir.makepyfile("""
            import pytest

            @pytest.fixture(params=[0, 1, 2])
            def fix_with_param(request):
                return request.param

            @pytest.fixture
            def get_named_fixture(request):
                return request.getfixturevalue('fix_with_param')

            def test_foo(request, get_named_fixture):
                pass
            """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines("""
            E*Failed: The requested fixture has no parameter defined for the current test.
            E*
            E*Requested fixture 'fix_with_param' defined in:
            E*{0}:4
            E*Requested here:
            E*{1}:9
            *1 error*
            """.format(testfile.basename, testfile.basename))

    def test_call_from_test(self, testdir):
        testfile = testdir.makepyfile("""
            import pytest

            @pytest.fixture(params=[0, 1, 2])
            def fix_with_param(request):
                return request.param

            def test_foo(request):
                request.getfixturevalue('fix_with_param')
            """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines("""
            E*Failed: The requested fixture has no parameter defined for the current test.
            E*
            E*Requested fixture 'fix_with_param' defined in:
            E*{0}:4
            E*Requested here:
            E*{1}:8
            *1 failed*
            """.format(testfile.basename, testfile.basename))

    def test_external_fixture(self, testdir):
        conffile = testdir.makeconftest("""
            import pytest

            @pytest.fixture(params=[0, 1, 2])
            def fix_with_param(request):
                return request.param
            """)

        testfile = testdir.makepyfile("""
            def test_foo(request):
                request.getfixturevalue('fix_with_param')
            """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines("""
            E*Failed: The requested fixture has no parameter defined for the current test.
            E*
            E*Requested fixture 'fix_with_param' defined in:
            E*{0}:4
            E*Requested here:
            E*{1}:2
            *1 failed*
            """.format(conffile.basename, testfile.basename))

    def test_non_relative_path(self, testdir):
        tests_dir = testdir.mkdir('tests')
        fixdir = testdir.mkdir('fixtures')
        fixfile = fixdir.join("fix.py")
        fixfile.write(_pytest._code.Source("""
            import pytest

            @pytest.fixture(params=[0, 1, 2])
            def fix_with_param(request):
                return request.param
            """))

        testfile = tests_dir.join("test_foos.py")
        testfile.write(_pytest._code.Source("""
            from fix import fix_with_param

            def test_foo(request):
                request.getfixturevalue('fix_with_param')
            """))

        tests_dir.chdir()
        testdir.syspathinsert(fixdir)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines("""
            E*Failed: The requested fixture has no parameter defined for the current test.
            E*
            E*Requested fixture 'fix_with_param' defined in:
            E*{0}:5
            E*Requested here:
            E*{1}:5
            *1 failed*
            """.format(fixfile.strpath, testfile.basename))


def test_pytest_fixture_setup_and_post_finalizer_hook(testdir):
    testdir.makeconftest("""
        from __future__ import print_function
        def pytest_fixture_setup(fixturedef, request):
            print('ROOT setup hook called for {0} from {1}'.format(fixturedef.argname, request.node.name))
        def pytest_fixture_post_finalizer(fixturedef, request):
            print('ROOT finalizer hook called for {0} from {1}'.format(fixturedef.argname, request.node.name))
    """)
    testdir.makepyfile(**{
        'tests/conftest.py': """
            from __future__ import print_function
            def pytest_fixture_setup(fixturedef, request):
                print('TESTS setup hook called for {0} from {1}'.format(fixturedef.argname, request.node.name))
            def pytest_fixture_post_finalizer(fixturedef, request):
                print('TESTS finalizer hook called for {0} from {1}'.format(fixturedef.argname, request.node.name))
        """,
        'tests/test_hooks.py': """
            from __future__ import print_function
            import pytest

            @pytest.fixture()
            def my_fixture():
                return 'some'

            def test_func(my_fixture):
                print('TEST test_func')
                assert my_fixture == 'some'
        """
    })
    result = testdir.runpytest("-s")
    assert result.ret == 0
    result.stdout.fnmatch_lines([
        "*TESTS setup hook called for my_fixture from test_func*",
        "*ROOT setup hook called for my_fixture from test_func*",
        "*TEST test_func*",
        "*TESTS finalizer hook called for my_fixture from test_func*",
        "*ROOT finalizer hook called for my_fixture from test_func*",
    ])
