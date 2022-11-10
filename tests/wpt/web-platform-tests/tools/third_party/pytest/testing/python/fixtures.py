import os
import sys
import textwrap
from pathlib import Path

import pytest
from _pytest import fixtures
from _pytest.compat import getfuncargnames
from _pytest.config import ExitCode
from _pytest.fixtures import FixtureRequest
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pytester import get_public_names
from _pytest.pytester import Pytester
from _pytest.python import Function


def test_getfuncargnames_functions():
    """Test getfuncargnames for normal functions"""

    def f():
        raise NotImplementedError()

    assert not getfuncargnames(f)

    def g(arg):
        raise NotImplementedError()

    assert getfuncargnames(g) == ("arg",)

    def h(arg1, arg2="hello"):
        raise NotImplementedError()

    assert getfuncargnames(h) == ("arg1",)

    def j(arg1, arg2, arg3="hello"):
        raise NotImplementedError()

    assert getfuncargnames(j) == ("arg1", "arg2")


def test_getfuncargnames_methods():
    """Test getfuncargnames for normal methods"""

    class A:
        def f(self, arg1, arg2="hello"):
            raise NotImplementedError()

    assert getfuncargnames(A().f) == ("arg1",)


def test_getfuncargnames_staticmethod():
    """Test getfuncargnames for staticmethods"""

    class A:
        @staticmethod
        def static(arg1, arg2, x=1):
            raise NotImplementedError()

    assert getfuncargnames(A.static, cls=A) == ("arg1", "arg2")


def test_getfuncargnames_staticmethod_inherited() -> None:
    """Test getfuncargnames for inherited staticmethods (#8061)"""

    class A:
        @staticmethod
        def static(arg1, arg2, x=1):
            raise NotImplementedError()

    class B(A):
        pass

    assert getfuncargnames(B.static, cls=B) == ("arg1", "arg2")


def test_getfuncargnames_partial():
    """Check getfuncargnames for methods defined with functools.partial (#5701)"""
    import functools

    def check(arg1, arg2, i):
        raise NotImplementedError()

    class T:
        test_ok = functools.partial(check, i=2)

    values = getfuncargnames(T().test_ok, name="test_ok")
    assert values == ("arg1", "arg2")


def test_getfuncargnames_staticmethod_partial():
    """Check getfuncargnames for staticmethods defined with functools.partial (#5701)"""
    import functools

    def check(arg1, arg2, i):
        raise NotImplementedError()

    class T:
        test_ok = staticmethod(functools.partial(check, i=2))

    values = getfuncargnames(T().test_ok, name="test_ok")
    assert values == ("arg1", "arg2")


