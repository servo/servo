# mypy: allow-untyped-defs
import os
import sys
import textwrap
from typing import Any
from typing import Dict

import _pytest._code
from _pytest.config import ExitCode
from _pytest.main import Session
from _pytest.monkeypatch import MonkeyPatch
from _pytest.nodes import Collector
from _pytest.pytester import Pytester
from _pytest.python import Class
from _pytest.python import Function
import pytest


class TestModule:
    def test_failing_import(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol("import alksdjalskdjalkjals")
        pytest.raises(Collector.CollectError, modcol.collect)

    def test_import_duplicate(self, pytester: Pytester) -> None:
        a = pytester.mkdir("a")
        b = pytester.mkdir("b")
        p1 = a.joinpath("test_whatever.py")
        p1.touch()
        p2 = b.joinpath("test_whatever.py")
        p2.touch()
        # ensure we don't have it imported already
        sys.modules.pop(p1.stem, None)

        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*import*mismatch*",
                "*imported*test_whatever*",
                "*%s*" % p1,
                "*not the same*",
                "*%s*" % p2,
                "*HINT*",
            ]
        )

    def test_import_prepend_append(
        self, pytester: Pytester, monkeypatch: MonkeyPatch
    ) -> None:
        root1 = pytester.mkdir("root1")
        root2 = pytester.mkdir("root2")
        root1.joinpath("x456.py").touch()
        root2.joinpath("x456.py").touch()
        p = root2.joinpath("test_x456.py")
        monkeypatch.syspath_prepend(str(root1))
        p.write_text(
            textwrap.dedent(
                f"""\
                import x456
                def test():
                    assert x456.__file__.startswith({str(root2)!r})
                """
            ),
            encoding="utf-8",
        )
        with monkeypatch.context() as mp:
            mp.chdir(root2)
            reprec = pytester.inline_run("--import-mode=append")
            reprec.assertoutcome(passed=0, failed=1)
            reprec = pytester.inline_run()
            reprec.assertoutcome(passed=1)

    def test_syntax_error_in_module(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol("this is a syntax error")
        pytest.raises(modcol.CollectError, modcol.collect)
        pytest.raises(modcol.CollectError, modcol.collect)

    def test_module_considers_pluginmanager_at_import(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol("pytest_plugins='xasdlkj',")
        pytest.raises(ImportError, lambda: modcol.obj)

    def test_invalid_test_module_name(self, pytester: Pytester) -> None:
        a = pytester.mkdir("a")
        a.joinpath("test_one.part1.py").touch()
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "ImportError while importing test module*test_one.part1*",
                "Hint: make sure your test modules/packages have valid Python names.",
            ]
        )

    @pytest.mark.parametrize("verbose", [0, 1, 2])
    def test_show_traceback_import_error(
        self, pytester: Pytester, verbose: int
    ) -> None:
        """Import errors when collecting modules should display the traceback (#1976).

        With low verbosity we omit pytest and internal modules, otherwise show all traceback entries.
        """
        pytester.makepyfile(
            foo_traceback_import_error="""
               from bar_traceback_import_error import NOT_AVAILABLE
           """,
            bar_traceback_import_error="",
        )
        pytester.makepyfile(
            """
               import foo_traceback_import_error
        """
        )
        args = ("-v",) * verbose
        result = pytester.runpytest(*args)
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
        if verbose == 2:
            assert "_pytest" in stdout
        else:
            assert "_pytest" not in stdout

    def test_show_traceback_import_error_unicode(self, pytester: Pytester) -> None:
        """Check test modules collected which raise ImportError with unicode messages
        are handled properly (#2336).
        """
        pytester.makepyfile("raise ImportError('Something bad happened ☺')")
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "ImportError while importing test module*",
                "Traceback:",
                "*raise ImportError*Something bad happened*",
            ]
        )
        assert result.ret == 2


