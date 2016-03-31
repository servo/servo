# -*- coding: utf-8 -*-
import sys
from textwrap import dedent

import _pytest._code
import py
import pytest
from _pytest.main import EXIT_NOTESTSCOLLECTED


class TestModule:
    def test_failing_import(self, testdir):
        modcol = testdir.getmodulecol("import alksdjalskdjalkjals")
        pytest.raises(ImportError, modcol.collect)
        pytest.raises(ImportError, modcol.collect)

    def test_import_duplicate(self, testdir):
        a = testdir.mkdir("a")
        b = testdir.mkdir("b")
        p = a.ensure("test_whatever.py")
        p.pyimport()
        del py.std.sys.modules['test_whatever']
        b.ensure("test_whatever.py")
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*import*mismatch*",
            "*imported*test_whatever*",
            "*%s*" % a.join("test_whatever.py"),
            "*not the same*",
            "*%s*" % b.join("test_whatever.py"),
            "*HINT*",
        ])

    def test_import_prepend_append(self, testdir, monkeypatch):
        syspath = list(sys.path)
        monkeypatch.setattr(sys, "path", syspath)
        root1 = testdir.mkdir("root1")
        root2 = testdir.mkdir("root2")
        root1.ensure("x456.py")
        root2.ensure("x456.py")
        p = root2.join("test_x456.py")
        monkeypatch.syspath_prepend(str(root1))
        p.write(dedent("""\
            import x456
            def test():
                assert x456.__file__.startswith(%r)
        """ % str(root2)))
        with root2.as_cwd():
            reprec = testdir.inline_run("--import-mode=append")
            reprec.assertoutcome(passed=0, failed=1)
            reprec = testdir.inline_run()
            reprec.assertoutcome(passed=1)

    def test_syntax_error_in_module(self, testdir):
        modcol = testdir.getmodulecol("this is a syntax error")
        pytest.raises(modcol.CollectError, modcol.collect)
        pytest.raises(modcol.CollectError, modcol.collect)

    def test_module_considers_pluginmanager_at_import(self, testdir):
        modcol = testdir.getmodulecol("pytest_plugins='xasdlkj',")
        pytest.raises(ImportError, lambda: modcol.obj)

class TestClass:
    def test_class_with_init_warning(self, testdir):
        testdir.makepyfile("""
            class TestClass1:
                def __init__(self):
                    pass
        """)
        result = testdir.runpytest("-rw")
        result.stdout.fnmatch_lines_random("""
            WC1*test_class_with_init_warning.py*__init__*
        """)

    def test_class_subclassobject(self, testdir):
        testdir.getmodulecol("""
            class test(object):
                pass
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*collected 0*",
        ])

    def test_setup_teardown_class_as_classmethod(self, testdir):
        testdir.makepyfile(test_mod1="""
            class TestClassMethod:
                @classmethod
                def setup_class(cls):
                    pass
                def test_1(self):
                    pass
                @classmethod
                def teardown_class(cls):
                    pass
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*1 passed*",
        ])

    def test_issue1035_obj_has_getattr(self, testdir):
        modcol = testdir.getmodulecol("""
            class Chameleon(object):
                def __getattr__(self, name):
                    return True
            chameleon = Chameleon()
        """)
        colitems = modcol.collect()
        assert len(colitems) == 0