@pytest.mark.pytester_example_path("fixtures/fill_fixtures")
class TestFillFixtures:
    def test_fillfuncargs_exposed(self):
        # used by oejskit, kept for compatibility
        assert pytest._fillfuncargs == fixtures._fillfuncargs

    def test_funcarg_lookupfails(self, pytester: Pytester) -> None:
        pytester.copy_example()
        result = pytester.runpytest()  # "--collect-only")
        assert result.ret != 0
        result.stdout.fnmatch_lines(
            """
            *def test_func(some)*
            *fixture*some*not found*
            *xyzsomething*
            """
        )

    def test_detect_recursive_dependency_error(self, pytester: Pytester) -> None:
        pytester.copy_example()
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            ["*recursive dependency involving fixture 'fix1' detected*"]
        )

    def test_funcarg_basic(self, pytester: Pytester) -> None:
        pytester.copy_example()
        item = pytester.getitem(Path("test_funcarg_basic.py"))
        assert isinstance(item, Function)
        # Execute's item's setup, which fills fixtures.
        item.session._setupstate.setup(item)
        del item.funcargs["request"]
        assert len(get_public_names(item.funcargs)) == 2
        assert item.funcargs["some"] == "test_func"
        assert item.funcargs["other"] == 42

    def test_funcarg_lookup_modulelevel(self, pytester: Pytester) -> None:
        pytester.copy_example()
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_funcarg_lookup_classlevel(self, pytester: Pytester) -> None:
        p = pytester.copy_example()
        result = pytester.runpytest(p)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_conftest_funcargs_only_available_in_subdir(
        self, pytester: Pytester
    ) -> None:
        pytester.copy_example()
        result = pytester.runpytest("-v")
        result.assert_outcomes(passed=2)

    def test_extend_fixture_module_class(self, pytester: Pytester) -> None:
        testfile = pytester.copy_example()
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = pytester.runpytest(testfile)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_extend_fixture_conftest_module(self, pytester: Pytester) -> None:
        p = pytester.copy_example()
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = pytester.runpytest(str(next(Path(str(p)).rglob("test_*.py"))))
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_extend_fixture_conftest_conftest(self, pytester: Pytester) -> None:
        p = pytester.copy_example()
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = pytester.runpytest(str(next(Path(str(p)).rglob("test_*.py"))))
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_extend_fixture_conftest_plugin(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            testplugin="""
            import pytest

            @pytest.fixture
            def foo():
                return 7
        """
        )
        pytester.syspathinsert()
        pytester.makeconftest(
            """
            import pytest

            pytest_plugins = 'testplugin'

            @pytest.fixture
            def foo(foo):
                return foo + 7
        """
        )
        pytester.makepyfile(
            """
            def test_foo(foo):
                assert foo == 14
        """
        )
        result = pytester.runpytest("-s")
        assert result.ret == 0

    def test_extend_fixture_plugin_plugin(self, pytester: Pytester) -> None:
        # Two plugins should extend each order in loading order
        pytester.makepyfile(
            testplugin0="""
            import pytest

            @pytest.fixture
            def foo():
                return 7
        """
        )
        pytester.makepyfile(
            testplugin1="""
            import pytest

            @pytest.fixture
            def foo(foo):
                return foo + 7
        """
        )
        pytester.syspathinsert()
        pytester.makepyfile(
            """
            pytest_plugins = ['testplugin0', 'testplugin1']

            def test_foo(foo):
                assert foo == 14
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_override_parametrized_fixture_conftest_module(
        self, pytester: Pytester
    ) -> None:
        """Test override of the parametrized fixture with non-parametrized one on the test module level."""
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(params=[1, 2, 3])
            def spam(request):
                return request.param
        """
        )
        testfile = pytester.makepyfile(
            """
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'

            def test_spam(spam):
                assert spam == 'spam'
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = pytester.runpytest(testfile)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_override_parametrized_fixture_conftest_conftest(
        self, pytester: Pytester
    ) -> None:
        """Test override of the parametrized fixture with non-parametrized one on the conftest level."""
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(params=[1, 2, 3])
            def spam(request):
                return request.param
        """
        )
        subdir = pytester.mkpydir("subdir")
        subdir.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest

                @pytest.fixture
                def spam():
                    return 'spam'
                """
            )
        )
        testfile = subdir.joinpath("test_spam.py")
        testfile.write_text(
            textwrap.dedent(
                """\
                def test_spam(spam):
                    assert spam == "spam"
                """
            )
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = pytester.runpytest(testfile)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_override_non_parametrized_fixture_conftest_module(
        self, pytester: Pytester
    ) -> None:
        """Test override of the non-parametrized fixture with parametrized one on the test module level."""
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'
        """
        )
        testfile = pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(params=[1, 2, 3])
            def spam(request):
                return request.param

            params = {'spam': 1}

            def test_spam(spam):
                assert spam == params['spam']
                params['spam'] += 1
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*3 passed*"])
        result = pytester.runpytest(testfile)
        result.stdout.fnmatch_lines(["*3 passed*"])

    def test_override_non_parametrized_fixture_conftest_conftest(
        self, pytester: Pytester
    ) -> None:
        """Test override of the non-parametrized fixture with parametrized one on the conftest level."""
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture
            def spam():
                return 'spam'
        """
        )
        subdir = pytester.mkpydir("subdir")
        subdir.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest

                @pytest.fixture(params=[1, 2, 3])
                def spam(request):
                    return request.param
                """
            )
        )
        testfile = subdir.joinpath("test_spam.py")
        testfile.write_text(
            textwrap.dedent(
                """\
                params = {'spam': 1}

                def test_spam(spam):
                    assert spam == params['spam']
                    params['spam'] += 1
                """
            )
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*3 passed*"])
        result = pytester.runpytest(testfile)
        result.stdout.fnmatch_lines(["*3 passed*"])

    def test_override_autouse_fixture_with_parametrized_fixture_conftest_conftest(
        self, pytester: Pytester
    ) -> None:
        """Test override of the autouse fixture with parametrized one on the conftest level.
        This test covers the issue explained in issue 1601
        """
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(autouse=True)
            def spam():
                return 'spam'
        """
        )
        subdir = pytester.mkpydir("subdir")
        subdir.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest

                @pytest.fixture(params=[1, 2, 3])
                def spam(request):
                    return request.param
                """
            )
        )
        testfile = subdir.joinpath("test_spam.py")
        testfile.write_text(
            textwrap.dedent(
                """\
                params = {'spam': 1}

                def test_spam(spam):
                    assert spam == params['spam']
                    params['spam'] += 1
                """
            )
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*3 passed*"])
        result = pytester.runpytest(testfile)
        result.stdout.fnmatch_lines(["*3 passed*"])

    def test_override_fixture_reusing_super_fixture_parametrization(
        self, pytester: Pytester
    ) -> None:
        """Override a fixture at a lower level, reusing the higher-level fixture that
        is parametrized (#1953).
        """
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(params=[1, 2])
            def foo(request):
                return request.param
            """
        )
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture
            def foo(foo):
                return foo * 2

            def test_spam(foo):
                assert foo in (2, 4)
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*2 passed*"])

    def test_override_parametrize_fixture_and_indirect(
        self, pytester: Pytester
    ) -> None:
        """Override a fixture at a lower level, reusing the higher-level fixture that
        is parametrized, while also using indirect parametrization.
        """
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(params=[1, 2])
            def foo(request):
                return request.param
            """
        )
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture
            def foo(foo):
                return foo * 2

            @pytest.fixture
            def bar(request):
                return request.param * 100

            @pytest.mark.parametrize("bar", [42], indirect=True)
            def test_spam(bar, foo):
                assert bar == 4200
                assert foo in (2, 4)
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*2 passed*"])

    def test_override_top_level_fixture_reusing_super_fixture_parametrization(
        self, pytester: Pytester
    ) -> None:
        """Same as the above test, but with another level of overwriting."""
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(params=['unused', 'unused'])
            def foo(request):
                return request.param
            """
        )
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(params=[1, 2])
            def foo(request):
                return request.param

            class Test:

                @pytest.fixture
                def foo(self, foo):
                    return foo * 2

                def test_spam(self, foo):
                    assert foo in (2, 4)
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*2 passed*"])

    def test_override_parametrized_fixture_with_new_parametrized_fixture(
        self, pytester: Pytester
    ) -> None:
        """Overriding a parametrized fixture, while also parametrizing the new fixture and
        simultaneously requesting the overwritten fixture as parameter, yields the same value
        as ``request.param``.
        """
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(params=['ignored', 'ignored'])
            def foo(request):
                return request.param
            """
        )
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(params=[10, 20])
            def foo(foo, request):
                assert request.param == foo
                return foo * 2

            def test_spam(foo):
                assert foo in (20, 40)
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*2 passed*"])

    def test_autouse_fixture_plugin(self, pytester: Pytester) -> None:
        # A fixture from a plugin has no baseid set, which screwed up
        # the autouse fixture handling.
        pytester.makepyfile(
            testplugin="""
            import pytest

            @pytest.fixture(autouse=True)
            def foo(request):
                request.function.foo = 7
        """
        )
        pytester.syspathinsert()
        pytester.makepyfile(
            """
            pytest_plugins = 'testplugin'

            def test_foo(request):
                assert request.function.foo == 7
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_funcarg_lookup_error(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture
            def a_fixture(): pass

            @pytest.fixture
            def b_fixture(): pass

            @pytest.fixture
            def c_fixture(): pass

            @pytest.fixture
            def d_fixture(): pass
        """
        )
        pytester.makepyfile(
            """
            def test_lookup_error(unknown):
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*ERROR at setup of test_lookup_error*",
                "  def test_lookup_error(unknown):*",
                "E       fixture 'unknown' not found",
                ">       available fixtures:*a_fixture,*b_fixture,*c_fixture,*d_fixture*monkeypatch,*",
                # sorted
                ">       use 'py*test --fixtures *' for help on them.",
                "*1 error*",
            ]
        )
        result.stdout.no_fnmatch_line("*INTERNAL*")

    def test_fixture_excinfo_leak(self, pytester: Pytester) -> None:
        # on python2 sys.excinfo would leak into fixture executions
        pytester.makepyfile(
            """
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
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0


class TestRequestBasic:
    def test_request_attributes(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest

            @pytest.fixture
            def something(request): pass
            def test_func(something): pass
        """
        )
        assert isinstance(item, Function)
        req = fixtures.FixtureRequest(item, _ispytest=True)
        assert req.function == item.obj
        assert req.keywords == item.keywords
        assert hasattr(req.module, "test_func")
        assert req.cls is None
        assert req.function.__name__ == "test_func"
        assert req.config == item.config
        assert repr(req).find(req.function.__name__) != -1

    def test_request_attributes_method(self, pytester: Pytester) -> None:
        (item,) = pytester.getitems(
            """
            import pytest
            class TestB(object):

                @pytest.fixture
                def something(self, request):
                    return 1
                def test_func(self, something):
                    pass
        """
        )
        assert isinstance(item, Function)
        req = item._request
        assert req.cls.__name__ == "TestB"
        assert req.instance.__class__ == req.cls

    def test_request_contains_funcarg_arg2fixturedefs(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
            """
            import pytest
            @pytest.fixture
            def something(request):
                pass
            class TestClass(object):
                def test_method(self, something):
                    pass
        """
        )
        (item1,) = pytester.genitems([modcol])
        assert item1.name == "test_method"
        arg2fixturedefs = fixtures.FixtureRequest(
            item1, _ispytest=True
        )._arg2fixturedefs
        assert len(arg2fixturedefs) == 1
        assert arg2fixturedefs["something"][0].argname == "something"

    @pytest.mark.skipif(
        hasattr(sys, "pypy_version_info"),
        reason="this method of test doesn't work on pypy",
    )
    def test_request_garbage(self, pytester: Pytester) -> None:
        try:
            import xdist  # noqa
        except ImportError:
            pass
        else:
            pytest.xfail("this test is flaky when executed with xdist")
        pytester.makepyfile(
            """
            import sys
            import pytest
            from _pytest.fixtures import PseudoFixtureDef
            import gc

            @pytest.fixture(autouse=True)
            def something(request):
                original = gc.get_debug()
                gc.set_debug(gc.DEBUG_SAVEALL)
                gc.collect()

                yield

                try:
                    gc.collect()
                    leaked = [x for _ in gc.garbage if isinstance(_, PseudoFixtureDef)]
                    assert leaked == []
                finally:
                    gc.set_debug(original)

            def test_func():
                pass
        """
        )
        result = pytester.runpytest_subprocess()
        result.stdout.fnmatch_lines(["* 1 passed in *"])

    def test_getfixturevalue_recursive(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture
            def something(request):
                return 1
        """
        )
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture
            def something(request):
                return request.getfixturevalue("something") + 1
            def test_func(something):
                assert something == 2
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_getfixturevalue_teardown(self, pytester: Pytester) -> None:
        """
        Issue #1895

        `test_inner` requests `inner` fixture, which in turn requests `resource`
        using `getfixturevalue`. `test_func` then requests `resource`.

        `resource` is teardown before `inner` because the fixture mechanism won't consider
        `inner` dependent on `resource` when it is used via `getfixturevalue`: `test_func`
        will then cause the `resource`'s finalizer to be called first because of this.
        """
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(scope='session')
            def resource():
                r = ['value']
                yield r
                r.pop()

            @pytest.fixture(scope='session')
            def inner(request):
                resource = request.getfixturevalue('resource')
                assert resource == ['value']
                yield
                assert resource == ['value']

            def test_inner(inner):
                pass

            def test_func(resource):
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed in *"])

    def test_getfixturevalue(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest

            @pytest.fixture
            def something(request):
                return 1

            values = [2]
            @pytest.fixture
            def other(request):
                return values.pop()

            def test_func(something): pass
        """
        )
        assert isinstance(item, Function)
        req = item._request

        # Execute item's setup.
        item.session._setupstate.setup(item)

        with pytest.raises(pytest.FixtureLookupError):
            req.getfixturevalue("notexists")
        val = req.getfixturevalue("something")
        assert val == 1
        val = req.getfixturevalue("something")
        assert val == 1
        val2 = req.getfixturevalue("other")
        assert val2 == 2
        val2 = req.getfixturevalue("other")  # see about caching
        assert val2 == 2
        assert item.funcargs["something"] == 1
        assert len(get_public_names(item.funcargs)) == 2
        assert "request" in item.funcargs

    def test_request_addfinalizer(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest
            teardownlist = []
            @pytest.fixture
            def something(request):
                request.addfinalizer(lambda: teardownlist.append(1))
            def test_func(something): pass
        """
        )
        assert isinstance(item, Function)
        item.session._setupstate.setup(item)
        item._request._fillfixtures()
        # successively check finalization calls
        parent = item.getparent(pytest.Module)
        assert parent is not None
        teardownlist = parent.obj.teardownlist
        ss = item.session._setupstate
        assert not teardownlist
        ss.teardown_exact(None)
        print(ss.stack)
        assert teardownlist == [1]

    def test_request_addfinalizer_failing_setup(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(failed=1, passed=1)

    def test_request_addfinalizer_failing_setup_module(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            import pytest
            values = [1, 2]
            @pytest.fixture(scope="module")
            def myfix(request):
                request.addfinalizer(values.pop)
                request.addfinalizer(values.pop)
                assert 0
            def test_fix(myfix):
                pass
        """
        )
        reprec = pytester.inline_run("-s")
        mod = reprec.getcalls("pytest_runtest_setup")[0].item.module
        assert not mod.values

    def test_request_addfinalizer_partial_setup_failure(
        self, pytester: Pytester
    ) -> None:
        p = pytester.makepyfile(
            """
            import pytest
            values = []
            @pytest.fixture
            def something(request):
                request.addfinalizer(lambda: values.append(None))
            def test_func(something, missingarg):
                pass
            def test_second():
                assert len(values) == 1
        """
        )
        result = pytester.runpytest(p)
        result.stdout.fnmatch_lines(
            ["*1 error*"]  # XXX the whole module collection fails
        )

    def test_request_subrequest_addfinalizer_exceptions(
        self, pytester: Pytester
    ) -> None:
        """
        Ensure exceptions raised during teardown by a finalizer are suppressed
        until all finalizers are called, re-raising the first exception (#2440)
        """
        pytester.makepyfile(
            """
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
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            ["*Exception: Error in excepts fixture", "* 2 passed, 1 error in *"]
        )

    def test_request_getmodulepath(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol("def test_somefunc(): pass")
        (item,) = pytester.genitems([modcol])
        req = fixtures.FixtureRequest(item, _ispytest=True)
        assert req.path == modcol.path

    def test_request_fixturenames(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            from _pytest.pytester import get_public_names
            @pytest.fixture()
            def arg1():
                pass
            @pytest.fixture()
            def farg(arg1):
                pass
            @pytest.fixture(autouse=True)
            def sarg(tmp_path):
                pass
            def test_function(request, farg):
                assert set(get_public_names(request.fixturenames)) == \
                       set(["sarg", "arg1", "request", "farg",
                            "tmp_path", "tmp_path_factory"])
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_request_fixturenames_dynamic_fixture(self, pytester: Pytester) -> None:
        """Regression test for #3057"""
        pytester.copy_example("fixtures/test_getfixturevalue_dynamic.py")
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_setupdecorator_and_xunit(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run("-v")
        reprec.assertoutcome(passed=3)

    def test_fixtures_sub_subdir_normalize_sep(self, pytester: Pytester) -> None:
        # this tests that normalization of nodeids takes place
        b = pytester.path.joinpath("tests", "unit")
        b.mkdir(parents=True)
        b.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest
                @pytest.fixture
                def arg1():
                    pass
                """
            )
        )
        p = b.joinpath("test_module.py")
        p.write_text("def test_func(arg1): pass")
        result = pytester.runpytest(p, "--fixtures")
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            """
            *fixtures defined*conftest*
            *arg1*
        """
        )

    def test_show_fixtures_color_yes(self, pytester: Pytester) -> None:
        pytester.makepyfile("def test_this(): assert 1")
        result = pytester.runpytest("--color=yes", "--fixtures")
        assert "\x1b[32mtmp_path" in result.stdout.str()

    def test_newstyle_with_request(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture()
            def arg(request):
                pass
            def test_1(arg):
                pass
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_setupcontext_no_param(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(params=[1,2])
            def arg(request):
                return request.param

            @pytest.fixture(autouse=True)
            def mysetup(request, arg):
                assert not hasattr(request, "param")
            def test_1(arg):
                assert arg in (1,2)
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)


class TestRequestSessionScoped:
    @pytest.fixture(scope="session")
    def session_request(self, request):
        return request

    @pytest.mark.parametrize("name", ["path", "module"])
    def test_session_scoped_unavailable_attributes(self, session_request, name):
        with pytest.raises(
            AttributeError,
            match=f"{name} not available in session-scoped context",
        ):
            getattr(session_request, name)


class TestRequestMarking:
    def test_applymarker(self, pytester: Pytester) -> None:
        item1, item2 = pytester.getitems(
            """
            import pytest

            @pytest.fixture
            def something(request):
                pass
            class TestClass(object):
                def test_func1(self, something):
                    pass
                def test_func2(self, something):
                    pass
        """
        )
        req1 = fixtures.FixtureRequest(item1, _ispytest=True)
        assert "xfail" not in item1.keywords
        req1.applymarker(pytest.mark.xfail)
        assert "xfail" in item1.keywords
        assert "skipif" not in item1.keywords
        req1.applymarker(pytest.mark.skipif)
        assert "skipif" in item1.keywords
        with pytest.raises(ValueError):
            req1.applymarker(42)  # type: ignore[arg-type]

    def test_accesskeywords(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture()
            def keywords(request):
                return request.keywords
            @pytest.mark.XYZ
            def test_function(keywords):
                assert keywords["XYZ"]
                assert "abc" not in keywords
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_accessmarker_dynamic(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            @pytest.fixture()
            def keywords(request):
                return request.keywords

            @pytest.fixture(scope="class", autouse=True)
            def marking(request):
                request.applymarker(pytest.mark.XYZ("hello"))
        """
        )
        pytester.makepyfile(
            """
            import pytest
            def test_fun1(keywords):
                assert keywords["XYZ"] is not None
                assert "abc" not in keywords
            def test_fun2(keywords):
                assert keywords["XYZ"] is not None
                assert "abc" not in keywords
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)


class TestFixtureUsages:
    def test_noargfixturedec(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture
            def arg1():
                return 1

            def test_func(arg1):
                assert arg1 == 1
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_receives_funcargs(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_receives_funcargs_scope_mismatch(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="function")
            def arg1():
                return 1

            @pytest.fixture(scope="module")
            def arg2(arg1):
                return arg1 + 1

            def test_add(arg2):
                assert arg2 == 2
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*ScopeMismatch*involved factories*",
                "test_receives_funcargs_scope_mismatch.py:6:  def arg2(arg1)",
                "test_receives_funcargs_scope_mismatch.py:2:  def arg1()",
                "*1 error*",
            ]
        )

    def test_receives_funcargs_scope_mismatch_issue660(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="function")
            def arg1():
                return 1

            @pytest.fixture(scope="module")
            def arg2(arg1):
                return arg1 + 1

            def test_add(arg1, arg2):
                assert arg2 == 2
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            ["*ScopeMismatch*involved factories*", "* def arg2*", "*1 error*"]
        )

    def test_invalid_scope(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="functions")
            def badscope():
                pass

            def test_nothing(badscope):
                pass
        """
        )
        result = pytester.runpytest_inprocess()
        result.stdout.fnmatch_lines(
            "*Fixture 'badscope' from test_invalid_scope.py got an unexpected scope value 'functions'"
        )

    @pytest.mark.parametrize("scope", ["function", "session"])
    def test_parameters_without_eq_semantics(self, scope, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            class NoEq1:  # fails on `a == b` statement
                def __eq__(self, _):
                    raise RuntimeError

            class NoEq2:  # fails on `if a == b:` statement
                def __eq__(self, _):
                    class NoBool:
                        def __bool__(self):
                            raise RuntimeError
                    return NoBool()

            import pytest
            @pytest.fixture(params=[NoEq1(), NoEq2()], scope={scope!r})
            def no_eq(request):
                return request.param

            def test1(no_eq):
                pass

            def test2(no_eq):
                pass
        """.format(
                scope=scope
            )
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*4 passed*"])

    def test_funcarg_parametrized_and_used_twice(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*2 passed*"])

    def test_factory_uses_unknown_funcarg_as_dependency_error(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture()
            def fail(missing):
                return

            @pytest.fixture()
            def call_fail(fail):
                return

            def test_missing(call_fail):
                pass
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            """
            *pytest.fixture()*
            *def call_fail(fail)*
            *pytest.fixture()*
            *def fail*
            *fixture*'missing'*not found*
        """
        )

    def test_factory_setup_as_classes_fails(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            class arg1(object):
                def __init__(self, request):
                    self.x = 1
            arg1 = pytest.fixture()(arg1)

        """
        )
        reprec = pytester.inline_run()
        values = reprec.getfailedcollections()
        assert len(values) == 1

    def test_usefixtures_marker(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_usefixtures_ini(self, pytester: Pytester) -> None:
        pytester.makeini(
            """
            [pytest]
            usefixtures = myfix
        """
        )
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(scope="class")
            def myfix(request):
                request.cls.hello = "world"

        """
        )
        pytester.makepyfile(
            """
            class TestClass(object):
                def test_one(self):
                    assert self.hello == "world"
                def test_two(self):
                    assert self.hello == "world"
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_usefixtures_seen_in_showmarkers(self, pytester: Pytester) -> None:
        result = pytester.runpytest("--markers")
        result.stdout.fnmatch_lines(
            """
            *usefixtures(fixturename1*mark tests*fixtures*
        """
        )

    def test_request_instance_issue203(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            class TestClass(object):
                @pytest.fixture
                def setup1(self, request):
                    assert self == request.instance
                    self.arg1 = 1
                def test_hello(self, setup1):
                    assert self.arg1 == 1
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_fixture_parametrized_with_iterator(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run("-v")
        reprec.assertoutcome(passed=4)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert values == [1, 2, 10, 20]

    def test_setup_functions_as_fixtures(self, pytester: Pytester) -> None:
        """Ensure setup_* methods obey fixture scope rules (#517, #3094)."""
        pytester.makepyfile(
            """
            import pytest

            DB_INITIALIZED = None

            @pytest.fixture(scope="session", autouse=True)
            def db():
                global DB_INITIALIZED
                DB_INITIALIZED = True
                yield
                DB_INITIALIZED = False

            def setup_module():
                assert DB_INITIALIZED

            def teardown_module():
                assert DB_INITIALIZED

            class TestClass(object):

                def setup_method(self, method):
                    assert DB_INITIALIZED

                def teardown_method(self, method):
                    assert DB_INITIALIZED

                def test_printer_1(self):
                    pass

                def test_printer_2(self):
                    pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed in *"])


class TestFixtureManagerParseFactories:
    @pytest.fixture
    def pytester(self, pytester: Pytester) -> Pytester:
        pytester.makeconftest(
            """
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
        """
        )
        return pytester

    def test_parsefactories_evil_objects_issue214(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            class A(object):
                def __call__(self):
                    pass
                def __getattr__(self, name):
                    raise RuntimeError()
            a = A()
            def test_hello():
                pass
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1, failed=0)

    def test_parsefactories_conftest(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_hello(item, fm):
                for name in ("fm", "hello", "item"):
                    faclist = fm.getfixturedefs(name, item.nodeid)
                    assert len(faclist) == 1
                    fac = faclist[0]
                    assert fac.func.__name__ == name
        """
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(passed=1)

    def test_parsefactories_conftest_and_module_and_class(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """\
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
                    print(faclist)
                    assert len(faclist) == 3

                    assert faclist[0].func(item._request) == "conftest"
                    assert faclist[1].func(item._request) == "module"
                    assert faclist[2].func(item._request) == "class"
            """
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(passed=1)

    def test_parsefactories_relative_node_ids(
        self, pytester: Pytester, monkeypatch: MonkeyPatch
    ) -> None:
        # example mostly taken from:
        # https://mail.python.org/pipermail/pytest-dev/2014-September/002617.html
        runner = pytester.mkdir("runner")
        package = pytester.mkdir("package")
        package.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
            import pytest
            @pytest.fixture
            def one():
                return 1
            """
            )
        )
        package.joinpath("test_x.py").write_text(
            textwrap.dedent(
                """\
                def test_x(one):
                    assert one == 1
                """
            )
        )
        sub = package.joinpath("sub")
        sub.mkdir()
        sub.joinpath("__init__.py").touch()
        sub.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest
                @pytest.fixture
                def one():
                    return 2
                """
            )
        )
        sub.joinpath("test_y.py").write_text(
            textwrap.dedent(
                """\
                def test_x(one):
                    assert one == 2
                """
            )
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)
        with monkeypatch.context() as mp:
            mp.chdir(runner)
            reprec = pytester.inline_run("..")
            reprec.assertoutcome(passed=2)

    def test_package_xunit_fixture(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            __init__="""\
            values = []
        """
        )
        package = pytester.mkdir("package")
        package.joinpath("__init__.py").write_text(
            textwrap.dedent(
                """\
                from .. import values
                def setup_module():
                    values.append("package")
                def teardown_module():
                    values[:] = []
                """
            )
        )
        package.joinpath("test_x.py").write_text(
            textwrap.dedent(
                """\
                from .. import values
                def test_x():
                    assert values == ["package"]
                """
            )
        )
        package = pytester.mkdir("package2")
        package.joinpath("__init__.py").write_text(
            textwrap.dedent(
                """\
                from .. import values
                def setup_module():
                    values.append("package2")
                def teardown_module():
                    values[:] = []
                """
            )
        )
        package.joinpath("test_x.py").write_text(
            textwrap.dedent(
                """\
                from .. import values
                def test_x():
                    assert values == ["package2"]
                """
            )
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_package_fixture_complex(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            __init__="""\
            values = []
        """
        )
        pytester.syspathinsert(pytester.path.name)
        package = pytester.mkdir("package")
        package.joinpath("__init__.py").write_text("")
        package.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest
                from .. import values
                @pytest.fixture(scope="package")
                def one():
                    values.append("package")
                    yield values
                    values.pop()
                @pytest.fixture(scope="package", autouse=True)
                def two():
                    values.append("package-auto")
                    yield values
                    values.pop()
                """
            )
        )
        package.joinpath("test_x.py").write_text(
            textwrap.dedent(
                """\
                from .. import values
                def test_package_autouse():
                    assert values == ["package-auto"]
                def test_package(one):
                    assert values == ["package-auto", "package"]
                """
            )
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_collect_custom_items(self, pytester: Pytester) -> None:
        pytester.copy_example("fixtures/custom_item")
        result = pytester.runpytest("foo")
        result.stdout.fnmatch_lines(["*passed*"])


class TestAutouseDiscovery:
    @pytest.fixture
    def pytester(self, pytester: Pytester) -> Pytester:
        pytester.makeconftest(
            """
            import pytest
            @pytest.fixture(autouse=True)
            def perfunction(request, tmp_path):
                pass

            @pytest.fixture()
            def arg1(tmp_path):
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
        """
        )
        return pytester

    def test_parsefactories_conftest(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            from _pytest.pytester import get_public_names
            def test_check_setup(item, fm):
                autousenames = list(fm._getautousenames(item.nodeid))
                assert len(get_public_names(autousenames)) == 2
                assert "perfunction2" in autousenames
                assert "perfunction" in autousenames
        """
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(passed=1)

    def test_two_classes_separated_autouse(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_setup_at_classlevel(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            class TestClass(object):
                @pytest.fixture(autouse=True)
                def permethod(self, request):
                    request.instance.funcname = request.function.__name__
                def test_method1(self):
                    assert self.funcname == "test_method1"
                def test_method2(self):
                    assert self.funcname == "test_method2"
        """
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(passed=2)

    @pytest.mark.xfail(reason="'enabled' feature not implemented")
    def test_setup_enabled_functionnode(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(passed=2)

    def test_callables_nocode(self, pytester: Pytester) -> None:
        """An imported mock.call would break setup/factory discovery due to
        it being callable and __code__ not being a code object."""
        pytester.makepyfile(
            """
           class _call(tuple):
               def __call__(self, *k, **kw):
                   pass
               def __getattr__(self, k):
                   return self

           call = _call()
        """
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(failed=0, passed=0)

    def test_autouse_in_conftests(self, pytester: Pytester) -> None:
        a = pytester.mkdir("a")
        b = pytester.mkdir("a1")
        conftest = pytester.makeconftest(
            """
            import pytest
            @pytest.fixture(autouse=True)
            def hello():
                xxx
        """
        )
        conftest.rename(a.joinpath(conftest.name))
        a.joinpath("test_something.py").write_text("def test_func(): pass")
        b.joinpath("test_otherthing.py").write_text("def test_func(): pass")
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            """
            *1 passed*1 error*
        """
        )

    def test_autouse_in_module_and_two_classes(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=3)


class TestAutouseManagement:
    def test_autouse_conftest_mid_directory(self, pytester: Pytester) -> None:
        pkgdir = pytester.mkpydir("xyz123")
        pkgdir.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest
                @pytest.fixture(autouse=True)
                def app():
                    import sys
                    sys._myapp = "hello"
                """
            )
        )
        sub = pkgdir.joinpath("tests")
        sub.mkdir()
        t = sub.joinpath("test_app.py")
        t.touch()
        t.write_text(
            textwrap.dedent(
                """\
                import sys
                def test_app():
                    assert sys._myapp == "hello"
                """
            )
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(passed=1)

    def test_funcarg_and_setup(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_uses_parametrized_resource(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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

        """
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(passed=2)

    def test_session_parametrized_function(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run("-v", "-s")
        reprec.assertoutcome(passed=4)

    def test_class_function_parametrization_finalization(
        self, pytester: Pytester
    ) -> None:
        p = pytester.makeconftest(
            """
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
        """
        )
        pytester.makepyfile(
            """
            import pytest

            class TestClass(object):
                def test_1(self):
                    pass
            class TestClass2(object):
                def test_2(self):
                    pass
        """
        )
        reprec = pytester.inline_run("-v", "-s", "--confcutdir", pytester.path)
        reprec.assertoutcome(passed=8)
        config = reprec.getcalls("pytest_unconfigure")[0].config
        values = config.pluginmanager._getconftestmodules(
            p, importmode="prepend", rootpath=pytester.path
        )[0].values
        assert values == ["fin_a1", "fin_a2", "fin_b1", "fin_b2"] * 2

    def test_scope_ordering(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_parametrization_setup_teardown_ordering(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            values = []
            def pytest_generate_tests(metafunc):
                if metafunc.cls is None:
                    assert metafunc.function is test_finish
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
                print(values)
                assert values == ["setup-1", "step1-1", "step2-1", "teardown-1",
                             "setup-2", "step1-2", "step2-2", "teardown-2",]
        """
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(passed=5)

    def test_ordering_autouse_before_explicit(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    @pytest.mark.parametrize("param1", ["", "params=[1]"], ids=["p00", "p01"])
    @pytest.mark.parametrize("param2", ["", "params=[1]"], ids=["p10", "p11"])
    def test_ordering_dependencies_torndown_first(
        self, pytester: Pytester, param1, param2
    ) -> None:
        """#226"""
        pytester.makepyfile(
            """
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
        """
            % locals()
        )
        reprec = pytester.inline_run("-s")
        reprec.assertoutcome(passed=2)


class TestFixtureMarker:
    def test_parametrize(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(params=["a", "b", "c"])
            def arg(request):
                return request.param
            values = []
            def test_param(arg):
                values.append(arg)
            def test_result():
                assert values == list("abc")
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=4)

    def test_multiple_parametrization_issue_736(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(params=[1,2,3])
            def foo(request):
                return request.param

            @pytest.mark.parametrize('foobar', [4,5,6])
            def test_issue(foo, foobar):
                assert foo in [1,2,3]
                assert foobar in [4,5,6]
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=9)

    @pytest.mark.parametrize(
        "param_args",
        ["'fixt, val'", "'fixt,val'", "['fixt', 'val']", "('fixt', 'val')"],
    )
    def test_override_parametrized_fixture_issue_979(
        self, pytester: Pytester, param_args
    ) -> None:
        """Make sure a parametrized argument can override a parametrized fixture.

        This was a regression introduced in the fix for #736.
        """
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(params=[1, 2])
            def fixt(request):
                return request.param

            @pytest.mark.parametrize(%s, [(3, 'x'), (4, 'x')])
            def test_foo(fixt, val):
                pass
        """
            % param_args
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_scope_session(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=3)

    def test_scope_session_exc(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(skipped=2, passed=1)

    def test_scope_session_exc_two_fix(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(skipped=2, passed=1)

    def test_scope_exc(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
            """,
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(skipped=2, passed=1)

    def test_scope_module_uses_session(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=3)

    def test_scope_module_and_finalizer(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
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
        """
        )
        pytester.makepyfile(
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
            """,
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=4)

    def test_scope_mismatch_various(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            finalized = []
            created = []
            @pytest.fixture(scope="function")
            def arg(request):
                pass
        """
        )
        pytester.makepyfile(
            test_mod1="""
                import pytest
                @pytest.fixture(scope="session")
                def arg(request):
                    request.getfixturevalue("arg")
                def test_1(arg):
                    pass
            """
        )
        result = pytester.runpytest()
        assert result.ret != 0
        result.stdout.fnmatch_lines(
            ["*ScopeMismatch*You tried*function*session*request*"]
        )

    def test_dynamic_scope(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest


            def pytest_addoption(parser):
                parser.addoption("--extend-scope", action="store_true", default=False)


            def dynamic_scope(fixture_name, config):
                if config.getoption("--extend-scope"):
                    return "session"
                return "function"


            @pytest.fixture(scope=dynamic_scope)
            def dynamic_fixture(calls=[]):
                calls.append("call")
                return len(calls)

        """
        )

        pytester.makepyfile(
            """
            def test_first(dynamic_fixture):
                assert dynamic_fixture == 1


            def test_second(dynamic_fixture):
                assert dynamic_fixture == 2

        """
        )

        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

        reprec = pytester.inline_run("--extend-scope")
        reprec.assertoutcome(passed=1, failed=1)

    def test_dynamic_scope_bad_return(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            def dynamic_scope(**_):
                return "wrong-scope"

            @pytest.fixture(scope=dynamic_scope)
            def fixture():
                pass

        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            "Fixture 'fixture' from test_dynamic_scope_bad_return.py "
            "got an unexpected scope value 'wrong-scope'"
        )

    def test_register_only_with_mark(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            @pytest.fixture()
            def arg():
                return 1
        """
        )
        pytester.makepyfile(
            test_mod1="""
                import pytest
                @pytest.fixture()
                def arg(arg):
                    return arg + 1
                def test_1(arg):
                    assert arg == 2
            """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_parametrize_and_scope(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="module", params=["a", "b", "c"])
            def arg(request):
                return request.param
            values = []
            def test_param(arg):
                values.append(arg)
        """
        )
        reprec = pytester.inline_run("-v")
        reprec.assertoutcome(passed=3)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert len(values) == 3
        assert "a" in values
        assert "b" in values
        assert "c" in values

    def test_scope_mismatch(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            @pytest.fixture(scope="function")
            def arg(request):
                pass
        """
        )
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="session")
            def arg(arg):
                pass
            def test_mismatch(arg):
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*ScopeMismatch*", "*1 error*"])

    def test_parametrize_separated_order(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(scope="module", params=[1, 2])
            def arg(request):
                return request.param

            values = []
            def test_1(arg):
                values.append(arg)
            def test_2(arg):
                values.append(arg)
        """
        )
        reprec = pytester.inline_run("-v")
        reprec.assertoutcome(passed=4)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert values == [1, 1, 2, 2]

    def test_module_parametrized_ordering(self, pytester: Pytester) -> None:
        pytester.makeini(
            """
            [pytest]
            console_output_style=classic
        """
        )
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(scope="session", params="s1 s2".split())
            def sarg():
                pass
            @pytest.fixture(scope="module", params="m1 m2".split())
            def marg():
                pass
        """
        )
        pytester.makepyfile(
            test_mod1="""
            def test_func(sarg):
                pass
            def test_func1(marg):
                pass
        """,
            test_mod2="""
            def test_func2(sarg):
                pass
            def test_func3(sarg, marg):
                pass
            def test_func3b(sarg, marg):
                pass
            def test_func4(marg):
                pass
        """,
        )
        result = pytester.runpytest("-v")
        result.stdout.fnmatch_lines(
            """
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
        """
        )

    def test_dynamic_parametrized_ordering(self, pytester: Pytester) -> None:
        pytester.makeini(
            """
            [pytest]
            console_output_style=classic
        """
        )
        pytester.makeconftest(
            """
            import pytest

            def pytest_configure(config):
                class DynamicFixturePlugin(object):
                    @pytest.fixture(scope='session', params=['flavor1', 'flavor2'])
                    def flavor(self, request):
                        return request.param
                config.pluginmanager.register(DynamicFixturePlugin(), 'flavor-fixture')

            @pytest.fixture(scope='session', params=['vxlan', 'vlan'])
            def encap(request):
                return request.param

            @pytest.fixture(scope='session', autouse='True')
            def reprovision(request, flavor, encap):
                pass
        """
        )
        pytester.makepyfile(
            """
            def test(reprovision):
                pass
            def test2(reprovision):
                pass
        """
        )
        result = pytester.runpytest("-v")
        result.stdout.fnmatch_lines(
            """
            test_dynamic_parametrized_ordering.py::test[flavor1-vxlan] PASSED
            test_dynamic_parametrized_ordering.py::test2[flavor1-vxlan] PASSED
            test_dynamic_parametrized_ordering.py::test[flavor2-vxlan] PASSED
            test_dynamic_parametrized_ordering.py::test2[flavor2-vxlan] PASSED
            test_dynamic_parametrized_ordering.py::test[flavor2-vlan] PASSED
            test_dynamic_parametrized_ordering.py::test2[flavor2-vlan] PASSED
            test_dynamic_parametrized_ordering.py::test[flavor1-vlan] PASSED
            test_dynamic_parametrized_ordering.py::test2[flavor1-vlan] PASSED
        """
        )

    def test_class_ordering(self, pytester: Pytester) -> None:
        pytester.makeini(
            """
            [pytest]
            console_output_style=classic
        """
        )
        pytester.makeconftest(
            """
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
        """
        )
        pytester.makepyfile(
            """
            import pytest

            class TestClass2(object):
                def test_1(self):
                    pass
                def test_2(self):
                    pass
            class TestClass(object):
                def test_3(self):
                    pass
        """
        )
        result = pytester.runpytest("-vs")
        result.stdout.re_match_lines(
            r"""
            test_class_ordering.py::TestClass2::test_1\[a-1\] PASSED
            test_class_ordering.py::TestClass2::test_1\[a-2\] PASSED
            test_class_ordering.py::TestClass2::test_2\[a-1\] PASSED
            test_class_ordering.py::TestClass2::test_2\[a-2\] PASSED
            test_class_ordering.py::TestClass2::test_1\[b-1\] PASSED
            test_class_ordering.py::TestClass2::test_1\[b-2\] PASSED
            test_class_ordering.py::TestClass2::test_2\[b-1\] PASSED
            test_class_ordering.py::TestClass2::test_2\[b-2\] PASSED
            test_class_ordering.py::TestClass::test_3\[a-1\] PASSED
            test_class_ordering.py::TestClass::test_3\[a-2\] PASSED
            test_class_ordering.py::TestClass::test_3\[b-1\] PASSED
            test_class_ordering.py::TestClass::test_3\[b-2\] PASSED
        """
        )

    def test_parametrize_separated_order_higher_scope_first(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run("-v")
        reprec.assertoutcome(passed=12)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        expected = [
            "create:1",
            "test1",
            "fin:1",
            "create:2",
            "test1",
            "fin:2",
            "create:mod1",
            "test2",
            "create:1",
            "test3",
            "fin:1",
            "create:2",
            "test3",
            "fin:2",
            "create:1",
            "test4",
            "fin:1",
            "create:2",
            "test4",
            "fin:2",
            "fin:mod1",
            "create:mod2",
            "test2",
            "create:1",
            "test3",
            "fin:1",
            "create:2",
            "test3",
            "fin:2",
            "create:1",
            "test4",
            "fin:1",
            "create:2",
            "test4",
            "fin:2",
            "fin:mod2",
        ]
        import pprint

        pprint.pprint(list(zip(values, expected)))
        assert values == expected

    def test_parametrized_fixture_teardown_order(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        result = pytester.runpytest("-v")
        result.stdout.fnmatch_lines(
            """
            *3 passed*
        """
        )
        result.stdout.no_fnmatch_line("*error*")

    def test_fixture_finalizer(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            import sys

            @pytest.fixture
            def browser(request):

                def finalize():
                    sys.stdout.write_text('Finalized')
                request.addfinalizer(finalize)
                return {}
        """
        )
        b = pytester.mkdir("subdir")
        b.joinpath("test_overridden_fixture_finalizer.py").write_text(
            textwrap.dedent(
                """\
                import pytest
                @pytest.fixture
                def browser(browser):
                    browser['visited'] = True
                    return browser

                def test_browser(browser):
                    assert browser['visited'] is True
                """
            )
        )
        reprec = pytester.runpytest("-s")
        for test in ["test_browser"]:
            reprec.stdout.fnmatch_lines(["*Finalized*"])

    def test_class_scope_with_normal_tests(self, pytester: Pytester) -> None:
        testpath = pytester.makepyfile(
            """
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
                    assert a == 3"""
        )
        reprec = pytester.inline_run(testpath)
        for test in ["test_a", "test_b", "test_c"]:
            assert reprec.matchreport(test).passed

    def test_request_is_clean(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            values = []
            @pytest.fixture(params=[1, 2])
            def fix(request):
                request.addfinalizer(lambda: values.append(request.param))
            def test_fix(fix):
                pass
        """
        )
        reprec = pytester.inline_run("-s")
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert values == [1, 2]

    def test_parametrize_separated_lifecycle(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run("-vs")
        reprec.assertoutcome(passed=4)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        import pprint

        pprint.pprint(values)
        # assert len(values) == 6
        assert values[0] == values[1] == 1
        assert values[2] == "fin1"
        assert values[3] == values[4] == 2
        assert values[5] == "fin2"

    def test_parametrize_function_scoped_finalizers_called(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run("-v")
        reprec.assertoutcome(passed=5)

    @pytest.mark.parametrize("scope", ["session", "function", "module"])
    def test_finalizer_order_on_parametrization(
        self, scope, pytester: Pytester
    ) -> None:
        """#246"""
        pytester.makepyfile(
            """
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
                    print("finalizing base")
                request.addfinalizer(cleanup_base)

            def test_begin():
                pass
            def test_baz(base, fix2):
                pass
            def test_other():
                pass
        """
            % {"scope": scope}
        )
        reprec = pytester.inline_run("-lvs")
        reprec.assertoutcome(passed=3)

    def test_class_scope_parametrization_ordering(self, pytester: Pytester) -> None:
        """#396"""
        pytester.makepyfile(
            """
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
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=6)
        values = reprec.getcalls("pytest_runtest_call")[0].item.module.values
        assert values == [
            "test_hello",
            "fin John",
            "test_hello",
            "fin Doe",
            "test_name",
            "test_population",
            "fin John",
            "test_name",
            "test_population",
            "fin Doe",
        ]

    def test_parametrize_setup_function(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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

        """
        )
        reprec = pytester.inline_run("-v")
        reprec.assertoutcome(passed=6)

    def test_fixture_marked_function_not_collected_as_test(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture
            def test_app():
                return 1

            def test_something(test_app):
                assert test_app == 1
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_params_and_ids(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(params=[object(), object()],
                            ids=['alpha', 'beta'])
            def fix(request):
                return request.param

            def test_foo(fix):
                assert 1
        """
        )
        res = pytester.runpytest("-v")
        res.stdout.fnmatch_lines(["*test_foo*alpha*", "*test_foo*beta*"])

    def test_params_and_ids_yieldfixture(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(params=[object(), object()], ids=['alpha', 'beta'])
            def fix(request):
                 yield request.param

            def test_foo(fix):
                assert 1
        """
        )
        res = pytester.runpytest("-v")
        res.stdout.fnmatch_lines(["*test_foo*alpha*", "*test_foo*beta*"])

    def test_deterministic_fixture_collection(
        self, pytester: Pytester, monkeypatch
    ) -> None:
        """#920"""
        pytester.makepyfile(
            """
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
            """
        )
        monkeypatch.setenv("PYTHONHASHSEED", "1")
        out1 = pytester.runpytest_subprocess("-v")
        monkeypatch.setenv("PYTHONHASHSEED", "2")
        out2 = pytester.runpytest_subprocess("-v")
        output1 = [
            line
            for line in out1.outlines
            if line.startswith("test_deterministic_fixture_collection.py::test_foo")
        ]
        output2 = [
            line
            for line in out2.outlines
            if line.startswith("test_deterministic_fixture_collection.py::test_foo")
        ]
        assert len(output1) == 12
        assert output1 == output2


class TestRequestScopeAccess:
    pytestmark = pytest.mark.parametrize(
        ("scope", "ok", "error"),
        [
            ["session", "", "path class function module"],
            ["module", "module path", "cls function"],
            ["class", "module path cls", "function"],
            ["function", "module path cls function", ""],
        ],
    )

    def test_setup(self, pytester: Pytester, scope, ok, error) -> None:
        pytester.makepyfile(
            """
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
        """
            % (scope, ok.split(), error.split())
        )
        reprec = pytester.inline_run("-l")
        reprec.assertoutcome(passed=1)

    def test_funcarg(self, pytester: Pytester, scope, ok, error) -> None:
        pytester.makepyfile(
            """
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
        """
            % (scope, ok.split(), error.split())
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)


class TestErrors:
    def test_subfactory_missing_funcarg(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture()
            def gen(qwe123):
                return 1
            def test_something(gen):
                pass
        """
        )
        result = pytester.runpytest()
        assert result.ret != 0
        result.stdout.fnmatch_lines(
            ["*def gen(qwe123):*", "*fixture*qwe123*not found*", "*1 error*"]
        )

    def test_issue498_fixture_finalizer_failing(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
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
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            """
            *ERROR*teardown*test_1*
            *KeyError*
            *ERROR*teardown*test_2*
            *KeyError*
            *3 pass*2 errors*
        """
        )

    def test_setupfunc_missing_funcarg(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(autouse=True)
            def gen(qwe123):
                return 1
            def test_something():
                pass
        """
        )
        result = pytester.runpytest()
        assert result.ret != 0
        result.stdout.fnmatch_lines(
            ["*def gen(qwe123):*", "*fixture*qwe123*not found*", "*1 error*"]
        )


class TestShowFixtures:
    def test_funcarg_compat(self, pytester: Pytester) -> None:
        config = pytester.parseconfigure("--funcargs")
        assert config.option.showfixtures

    def test_show_fixtures(self, pytester: Pytester) -> None:
        result = pytester.runpytest("--fixtures")
        result.stdout.fnmatch_lines(
            [
                "tmp_path_factory [[]session scope[]] -- .../_pytest/tmpdir.py:*",
                "*for the test session*",
                "tmp_path -- .../_pytest/tmpdir.py:*",
                "*temporary directory*",
            ]
        )

    def test_show_fixtures_verbose(self, pytester: Pytester) -> None:
        result = pytester.runpytest("--fixtures", "-v")
        result.stdout.fnmatch_lines(
            [
                "tmp_path_factory [[]session scope[]] -- .../_pytest/tmpdir.py:*",
                "*for the test session*",
                "tmp_path -- .../_pytest/tmpdir.py:*",
                "*temporary directory*",
            ]
        )

    def test_show_fixtures_testmodule(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            '''
            import pytest
            @pytest.fixture
            def _arg0():
                """ hidden """
            @pytest.fixture
            def arg1():
                """  hello world """
        '''
        )
        result = pytester.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines(
            """
            *tmp_path -- *
            *fixtures defined from*
            *arg1 -- test_show_fixtures_testmodule.py:6*
            *hello world*
        """
        )
        result.stdout.no_fnmatch_line("*arg0*")

    @pytest.mark.parametrize("testmod", [True, False])
    def test_show_fixtures_conftest(self, pytester: Pytester, testmod) -> None:
        pytester.makeconftest(
            '''
            import pytest
            @pytest.fixture
            def arg1():
                """  hello world """
        '''
        )
        if testmod:
            pytester.makepyfile(
                """
                def test_hello():
                    pass
            """
            )
        result = pytester.runpytest("--fixtures")
        result.stdout.fnmatch_lines(
            """
            *tmp_path*
            *fixtures defined from*conftest*
            *arg1*
            *hello world*
        """
        )

    def test_show_fixtures_trimmed_doc(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            textwrap.dedent(
                '''\
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
                '''
            )
        )
        result = pytester.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines(
            textwrap.dedent(
                """\
                * fixtures defined from test_show_fixtures_trimmed_doc *
                arg2 -- test_show_fixtures_trimmed_doc.py:10
                    line1
                    line2
                arg1 -- test_show_fixtures_trimmed_doc.py:3
                    line1
                    line2
                """
            )
        )

    def test_show_fixtures_indented_doc(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            textwrap.dedent(
                '''\
                import pytest
                @pytest.fixture
                def fixture1():
                    """
                    line1
                        indented line
                    """
                '''
            )
        )
        result = pytester.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines(
            textwrap.dedent(
                """\
                * fixtures defined from test_show_fixtures_indented_doc *
                fixture1 -- test_show_fixtures_indented_doc.py:3
                    line1
                        indented line
                """
            )
        )

    def test_show_fixtures_indented_doc_first_line_unindented(
        self, pytester: Pytester
    ) -> None:
        p = pytester.makepyfile(
            textwrap.dedent(
                '''\
                import pytest
                @pytest.fixture
                def fixture1():
                    """line1
                    line2
                        indented line
                    """
                '''
            )
        )
        result = pytester.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines(
            textwrap.dedent(
                """\
                * fixtures defined from test_show_fixtures_indented_doc_first_line_unindented *
                fixture1 -- test_show_fixtures_indented_doc_first_line_unindented.py:3
                    line1
                    line2
                        indented line
                """
            )
        )

    def test_show_fixtures_indented_in_class(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            textwrap.dedent(
                '''\
                import pytest
                class TestClass(object):
                    @pytest.fixture
                    def fixture1(self):
                        """line1
                        line2
                            indented line
                        """
                '''
            )
        )
        result = pytester.runpytest("--fixtures", p)
        result.stdout.fnmatch_lines(
            textwrap.dedent(
                """\
                * fixtures defined from test_show_fixtures_indented_in_class *
                fixture1 -- test_show_fixtures_indented_in_class.py:4
                    line1
                    line2
                        indented line
                """
            )
        )

    def test_show_fixtures_different_files(self, pytester: Pytester) -> None:
        """`--fixtures` only shows fixtures from first file (#833)."""
        pytester.makepyfile(
            test_a='''
            import pytest

            @pytest.fixture
            def fix_a():
                """Fixture A"""
                pass

            def test_a(fix_a):
                pass
        '''
        )
        pytester.makepyfile(
            test_b='''
            import pytest

            @pytest.fixture
            def fix_b():
                """Fixture B"""
                pass

            def test_b(fix_b):
                pass
        '''
        )
        result = pytester.runpytest("--fixtures")
        result.stdout.fnmatch_lines(
            """
            * fixtures defined from test_a *
            fix_a -- test_a.py:4
                Fixture A

            * fixtures defined from test_b *
            fix_b -- test_b.py:4
                Fixture B
        """
        )

    def test_show_fixtures_with_same_name(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            '''
            import pytest
            @pytest.fixture
            def arg1():
                """Hello World in conftest.py"""
                return "Hello World"
        '''
        )
        pytester.makepyfile(
            """
            def test_foo(arg1):
                assert arg1 == "Hello World"
        """
        )
        pytester.makepyfile(
            '''
            import pytest
            @pytest.fixture
            def arg1():
                """Hi from test module"""
                return "Hi"
            def test_bar(arg1):
                assert arg1 == "Hi"
        '''
        )
        result = pytester.runpytest("--fixtures")
        result.stdout.fnmatch_lines(
            """
            * fixtures defined from conftest *
            arg1 -- conftest.py:3
                Hello World in conftest.py

            * fixtures defined from test_show_fixtures_with_same_name *
            arg1 -- test_show_fixtures_with_same_name.py:3
                Hi from test module
        """
        )

    def test_fixture_disallow_twice(self):
        """Test that applying @pytest.fixture twice generates an error (#2334)."""
        with pytest.raises(ValueError):

            @pytest.fixture
            @pytest.fixture
            def foo():
                raise NotImplementedError()


class TestContextManagerFixtureFuncs:
    def test_simple(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture
            def arg1():
                print("setup")
                yield 1
                print("teardown")
            def test_1(arg1):
                print("test1", arg1)
            def test_2(arg1):
                print("test2", arg1)
                assert 0
        """
        )
        result = pytester.runpytest("-s")
        result.stdout.fnmatch_lines(
            """
            *setup*
            *test1 1*
            *teardown*
            *setup*
            *test2 1*
            *teardown*
        """
        )

    def test_scoped(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="module")
            def arg1():
                print("setup")
                yield 1
                print("teardown")
            def test_1(arg1):
                print("test1", arg1)
            def test_2(arg1):
                print("test2", arg1)
        """
        )
        result = pytester.runpytest("-s")
        result.stdout.fnmatch_lines(
            """
            *setup*
            *test1 1*
            *test2 1*
            *teardown*
        """
        )

    def test_setup_exception(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="module")
            def arg1():
                pytest.fail("setup")
                yield 1
            def test_1(arg1):
                pass
        """
        )
        result = pytester.runpytest("-s")
        result.stdout.fnmatch_lines(
            """
            *pytest.fail*setup*
            *1 error*
        """
        )

    def test_teardown_exception(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="module")
            def arg1():
                yield 1
                pytest.fail("teardown")
            def test_1(arg1):
                pass
        """
        )
        result = pytester.runpytest("-s")
        result.stdout.fnmatch_lines(
            """
            *pytest.fail*teardown*
            *1 passed*1 error*
        """
        )

    def test_yields_more_than_one(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="module")
            def arg1():
                yield 1
                yield 2
            def test_1(arg1):
                pass
        """
        )
        result = pytester.runpytest("-s")
        result.stdout.fnmatch_lines(
            """
            *fixture function*
            *test_yields*:2*
        """
        )

    def test_custom_name(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(name='meow')
            def arg1():
                return 'mew'
            def test_1(meow):
                print(meow)
        """
        )
        result = pytester.runpytest("-s")
        result.stdout.fnmatch_lines(["*mew*"])


class TestParameterizedSubRequest:
    def test_call_from_fixture(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            test_call_from_fixture="""
            import pytest

            @pytest.fixture(params=[0, 1, 2])
            def fix_with_param(request):
                return request.param

            @pytest.fixture
            def get_named_fixture(request):
                return request.getfixturevalue('fix_with_param')

            def test_foo(request, get_named_fixture):
                pass
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "The requested fixture has no parameter defined for test:",
                "    test_call_from_fixture.py::test_foo",
                "Requested fixture 'fix_with_param' defined in:",
                "test_call_from_fixture.py:4",
                "Requested here:",
                "test_call_from_fixture.py:9",
                "*1 error in*",
            ]
        )

    def test_call_from_test(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            test_call_from_test="""
            import pytest

            @pytest.fixture(params=[0, 1, 2])
            def fix_with_param(request):
                return request.param

            def test_foo(request):
                request.getfixturevalue('fix_with_param')
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "The requested fixture has no parameter defined for test:",
                "    test_call_from_test.py::test_foo",
                "Requested fixture 'fix_with_param' defined in:",
                "test_call_from_test.py:4",
                "Requested here:",
                "test_call_from_test.py:8",
                "*1 failed*",
            ]
        )

    def test_external_fixture(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(params=[0, 1, 2])
            def fix_with_param(request):
                return request.param
            """
        )

        pytester.makepyfile(
            test_external_fixture="""
            def test_foo(request):
                request.getfixturevalue('fix_with_param')
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "The requested fixture has no parameter defined for test:",
                "    test_external_fixture.py::test_foo",
                "",
                "Requested fixture 'fix_with_param' defined in:",
                "conftest.py:4",
                "Requested here:",
                "test_external_fixture.py:2",
                "*1 failed*",
            ]
        )

    def test_non_relative_path(self, pytester: Pytester) -> None:
        tests_dir = pytester.mkdir("tests")
        fixdir = pytester.mkdir("fixtures")
        fixfile = fixdir.joinpath("fix.py")
        fixfile.write_text(
            textwrap.dedent(
                """\
                import pytest

                @pytest.fixture(params=[0, 1, 2])
                def fix_with_param(request):
                    return request.param
                """
            )
        )

        testfile = tests_dir.joinpath("test_foos.py")
        testfile.write_text(
            textwrap.dedent(
                """\
                from fix import fix_with_param

                def test_foo(request):
                    request.getfixturevalue('fix_with_param')
                """
            )
        )

        os.chdir(tests_dir)
        pytester.syspathinsert(fixdir)
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "The requested fixture has no parameter defined for test:",
                "    test_foos.py::test_foo",
                "",
                "Requested fixture 'fix_with_param' defined in:",
                f"{fixfile}:4",
                "Requested here:",
                "test_foos.py:4",
                "*1 failed*",
            ]
        )

        # With non-overlapping rootdir, passing tests_dir.
        rootdir = pytester.mkdir("rootdir")
        os.chdir(rootdir)
        result = pytester.runpytest("--rootdir", rootdir, tests_dir)
        result.stdout.fnmatch_lines(
            [
                "The requested fixture has no parameter defined for test:",
                "    test_foos.py::test_foo",
                "",
                "Requested fixture 'fix_with_param' defined in:",
                f"{fixfile}:4",
                "Requested here:",
                f"{testfile}:4",
                "*1 failed*",
            ]
        )


def test_pytest_fixture_setup_and_post_finalizer_hook(pytester: Pytester) -> None:
    pytester.makeconftest(
        """
        def pytest_fixture_setup(fixturedef, request):
            print('ROOT setup hook called for {0} from {1}'.format(fixturedef.argname, request.node.name))
        def pytest_fixture_post_finalizer(fixturedef, request):
            print('ROOT finalizer hook called for {0} from {1}'.format(fixturedef.argname, request.node.name))
    """
    )
    pytester.makepyfile(
        **{
            "tests/conftest.py": """
            def pytest_fixture_setup(fixturedef, request):
                print('TESTS setup hook called for {0} from {1}'.format(fixturedef.argname, request.node.name))
            def pytest_fixture_post_finalizer(fixturedef, request):
                print('TESTS finalizer hook called for {0} from {1}'.format(fixturedef.argname, request.node.name))
        """,
            "tests/test_hooks.py": """
            import pytest

            @pytest.fixture()
            def my_fixture():
                return 'some'

            def test_func(my_fixture):
                print('TEST test_func')
                assert my_fixture == 'some'
        """,
        }
    )
    result = pytester.runpytest("-s")
    assert result.ret == 0
    result.stdout.fnmatch_lines(
        [
            "*TESTS setup hook called for my_fixture from test_func*",
            "*ROOT setup hook called for my_fixture from test_func*",
            "*TEST test_func*",
            "*TESTS finalizer hook called for my_fixture from test_func*",
            "*ROOT finalizer hook called for my_fixture from test_func*",
        ]
    )


class TestScopeOrdering:
    """Class of tests that ensure fixtures are ordered based on their scopes (#2405)"""

    @pytest.mark.parametrize("variant", ["mark", "autouse"])
    def test_func_closure_module_auto(
        self, pytester: Pytester, variant, monkeypatch
    ) -> None:
        """Semantically identical to the example posted in #2405 when ``use_mark=True``"""
        monkeypatch.setenv("FIXTURE_ACTIVATION_VARIANT", variant)
        pytester.makepyfile(
            """
            import warnings
            import os
            import pytest
            VAR = 'FIXTURE_ACTIVATION_VARIANT'
            VALID_VARS = ('autouse', 'mark')

            VARIANT = os.environ.get(VAR)
            if VARIANT is None or VARIANT not in VALID_VARS:
                warnings.warn("{!r} is not  in {}, assuming autouse".format(VARIANT, VALID_VARS) )
                variant = 'mark'

            @pytest.fixture(scope='module', autouse=VARIANT == 'autouse')
            def m1(): pass

            if VARIANT=='mark':
                pytestmark = pytest.mark.usefixtures('m1')

            @pytest.fixture(scope='function', autouse=True)
            def f1(): pass

            def test_func(m1):
                pass
        """
        )
        items, _ = pytester.inline_genitems()
        request = FixtureRequest(items[0], _ispytest=True)
        assert request.fixturenames == "m1 f1".split()

    def test_func_closure_with_native_fixtures(
        self, pytester: Pytester, monkeypatch: MonkeyPatch
    ) -> None:
        """Sanity check that verifies the order returned by the closures and the actual fixture execution order:
        The execution order may differ because of fixture inter-dependencies.
        """
        monkeypatch.setattr(pytest, "FIXTURE_ORDER", [], raising=False)
        pytester.makepyfile(
            """
            import pytest

            FIXTURE_ORDER = pytest.FIXTURE_ORDER

            @pytest.fixture(scope="session")
            def s1():
                FIXTURE_ORDER.append('s1')

            @pytest.fixture(scope="package")
            def p1():
                FIXTURE_ORDER.append('p1')

            @pytest.fixture(scope="module")
            def m1():
                FIXTURE_ORDER.append('m1')

            @pytest.fixture(scope='session')
            def my_tmp_path_factory():
                FIXTURE_ORDER.append('my_tmp_path_factory')

            @pytest.fixture
            def my_tmp_path(my_tmp_path_factory):
                FIXTURE_ORDER.append('my_tmp_path')

            @pytest.fixture
            def f1(my_tmp_path):
                FIXTURE_ORDER.append('f1')

            @pytest.fixture
            def f2():
                FIXTURE_ORDER.append('f2')

            def test_foo(f1, p1, m1, f2, s1): pass
        """
        )
        items, _ = pytester.inline_genitems()
        request = FixtureRequest(items[0], _ispytest=True)
        # order of fixtures based on their scope and position in the parameter list
        assert (
            request.fixturenames
            == "s1 my_tmp_path_factory p1 m1 f1 f2 my_tmp_path".split()
        )
        pytester.runpytest()
        # actual fixture execution differs: dependent fixtures must be created first ("my_tmp_path")
        FIXTURE_ORDER = pytest.FIXTURE_ORDER  # type: ignore[attr-defined]
        assert FIXTURE_ORDER == "s1 my_tmp_path_factory p1 m1 my_tmp_path f1 f2".split()

    def test_func_closure_module(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(scope='module')
            def m1(): pass

            @pytest.fixture(scope='function')
            def f1(): pass

            def test_func(f1, m1):
                pass
        """
        )
        items, _ = pytester.inline_genitems()
        request = FixtureRequest(items[0], _ispytest=True)
        assert request.fixturenames == "m1 f1".split()

    def test_func_closure_scopes_reordered(self, pytester: Pytester) -> None:
        """Test ensures that fixtures are ordered by scope regardless of the order of the parameters, although
        fixtures of same scope keep the declared order
        """
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(scope='session')
            def s1(): pass

            @pytest.fixture(scope='module')
            def m1(): pass

            @pytest.fixture(scope='function')
            def f1(): pass

            @pytest.fixture(scope='function')
            def f2(): pass

            class Test:

                @pytest.fixture(scope='class')
                def c1(cls): pass

                def test_func(self, f2, f1, c1, m1, s1):
                    pass
        """
        )
        items, _ = pytester.inline_genitems()
        request = FixtureRequest(items[0], _ispytest=True)
        assert request.fixturenames == "s1 m1 c1 f2 f1".split()

    def test_func_closure_same_scope_closer_root_first(
        self, pytester: Pytester
    ) -> None:
        """Auto-use fixtures of same scope are ordered by closer-to-root first"""
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(scope='module', autouse=True)
            def m_conf(): pass
        """
        )
        pytester.makepyfile(
            **{
                "sub/conftest.py": """
                import pytest

                @pytest.fixture(scope='package', autouse=True)
                def p_sub(): pass

                @pytest.fixture(scope='module', autouse=True)
                def m_sub(): pass
            """,
                "sub/__init__.py": "",
                "sub/test_func.py": """
                import pytest

                @pytest.fixture(scope='module', autouse=True)
                def m_test(): pass

                @pytest.fixture(scope='function')
                def f1(): pass

                def test_func(m_test, f1):
                    pass
        """,
            }
        )
        items, _ = pytester.inline_genitems()
        request = FixtureRequest(items[0], _ispytest=True)
        assert request.fixturenames == "p_sub m_conf m_sub m_test f1".split()

    def test_func_closure_all_scopes_complex(self, pytester: Pytester) -> None:
        """Complex test involving all scopes and mixing autouse with normal fixtures"""
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(scope='session')
            def s1(): pass

            @pytest.fixture(scope='package', autouse=True)
            def p1(): pass
        """
        )
        pytester.makepyfile(**{"__init__.py": ""})
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(scope='module', autouse=True)
            def m1(): pass

            @pytest.fixture(scope='module')
            def m2(s1): pass

            @pytest.fixture(scope='function')
            def f1(): pass

            @pytest.fixture(scope='function')
            def f2(): pass

            class Test:

                @pytest.fixture(scope='class', autouse=True)
                def c1(self):
                    pass

                def test_func(self, f2, f1, m2):
                    pass
        """
        )
        items, _ = pytester.inline_genitems()
        request = FixtureRequest(items[0], _ispytest=True)
        assert request.fixturenames == "s1 p1 m1 m2 c1 f2 f1".split()

    def test_multiple_packages(self, pytester: Pytester) -> None:
        """Complex test involving multiple package fixtures. Make sure teardowns
        are executed in order.
        .
        └── root
            ├── __init__.py
            ├── sub1
            │   ├── __init__.py
            │   ├── conftest.py
            │   └── test_1.py
            └── sub2
                ├── __init__.py
                ├── conftest.py
                └── test_2.py
        """
        root = pytester.mkdir("root")
        root.joinpath("__init__.py").write_text("values = []")
        sub1 = root.joinpath("sub1")
        sub1.mkdir()
        sub1.joinpath("__init__.py").touch()
        sub1.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
            import pytest
            from .. import values
            @pytest.fixture(scope="package")
            def fix():
                values.append("pre-sub1")
                yield values
                assert values.pop() == "pre-sub1"
        """
            )
        )
        sub1.joinpath("test_1.py").write_text(
            textwrap.dedent(
                """\
            from .. import values
            def test_1(fix):
                assert values == ["pre-sub1"]
        """
            )
        )
        sub2 = root.joinpath("sub2")
        sub2.mkdir()
        sub2.joinpath("__init__.py").touch()
        sub2.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
            import pytest
            from .. import values
            @pytest.fixture(scope="package")
            def fix():
                values.append("pre-sub2")
                yield values
                assert values.pop() == "pre-sub2"
        """
            )
        )
        sub2.joinpath("test_2.py").write_text(
            textwrap.dedent(
                """\
            from .. import values
            def test_2(fix):
                assert values == ["pre-sub2"]
        """
            )
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_class_fixture_self_instance(self, pytester: Pytester) -> None:
        """Check that plugin classes which implement fixtures receive the plugin instance
        as self (see #2270).
        """
        pytester.makeconftest(
            """
            import pytest

            def pytest_configure(config):
                config.pluginmanager.register(MyPlugin())

            class MyPlugin():
                def __init__(self):
                    self.arg = 1

                @pytest.fixture(scope='function')
                def myfix(self):
                    assert isinstance(self, MyPlugin)
                    return self.arg
        """
        )

        pytester.makepyfile(
            """
            class TestClass(object):
                def test_1(self, myfix):
                    assert myfix == 1
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)


def test_call_fixture_function_error():
    """Check if an error is raised if a fixture function is called directly (#4545)"""

    @pytest.fixture
    def fix():
        raise NotImplementedError()

    with pytest.raises(pytest.fail.Exception):
        assert fix() == 1


def test_fixture_param_shadowing(pytester: Pytester) -> None:
    """Parametrized arguments would be shadowed if a fixture with the same name also exists (#5036)"""
    pytester.makepyfile(
        """
        import pytest

        @pytest.fixture(params=['a', 'b'])
        def argroot(request):
            return request.param

        @pytest.fixture
        def arg(argroot):
            return argroot

        # This should only be parametrized directly
        @pytest.mark.parametrize("arg", [1])
        def test_direct(arg):
            assert arg == 1

        # This should be parametrized based on the fixtures
        def test_normal_fixture(arg):
            assert isinstance(arg, str)

        # Indirect should still work:

        @pytest.fixture
        def arg2(request):
            return 2*request.param

        @pytest.mark.parametrize("arg2", [1], indirect=True)
        def test_indirect(arg2):
            assert arg2 == 2
    """
    )
    # Only one test should have run
    result = pytester.runpytest("-v")
    result.assert_outcomes(passed=4)
    result.stdout.fnmatch_lines(["*::test_direct[[]1[]]*"])
    result.stdout.fnmatch_lines(["*::test_normal_fixture[[]a[]]*"])
    result.stdout.fnmatch_lines(["*::test_normal_fixture[[]b[]]*"])
    result.stdout.fnmatch_lines(["*::test_indirect[[]1[]]*"])


def test_fixture_named_request(pytester: Pytester) -> None:
    pytester.copy_example("fixtures/test_fixture_named_request.py")
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*'request' is a reserved word for fixtures, use another name:",
            "  *test_fixture_named_request.py:5",
        ]
    )


def test_indirect_fixture_does_not_break_scope(pytester: Pytester) -> None:
    """Ensure that fixture scope is respected when using indirect fixtures (#570)"""
    pytester.makepyfile(
        """
        import pytest
        instantiated  = []

        @pytest.fixture(scope="session")
        def fixture_1(request):
            instantiated.append(("fixture_1", request.param))


        @pytest.fixture(scope="session")
        def fixture_2(request):
            instantiated.append(("fixture_2", request.param))


        scenarios = [
            ("A", "a1"),
            ("A", "a2"),
            ("B", "b1"),
            ("B", "b2"),
            ("C", "c1"),
            ("C", "c2"),
        ]

        @pytest.mark.parametrize(
            "fixture_1,fixture_2", scenarios, indirect=["fixture_1", "fixture_2"]
        )
        def test_create_fixtures(fixture_1, fixture_2):
            pass


        def test_check_fixture_instantiations():
            assert instantiated == [
                ('fixture_1', 'A'),
                ('fixture_2', 'a1'),
                ('fixture_2', 'a2'),
                ('fixture_1', 'B'),
                ('fixture_2', 'b1'),
                ('fixture_2', 'b2'),
                ('fixture_1', 'C'),
                ('fixture_2', 'c1'),
                ('fixture_2', 'c2'),
            ]
    """
    )
    result = pytester.runpytest()
    result.assert_outcomes(passed=7)


def test_fixture_parametrization_nparray(pytester: Pytester) -> None:
    pytest.importorskip("numpy")

    pytester.makepyfile(
        """
        from numpy import linspace
        from pytest import fixture

        @fixture(params=linspace(1, 10, 10))
        def value(request):
            return request.param

        def test_bug(value):
            assert value == value
    """
    )
    result = pytester.runpytest()
    result.assert_outcomes(passed=10)


def test_fixture_arg_ordering(pytester: Pytester) -> None:
    """
    This test describes how fixtures in the same scope but without explicit dependencies
    between them are created. While users should make dependencies explicit, often
    they rely on this order, so this test exists to catch regressions in this regard.
    See #6540 and #6492.
    """
    p1 = pytester.makepyfile(
        """
        import pytest

        suffixes = []

        @pytest.fixture
        def fix_1(): suffixes.append("fix_1")
        @pytest.fixture
        def fix_2(): suffixes.append("fix_2")
        @pytest.fixture
        def fix_3(): suffixes.append("fix_3")
        @pytest.fixture
        def fix_4(): suffixes.append("fix_4")
        @pytest.fixture
        def fix_5(): suffixes.append("fix_5")

        @pytest.fixture
        def fix_combined(fix_1, fix_2, fix_3, fix_4, fix_5): pass

        def test_suffix(fix_combined):
            assert suffixes == ["fix_1", "fix_2", "fix_3", "fix_4", "fix_5"]
        """
    )
    result = pytester.runpytest("-vv", str(p1))
    assert result.ret == 0


def test_yield_fixture_with_no_value(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest
        @pytest.fixture(name='custom')
        def empty_yield():
            if False:
                yield

        def test_fixt(custom):
            pass
        """
    )
    expected = "E               ValueError: custom did not yield a value"
    result = pytester.runpytest()
    result.assert_outcomes(errors=1)
    result.stdout.fnmatch_lines([expected])
    assert result.ret == ExitCode.TESTS_FAILED
