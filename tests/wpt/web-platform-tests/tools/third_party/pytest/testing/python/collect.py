# -*- coding: utf-8 -*-
import os
import sys
import textwrap

import _pytest._code
import pytest
from _pytest.main import EXIT_NOTESTSCOLLECTED
from _pytest.nodes import Collector


class TestModule(object):
    def test_failing_import(self, testdir):
        modcol = testdir.getmodulecol("import alksdjalskdjalkjals")
        pytest.raises(Collector.CollectError, modcol.collect)

    def test_import_duplicate(self, testdir):
        a = testdir.mkdir("a")
        b = testdir.mkdir("b")
        p = a.ensure("test_whatever.py")
        p.pyimport()
        del sys.modules["test_whatever"]
        b.ensure("test_whatever.py")
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*import*mismatch*",
                "*imported*test_whatever*",
                "*%s*" % a.join("test_whatever.py"),
                "*not the same*",
                "*%s*" % b.join("test_whatever.py"),
                "*HINT*",
            ]
        )

    def test_import_prepend_append(self, testdir, monkeypatch):
        root1 = testdir.mkdir("root1")
        root2 = testdir.mkdir("root2")
        root1.ensure("x456.py")
        root2.ensure("x456.py")
        p = root2.join("test_x456.py")
        monkeypatch.syspath_prepend(str(root1))
        p.write(
            textwrap.dedent(
                """\
                import x456
                def test():
                    assert x456.__file__.startswith({!r})
                """.format(
                    str(root2)
                )
            )
        )
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

    def test_invalid_test_module_name(self, testdir):
        a = testdir.mkdir("a")
        a.ensure("test_one.part1.py")
        result = testdir.runpytest("-rw")
        result.stdout.fnmatch_lines(
            [
                "ImportError while importing test module*test_one.part1*",
                "Hint: make sure your test modules/packages have valid Python names.",
            ]
        )

    @pytest.mark.parametrize("verbose", [0, 1, 2])
    def test_show_traceback_import_error(self, testdir, verbose):
        """Import errors when collecting modules should display the traceback (#1976).

        With low verbosity we omit pytest and internal modules, otherwise show all traceback entries.
        """
        testdir.makepyfile(
            foo_traceback_import_error="""
               from bar_traceback_import_error import NOT_AVAILABLE
           """,
            bar_traceback_import_error="",
        )
        testdir.makepyfile(
            """
               import foo_traceback_import_error
        """
        )
        args = ("-v",) * verbose
        result = testdir.runpytest(*args)
        result.stdout.fnmatch_lines(
            [
                "ImportError while importing test module*",
                "Traceback:",
                "*from bar_traceback_import_error import NOT_AVAILABLE",
                "*cannot import name *NOT_AVAILABLE*",
            ]
        )
        assert result.ret == 2

        stdout = result.stdout.str()
        for name in ("_pytest", os.path.join("py", "_path")):
            if verbose == 2:
                assert name in stdout
            else:
                assert name not in stdout

    def test_show_traceback_import_error_unicode(self, testdir):
        """Check test modules collected which raise ImportError with unicode messages
        are handled properly (#2336).
        """
        testdir.makepyfile(
            u"""
            # -*- coding: utf-8 -*-
            raise ImportError(u'Something bad happened ☺')
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            [
                "ImportError while importing test module*",
                "Traceback:",
                "*raise ImportError*Something bad happened*",
            ]
        )
        assert result.ret == 2


class TestClass(object):
    def test_class_with_init_warning(self, testdir):
        testdir.makepyfile(
            """
            class TestClass1(object):
                def __init__(self):
                    pass
        """
        )
        result = testdir.runpytest("-rw")
        result.stdout.fnmatch_lines(
            [
                "*cannot collect test class 'TestClass1' because it has "
                "a __init__ constructor (from: test_class_with_init_warning.py)"
            ]
        )

    def test_class_with_new_warning(self, testdir):
        testdir.makepyfile(
            """
            class TestClass1(object):
                def __new__(self):
                    pass
        """
        )
        result = testdir.runpytest("-rw")
        result.stdout.fnmatch_lines(
            [
                "*cannot collect test class 'TestClass1' because it has "
                "a __new__ constructor (from: test_class_with_new_warning.py)"
            ]
        )

    def test_class_subclassobject(self, testdir):
        testdir.getmodulecol(
            """
            class test(object):
                pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*collected 0*"])

    def test_static_method(self, testdir):
        """Support for collecting staticmethod tests (#2528, #2699)"""
        testdir.getmodulecol(
            """
            import pytest
            class Test(object):
                @staticmethod
                def test_something():
                    pass

                @pytest.fixture
                def fix(self):
                    return 1

                @staticmethod
                def test_fix(fix):
                    assert fix == 1
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*collected 2 items*", "*2 passed in*"])

    def test_setup_teardown_class_as_classmethod(self, testdir):
        testdir.makepyfile(
            test_mod1="""
            class TestClassMethod(object):
                @classmethod
                def setup_class(cls):
                    pass
                def test_1(self):
                    pass
                @classmethod
                def teardown_class(cls):
                    pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_issue1035_obj_has_getattr(self, testdir):
        modcol = testdir.getmodulecol(
            """
            class Chameleon(object):
                def __getattr__(self, name):
                    return True
            chameleon = Chameleon()
        """
        )
        colitems = modcol.collect()
        assert len(colitems) == 0

    def test_issue1579_namedtuple(self, testdir):
        testdir.makepyfile(
            """
            import collections

            TestCase = collections.namedtuple('TestCase', ['a'])
        """
        )
        result = testdir.runpytest("-rw")
        result.stdout.fnmatch_lines(
            "*cannot collect test class 'TestCase' "
            "because it has a __new__ constructor*"
        )

    def test_issue2234_property(self, testdir):
        testdir.makepyfile(
            """
            class TestCase(object):
                @property
                def prop(self):
                    raise NotImplementedError()
        """
        )
        result = testdir.runpytest()
        assert result.ret == EXIT_NOTESTSCOLLECTED


class TestFunction(object):
    def test_getmodulecollector(self, testdir):
        item = testdir.getitem("def test_func(): pass")
        modcol = item.getparent(pytest.Module)
        assert isinstance(modcol, pytest.Module)
        assert hasattr(modcol.obj, "test_func")

    @pytest.mark.filterwarnings("default")
    def test_function_as_object_instance_ignored(self, testdir):
        testdir.makepyfile(
            """
            class A(object):
                def __call__(self, tmpdir):
                    0/0

            test_a = A()
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            [
                "collected 0 items",
                "*test_function_as_object_instance_ignored.py:2: "
                "*cannot collect 'test_a' because it is not a function.",
            ]
        )

    @staticmethod
    def make_function(testdir, **kwargs):
        from _pytest.fixtures import FixtureManager

        config = testdir.parseconfigure()
        session = testdir.Session(config)
        session._fixturemanager = FixtureManager(session)

        return pytest.Function(config=config, parent=session, **kwargs)

    def test_function_equality(self, testdir, tmpdir):
        def func1():
            pass

        def func2():
            pass

        f1 = self.make_function(testdir, name="name", args=(1,), callobj=func1)
        assert f1 == f1
        f2 = self.make_function(testdir, name="name", callobj=func2)
        assert f1 != f2

    def test_repr_produces_actual_test_id(self, testdir):
        f = self.make_function(
            testdir, name=r"test[\xe5]", callobj=self.test_repr_produces_actual_test_id
        )
        assert repr(f) == r"<Function test[\xe5]>"

    def test_issue197_parametrize_emptyset(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.parametrize('arg', [])
            def test_function(arg):
                pass
        """
        )
        reprec = testdir.inline_run()
        reprec.assertoutcome(skipped=1)

    def test_single_tuple_unwraps_values(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.parametrize(('arg',), [(1,)])
            def test_function(arg):
                assert arg == 1
        """
        )
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_issue213_parametrize_value_no_equal(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            class A(object):
                def __eq__(self, other):
                    raise ValueError("not possible")
            @pytest.mark.parametrize('arg', [A()])
            def test_function(arg):
                assert arg.__class__.__name__ == "A"
        """
        )
        reprec = testdir.inline_run("--fulltrace")
        reprec.assertoutcome(passed=1)

    def test_parametrize_with_non_hashable_values(self, testdir):
        """Test parametrization with non-hashable values."""
        testdir.makepyfile(
            """
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
        """
        )
        rec = testdir.inline_run()
        rec.assertoutcome(passed=2)

    def test_parametrize_with_non_hashable_values_indirect(self, testdir):
        """Test parametrization with non-hashable values with indirect parametrization."""
        testdir.makepyfile(
            """
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
        """
        )
        rec = testdir.inline_run()
        rec.assertoutcome(passed=2)

    def test_parametrize_overrides_fixture(self, testdir):
        """Test parametrization when parameter overrides existing fixture with same name."""
        testdir.makepyfile(
            """
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
        """
        )
        rec = testdir.inline_run()
        rec.assertoutcome(passed=3)

    def test_parametrize_overrides_parametrized_fixture(self, testdir):
        """Test parametrization when parameter overrides existing parametrized fixture with same name."""
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture(params=[1, 2])
            def value(request):
                return request.param

            @pytest.mark.parametrize('value',
                                     ['overridden'])
            def test_overridden_via_param(value):
                assert value == 'overridden'
        """
        )
        rec = testdir.inline_run()
        rec.assertoutcome(passed=1)

    def test_parametrize_overrides_indirect_dependency_fixture(self, testdir):
        """Test parametrization when parameter overrides a fixture that a test indirectly depends on"""
        testdir.makepyfile(
            """
            import pytest

            fix3_instantiated = False

            @pytest.fixture
            def fix1(fix2):
               return fix2 + '1'

            @pytest.fixture
            def fix2(fix3):
               return fix3 + '2'

            @pytest.fixture
            def fix3():
               global fix3_instantiated
               fix3_instantiated = True
               return '3'

            @pytest.mark.parametrize('fix2', ['2'])
            def test_it(fix1):
               assert fix1 == '21'
               assert not fix3_instantiated
        """
        )
        rec = testdir.inline_run()
        rec.assertoutcome(passed=1)

    def test_parametrize_with_mark(self, testdir):
        items = testdir.getitems(
            """
            import pytest
            @pytest.mark.foo
            @pytest.mark.parametrize('arg', [
                1,
                pytest.param(2, marks=[pytest.mark.baz, pytest.mark.bar])
            ])
            def test_function(arg):
                pass
        """
        )
        keywords = [item.keywords for item in items]
        assert (
            "foo" in keywords[0]
            and "bar" not in keywords[0]
            and "baz" not in keywords[0]
        )
        assert "foo" in keywords[1] and "bar" in keywords[1] and "baz" in keywords[1]

    def test_function_equality_with_callspec(self, testdir, tmpdir):
        items = testdir.getitems(
            """
            import pytest
            @pytest.mark.parametrize('arg', [1,2])
            def test_function(arg):
                pass
        """
        )
        assert items[0] != items[1]
        assert not (items[0] == items[1])

    def test_pyfunc_call(self, testdir):
        item = testdir.getitem("def test_func(): raise ValueError")
        config = item.config

        class MyPlugin1(object):
            def pytest_pyfunc_call(self, pyfuncitem):
                raise ValueError

        class MyPlugin2(object):
            def pytest_pyfunc_call(self, pyfuncitem):
                return True

        config.pluginmanager.register(MyPlugin1())
        config.pluginmanager.register(MyPlugin2())
        config.hook.pytest_runtest_setup(item=item)
        config.hook.pytest_pyfunc_call(pyfuncitem=item)

    def test_multiple_parametrize(self, testdir):
        modcol = testdir.getmodulecol(
            """
            import pytest
            @pytest.mark.parametrize('x', [0, 1])
            @pytest.mark.parametrize('y', [2, 3])
            def test1(x, y):
                pass
        """
        )
        colitems = modcol.collect()
        assert colitems[0].name == "test1[2-0]"
        assert colitems[1].name == "test1[2-1]"
        assert colitems[2].name == "test1[3-0]"
        assert colitems[3].name == "test1[3-1]"

    def test_issue751_multiple_parametrize_with_ids(self, testdir):
        modcol = testdir.getmodulecol(
            """
            import pytest
            @pytest.mark.parametrize('x', [0], ids=['c'])
            @pytest.mark.parametrize('y', [0, 1], ids=['a', 'b'])
            class Test(object):
                def test1(self, x, y):
                    pass
                def test2(self, x, y):
                    pass
        """
        )
        colitems = modcol.collect()[0].collect()[0].collect()
        assert colitems[0].name == "test1[a-c]"
        assert colitems[1].name == "test1[b-c]"
        assert colitems[2].name == "test2[a-c]"
        assert colitems[3].name == "test2[b-c]"

    def test_parametrize_skipif(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            m = pytest.mark.skipif('True')

            @pytest.mark.parametrize('x', [0, 1, pytest.param(2, marks=m)])
            def test_skip_if(x):
                assert x < 2
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed, 1 skipped in *"])

    def test_parametrize_skip(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            m = pytest.mark.skip('')

            @pytest.mark.parametrize('x', [0, 1, pytest.param(2, marks=m)])
            def test_skip(x):
                assert x < 2
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed, 1 skipped in *"])

    def test_parametrize_skipif_no_skip(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            m = pytest.mark.skipif('False')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_skipif_no_skip(x):
                assert x < 2
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 1 failed, 2 passed in *"])

    def test_parametrize_xfail(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            m = pytest.mark.xfail('True')

            @pytest.mark.parametrize('x', [0, 1, pytest.param(2, marks=m)])
            def test_xfail(x):
                assert x < 2
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed, 1 xfailed in *"])

    def test_parametrize_passed(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            m = pytest.mark.xfail('True')

            @pytest.mark.parametrize('x', [0, 1, pytest.param(2, marks=m)])
            def test_xfail(x):
                pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed, 1 xpassed in *"])

    def test_parametrize_xfail_passed(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            m = pytest.mark.xfail('False')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_passed(x):
                pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 3 passed in *"])

    def test_function_original_name(self, testdir):
        items = testdir.getitems(
            """
            import pytest
            @pytest.mark.parametrize('arg', [1,2])
            def test_func(arg):
                pass
        """
        )
        assert [x.originalname for x in items] == ["test_func", "test_func"]


class TestSorting(object):
    def test_check_equality(self, testdir):
        modcol = testdir.getmodulecol(
            """
            def test_pass(): pass
            def test_fail(): assert 0
        """
        )
        fn1 = testdir.collect_by_name(modcol, "test_pass")
        assert isinstance(fn1, pytest.Function)
        fn2 = testdir.collect_by_name(modcol, "test_pass")
        assert isinstance(fn2, pytest.Function)

        assert fn1 == fn2
        assert fn1 != modcol
        if sys.version_info < (3, 0):
            assert cmp(fn1, fn2) == 0  # NOQA
        assert hash(fn1) == hash(fn2)

        fn3 = testdir.collect_by_name(modcol, "test_fail")
        assert isinstance(fn3, pytest.Function)
        assert not (fn1 == fn3)
        assert fn1 != fn3

        for fn in fn1, fn2, fn3:
            assert fn != 3
            assert fn != modcol
            assert fn != [1, 2, 3]
            assert [1, 2, 3] != fn
            assert modcol != fn

    def test_allow_sane_sorting_for_decorators(self, testdir):
        modcol = testdir.getmodulecol(
            """
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
        """
        )
        colitems = modcol.collect()
        assert len(colitems) == 2
        assert [item.name for item in colitems] == ["test_b", "test_a"]


class TestConftestCustomization(object):
    def test_pytest_pycollect_module(self, testdir):
        testdir.makeconftest(
            """
            import pytest
            class MyModule(pytest.Module):
                pass
            def pytest_pycollect_makemodule(path, parent):
                if path.basename == "test_xyz.py":
                    return MyModule(path, parent)
        """
        )
        testdir.makepyfile("def test_some(): pass")
        testdir.makepyfile(test_xyz="def test_func(): pass")
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*<Module*test_pytest*", "*<MyModule*xyz*"])

    def test_customized_pymakemodule_issue205_subdir(self, testdir):
        b = testdir.mkdir("a").mkdir("b")
        b.join("conftest.py").write(
            textwrap.dedent(
                """\
                import pytest
                @pytest.hookimpl(hookwrapper=True)
                def pytest_pycollect_makemodule():
                    outcome = yield
                    mod = outcome.get_result()
                    mod.obj.hello = "world"
                """
            )
        )
        b.join("test_module.py").write(
            textwrap.dedent(
                """\
                def test_hello():
                    assert hello == "world"
                """
            )
        )
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_customized_pymakeitem(self, testdir):
        b = testdir.mkdir("a").mkdir("b")
        b.join("conftest.py").write(
            textwrap.dedent(
                """\
                import pytest
                @pytest.hookimpl(hookwrapper=True)
                def pytest_pycollect_makeitem():
                    outcome = yield
                    if outcome.excinfo is None:
                        result = outcome.get_result()
                        if result:
                            for func in result:
                                func._some123 = "world"
                """
            )
        )
        b.join("test_module.py").write(
            textwrap.dedent(
                """\
                import pytest

                @pytest.fixture()
                def obj(request):
                    return request.node._some123
                def test_hello(obj):
                    assert obj == "world"
                """
            )
        )
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_pytest_pycollect_makeitem(self, testdir):
        testdir.makeconftest(
            """
            import pytest
            class MyFunction(pytest.Function):
                pass
            def pytest_pycollect_makeitem(collector, name, obj):
                if name == "some":
                    return MyFunction(name, collector)
        """
        )
        testdir.makepyfile("def some(): pass")
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*MyFunction*some*"])

    def test_makeitem_non_underscore(self, testdir, monkeypatch):
        modcol = testdir.getmodulecol("def _hello(): pass")
        values = []
        monkeypatch.setattr(
            pytest.Module, "_makeitem", lambda self, name, obj: values.append(name)
        )
        values = modcol.collect()
        assert "_hello" not in values

    def test_issue2369_collect_module_fileext(self, testdir):
        """Ensure we can collect files with weird file extensions as Python
        modules (#2369)"""
        # We'll implement a little finder and loader to import files containing
        # Python source code whose file extension is ".narf".
        testdir.makeconftest(
            """
            import sys, os, imp
            from _pytest.python import Module

            class Loader(object):
                def load_module(self, name):
                    return imp.load_source(name, name + ".narf")
            class Finder(object):
                def find_module(self, name, path=None):
                    if os.path.exists(name + ".narf"):
                        return Loader()
            sys.meta_path.append(Finder())

            def pytest_collect_file(path, parent):
                if path.ext == ".narf":
                    return Module(path, parent)"""
        )
        testdir.makefile(
            ".narf",
            """\
            def test_something():
                assert 1 + 1 == 2""",
        )
        # Use runpytest_subprocess, since we're futzing with sys.meta_path.
        result = testdir.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*1 passed*"])


def test_setup_only_available_in_subdir(testdir):
    sub1 = testdir.mkpydir("sub1")
    sub2 = testdir.mkpydir("sub2")
    sub1.join("conftest.py").write(
        textwrap.dedent(
            """\
            import pytest
            def pytest_runtest_setup(item):
                assert item.fspath.purebasename == "test_in_sub1"
            def pytest_runtest_call(item):
                assert item.fspath.purebasename == "test_in_sub1"
            def pytest_runtest_teardown(item):
                assert item.fspath.purebasename == "test_in_sub1"
            """
        )
    )
    sub2.join("conftest.py").write(
        textwrap.dedent(
            """\
            import pytest
            def pytest_runtest_setup(item):
                assert item.fspath.purebasename == "test_in_sub2"
            def pytest_runtest_call(item):
                assert item.fspath.purebasename == "test_in_sub2"
            def pytest_runtest_teardown(item):
                assert item.fspath.purebasename == "test_in_sub2"
            """
        )
    )
    sub1.join("test_in_sub1.py").write("def test_1(): pass")
    sub2.join("test_in_sub2.py").write("def test_2(): pass")
    result = testdir.runpytest("-v", "-s")
    result.assert_outcomes(passed=2)


def test_modulecol_roundtrip(testdir):
    modcol = testdir.getmodulecol("pass", withinit=False)
    trail = modcol.nodeid
    newcol = modcol.session.perform_collect([trail], genitems=0)[0]
    assert modcol.name == newcol.name


class TestTracebackCutting(object):
    def test_skip_simple(self):
        with pytest.raises(pytest.skip.Exception) as excinfo:
            pytest.skip("xxx")
        assert excinfo.traceback[-1].frame.code.name == "skip"
        assert excinfo.traceback[-1].ishidden()

    def test_traceback_argsetup(self, testdir):
        testdir.makeconftest(
            """
            import pytest

            @pytest.fixture
            def hello(request):
                raise ValueError("xyz")
        """
        )
        p = testdir.makepyfile("def test(hello): pass")
        result = testdir.runpytest(p)
        assert result.ret != 0
        out = result.stdout.str()
        assert "xyz" in out
        assert "conftest.py:5: ValueError" in out
        numentries = out.count("_ _ _")  # separator for traceback entries
        assert numentries == 0

        result = testdir.runpytest("--fulltrace", p)
        out = result.stdout.str()
        assert "conftest.py:5: ValueError" in out
        numentries = out.count("_ _ _ _")  # separator for traceback entries
        assert numentries > 3

    def test_traceback_error_during_import(self, testdir):
        testdir.makepyfile(
            """
            x = 1
            x = 2
            x = 17
            asd
        """
        )
        result = testdir.runpytest()
        assert result.ret != 0
        out = result.stdout.str()
        assert "x = 1" not in out
        assert "x = 2" not in out
        result.stdout.fnmatch_lines([" *asd*", "E*NameError*"])
        result = testdir.runpytest("--fulltrace")
        out = result.stdout.str()
        assert "x = 1" in out
        assert "x = 2" in out
        result.stdout.fnmatch_lines([">*asd*", "E*NameError*"])

    def test_traceback_filter_error_during_fixture_collection(self, testdir):
        """integration test for issue #995.
        """
        testdir.makepyfile(
            """
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
        """
        )
        result = testdir.runpytest()
        assert result.ret != 0
        out = result.stdout.str()
        assert "INTERNALERROR>" not in out
        result.stdout.fnmatch_lines(["*ValueError: fail me*", "* 1 error in *"])

    def test_filter_traceback_generated_code(self):
        """test that filter_traceback() works with the fact that
        _pytest._code.code.Code.path attribute might return an str object.
        In this case, one of the entries on the traceback was produced by
        dynamically generated code.
        See: https://bitbucket.org/pytest-dev/py/issues/71
        This fixes #995.
        """
        from _pytest.python import filter_traceback

        try:
            ns = {}
            exec("def foo(): raise ValueError", ns)
            ns["foo"]()
        except ValueError:
            _, _, tb = sys.exc_info()

        tb = _pytest._code.Traceback(tb)
        assert isinstance(tb[-1].path, str)
        assert not filter_traceback(tb[-1])

    def test_filter_traceback_path_no_longer_valid(self, testdir):
        """test that filter_traceback() works with the fact that
        _pytest._code.code.Code.path attribute might return an str object.
        In this case, one of the files in the traceback no longer exists.
        This fixes #1133.
        """
        from _pytest.python import filter_traceback

        testdir.syspathinsert()
        testdir.makepyfile(
            filter_traceback_entry_as_str="""
            def foo():
                raise ValueError
        """
        )
        try:
            import filter_traceback_entry_as_str

            filter_traceback_entry_as_str.foo()
        except ValueError:
            _, _, tb = sys.exc_info()

        testdir.tmpdir.join("filter_traceback_entry_as_str.py").remove()
        tb = _pytest._code.Traceback(tb)
        assert isinstance(tb[-1].path, str)
        assert filter_traceback(tb[-1])


class TestReportInfo(object):
    def test_itemreport_reportinfo(self, testdir, linecomp):
        testdir.makeconftest(
            """
            import pytest
            class MyFunction(pytest.Function):
                def reportinfo(self):
                    return "ABCDE", 42, "custom"
            def pytest_pycollect_makeitem(collector, name, obj):
                if name == "test_func":
                    return MyFunction(name, parent=collector)
        """
        )
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
        modcol = testdir.getmodulecol(
            """
            # lineno 0
            class TestClass(object):
                def test_hello(self): pass
        """
        )
        classcol = testdir.collect_by_name(modcol, "TestClass")
        fspath, lineno, msg = classcol.reportinfo()
        assert fspath == modcol.fspath
        assert lineno == 1
        assert msg == "TestClass"

    @pytest.mark.filterwarnings(
        "ignore:usage of Generator.Function is deprecated, please use pytest.Function instead"
    )
    def test_reportinfo_with_nasty_getattr(self, testdir):
        # https://github.com/pytest-dev/pytest/issues/1204
        modcol = testdir.getmodulecol(
            """
            # lineno 0
            class TestClass(object):
                def __getattr__(self, name):
                    return "this is not an int"

                def test_foo(self):
                    pass
        """
        )
        classcol = testdir.collect_by_name(modcol, "TestClass")
        instance = classcol.collect()[0]
        fspath, lineno, msg = instance.reportinfo()


def test_customized_python_discovery(testdir):
    testdir.makeini(
        """
        [pytest]
        python_files=check_*.py
        python_classes=Check
        python_functions=check
    """
    )
    p = testdir.makepyfile(
        """
        def check_simple():
            pass
        class CheckMyApp(object):
            def check_meth(self):
                pass
    """
    )
    p2 = p.new(basename=p.basename.replace("test", "check"))
    p.move(p2)
    result = testdir.runpytest("--collect-only", "-s")
    result.stdout.fnmatch_lines(
        ["*check_customized*", "*check_simple*", "*CheckMyApp*", "*check_meth*"]
    )

    result = testdir.runpytest()
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*2 passed*"])


def test_customized_python_discovery_functions(testdir):
    testdir.makeini(
        """
        [pytest]
        python_functions=_test
    """
    )
    testdir.makepyfile(
        """
        def _test_underscore():
            pass
    """
    )
    result = testdir.runpytest("--collect-only", "-s")
    result.stdout.fnmatch_lines(["*_test_underscore*"])

    result = testdir.runpytest()
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_unorderable_types(testdir):
    testdir.makepyfile(
        """
        class TestJoinEmpty(object):
            pass

        def make_test():
            class Test(object):
                pass
            Test.__name__ = "TestFoo"
            return Test
        TestFoo = make_test()
    """
    )
    result = testdir.runpytest()
    assert "TypeError" not in result.stdout.str()
    assert result.ret == EXIT_NOTESTSCOLLECTED


def test_collect_functools_partial(testdir):
    """
    Test that collection of functools.partial object works, and arguments
    to the wrapped functions are dealt correctly (see #811).
    """
    testdir.makepyfile(
        """
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
    """
    )
    result = testdir.inline_run()
    result.assertoutcome(passed=6, failed=2)


@pytest.mark.filterwarnings("default")
def test_dont_collect_non_function_callable(testdir):
    """Test for issue https://github.com/pytest-dev/pytest/issues/331

    In this case an INTERNALERROR occurred trying to report the failure of
    a test like this one because py test failed to get the source lines.
    """
    testdir.makepyfile(
        """
        class Oh(object):
            def __call__(self):
                pass

        test_a = Oh()

        def test_real():
            pass
    """
    )
    result = testdir.runpytest("-rw")
    result.stdout.fnmatch_lines(
        [
            "*collected 1 item*",
            "*test_dont_collect_non_function_callable.py:2: *cannot collect 'test_a' because it is not a function*",
            "*1 passed, 1 warnings in *",
        ]
    )


def test_class_injection_does_not_break_collection(testdir):
    """Tests whether injection during collection time will terminate testing.

    In this case the error should not occur if the TestClass itself
    is modified during collection time, and the original method list
    is still used for collection.
    """
    testdir.makeconftest(
        """
        from test_inject import TestClass
        def pytest_generate_tests(metafunc):
            TestClass.changed_var = {}
    """
    )
    testdir.makepyfile(
        test_inject='''
         class TestClass(object):
            def test_injection(self):
                """Test being parametrized."""
                pass
    '''
    )
    result = testdir.runpytest()
    assert (
        "RuntimeError: dictionary changed size during iteration"
        not in result.stdout.str()
    )
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_syntax_error_with_non_ascii_chars(testdir):
    """Fix decoding issue while formatting SyntaxErrors during collection (#578)
    """
    testdir.makepyfile(
        u"""
    # -*- coding: utf-8 -*-

    ☃
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*ERROR collecting*", "*SyntaxError*", "*1 error in*"])


def test_skip_duplicates_by_default(testdir):
    """Test for issue https://github.com/pytest-dev/pytest/issues/1609 (#1609)

    Ignore duplicate directories.
    """
    a = testdir.mkdir("a")
    fh = a.join("test_a.py")
    fh.write(
        textwrap.dedent(
            """\
            import pytest
            def test_real():
                pass
            """
        )
    )
    result = testdir.runpytest(a.strpath, a.strpath)
    result.stdout.fnmatch_lines(["*collected 1 item*"])


def test_keep_duplicates(testdir):
    """Test for issue https://github.com/pytest-dev/pytest/issues/1609 (#1609)

    Use --keep-duplicates to collect tests from duplicate directories.
    """
    a = testdir.mkdir("a")
    fh = a.join("test_a.py")
    fh.write(
        textwrap.dedent(
            """\
            import pytest
            def test_real():
                pass
            """
        )
    )
    result = testdir.runpytest("--keep-duplicates", a.strpath, a.strpath)
    result.stdout.fnmatch_lines(["*collected 2 item*"])


def test_package_collection_infinite_recursion(testdir):
    testdir.copy_example("collect/package_infinite_recursion")
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_package_collection_init_given_as_argument(testdir):
    """Regression test for #3749"""
    p = testdir.copy_example("collect/package_init_given_as_arg")
    result = testdir.runpytest(p / "pkg" / "__init__.py")
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_package_with_modules(testdir):
    """
    .
    └── root
        ├── __init__.py
        ├── sub1
        │   ├── __init__.py
        │   └── sub1_1
        │       ├── __init__.py
        │       └── test_in_sub1.py
        └── sub2
            └── test
                └── test_in_sub2.py

    """
    root = testdir.mkpydir("root")
    sub1 = root.mkdir("sub1")
    sub1.ensure("__init__.py")
    sub1_test = sub1.mkdir("sub1_1")
    sub1_test.ensure("__init__.py")
    sub2 = root.mkdir("sub2")
    sub2_test = sub2.mkdir("sub2")

    sub1_test.join("test_in_sub1.py").write("def test_1(): pass")
    sub2_test.join("test_in_sub2.py").write("def test_2(): pass")

    # Execute from .
    result = testdir.runpytest("-v", "-s")
    result.assert_outcomes(passed=2)

    # Execute from . with one argument "root"
    result = testdir.runpytest("-v", "-s", "root")
    result.assert_outcomes(passed=2)

    # Chdir into package's root and execute with no args
    root.chdir()
    result = testdir.runpytest("-v", "-s")
    result.assert_outcomes(passed=2)


def test_package_ordering(testdir):
    """
    .
    └── root
        ├── Test_root.py
        ├── __init__.py
        ├── sub1
        │   ├── Test_sub1.py
        │   └── __init__.py
        └── sub2
            └── test
                └── test_sub2.py

    """
    testdir.makeini(
        """
        [pytest]
        python_files=*.py
    """
    )
    root = testdir.mkpydir("root")
    sub1 = root.mkdir("sub1")
    sub1.ensure("__init__.py")
    sub2 = root.mkdir("sub2")
    sub2_test = sub2.mkdir("sub2")

    root.join("Test_root.py").write("def test_1(): pass")
    sub1.join("Test_sub1.py").write("def test_2(): pass")
    sub2_test.join("test_sub2.py").write("def test_3(): pass")

    # Execute from .
    result = testdir.runpytest("-v", "-s")
    result.assert_outcomes(passed=3)