class TestClass:
    def test_class_with_init_warning(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            class TestClass1(object):
                def __init__(self):
                    pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*cannot collect test class 'TestClass1' because it has "
                "a __init__ constructor (from: test_class_with_init_warning.py)"
            ]
        )

    def test_class_with_new_warning(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            class TestClass1(object):
                def __new__(self):
                    pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*cannot collect test class 'TestClass1' because it has "
                "a __new__ constructor (from: test_class_with_new_warning.py)"
            ]
        )

    def test_class_subclassobject(self, pytester: Pytester) -> None:
        pytester.getmodulecol(
            """
            class test(object):
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*collected 0*"])

    def test_static_method(self, pytester: Pytester) -> None:
        """Support for collecting staticmethod tests (#2528, #2699)"""
        pytester.getmodulecol(
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
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*collected 2 items*", "*2 passed in*"])

    def test_setup_teardown_class_as_classmethod(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_issue1035_obj_has_getattr(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
            """
            class Chameleon(object):
                def __getattr__(self, name):
                    return True
            chameleon = Chameleon()
        """
        )
        colitems = modcol.collect()
        assert len(colitems) == 0

    def test_issue1579_namedtuple(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import collections

            TestCase = collections.namedtuple('TestCase', ['a'])
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            "*cannot collect test class 'TestCase' "
            "because it has a __new__ constructor*"
        )

    def test_issue2234_property(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            class TestCase(object):
                @property
                def prop(self):
                    raise NotImplementedError()
        """
        )
        result = pytester.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED


class TestFunction:
    def test_getmodulecollector(self, pytester: Pytester) -> None:
        item = pytester.getitem("def test_func(): pass")
        modcol = item.getparent(pytest.Module)
        assert isinstance(modcol, pytest.Module)
        assert hasattr(modcol.obj, "test_func")

    @pytest.mark.filterwarnings("default")
    def test_function_as_object_instance_ignored(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            class A(object):
                def __call__(self, tmp_path):
                    0/0

            test_a = A()
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "collected 0 items",
                "*test_function_as_object_instance_ignored.py:2: "
                "*cannot collect 'test_a' because it is not a function.",
            ]
        )

    @staticmethod
    def make_function(pytester: Pytester, **kwargs: Any) -> Any:
        from _pytest.fixtures import FixtureManager

        config = pytester.parseconfigure()
        session = Session.from_config(config)
        session._fixturemanager = FixtureManager(session)

        return pytest.Function.from_parent(parent=session, **kwargs)

    def test_function_equality(self, pytester: Pytester) -> None:
        def func1():
            pass

        def func2():
            pass

        f1 = self.make_function(pytester, name="name", callobj=func1)
        assert f1 == f1
        f2 = self.make_function(
            pytester, name="name", callobj=func2, originalname="foobar"
        )
        assert f1 != f2

    def test_repr_produces_actual_test_id(self, pytester: Pytester) -> None:
        f = self.make_function(
            pytester, name=r"test[\xe5]", callobj=self.test_repr_produces_actual_test_id
        )
        assert repr(f) == r"<Function test[\xe5]>"

    def test_issue197_parametrize_emptyset(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.parametrize('arg', [])
            def test_function(arg):
                pass
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(skipped=1)

    def test_single_tuple_unwraps_values(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.parametrize(('arg',), [(1,)])
            def test_function(arg):
                assert arg == 1
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_issue213_parametrize_value_no_equal(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        reprec = pytester.inline_run("--fulltrace")
        reprec.assertoutcome(passed=1)

    def test_parametrize_with_non_hashable_values(self, pytester: Pytester) -> None:
        """Test parametrization with non-hashable values."""
        pytester.makepyfile(
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
        rec = pytester.inline_run()
        rec.assertoutcome(passed=2)

    def test_parametrize_with_non_hashable_values_indirect(
        self, pytester: Pytester
    ) -> None:
        """Test parametrization with non-hashable values with indirect parametrization."""
        pytester.makepyfile(
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
        rec = pytester.inline_run()
        rec.assertoutcome(passed=2)

    def test_parametrize_overrides_fixture(self, pytester: Pytester) -> None:
        """Test parametrization when parameter overrides existing fixture with same name."""
        pytester.makepyfile(
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
        rec = pytester.inline_run()
        rec.assertoutcome(passed=3)

    def test_parametrize_overrides_parametrized_fixture(
        self, pytester: Pytester
    ) -> None:
        """Test parametrization when parameter overrides existing parametrized fixture with same name."""
        pytester.makepyfile(
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
        rec = pytester.inline_run()
        rec.assertoutcome(passed=1)

    def test_parametrize_overrides_indirect_dependency_fixture(
        self, pytester: Pytester
    ) -> None:
        """Test parametrization when parameter overrides a fixture that a test indirectly depends on"""
        pytester.makepyfile(
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
        rec = pytester.inline_run()
        rec.assertoutcome(passed=1)

    def test_parametrize_with_mark(self, pytester: Pytester) -> None:
        items = pytester.getitems(
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

    def test_parametrize_with_empty_string_arguments(self, pytester: Pytester) -> None:
        items = pytester.getitems(
            """\
            import pytest

            @pytest.mark.parametrize('v', ('', ' '))
            @pytest.mark.parametrize('w', ('', ' '))
            def test(v, w): ...
            """
        )
        names = {item.name for item in items}
        assert names == {"test[-]", "test[ -]", "test[- ]", "test[ - ]"}

    def test_function_equality_with_callspec(self, pytester: Pytester) -> None:
        items = pytester.getitems(
            """
            import pytest
            @pytest.mark.parametrize('arg', [1,2])
            def test_function(arg):
                pass
        """
        )
        assert items[0] != items[1]
        assert not (items[0] == items[1])

    def test_pyfunc_call(self, pytester: Pytester) -> None:
        item = pytester.getitem("def test_func(): raise ValueError")
        config = item.config

        class MyPlugin1:
            def pytest_pyfunc_call(self):
                raise ValueError

        class MyPlugin2:
            def pytest_pyfunc_call(self):
                return True

        config.pluginmanager.register(MyPlugin1())
        config.pluginmanager.register(MyPlugin2())
        config.hook.pytest_runtest_setup(item=item)
        config.hook.pytest_pyfunc_call(pyfuncitem=item)

    def test_multiple_parametrize(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
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

    def test_issue751_multiple_parametrize_with_ids(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
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
        colitems = modcol.collect()[0].collect()
        assert colitems[0].name == "test1[a-c]"
        assert colitems[1].name == "test1[b-c]"
        assert colitems[2].name == "test2[a-c]"
        assert colitems[3].name == "test2[b-c]"

    def test_parametrize_skipif(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            m = pytest.mark.skipif('True')

            @pytest.mark.parametrize('x', [0, 1, pytest.param(2, marks=m)])
            def test_skip_if(x):
                assert x < 2
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed, 1 skipped in *"])

    def test_parametrize_skip(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            m = pytest.mark.skip('')

            @pytest.mark.parametrize('x', [0, 1, pytest.param(2, marks=m)])
            def test_skip(x):
                assert x < 2
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed, 1 skipped in *"])

    def test_parametrize_skipif_no_skip(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            m = pytest.mark.skipif('False')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_skipif_no_skip(x):
                assert x < 2
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 1 failed, 2 passed in *"])

    def test_parametrize_xfail(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            m = pytest.mark.xfail('True')

            @pytest.mark.parametrize('x', [0, 1, pytest.param(2, marks=m)])
            def test_xfail(x):
                assert x < 2
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed, 1 xfailed in *"])

    def test_parametrize_passed(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            m = pytest.mark.xfail('True')

            @pytest.mark.parametrize('x', [0, 1, pytest.param(2, marks=m)])
            def test_xfail(x):
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed, 1 xpassed in *"])

    def test_parametrize_xfail_passed(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            m = pytest.mark.xfail('False')

            @pytest.mark.parametrize('x', [0, 1, m(2)])
            def test_passed(x):
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 3 passed in *"])

    def test_function_originalname(self, pytester: Pytester) -> None:
        items = pytester.getitems(
            """
            import pytest

            @pytest.mark.parametrize('arg', [1,2])
            def test_func(arg):
                pass

            def test_no_param():
                pass
        """
        )
        originalnames = []
        for x in items:
            assert isinstance(x, pytest.Function)
            originalnames.append(x.originalname)
        assert originalnames == [
            "test_func",
            "test_func",
            "test_no_param",
        ]

    def test_function_with_square_brackets(self, pytester: Pytester) -> None:
        """Check that functions with square brackets don't cause trouble."""
        p1 = pytester.makepyfile(
            """
            locals()["test_foo[name]"] = lambda: None
            """
        )
        result = pytester.runpytest("-v", str(p1))
        result.stdout.fnmatch_lines(
            [
                "test_function_with_square_brackets.py::test_foo[[]name[]] PASSED *",
                "*= 1 passed in *",
            ]
        )


class TestSorting:
    def test_check_equality(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
            """
            def test_pass(): pass
            def test_fail(): assert 0
        """
        )
        fn1 = pytester.collect_by_name(modcol, "test_pass")
        assert isinstance(fn1, pytest.Function)
        fn2 = pytester.collect_by_name(modcol, "test_pass")
        assert isinstance(fn2, pytest.Function)

        assert fn1 == fn2
        assert fn1 != modcol
        assert hash(fn1) == hash(fn2)

        fn3 = pytester.collect_by_name(modcol, "test_fail")
        assert isinstance(fn3, pytest.Function)
        assert not (fn1 == fn3)
        assert fn1 != fn3

        for fn in fn1, fn2, fn3:
            assert fn != 3  # type: ignore[comparison-overlap]
            assert fn != modcol
            assert fn != [1, 2, 3]  # type: ignore[comparison-overlap]
            assert [1, 2, 3] != fn  # type: ignore[comparison-overlap]
            assert modcol != fn

    def test_allow_sane_sorting_for_decorators(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
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

    def test_ordered_by_definition_order(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """\
            class Test1:
                def test_foo(self): pass
                def test_bar(self): pass
            class Test2:
                def test_foo(self): pass
                test_bar = Test1.test_bar
            class Test3(Test2):
                def test_baz(self): pass
            """
        )
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(
            [
                "*Class Test1*",
                "*Function test_foo*",
                "*Function test_bar*",
                "*Class Test2*",
                # previously the order was flipped due to Test1.test_bar reference
                "*Function test_foo*",
                "*Function test_bar*",
                "*Class Test3*",
                "*Function test_foo*",
                "*Function test_bar*",
                "*Function test_baz*",
            ]
        )


class TestConftestCustomization:
    def test_pytest_pycollect_module(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            class MyModule(pytest.Module):
                pass
            def pytest_pycollect_makemodule(module_path, parent):
                if module_path.name == "test_xyz.py":
                    return MyModule.from_parent(path=module_path, parent=parent)
        """
        )
        pytester.makepyfile("def test_some(): pass")
        pytester.makepyfile(test_xyz="def test_func(): pass")
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*<Module*test_pytest*", "*<MyModule*xyz*"])

    def test_customized_pymakemodule_issue205_subdir(self, pytester: Pytester) -> None:
        b = pytester.path.joinpath("a", "b")
        b.mkdir(parents=True)
        b.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest
                @pytest.hookimpl(wrapper=True)
                def pytest_pycollect_makemodule():
                    mod = yield
                    mod.obj.hello = "world"
                    return mod
                """
            ),
            encoding="utf-8",
        )
        b.joinpath("test_module.py").write_text(
            textwrap.dedent(
                """\
                def test_hello():
                    assert hello == "world"
                """
            ),
            encoding="utf-8",
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_customized_pymakeitem(self, pytester: Pytester) -> None:
        b = pytester.path.joinpath("a", "b")
        b.mkdir(parents=True)
        b.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest
                @pytest.hookimpl(wrapper=True)
                def pytest_pycollect_makeitem():
                    result = yield
                    if result:
                        for func in result:
                            func._some123 = "world"
                    return result
                """
            ),
            encoding="utf-8",
        )
        b.joinpath("test_module.py").write_text(
            textwrap.dedent(
                """\
                import pytest

                @pytest.fixture()
                def obj(request):
                    return request.node._some123
                def test_hello(obj):
                    assert obj == "world"
                """
            ),
            encoding="utf-8",
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_pytest_pycollect_makeitem(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            class MyFunction(pytest.Function):
                pass
            def pytest_pycollect_makeitem(collector, name, obj):
                if name == "some":
                    return MyFunction.from_parent(name=name, parent=collector)
        """
        )
        pytester.makepyfile("def some(): pass")
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*MyFunction*some*"])

    def test_issue2369_collect_module_fileext(self, pytester: Pytester) -> None:
        """Ensure we can collect files with weird file extensions as Python
        modules (#2369)"""
        # Implement a little meta path finder to import files containing
        # Python source code whose file extension is ".narf".
        pytester.makeconftest(
            """
            import sys
            import os.path
            from importlib.util import spec_from_loader
            from importlib.machinery import SourceFileLoader
            from _pytest.python import Module

            class MetaPathFinder:
                def find_spec(self, fullname, path, target=None):
                    if os.path.exists(fullname + ".narf"):
                        return spec_from_loader(
                            fullname,
                            SourceFileLoader(fullname, fullname + ".narf"),
                        )
            sys.meta_path.append(MetaPathFinder())

            def pytest_collect_file(file_path, parent):
                if file_path.suffix == ".narf":
                    return Module.from_parent(path=file_path, parent=parent)
            """
        )
        pytester.makefile(
            ".narf",
            """\
            def test_something():
                assert 1 + 1 == 2""",
        )
        # Use runpytest_subprocess, since we're futzing with sys.meta_path.
        result = pytester.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_early_ignored_attributes(self, pytester: Pytester) -> None:
        """Builtin attributes should be ignored early on, even if
        configuration would otherwise allow them.

        This tests a performance optimization, not correctness, really,
        although it tests PytestCollectionWarning is not raised, while
        it would have been raised otherwise.
        """
        pytester.makeini(
            """
            [pytest]
            python_classes=*
            python_functions=*
        """
        )
        pytester.makepyfile(
            """
            class TestEmpty:
                pass
            test_empty = TestEmpty()
            def test_real():
                pass
        """
        )
        items, rec = pytester.inline_genitems()
        assert rec.ret == 0
        assert len(items) == 1


def test_setup_only_available_in_subdir(pytester: Pytester) -> None:
    sub1 = pytester.mkpydir("sub1")
    sub2 = pytester.mkpydir("sub2")
    sub1.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            import pytest
            def pytest_runtest_setup(item):
                assert item.path.stem == "test_in_sub1"
            def pytest_runtest_call(item):
                assert item.path.stem == "test_in_sub1"
            def pytest_runtest_teardown(item):
                assert item.path.stem == "test_in_sub1"
            """
        ),
        encoding="utf-8",
    )
    sub2.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            import pytest
            def pytest_runtest_setup(item):
                assert item.path.stem == "test_in_sub2"
            def pytest_runtest_call(item):
                assert item.path.stem == "test_in_sub2"
            def pytest_runtest_teardown(item):
                assert item.path.stem == "test_in_sub2"
            """
        ),
        encoding="utf-8",
    )
    sub1.joinpath("test_in_sub1.py").write_text("def test_1(): pass", encoding="utf-8")
    sub2.joinpath("test_in_sub2.py").write_text("def test_2(): pass", encoding="utf-8")
    result = pytester.runpytest("-v", "-s")
    result.assert_outcomes(passed=2)


def test_modulecol_roundtrip(pytester: Pytester) -> None:
    modcol = pytester.getmodulecol("pass", withinit=False)
    trail = modcol.nodeid
    newcol = modcol.session.perform_collect([trail], genitems=0)[0]
    assert modcol.name == newcol.name


class TestTracebackCutting:
    def test_skip_simple(self):
        with pytest.raises(pytest.skip.Exception) as excinfo:
            pytest.skip("xxx")
        assert excinfo.traceback[-1].frame.code.name == "skip"
        assert excinfo.traceback[-1].ishidden(excinfo)
        assert excinfo.traceback[-2].frame.code.name == "test_skip_simple"
        assert not excinfo.traceback[-2].ishidden(excinfo)

    def test_traceback_argsetup(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture
            def hello(request):
                raise ValueError("xyz")
        """
        )
        p = pytester.makepyfile("def test(hello): pass")
        result = pytester.runpytest(p)
        assert result.ret != 0
        out = result.stdout.str()
        assert "xyz" in out
        assert "conftest.py:5: ValueError" in out
        numentries = out.count("_ _ _")  # separator for traceback entries
        assert numentries == 0

        result = pytester.runpytest("--fulltrace", p)
        out = result.stdout.str()
        assert "conftest.py:5: ValueError" in out
        numentries = out.count("_ _ _ _")  # separator for traceback entries
        assert numentries > 3

    def test_traceback_error_during_import(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            x = 1
            x = 2
            x = 17
            asd
        """
        )
        result = pytester.runpytest()
        assert result.ret != 0
        out = result.stdout.str()
        assert "x = 1" not in out
        assert "x = 2" not in out
        result.stdout.fnmatch_lines([" *asd*", "E*NameError*"])
        result = pytester.runpytest("--fulltrace")
        out = result.stdout.str()
        assert "x = 1" in out
        assert "x = 2" in out
        result.stdout.fnmatch_lines([">*asd*", "E*NameError*"])

    def test_traceback_filter_error_during_fixture_collection(
        self, pytester: Pytester
    ) -> None:
        """Integration test for issue #995."""
        pytester.makepyfile(
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
        result = pytester.runpytest()
        assert result.ret != 0
        out = result.stdout.str()
        assert "INTERNALERROR>" not in out
        result.stdout.fnmatch_lines(["*ValueError: fail me*", "* 1 error in *"])

    def test_filter_traceback_generated_code(self) -> None:
        """Test that filter_traceback() works with the fact that
        _pytest._code.code.Code.path attribute might return an str object.

        In this case, one of the entries on the traceback was produced by
        dynamically generated code.
        See: https://bitbucket.org/pytest-dev/py/issues/71
        This fixes #995.
        """
        from _pytest._code import filter_traceback

        tb = None
        try:
            ns: Dict[str, Any] = {}
            exec("def foo(): raise ValueError", ns)
            ns["foo"]()
        except ValueError:
            _, _, tb = sys.exc_info()

        assert tb is not None
        traceback = _pytest._code.Traceback(tb)
        assert isinstance(traceback[-1].path, str)
        assert not filter_traceback(traceback[-1])

    def test_filter_traceback_path_no_longer_valid(self, pytester: Pytester) -> None:
        """Test that filter_traceback() works with the fact that
        _pytest._code.code.Code.path attribute might return an str object.

        In this case, one of the files in the traceback no longer exists.
        This fixes #1133.
        """
        from _pytest._code import filter_traceback

        pytester.syspathinsert()
        pytester.makepyfile(
            filter_traceback_entry_as_str="""
            def foo():
                raise ValueError
        """
        )
        tb = None
        try:
            import filter_traceback_entry_as_str

            filter_traceback_entry_as_str.foo()
        except ValueError:
            _, _, tb = sys.exc_info()

        assert tb is not None
        pytester.path.joinpath("filter_traceback_entry_as_str.py").unlink()
        traceback = _pytest._code.Traceback(tb)
        assert isinstance(traceback[-1].path, str)
        assert filter_traceback(traceback[-1])


class TestReportInfo:
    def test_itemreport_reportinfo(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            class MyFunction(pytest.Function):
                def reportinfo(self):
                    return "ABCDE", 42, "custom"
            def pytest_pycollect_makeitem(collector, name, obj):
                if name == "test_func":
                    return MyFunction.from_parent(name=name, parent=collector)
        """
        )
        item = pytester.getitem("def test_func(): pass")
        item.config.pluginmanager.getplugin("runner")
        assert item.location == ("ABCDE", 42, "custom")

    def test_func_reportinfo(self, pytester: Pytester) -> None:
        item = pytester.getitem("def test_func(): pass")
        path, lineno, modpath = item.reportinfo()
        assert os.fspath(path) == str(item.path)
        assert lineno == 0
        assert modpath == "test_func"

    def test_class_reportinfo(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
            """
            # lineno 0
            class TestClass(object):
                def test_hello(self): pass
        """
        )
        classcol = pytester.collect_by_name(modcol, "TestClass")
        assert isinstance(classcol, Class)
        path, lineno, msg = classcol.reportinfo()
        assert os.fspath(path) == str(modcol.path)
        assert lineno == 1
        assert msg == "TestClass"

    @pytest.mark.filterwarnings(
        "ignore:usage of Generator.Function is deprecated, please use pytest.Function instead"
    )
    def test_reportinfo_with_nasty_getattr(self, pytester: Pytester) -> None:
        # https://github.com/pytest-dev/pytest/issues/1204
        modcol = pytester.getmodulecol(
            """
            # lineno 0
            class TestClass:
                def __getattr__(self, name):
                    return "this is not an int"

                def __class_getattr__(cls, name):
                    return "this is not an int"

                def intest_foo(self):
                    pass

                def test_bar(self):
                    pass
        """
        )
        classcol = pytester.collect_by_name(modcol, "TestClass")
        assert isinstance(classcol, Class)
        path, lineno, msg = classcol.reportinfo()
        func = next(iter(classcol.collect()))
        assert isinstance(func, Function)
        path, lineno, msg = func.reportinfo()


def test_customized_python_discovery(pytester: Pytester) -> None:
    pytester.makeini(
        """
        [pytest]
        python_files=check_*.py
        python_classes=Check
        python_functions=check
    """
    )
    p = pytester.makepyfile(
        """
        def check_simple():
            pass
        class CheckMyApp(object):
            def check_meth(self):
                pass
    """
    )
    p2 = p.with_name(p.name.replace("test", "check"))
    p.rename(p2)
    result = pytester.runpytest("--collect-only", "-s")
    result.stdout.fnmatch_lines(
        ["*check_customized*", "*check_simple*", "*CheckMyApp*", "*check_meth*"]
    )

    result = pytester.runpytest()
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*2 passed*"])


def test_customized_python_discovery_functions(pytester: Pytester) -> None:
    pytester.makeini(
        """
        [pytest]
        python_functions=_test
    """
    )
    pytester.makepyfile(
        """
        def _test_underscore():
            pass
    """
    )
    result = pytester.runpytest("--collect-only", "-s")
    result.stdout.fnmatch_lines(["*_test_underscore*"])

    result = pytester.runpytest()
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_unorderable_types(pytester: Pytester) -> None:
    pytester.makepyfile(
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
    result = pytester.runpytest()
    result.stdout.no_fnmatch_line("*TypeError*")
    assert result.ret == ExitCode.NO_TESTS_COLLECTED


@pytest.mark.filterwarnings("default::pytest.PytestCollectionWarning")
def test_dont_collect_non_function_callable(pytester: Pytester) -> None:
    """Test for issue https://github.com/pytest-dev/pytest/issues/331

    In this case an INTERNALERROR occurred trying to report the failure of
    a test like this one because pytest failed to get the source lines.
    """
    pytester.makepyfile(
        """
        class Oh(object):
            def __call__(self):
                pass

        test_a = Oh()

        def test_real():
            pass
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*collected 1 item*",
            "*test_dont_collect_non_function_callable.py:2: *cannot collect 'test_a' because it is not a function*",
            "*1 passed, 1 warning in *",
        ]
    )


def test_class_injection_does_not_break_collection(pytester: Pytester) -> None:
    """Tests whether injection during collection time will terminate testing.

    In this case the error should not occur if the TestClass itself
    is modified during collection time, and the original method list
    is still used for collection.
    """
    pytester.makeconftest(
        """
        from test_inject import TestClass
        def pytest_generate_tests(metafunc):
            TestClass.changed_var = {}
    """
    )
    pytester.makepyfile(
        test_inject='''
         class TestClass(object):
            def test_injection(self):
                """Test being parametrized."""
                pass
    '''
    )
    result = pytester.runpytest()
    assert (
        "RuntimeError: dictionary changed size during iteration"
        not in result.stdout.str()
    )
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_syntax_error_with_non_ascii_chars(pytester: Pytester) -> None:
    """Fix decoding issue while formatting SyntaxErrors during collection (#578)."""
    pytester.makepyfile("☃")
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*ERROR collecting*", "*SyntaxError*", "*1 error in*"])


def test_collect_error_with_fulltrace(pytester: Pytester) -> None:
    pytester.makepyfile("assert 0")
    result = pytester.runpytest("--fulltrace")
    result.stdout.fnmatch_lines(
        [
            "collected 0 items / 1 error",
            "",
            "*= ERRORS =*",
            "*_ ERROR collecting test_collect_error_with_fulltrace.py _*",
            "",
            ">   assert 0",
            "E   assert 0",
            "",
            "test_collect_error_with_fulltrace.py:1: AssertionError",
            "*! Interrupted: 1 error during collection !*",
        ]
    )


def test_skip_duplicates_by_default(pytester: Pytester) -> None:
    """Test for issue https://github.com/pytest-dev/pytest/issues/1609 (#1609)

    Ignore duplicate directories.
    """
    a = pytester.mkdir("a")
    fh = a.joinpath("test_a.py")
    fh.write_text(
        textwrap.dedent(
            """\
            import pytest
            def test_real():
                pass
            """
        ),
        encoding="utf-8",
    )
    result = pytester.runpytest(str(a), str(a))
    result.stdout.fnmatch_lines(["*collected 1 item*"])


def test_keep_duplicates(pytester: Pytester) -> None:
    """Test for issue https://github.com/pytest-dev/pytest/issues/1609 (#1609)

    Use --keep-duplicates to collect tests from duplicate directories.
    """
    a = pytester.mkdir("a")
    fh = a.joinpath("test_a.py")
    fh.write_text(
        textwrap.dedent(
            """\
            import pytest
            def test_real():
                pass
            """
        ),
        encoding="utf-8",
    )
    result = pytester.runpytest("--keep-duplicates", str(a), str(a))
    result.stdout.fnmatch_lines(["*collected 2 item*"])


def test_package_collection_infinite_recursion(pytester: Pytester) -> None:
    pytester.copy_example("collect/package_infinite_recursion")
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_package_collection_init_given_as_argument(pytester: Pytester) -> None:
    """Regression test for #3749, #8976, #9263, #9313.

    Specifying an __init__.py file directly should collect only the __init__.py
    Module, not the entire package.
    """
    p = pytester.copy_example("collect/package_init_given_as_arg")
    items, hookrecorder = pytester.inline_genitems(p / "pkg" / "__init__.py")
    assert len(items) == 1
    assert items[0].name == "test_init"


def test_package_with_modules(pytester: Pytester) -> None:
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
    root = pytester.mkpydir("root")
    sub1 = root.joinpath("sub1")
    sub1_test = sub1.joinpath("sub1_1")
    sub1_test.mkdir(parents=True)
    for d in (sub1, sub1_test):
        d.joinpath("__init__.py").touch()

    sub2 = root.joinpath("sub2")
    sub2_test = sub2.joinpath("test")
    sub2_test.mkdir(parents=True)

    sub1_test.joinpath("test_in_sub1.py").write_text(
        "def test_1(): pass", encoding="utf-8"
    )
    sub2_test.joinpath("test_in_sub2.py").write_text(
        "def test_2(): pass", encoding="utf-8"
    )

    # Execute from .
    result = pytester.runpytest("-v", "-s")
    result.assert_outcomes(passed=2)

    # Execute from . with one argument "root"
    result = pytester.runpytest("-v", "-s", "root")
    result.assert_outcomes(passed=2)

    # Chdir into package's root and execute with no args
    os.chdir(root)
    result = pytester.runpytest("-v", "-s")
    result.assert_outcomes(passed=2)


def test_package_ordering(pytester: Pytester) -> None:
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
    pytester.makeini(
        """
        [pytest]
        python_files=*.py
    """
    )
    root = pytester.mkpydir("root")
    sub1 = root.joinpath("sub1")
    sub1.mkdir()
    sub1.joinpath("__init__.py").touch()
    sub2 = root.joinpath("sub2")
    sub2_test = sub2.joinpath("test")
    sub2_test.mkdir(parents=True)

    root.joinpath("Test_root.py").write_text("def test_1(): pass", encoding="utf-8")
    sub1.joinpath("Test_sub1.py").write_text("def test_2(): pass", encoding="utf-8")
    sub2_test.joinpath("test_sub2.py").write_text(
        "def test_3(): pass", encoding="utf-8"
    )

    # Execute from .
    result = pytester.runpytest("-v", "-s")
    result.assert_outcomes(passed=3)


def test_collection_hierarchy(pytester: Pytester) -> None:
    """A general test checking that a filesystem hierarchy is collected as
    expected in various scenarios.

    top/
    ├── aaa
    │   ├── pkg
    │   │   ├── __init__.py
    │   │   └── test_pkg.py
    │   └── test_aaa.py
    ├── test_a.py
    ├── test_b
    │   ├── __init__.py
    │   └── test_b.py
    ├── test_c.py
    └── zzz
        ├── dir
        │   └── test_dir.py
        ├── __init__.py
        └── test_zzz.py
    """
    pytester.makepyfile(
        **{
            "top/aaa/test_aaa.py": "def test_it(): pass",
            "top/aaa/pkg/__init__.py": "",
            "top/aaa/pkg/test_pkg.py": "def test_it(): pass",
            "top/test_a.py": "def test_it(): pass",
            "top/test_b/__init__.py": "",
            "top/test_b/test_b.py": "def test_it(): pass",
            "top/test_c.py": "def test_it(): pass",
            "top/zzz/__init__.py": "",
            "top/zzz/test_zzz.py": "def test_it(): pass",
            "top/zzz/dir/test_dir.py": "def test_it(): pass",
        }
    )

    full = [
        "<Dir test_collection_hierarchy*>",
        "  <Dir top>",
        "    <Dir aaa>",
        "      <Package pkg>",
        "        <Module test_pkg.py>",
        "          <Function test_it>",
        "      <Module test_aaa.py>",
        "        <Function test_it>",
        "    <Module test_a.py>",
        "      <Function test_it>",
        "    <Package test_b>",
        "      <Module test_b.py>",
        "        <Function test_it>",
        "    <Module test_c.py>",
        "      <Function test_it>",
        "    <Package zzz>",
        "      <Dir dir>",
        "        <Module test_dir.py>",
        "          <Function test_it>",
        "      <Module test_zzz.py>",
        "        <Function test_it>",
    ]
    result = pytester.runpytest("--collect-only")
    result.stdout.fnmatch_lines(full, consecutive=True)
    result = pytester.runpytest("top", "--collect-only")
    result.stdout.fnmatch_lines(full, consecutive=True)
    result = pytester.runpytest("top", "top", "--collect-only")
    result.stdout.fnmatch_lines(full, consecutive=True)

    result = pytester.runpytest(
        "top/aaa", "top/aaa/pkg", "--collect-only", "--keep-duplicates"
    )
    result.stdout.fnmatch_lines(
        [
            "<Dir test_collection_hierarchy*>",
            "  <Dir top>",
            "    <Dir aaa>",
            "      <Package pkg>",
            "        <Module test_pkg.py>",
            "          <Function test_it>",
            "      <Module test_aaa.py>",
            "        <Function test_it>",
            "      <Package pkg>",
            "        <Module test_pkg.py>",
            "          <Function test_it>",
        ],
        consecutive=True,
    )

    result = pytester.runpytest(
        "top/aaa/pkg", "top/aaa", "--collect-only", "--keep-duplicates"
    )
    result.stdout.fnmatch_lines(
        [
            "<Dir test_collection_hierarchy*>",
            "  <Dir top>",
            "    <Dir aaa>",
            "      <Package pkg>",
            "        <Module test_pkg.py>",
            "          <Function test_it>",
            "          <Function test_it>",
            "      <Module test_aaa.py>",
            "        <Function test_it>",
        ],
        consecutive=True,
    )