class TestGenerator:
    def test_generative_functions(self, testdir):
        modcol = testdir.getmodulecol("""
            def func1(arg, arg2):
                assert arg == arg2

            def test_gen():
                yield func1, 17, 3*5
                yield func1, 42, 6*7
        """)
        colitems = modcol.collect()
        assert len(colitems) == 1
        gencol = colitems[0]
        assert isinstance(gencol, pytest.Generator)
        gencolitems = gencol.collect()
        assert len(gencolitems) == 2
        assert isinstance(gencolitems[0], pytest.Function)
        assert isinstance(gencolitems[1], pytest.Function)
        assert gencolitems[0].name == '[0]'
        assert gencolitems[0].obj.__name__ == 'func1'

    def test_generative_methods(self, testdir):
        modcol = testdir.getmodulecol("""
            def func1(arg, arg2):
                assert arg == arg2
            class TestGenMethods:
                def test_gen(self):
                    yield func1, 17, 3*5
                    yield func1, 42, 6*7
        """)
        gencol = modcol.collect()[0].collect()[0].collect()[0]
        assert isinstance(gencol, pytest.Generator)
        gencolitems = gencol.collect()
        assert len(gencolitems) == 2
        assert isinstance(gencolitems[0], pytest.Function)
        assert isinstance(gencolitems[1], pytest.Function)
        assert gencolitems[0].name == '[0]'
        assert gencolitems[0].obj.__name__ == 'func1'

    def test_generative_functions_with_explicit_names(self, testdir):
        modcol = testdir.getmodulecol("""
            def func1(arg, arg2):
                assert arg == arg2

            def test_gen():
                yield "seventeen", func1, 17, 3*5
                yield "fortytwo", func1, 42, 6*7
        """)
        colitems = modcol.collect()
        assert len(colitems) == 1
        gencol = colitems[0]
        assert isinstance(gencol, pytest.Generator)
        gencolitems = gencol.collect()
        assert len(gencolitems) == 2
        assert isinstance(gencolitems[0], pytest.Function)
        assert isinstance(gencolitems[1], pytest.Function)
        assert gencolitems[0].name == "['seventeen']"
        assert gencolitems[0].obj.__name__ == 'func1'
        assert gencolitems[1].name == "['fortytwo']"
        assert gencolitems[1].obj.__name__ == 'func1'

    def test_generative_functions_unique_explicit_names(self, testdir):
        # generative
        modcol = testdir.getmodulecol("""
            def func(): pass
            def test_gen():
                yield "name", func
                yield "name", func
        """)
        colitems = modcol.collect()
        assert len(colitems) == 1
        gencol = colitems[0]
        assert isinstance(gencol, pytest.Generator)
        pytest.raises(ValueError, "gencol.collect()")

    def test_generative_methods_with_explicit_names(self, testdir):
        modcol = testdir.getmodulecol("""
            def func1(arg, arg2):
                assert arg == arg2
            class TestGenMethods:
                def test_gen(self):
                    yield "m1", func1, 17, 3*5
                    yield "m2", func1, 42, 6*7
        """)
        gencol = modcol.collect()[0].collect()[0].collect()[0]
        assert isinstance(gencol, pytest.Generator)
        gencolitems = gencol.collect()
        assert len(gencolitems) == 2
        assert isinstance(gencolitems[0], pytest.Function)
        assert isinstance(gencolitems[1], pytest.Function)
        assert gencolitems[0].name == "['m1']"
        assert gencolitems[0].obj.__name__ == 'func1'
        assert gencolitems[1].name == "['m2']"
        assert gencolitems[1].obj.__name__ == 'func1'

    def test_order_of_execution_generator_same_codeline(self, testdir, tmpdir):
        o = testdir.makepyfile("""
            def test_generative_order_of_execution():
                import py, pytest
                test_list = []
                expected_list = list(range(6))

                def list_append(item):
                    test_list.append(item)

                def assert_order_of_execution():
                    py.builtin.print_('expected order', expected_list)
                    py.builtin.print_('but got       ', test_list)
                    assert test_list == expected_list

                for i in expected_list:
                    yield list_append, i
                yield assert_order_of_execution
        """)
        reprec = testdir.inline_run(o)
        passed, skipped, failed = reprec.countoutcomes()
        assert passed == 7
        assert not skipped and not failed

    def test_order_of_execution_generator_different_codeline(self, testdir):
        o = testdir.makepyfile("""
            def test_generative_tests_different_codeline():
                import py, pytest
                test_list = []
                expected_list = list(range(3))

                def list_append_2():
                    test_list.append(2)

                def list_append_1():
                    test_list.append(1)

                def list_append_0():
                    test_list.append(0)

                def assert_order_of_execution():
                    py.builtin.print_('expected order', expected_list)
                    py.builtin.print_('but got       ', test_list)
                    assert test_list == expected_list

                yield list_append_0
                yield list_append_1
                yield list_append_2
                yield assert_order_of_execution
        """)
        reprec = testdir.inline_run(o)
        passed, skipped, failed = reprec.countoutcomes()
        assert passed == 4
        assert not skipped and not failed

    def test_setupstate_is_preserved_134(self, testdir):
        # yield-based tests are messy wrt to setupstate because
        # during collection they already invoke setup functions
        # and then again when they are run.  For now, we want to make sure
        # that the old 1.3.4 behaviour is preserved such that all
        # yielded functions all share the same "self" instance that
        # has been used during collection.
        o = testdir.makepyfile("""
            setuplist = []
            class TestClass:
                def setup_method(self, func):
                    #print "setup_method", self, func
                    setuplist.append(self)
                    self.init = 42

                def teardown_method(self, func):
                    self.init = None

                def test_func1(self):
                    pass

                def test_func2(self):
                    yield self.func2
                    yield self.func2

                def func2(self):
                    assert self.init

            def test_setuplist():
                # once for test_func2 during collection
                # once for test_func1 during test run
                # once for test_func2 during test run
                #print setuplist
                assert len(setuplist) == 3, len(setuplist)
                assert setuplist[0] == setuplist[2], setuplist
                assert setuplist[1] != setuplist[2], setuplist
        """)
        reprec = testdir.inline_run(o, '-v')
        passed, skipped, failed = reprec.countoutcomes()
        assert passed == 4
        assert not skipped and not failed


class TestFunction:
    def test_getmodulecollector(self, testdir):
        item = testdir.getitem("def test_func(): pass")
        modcol = item.getparent(pytest.Module)
        assert isinstance(modcol, pytest.Module)
        assert hasattr(modcol.obj, 'test_func')

    def test_function_as_object_instance_ignored(self, testdir):
        testdir.makepyfile("""
            class A:
                def __call__(self, tmpdir):
                    0/0

            test_a = A()
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome()

    def test_function_equality(self, testdir, tmpdir):
        from _pytest.python import FixtureManager
        config = testdir.parseconfigure()
        session = testdir.Session(config)
        session._fixturemanager = FixtureManager(session)
        def func1():
            pass
        def func2():
            pass
        f1 = pytest.Function(name="name", parent=session, config=config,
                args=(1,), callobj=func1)
        assert f1 == f1
        f2 = pytest.Function(name="name",config=config,
                callobj=func2, parent=session)
        assert f1 != f2

    def test_issue197_parametrize_emptyset(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.mark.parametrize('arg', [])
            def test_function(arg):
                pass
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(skipped=1)

    def test_single_tuple_unwraps_values(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.mark.parametrize(('arg',), [(1,)])
            def test_function(arg):
                assert arg == 1
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_issue213_parametrize_value_no_equal(self, testdir):
        testdir.makepyfile("""
            import pytest
            class A:
                def __eq__(self, other):
                    raise ValueError("not possible")
            @pytest.mark.parametrize('arg', [A()])
            def test_function(arg):
                assert arg.__class__.__name__ == "A"
        """)
        reprec = testdir.inline_run("--fulltrace")
        reprec.assertoutcome(passed=1)

    def test_parametrize_with_non_hashable_values(self, testdir):
        """Test parametrization with non-hashable values."""
        testdir.makepyfile("""
            archival_mapping = {
                '1.0': {'tag': '1.0'},
                '1.2.2a1': {'tag': 'release-1.2.2a1'},
            }

            import pytest
            @pytest.mark.parametrize('key value'.split(),
                                     archival_mapping.items())
            def test_archival_to_version(key, value):
                assert key in archival_mapping
                assert value == archival_mapping[key]
        """)
        rec = testdir.inline_run()
        rec.assertoutcome(passed=2)


    def test_parametrize_with_non_hashable_values_indirect(self, testdir):
        """Test parametrization with non-hashable values with indirect parametrization."""
        testdir.makepyfile("""
            archival_mapping = {
                '1.0': {'tag': '1.0'},
                '1.2.2a1': {'tag': 'release-1.2.2a1'},
            }

            import pytest

            @pytest.fixture
            def key(request):
                return request.param

            @pytest.fixture
            def value(request):
                return request.param

            @pytest.mark.parametrize('key value'.split(),
                                     archival_mapping.items(), indirect=True)
            def test_archival_to_version(key, value):
                assert key in archival_mapping
                assert value == archival_mapping[key]
        """)
        rec = testdir.inline_run()
        rec.assertoutcome(passed=2)


    def test_parametrize_overrides_fixture(self, testdir):
        """Test parametrization when parameter overrides existing fixture with same name."""
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def value():
                return 'value'

            @pytest.mark.parametrize('value',
                                     ['overridden'])
            def test_overridden_via_param(value):
                assert value == 'overridden'

            @pytest.mark.parametrize('somevalue', ['overridden'])
            def test_not_overridden(value, somevalue):
                assert value == 'value'
                assert somevalue == 'overridden'

            @pytest.mark.parametrize('other,value', [('foo', 'overridden')])
            def test_overridden_via_multiparam(other, value):
                assert other == 'foo'
                assert value == 'overridden'
        """)
        rec = testdir.inline_run()
        rec.assertoutcome(passed=3)


    def test_parametrize_overrides_parametrized_fixture(self, testdir):
        """Test parametrization when parameter overrides existing parametrized fixture with same name."""
        testdir.makepyfile("""
            import pytest

            @pytest.fixture(params=[1, 2])
            def value(request):
                return request.param

            @pytest.mark.parametrize('value',
                                     ['overridden'])
            def test_overridden_via_param(value):
                assert value == 'overridden'
        """)
        rec = testdir.inline_run()
        rec.assertoutcome(passed=1)

    def test_parametrize_with_mark(selfself, testdir):
        items = testdir.getitems("""
            import pytest
            @pytest.mark.foo
            @pytest.mark.parametrize('arg', [
                1,
                pytest.mark.bar(pytest.mark.baz(2))
            ])
            def test_function(arg):
                pass
        """)
        keywords = [item.keywords for item in items]
        assert 'foo' in keywords[0] and 'bar' not in keywords[0] and 'baz' not in keywords[0]
        assert 'foo' in keywords[1] and 'bar' in keywords[1] and 'baz' in keywords[1]

    def test_function_equality_with_callspec(self, testdir, tmpdir):
        items = testdir.getitems("""
            import pytest
            @pytest.mark.parametrize('arg', [1,2])
            def test_function(arg):
                pass
        """)
        assert items[0] != items[1]
        assert not (items[0] == items[1])

    def test_pyfunc_call(self, testdir):
        item = testdir.getitem("def test_func(): raise ValueError")
        config = item.config
        class MyPlugin1:
            def pytest_pyfunc_call(self, pyfuncitem):
                raise ValueError
        class MyPlugin2:
            def pytest_pyfunc_call(self, pyfuncitem):
                return True
        config.pluginmanager.register(MyPlugin1())
        config.pluginmanager.register(MyPlugin2())
        config.hook.pytest_runtest_setup(item=item)
        config.hook.pytest_pyfunc_call(pyfuncitem=item)

    def test_multiple_parametrize(self, testdir):
        modcol = testdir.getmodulecol("""
            import pytest
            @pytest.mark.parametrize('x', [0, 1])
            @pytest.mark.parametrize('y', [2, 3])
            def test1(x, y):
                pass
        """)
        colitems = modcol.collect()
        assert colitems[0].name == 'test1[2-0]'
        assert colitems[1].name == 'test1[2-1]'
        assert colitems[2].name == 'test1[3-0]'
        assert colitems[3].name == 'test1[3-1]'

    def test_issue751_multiple_parametrize_with_ids(self, testdir):
        modcol = testdir.getmodulecol("""
            import pytest
            @pytest.mark.parametrize('x', [0], ids=['c'])
            @pytest.mark.parametrize('y', [0, 1], ids=['a', 'b'])
            class Test(object):
                def test1(self, x, y):
                    pass
                def test2(self, x, y):
                    pass
        """)
        colitems = modcol.collect()[0].collect()[0].collect()
        assert colitems[0].name == 'test1[a-c]'
        assert colitems[1].name == 'test1[b-c]'
        assert colitems[2].name == 'test2[a-c]'
        assert colitems[3].name == 'test2[b-c]'

    def test_parametrize_skipif(self, testdir):
        testdir.makepyfile("""
            import pytest

            m = pytest.mark.skipif('True')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_skip_if(x):
                assert x < 2
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines('* 2 passed, 1 skipped in *')

    def test_parametrize_skip(self, testdir):
        testdir.makepyfile("""
            import pytest

            m = pytest.mark.skip('')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_skip(x):
                assert x < 2
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines('* 2 passed, 1 skipped in *')

    def test_parametrize_skipif_no_skip(self, testdir):
        testdir.makepyfile("""
            import pytest

            m = pytest.mark.skipif('False')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_skipif_no_skip(x):
                assert x < 2
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines('* 1 failed, 2 passed in *')

    def test_parametrize_xfail(self, testdir):
        testdir.makepyfile("""
            import pytest

            m = pytest.mark.xfail('True')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_xfail(x):
                assert x < 2
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines('* 2 passed, 1 xfailed in *')

    def test_parametrize_passed(self, testdir):
        testdir.makepyfile("""
            import pytest

            m = pytest.mark.xfail('True')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_xfail(x):
                pass
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines('* 2 passed, 1 xpassed in *')

    def test_parametrize_xfail_passed(self, testdir):
        testdir.makepyfile("""
            import pytest

            m = pytest.mark.xfail('False')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_passed(x):
                pass
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines('* 3 passed in *')


class TestSorting:
    def test_check_equality(self, testdir):
        modcol = testdir.getmodulecol("""
            def test_pass(): pass
            def test_fail(): assert 0
        """)
        fn1 = testdir.collect_by_name(modcol, "test_pass")
        assert isinstance(fn1, pytest.Function)
        fn2 = testdir.collect_by_name(modcol, "test_pass")
        assert isinstance(fn2, pytest.Function)

        assert fn1 == fn2
        assert fn1 != modcol
        if py.std.sys.version_info < (3, 0):
            assert cmp(fn1, fn2) == 0
        assert hash(fn1) == hash(fn2)

        fn3 = testdir.collect_by_name(modcol, "test_fail")
        assert isinstance(fn3, pytest.Function)
        assert not (fn1 == fn3)
        assert fn1 != fn3

        for fn in fn1,fn2,fn3:
            assert fn != 3
            assert fn != modcol
            assert fn != [1,2,3]
            assert [1,2,3] != fn
            assert modcol != fn

    def test_allow_sane_sorting_for_decorators(self, testdir):
        modcol = testdir.getmodulecol("""
            def dec(f):
                g = lambda: f(2)
                g.place_as = f
                return g


            def test_b(y):
                pass
            test_b = dec(test_b)

            def test_a(y):
                pass
            test_a = dec(test_a)
        """)
        colitems = modcol.collect()
        assert len(colitems) == 2
        assert [item.name for item in colitems] == ['test_b', 'test_a']


class TestConftestCustomization:
    def test_pytest_pycollect_module(self, testdir):
        testdir.makeconftest("""
            import pytest
            class MyModule(pytest.Module):
                pass
            def pytest_pycollect_makemodule(path, parent):
                if path.basename == "test_xyz.py":
                    return MyModule(path, parent)
        """)
        testdir.makepyfile("def test_some(): pass")
        testdir.makepyfile(test_xyz="def test_func(): pass")
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines([
            "*<Module*test_pytest*",
            "*<MyModule*xyz*",
        ])

    def test_customized_pymakemodule_issue205_subdir(self, testdir):
        b = testdir.mkdir("a").mkdir("b")
        b.join("conftest.py").write(_pytest._code.Source("""
            def pytest_pycollect_makemodule(__multicall__):
                mod = __multicall__.execute()
                mod.obj.hello = "world"
                return mod
        """))
        b.join("test_module.py").write(_pytest._code.Source("""
            def test_hello():
                assert hello == "world"
        """))
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_customized_pymakeitem(self, testdir):
        b = testdir.mkdir("a").mkdir("b")
        b.join("conftest.py").write(_pytest._code.Source("""
            import pytest
            @pytest.hookimpl(hookwrapper=True)
            def pytest_pycollect_makeitem():
                outcome = yield
                if outcome.excinfo is None:
                    result = outcome.result
                    if result:
                        for func in result:
                            func._some123 = "world"
        """))
        b.join("test_module.py").write(_pytest._code.Source("""
            import pytest

            @pytest.fixture()
            def obj(request):
                return request.node._some123
            def test_hello(obj):
                assert obj == "world"
        """))
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_pytest_pycollect_makeitem(self, testdir):
        testdir.makeconftest("""
            import pytest
            class MyFunction(pytest.Function):
                pass
            def pytest_pycollect_makeitem(collector, name, obj):
                if name == "some":
                    return MyFunction(name, collector)
        """)
        testdir.makepyfile("def some(): pass")
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines([
            "*MyFunction*some*",
        ])

    def test_makeitem_non_underscore(self, testdir, monkeypatch):
        modcol = testdir.getmodulecol("def _hello(): pass")
        l = []
        monkeypatch.setattr(pytest.Module, 'makeitem',
            lambda self, name, obj: l.append(name))
        l = modcol.collect()
        assert '_hello' not in l

def test_setup_only_available_in_subdir(testdir):
    sub1 = testdir.mkpydir("sub1")
    sub2 = testdir.mkpydir("sub2")
    sub1.join("conftest.py").write(_pytest._code.Source("""
        import pytest
        def pytest_runtest_setup(item):
            assert item.fspath.purebasename == "test_in_sub1"
        def pytest_runtest_call(item):
            assert item.fspath.purebasename == "test_in_sub1"
        def pytest_runtest_teardown(item):
            assert item.fspath.purebasename == "test_in_sub1"
    """))
    sub2.join("conftest.py").write(_pytest._code.Source("""
        import pytest
        def pytest_runtest_setup(item):
            assert item.fspath.purebasename == "test_in_sub2"
        def pytest_runtest_call(item):
            assert item.fspath.purebasename == "test_in_sub2"
        def pytest_runtest_teardown(item):
            assert item.fspath.purebasename == "test_in_sub2"
    """))
    sub1.join("test_in_sub1.py").write("def test_1(): pass")
    sub2.join("test_in_sub2.py").write("def test_2(): pass")
    result = testdir.runpytest("-v", "-s")
    result.assert_outcomes(passed=2)

def test_modulecol_roundtrip(testdir):
    modcol = testdir.getmodulecol("pass", withinit=True)
    trail = modcol.nodeid
    newcol = modcol.session.perform_collect([trail], genitems=0)[0]
    assert modcol.name == newcol.name


class TestTracebackCutting:
    def test_skip_simple(self):
        excinfo = pytest.raises(pytest.skip.Exception, 'pytest.skip("xxx")')
        assert excinfo.traceback[-1].frame.code.name == "skip"
        assert excinfo.traceback[-1].ishidden()

    def test_traceback_argsetup(self, testdir):
        testdir.makeconftest("""
            def pytest_funcarg__hello(request):
                raise ValueError("xyz")
        """)
        p = testdir.makepyfile("def test(hello): pass")
        result = testdir.runpytest(p)
        assert result.ret != 0
        out = result.stdout.str()
        assert out.find("xyz") != -1
        assert out.find("conftest.py:2: ValueError") != -1
        numentries = out.count("_ _ _") # separator for traceback entries
        assert numentries == 0

        result = testdir.runpytest("--fulltrace", p)
        out = result.stdout.str()
        assert out.find("conftest.py:2: ValueError") != -1
        numentries = out.count("_ _ _ _") # separator for traceback entries
        assert numentries > 3

    def test_traceback_error_during_import(self, testdir):
        testdir.makepyfile("""
            x = 1
            x = 2
            x = 17
            asd
        """)
        result = testdir.runpytest()
        assert result.ret != 0
        out = result.stdout.str()
        assert "x = 1" not in out
        assert "x = 2" not in out
        result.stdout.fnmatch_lines([
            " *asd*",
            "E*NameError*",
        ])
        result = testdir.runpytest("--fulltrace")
        out = result.stdout.str()
        assert "x = 1" in out
        assert "x = 2" in out
        result.stdout.fnmatch_lines([
            ">*asd*",
            "E*NameError*",
        ])

    def test_traceback_filter_error_during_fixture_collection(self, testdir):
        """integration test for issue #995.
        """
        testdir.makepyfile("""
            import pytest

            def fail_me(func):
                ns = {}
                exec('def w(): raise ValueError("fail me")', ns)
                return ns['w']

            @pytest.fixture(scope='class')
            @fail_me
            def fail_fixture():
                pass

            def test_failing_fixture(fail_fixture):
               pass
        """)
        result = testdir.runpytest()
        assert result.ret != 0
        out = result.stdout.str()
        assert "INTERNALERROR>" not in out
        result.stdout.fnmatch_lines([
            "*ValueError: fail me*",
            "* 1 error in *",
        ])

    def test_filter_traceback_generated_code(self):
        """test that filter_traceback() works with the fact that
        py.code.Code.path attribute might return an str object.
        In this case, one of the entries on the traceback was produced by
        dynamically generated code.
        See: https://bitbucket.org/pytest-dev/py/issues/71
        This fixes #995.
        """
        from _pytest.python import filter_traceback
        try:
            ns = {}
            exec('def foo(): raise ValueError', ns)
            ns['foo']()
        except ValueError:
            _, _, tb = sys.exc_info()

        tb = _pytest._code.Traceback(tb)
        assert isinstance(tb[-1].path, str)
        assert not filter_traceback(tb[-1])

    def test_filter_traceback_path_no_longer_valid(self, testdir):
        """test that filter_traceback() works with the fact that
        py.code.Code.path attribute might return an str object.
        In this case, one of the files in the traceback no longer exists.
        This fixes #1133.
        """
        from _pytest.python import filter_traceback
        testdir.syspathinsert()
        testdir.makepyfile(filter_traceback_entry_as_str='''
            def foo():
                raise ValueError
        ''')
        try:
            import filter_traceback_entry_as_str
            filter_traceback_entry_as_str.foo()
        except ValueError:
            _, _, tb = sys.exc_info()

        testdir.tmpdir.join('filter_traceback_entry_as_str.py').remove()
        tb = _pytest._code.Traceback(tb)
        assert isinstance(tb[-1].path, str)
        assert filter_traceback(tb[-1])


class TestReportInfo:
    def test_itemreport_reportinfo(self, testdir, linecomp):
        testdir.makeconftest("""
            import pytest
            class MyFunction(pytest.Function):
                def reportinfo(self):
                    return "ABCDE", 42, "custom"
            def pytest_pycollect_makeitem(collector, name, obj):
                if name == "test_func":
                    return MyFunction(name, parent=collector)
        """)
        item = testdir.getitem("def test_func(): pass")
        item.config.pluginmanager.getplugin("runner")
        assert item.location == ("ABCDE", 42, "custom")

    def test_func_reportinfo(self, testdir):
        item = testdir.getitem("def test_func(): pass")
        fspath, lineno, modpath = item.reportinfo()
        assert fspath == item.fspath
        assert lineno == 0
        assert modpath == "test_func"

    def test_class_reportinfo(self, testdir):
        modcol = testdir.getmodulecol("""
            # lineno 0
            class TestClass:
                def test_hello(self): pass
        """)
        classcol = testdir.collect_by_name(modcol, "TestClass")
        fspath, lineno, msg = classcol.reportinfo()
        assert fspath == modcol.fspath
        assert lineno == 1
        assert msg == "TestClass"

    def test_generator_reportinfo(self, testdir):
        modcol = testdir.getmodulecol("""
            # lineno 0
            def test_gen():
                def check(x):
                    assert x
                yield check, 3
        """)
        gencol = testdir.collect_by_name(modcol, "test_gen")
        fspath, lineno, modpath = gencol.reportinfo()
        assert fspath == modcol.fspath
        assert lineno == 1
        assert modpath == "test_gen"

        genitem = gencol.collect()[0]
        fspath, lineno, modpath = genitem.reportinfo()
        assert fspath == modcol.fspath
        assert lineno == 2
        assert modpath == "test_gen[0]"
        """
            def test_func():
                pass
            def test_genfunc():
                def check(x):
                    pass
                yield check, 3
            class TestClass:
                def test_method(self):
                    pass
       """

    def test_reportinfo_with_nasty_getattr(self, testdir):
        # https://github.com/pytest-dev/pytest/issues/1204
        modcol = testdir.getmodulecol("""
            # lineno 0
            class TestClass:
                def __getattr__(self, name):
                    return "this is not an int"

                def test_foo(self):
                    pass
        """)
        classcol = testdir.collect_by_name(modcol, "TestClass")
        instance = classcol.collect()[0]
        fspath, lineno, msg = instance.reportinfo()


def test_customized_python_discovery(testdir):
    testdir.makeini("""
        [pytest]
        python_files=check_*.py
        python_classes=Check
        python_functions=check
    """)
    p = testdir.makepyfile("""
        def check_simple():
            pass
        class CheckMyApp:
            def check_meth(self):
                pass
    """)
    p2 = p.new(basename=p.basename.replace("test", "check"))
    p.move(p2)
    result = testdir.runpytest("--collect-only", "-s")
    result.stdout.fnmatch_lines([
        "*check_customized*",
        "*check_simple*",
        "*CheckMyApp*",
        "*check_meth*",
    ])

    result = testdir.runpytest()
    assert result.ret == 0
    result.stdout.fnmatch_lines([
        "*2 passed*",
    ])


def test_customized_python_discovery_functions(testdir):
    testdir.makeini("""
        [pytest]
        python_functions=_test
    """)
    testdir.makepyfile("""
        def _test_underscore():
            pass
    """)
    result = testdir.runpytest("--collect-only", "-s")
    result.stdout.fnmatch_lines([
        "*_test_underscore*",
    ])

    result = testdir.runpytest()
    assert result.ret == 0
    result.stdout.fnmatch_lines([
        "*1 passed*",
    ])


def test_collector_attributes(testdir):
    testdir.makeconftest("""
        import pytest
        def pytest_pycollect_makeitem(collector):
            assert collector.Function == pytest.Function
            assert collector.Class == pytest.Class
            assert collector.Instance == pytest.Instance
            assert collector.Module == pytest.Module
    """)
    testdir.makepyfile("""
         def test_hello():
            pass
    """)
    result = testdir.runpytest()
    result.stdout.fnmatch_lines([
        "*1 passed*",
    ])

def test_customize_through_attributes(testdir):
    testdir.makeconftest("""
        import pytest
        class MyFunction(pytest.Function):
            pass
        class MyInstance(pytest.Instance):
            Function = MyFunction
        class MyClass(pytest.Class):
            Instance = MyInstance

        def pytest_pycollect_makeitem(collector, name, obj):
            if name.startswith("MyTestClass"):
                return MyClass(name, parent=collector)
    """)
    testdir.makepyfile("""
         class MyTestClass:
            def test_hello(self):
                pass
    """)
    result = testdir.runpytest("--collect-only")
    result.stdout.fnmatch_lines([
        "*MyClass*",
        "*MyInstance*",
        "*MyFunction*test_hello*",
    ])


def test_unorderable_types(testdir):
    testdir.makepyfile("""
        class TestJoinEmpty:
            pass

        def make_test():
            class Test:
                pass
            Test.__name__ = "TestFoo"
            return Test
        TestFoo = make_test()
    """)
    result = testdir.runpytest()
    assert "TypeError" not in result.stdout.str()
    assert result.ret == EXIT_NOTESTSCOLLECTED


def test_collect_functools_partial(testdir):
    """
    Test that collection of functools.partial object works, and arguments
    to the wrapped functions are dealt correctly (see #811).
    """
    testdir.makepyfile("""
        import functools
        import pytest

        @pytest.fixture
        def fix1():
            return 'fix1'

        @pytest.fixture
        def fix2():
            return 'fix2'

        def check1(i, fix1):
            assert i == 2
            assert fix1 == 'fix1'

        def check2(fix1, i):
            assert i == 2
            assert fix1 == 'fix1'

        def check3(fix1, i, fix2):
            assert i == 2
            assert fix1 == 'fix1'
            assert fix2 == 'fix2'

        test_ok_1 = functools.partial(check1, i=2)
        test_ok_2 = functools.partial(check1, i=2, fix1='fix1')
        test_ok_3 = functools.partial(check1, 2)
        test_ok_4 = functools.partial(check2, i=2)
        test_ok_5 = functools.partial(check3, i=2)
        test_ok_6 = functools.partial(check3, i=2, fix1='fix1')

        test_fail_1 = functools.partial(check2, 2)
        test_fail_2 = functools.partial(check3, 2)
    """)
    result = testdir.inline_run()
    result.assertoutcome(passed=6, failed=2)


def test_dont_collect_non_function_callable(testdir):
    """Test for issue https://github.com/pytest-dev/pytest/issues/331

    In this case an INTERNALERROR occurred trying to report the failure of
    a test like this one because py test failed to get the source lines.
    """
    testdir.makepyfile("""
        class Oh(object):
            def __call__(self):
                pass

        test_a = Oh()

        def test_real():
            pass
    """)
    result = testdir.runpytest('-rw')
    result.stdout.fnmatch_lines([
        '*collected 1 item*',
        'WC2 *',
        '*1 passed, 1 pytest-warnings in *',
    ])


def test_class_injection_does_not_break_collection(testdir):
    """Tests whether injection during collection time will terminate testing.

    In this case the error should not occur if the TestClass itself
    is modified during collection time, and the original method list
    is still used for collection.
    """
    testdir.makeconftest("""
        from test_inject import TestClass
        def pytest_generate_tests(metafunc):
            TestClass.changed_var = {}
    """)
    testdir.makepyfile(test_inject='''
         class TestClass(object):
            def test_injection(self):
                """Test being parametrized."""
                pass
    ''')
    result = testdir.runpytest()
    assert "RuntimeError: dictionary changed size during iteration" not in result.stdout.str()
    result.stdout.fnmatch_lines(['*1 passed*'])


def test_syntax_error_with_non_ascii_chars(testdir):
    """Fix decoding issue while formatting SyntaxErrors during collection (#578)
    """
    testdir.makepyfile(u"""
    # -*- coding: UTF-8 -*-

    ☃
    """)
    result = testdir.runpytest()
    result.stdout.fnmatch_lines([
        '*ERROR collecting*',
        '*SyntaxError*',
        '*1 error in*',
    ])
