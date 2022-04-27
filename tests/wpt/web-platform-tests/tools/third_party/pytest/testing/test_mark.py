import os
import sys
from typing import List
from typing import Optional
from unittest import mock

import pytest
from _pytest.config import ExitCode
from _pytest.mark import MarkGenerator
from _pytest.mark.structures import EMPTY_PARAMETERSET_OPTION
from _pytest.nodes import Collector
from _pytest.nodes import Node
from _pytest.pytester import Pytester


class TestMark:
    @pytest.mark.parametrize("attr", ["mark", "param"])
    def test_pytest_exists_in_namespace_all(self, attr: str) -> None:
        module = sys.modules["pytest"]
        assert attr in module.__all__  # type: ignore

    def test_pytest_mark_notcallable(self) -> None:
        mark = MarkGenerator(_ispytest=True)
        with pytest.raises(TypeError):
            mark()  # type: ignore[operator]

    def test_mark_with_param(self):
        def some_function(abc):
            pass

        class SomeClass:
            pass

        assert pytest.mark.foo(some_function) is some_function
        marked_with_args = pytest.mark.foo.with_args(some_function)
        assert marked_with_args is not some_function  # type: ignore[comparison-overlap]

        assert pytest.mark.foo(SomeClass) is SomeClass
        assert pytest.mark.foo.with_args(SomeClass) is not SomeClass  # type: ignore[comparison-overlap]

    def test_pytest_mark_name_starts_with_underscore(self) -> None:
        mark = MarkGenerator(_ispytest=True)
        with pytest.raises(AttributeError):
            mark._some_name


def test_marked_class_run_twice(pytester: Pytester) -> None:
    """Test fails file is run twice that contains marked class.
    See issue#683.
    """
    py_file = pytester.makepyfile(
        """
    import pytest
    @pytest.mark.parametrize('abc', [1, 2, 3])
    class Test1(object):
        def test_1(self, abc):
            assert abc in [1, 2, 3]
    """
    )
    file_name = os.path.basename(py_file)
    rec = pytester.inline_run(file_name, file_name)
    rec.assertoutcome(passed=6)


def test_ini_markers(pytester: Pytester) -> None:
    pytester.makeini(
        """
        [pytest]
        markers =
            a1: this is a webtest marker
            a2: this is a smoke marker
    """
    )
    pytester.makepyfile(
        """
        def test_markers(pytestconfig):
            markers = pytestconfig.getini("markers")
            print(markers)
            assert len(markers) >= 2
            assert markers[0].startswith("a1:")
            assert markers[1].startswith("a2:")
    """
    )
    rec = pytester.inline_run()
    rec.assertoutcome(passed=1)


def test_markers_option(pytester: Pytester) -> None:
    pytester.makeini(
        """
        [pytest]
        markers =
            a1: this is a webtest marker
            a1some: another marker
            nodescription
    """
    )
    result = pytester.runpytest("--markers")
    result.stdout.fnmatch_lines(
        ["*a1*this is a webtest*", "*a1some*another marker", "*nodescription*"]
    )


def test_ini_markers_whitespace(pytester: Pytester) -> None:
    pytester.makeini(
        """
        [pytest]
        markers =
            a1 : this is a whitespace marker
    """
    )
    pytester.makepyfile(
        """
        import pytest

        @pytest.mark.a1
        def test_markers():
            assert True
    """
    )
    rec = pytester.inline_run("--strict-markers", "-m", "a1")
    rec.assertoutcome(passed=1)


def test_marker_without_description(pytester: Pytester) -> None:
    pytester.makefile(
        ".cfg",
        setup="""
        [tool:pytest]
        markers=slow
    """,
    )
    pytester.makeconftest(
        """
        import pytest
        pytest.mark.xfail('FAIL')
    """
    )
    ftdir = pytester.mkdir("ft1_dummy")
    pytester.path.joinpath("conftest.py").replace(ftdir.joinpath("conftest.py"))
    rec = pytester.runpytest("--strict-markers")
    rec.assert_outcomes()


def test_markers_option_with_plugin_in_current_dir(pytester: Pytester) -> None:
    pytester.makeconftest('pytest_plugins = "flip_flop"')
    pytester.makepyfile(
        flip_flop="""\
        def pytest_configure(config):
            config.addinivalue_line("markers", "flip:flop")

        def pytest_generate_tests(metafunc):
            try:
                mark = metafunc.function.flipper
            except AttributeError:
                return
            metafunc.parametrize("x", (10, 20))"""
    )
    pytester.makepyfile(
        """\
        import pytest
        @pytest.mark.flipper
        def test_example(x):
            assert x"""
    )

    result = pytester.runpytest("--markers")
    result.stdout.fnmatch_lines(["*flip*flop*"])


def test_mark_on_pseudo_function(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest

        @pytest.mark.r(lambda x: 0/0)
        def test_hello():
            pass
    """
    )
    reprec = pytester.inline_run()
    reprec.assertoutcome(passed=1)


@pytest.mark.parametrize("option_name", ["--strict-markers", "--strict"])
def test_strict_prohibits_unregistered_markers(
    pytester: Pytester, option_name: str
) -> None:
    pytester.makepyfile(
        """
        import pytest
        @pytest.mark.unregisteredmark
        def test_hello():
            pass
    """
    )
    result = pytester.runpytest(option_name)
    assert result.ret != 0
    result.stdout.fnmatch_lines(
        ["'unregisteredmark' not found in `markers` configuration option"]
    )


@pytest.mark.parametrize(
    ("expr", "expected_passed"),
    [
        ("xyz", ["test_one"]),
        ("(((  xyz))  )", ["test_one"]),
        ("not not xyz", ["test_one"]),
        ("xyz and xyz2", []),
        ("xyz2", ["test_two"]),
        ("xyz or xyz2", ["test_one", "test_two"]),
    ],
)
def test_mark_option(
    expr: str, expected_passed: List[Optional[str]], pytester: Pytester
) -> None:
    pytester.makepyfile(
        """
        import pytest
        @pytest.mark.xyz
        def test_one():
            pass
        @pytest.mark.xyz2
        def test_two():
            pass
    """
    )
    rec = pytester.inline_run("-m", expr)
    passed, skipped, fail = rec.listoutcomes()
    passed_str = [x.nodeid.split("::")[-1] for x in passed]
    assert passed_str == expected_passed


@pytest.mark.parametrize(
    ("expr", "expected_passed"),
    [("interface", ["test_interface"]), ("not interface", ["test_nointer"])],
)
def test_mark_option_custom(
    expr: str, expected_passed: List[str], pytester: Pytester
) -> None:
    pytester.makeconftest(
        """
        import pytest
        def pytest_collection_modifyitems(items):
            for item in items:
                if "interface" in item.nodeid:
                    item.add_marker(pytest.mark.interface)
    """
    )
    pytester.makepyfile(
        """
        def test_interface():
            pass
        def test_nointer():
            pass
    """
    )
    rec = pytester.inline_run("-m", expr)
    passed, skipped, fail = rec.listoutcomes()
    passed_str = [x.nodeid.split("::")[-1] for x in passed]
    assert passed_str == expected_passed


@pytest.mark.parametrize(
    ("expr", "expected_passed"),
    [
        ("interface", ["test_interface"]),
        ("not interface", ["test_nointer", "test_pass", "test_1", "test_2"]),
        ("pass", ["test_pass"]),
        ("not pass", ["test_interface", "test_nointer", "test_1", "test_2"]),
        ("not not not (pass)", ["test_interface", "test_nointer", "test_1", "test_2"]),
        ("1 or 2", ["test_1", "test_2"]),
        ("not (1 or 2)", ["test_interface", "test_nointer", "test_pass"]),
    ],
)
def test_keyword_option_custom(
    expr: str, expected_passed: List[str], pytester: Pytester
) -> None:
    pytester.makepyfile(
        """
        def test_interface():
            pass
        def test_nointer():
            pass
        def test_pass():
            pass
        def test_1():
            pass
        def test_2():
            pass
    """
    )
    rec = pytester.inline_run("-k", expr)
    passed, skipped, fail = rec.listoutcomes()
    passed_str = [x.nodeid.split("::")[-1] for x in passed]
    assert passed_str == expected_passed


def test_keyword_option_considers_mark(pytester: Pytester) -> None:
    pytester.copy_example("marks/marks_considered_keywords")
    rec = pytester.inline_run("-k", "foo")
    passed = rec.listoutcomes()[0]
    assert len(passed) == 1


@pytest.mark.parametrize(
    ("expr", "expected_passed"),
    [
        ("None", ["test_func[None]"]),
        ("[1.3]", ["test_func[1.3]"]),
        ("2-3", ["test_func[2-3]"]),
    ],
)
def test_keyword_option_parametrize(
    expr: str, expected_passed: List[str], pytester: Pytester
) -> None:
    pytester.makepyfile(
        """
        import pytest
        @pytest.mark.parametrize("arg", [None, 1.3, "2-3"])
        def test_func(arg):
            pass
    """
    )
    rec = pytester.inline_run("-k", expr)
    passed, skipped, fail = rec.listoutcomes()
    passed_str = [x.nodeid.split("::")[-1] for x in passed]
    assert passed_str == expected_passed


def test_parametrize_with_module(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest
        @pytest.mark.parametrize("arg", [pytest,])
        def test_func(arg):
            pass
    """
    )
    rec = pytester.inline_run()
    passed, skipped, fail = rec.listoutcomes()
    expected_id = "test_func[" + pytest.__name__ + "]"
    assert passed[0].nodeid.split("::")[-1] == expected_id


@pytest.mark.parametrize(
    ("expr", "expected_error"),
    [
        (
            "foo or",
            "at column 7: expected not OR left parenthesis OR identifier; got end of input",
        ),
        (
            "foo or or",
            "at column 8: expected not OR left parenthesis OR identifier; got or",
        ),
        (
            "(foo",
            "at column 5: expected right parenthesis; got end of input",
        ),
        (
            "foo bar",
            "at column 5: expected end of input; got identifier",
        ),
        (
            "or or",
            "at column 1: expected not OR left parenthesis OR identifier; got or",
        ),
        (
            "not or",
            "at column 5: expected not OR left parenthesis OR identifier; got or",
        ),
    ],
)
def test_keyword_option_wrong_arguments(
    expr: str, expected_error: str, pytester: Pytester, capsys
) -> None:
    pytester.makepyfile(
        """
            def test_func(arg):
                pass
        """
    )
    pytester.inline_run("-k", expr)
    err = capsys.readouterr().err
    assert expected_error in err


def test_parametrized_collected_from_command_line(pytester: Pytester) -> None:
    """Parametrized test not collected if test named specified in command
    line issue#649."""
    py_file = pytester.makepyfile(
        """
        import pytest
        @pytest.mark.parametrize("arg", [None, 1.3, "2-3"])
        def test_func(arg):
            pass
    """
    )
    file_name = os.path.basename(py_file)
    rec = pytester.inline_run(file_name + "::" + "test_func")
    rec.assertoutcome(passed=3)


def test_parametrized_collect_with_wrong_args(pytester: Pytester) -> None:
    """Test collect parametrized func with wrong number of args."""
    py_file = pytester.makepyfile(
        """
        import pytest

        @pytest.mark.parametrize('foo, bar', [(1, 2, 3)])
        def test_func(foo, bar):
            pass
    """
    )

    result = pytester.runpytest(py_file)
    result.stdout.fnmatch_lines(
        [
            'test_parametrized_collect_with_wrong_args.py::test_func: in "parametrize" the number of names (2):',
            "  ['foo', 'bar']",
            "must be equal to the number of values (3):",
            "  (1, 2, 3)",
        ]
    )


def test_parametrized_with_kwargs(pytester: Pytester) -> None:
    """Test collect parametrized func with wrong number of args."""
    py_file = pytester.makepyfile(
        """
        import pytest

        @pytest.fixture(params=[1,2])
        def a(request):
            return request.param

        @pytest.mark.parametrize(argnames='b', argvalues=[1, 2])
        def test_func(a, b):
            pass
    """
    )

    result = pytester.runpytest(py_file)
    assert result.ret == 0


def test_parametrize_iterator(pytester: Pytester) -> None:
    """`parametrize` should work with generators (#5354)."""
    py_file = pytester.makepyfile(
        """\
        import pytest

        def gen():
            yield 1
            yield 2
            yield 3

        @pytest.mark.parametrize('a', gen())
        def test(a):
            assert a >= 1
        """
    )
    result = pytester.runpytest(py_file)
    assert result.ret == 0
    # should not skip any tests
    result.stdout.fnmatch_lines(["*3 passed*"])


class TestFunctional:
    def test_merging_markers_deep(self, pytester: Pytester) -> None:
        # issue 199 - propagate markers into nested classes
        p = pytester.makepyfile(
            """
            import pytest
            class TestA(object):
                pytestmark = pytest.mark.a
                def test_b(self):
                    assert True
                class TestC(object):
                    # this one didn't get marked
                    def test_d(self):
                        assert True
        """
        )
        items, rec = pytester.inline_genitems(p)
        for item in items:
            print(item, item.keywords)
            assert [x for x in item.iter_markers() if x.name == "a"]

    def test_mark_decorator_subclass_does_not_propagate_to_base(
        self, pytester: Pytester
    ) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.a
            class Base(object): pass

            @pytest.mark.b
            class Test1(Base):
                def test_foo(self): pass

            class Test2(Base):
                def test_bar(self): pass
        """
        )
        items, rec = pytester.inline_genitems(p)
        self.assert_markers(items, test_foo=("a", "b"), test_bar=("a",))

    def test_mark_should_not_pass_to_siebling_class(self, pytester: Pytester) -> None:
        """#568"""
        p = pytester.makepyfile(
            """
            import pytest

            class TestBase(object):
                def test_foo(self):
                    pass

            @pytest.mark.b
            class TestSub(TestBase):
                pass


            class TestOtherSub(TestBase):
                pass

        """
        )
        items, rec = pytester.inline_genitems(p)
        base_item, sub_item, sub_item_other = items
        print(items, [x.nodeid for x in items])
        # new api segregates
        assert not list(base_item.iter_markers(name="b"))
        assert not list(sub_item_other.iter_markers(name="b"))
        assert list(sub_item.iter_markers(name="b"))

    def test_mark_decorator_baseclasses_merged(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.a
            class Base(object): pass

            @pytest.mark.b
            class Base2(Base): pass

            @pytest.mark.c
            class Test1(Base2):
                def test_foo(self): pass

            class Test2(Base2):
                @pytest.mark.d
                def test_bar(self): pass
        """
        )
        items, rec = pytester.inline_genitems(p)
        self.assert_markers(items, test_foo=("a", "b", "c"), test_bar=("a", "b", "d"))

    def test_mark_closest(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.c(location="class")
            class Test:
                @pytest.mark.c(location="function")
                def test_has_own(self):
                    pass

                def test_has_inherited(self):
                    pass

        """
        )
        items, rec = pytester.inline_genitems(p)
        has_own, has_inherited = items
        has_own_marker = has_own.get_closest_marker("c")
        has_inherited_marker = has_inherited.get_closest_marker("c")
        assert has_own_marker is not None
        assert has_inherited_marker is not None
        assert has_own_marker.kwargs == {"location": "function"}
        assert has_inherited_marker.kwargs == {"location": "class"}
        assert has_own.get_closest_marker("missing") is None

    def test_mark_with_wrong_marker(self, pytester: Pytester) -> None:
        reprec = pytester.inline_runsource(
            """
                import pytest
                class pytestmark(object):
                    pass
                def test_func():
                    pass
        """
        )
        values = reprec.getfailedcollections()
        assert len(values) == 1
        assert "TypeError" in str(values[0].longrepr)

    def test_mark_dynamically_in_funcarg(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            @pytest.fixture
            def arg(request):
                request.applymarker(pytest.mark.hello)
            def pytest_terminal_summary(terminalreporter):
                values = terminalreporter.stats['passed']
                terminalreporter._tw.line("keyword: %s" % values[0].keywords)
        """
        )
        pytester.makepyfile(
            """
            def test_func(arg):
                pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["keyword: *hello*"])

    def test_no_marker_match_on_unmarked_names(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest
            @pytest.mark.shouldmatch
            def test_marked():
                assert 1

            def test_unmarked():
                assert 1
        """
        )
        reprec = pytester.inline_run("-m", "test_unmarked", p)
        passed, skipped, failed = reprec.listoutcomes()
        assert len(passed) + len(skipped) + len(failed) == 0
        dlist = reprec.getcalls("pytest_deselected")
        deselected_tests = dlist[0].items
        assert len(deselected_tests) == 2

    def test_keywords_at_node_level(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.fixture(scope="session", autouse=True)
            def some(request):
                request.keywords["hello"] = 42
                assert "world" not in request.keywords

            @pytest.fixture(scope="function", autouse=True)
            def funcsetup(request):
                assert "world" in request.keywords
                assert "hello" in  request.keywords

            @pytest.mark.world
            def test_function():
                pass
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_keyword_added_for_session(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            def pytest_collection_modifyitems(session):
                session.add_marker("mark1")
                session.add_marker(pytest.mark.mark2)
                session.add_marker(pytest.mark.mark3)
                pytest.raises(ValueError, lambda:
                        session.add_marker(10))
        """
        )
        pytester.makepyfile(
            """
            def test_some(request):
                assert "mark1" in request.keywords
                assert "mark2" in request.keywords
                assert "mark3" in request.keywords
                assert 10 not in request.keywords
                marker = request.node.get_closest_marker("mark1")
                assert marker.name == "mark1"
                assert marker.args == ()
                assert marker.kwargs == {}
        """
        )
        reprec = pytester.inline_run("-m", "mark1")
        reprec.assertoutcome(passed=1)

    def assert_markers(self, items, **expected) -> None:
        """Assert that given items have expected marker names applied to them.
        expected should be a dict of (item name -> seq of expected marker names).

        Note: this could be moved to ``pytester`` if proven to be useful
        to other modules.
        """
        items = {x.name: x for x in items}
        for name, expected_markers in expected.items():
            markers = {m.name for m in items[name].iter_markers()}
            assert markers == set(expected_markers)

    @pytest.mark.filterwarnings("ignore")
    def test_mark_from_parameters(self, pytester: Pytester) -> None:
        """#1540"""
        pytester.makepyfile(
            """
            import pytest

            pytestmark = pytest.mark.skipif(True, reason='skip all')

            # skipifs inside fixture params
            params = [pytest.mark.skipif(False, reason='dont skip')('parameter')]


            @pytest.fixture(params=params)
            def parameter(request):
                return request.param


            def test_1(parameter):
                assert True
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(skipped=1)

    def test_reevaluate_dynamic_expr(self, pytester: Pytester) -> None:
        """#7360"""
        py_file1 = pytester.makepyfile(
            test_reevaluate_dynamic_expr1="""
            import pytest

            skip = True

            @pytest.mark.skipif("skip")
            def test_should_skip():
                assert True
        """
        )
        py_file2 = pytester.makepyfile(
            test_reevaluate_dynamic_expr2="""
            import pytest

            skip = False

            @pytest.mark.skipif("skip")
            def test_should_not_skip():
                assert True
        """
        )

        file_name1 = os.path.basename(py_file1)
        file_name2 = os.path.basename(py_file2)
        reprec = pytester.inline_run(file_name1, file_name2)
        reprec.assertoutcome(passed=1, skipped=1)


class TestKeywordSelection:
    def test_select_simple(self, pytester: Pytester) -> None:
        file_test = pytester.makepyfile(
            """
            def test_one():
                assert 0
            class TestClass(object):
                def test_method_one(self):
                    assert 42 == 43
        """
        )

        def check(keyword, name):
            reprec = pytester.inline_run("-s", "-k", keyword, file_test)
            passed, skipped, failed = reprec.listoutcomes()
            assert len(failed) == 1
            assert failed[0].nodeid.split("::")[-1] == name
            assert len(reprec.getcalls("pytest_deselected")) == 1

        for keyword in ["test_one", "est_on"]:
            check(keyword, "test_one")
        check("TestClass and test", "test_method_one")

    @pytest.mark.parametrize(
        "keyword",
        [
            "xxx",
            "xxx and test_2",
            "TestClass",
            "xxx and not test_1",
            "TestClass and test_2",
            "xxx and TestClass and test_2",
        ],
    )
    def test_select_extra_keywords(self, pytester: Pytester, keyword) -> None:
        p = pytester.makepyfile(
            test_select="""
            def test_1():
                pass
            class TestClass(object):
                def test_2(self):
                    pass
        """
        )
        pytester.makepyfile(
            conftest="""
            import pytest
            @pytest.hookimpl(hookwrapper=True)
            def pytest_pycollect_makeitem(name):
                outcome = yield
                if name == "TestClass":
                    item = outcome.get_result()
                    item.extra_keyword_matches.add("xxx")
        """
        )
        reprec = pytester.inline_run(p.parent, "-s", "-k", keyword)
        print("keyword", repr(keyword))
        passed, skipped, failed = reprec.listoutcomes()
        assert len(passed) == 1
        assert passed[0].nodeid.endswith("test_2")
        dlist = reprec.getcalls("pytest_deselected")
        assert len(dlist) == 1
        assert dlist[0].items[0].name == "test_1"

    def test_select_starton(self, pytester: Pytester) -> None:
        threepass = pytester.makepyfile(
            test_threepass="""
            def test_one(): assert 1
            def test_two(): assert 1
            def test_three(): assert 1
        """
        )
        reprec = pytester.inline_run(
            "-Wignore::pytest.PytestRemovedIn7Warning", "-k", "test_two:", threepass
        )
        passed, skipped, failed = reprec.listoutcomes()
        assert len(passed) == 2
        assert not failed
        dlist = reprec.getcalls("pytest_deselected")
        assert len(dlist) == 1
        item = dlist[0].items[0]
        assert item.name == "test_one"

    def test_keyword_extra(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
           def test_one():
               assert 0
           test_one.mykeyword = True
        """
        )
        reprec = pytester.inline_run("-k", "mykeyword", p)
        passed, skipped, failed = reprec.countoutcomes()
        assert failed == 1

    @pytest.mark.xfail
    def test_keyword_extra_dash(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
           def test_one():
               assert 0
           test_one.mykeyword = True
        """
        )
        # with argparse the argument to an option cannot
        # start with '-'
        reprec = pytester.inline_run("-k", "-mykeyword", p)
        passed, skipped, failed = reprec.countoutcomes()
        assert passed + skipped + failed == 0

    @pytest.mark.parametrize(
        "keyword",
        ["__", "+", ".."],
    )
    def test_no_magic_values(self, pytester: Pytester, keyword: str) -> None:
        """Make sure the tests do not match on magic values,
        no double underscored values, like '__dict__' and '+'.
        """
        p = pytester.makepyfile(
            """
            def test_one(): assert 1
        """
        )

        reprec = pytester.inline_run("-k", keyword, p)
        passed, skipped, failed = reprec.countoutcomes()
        dlist = reprec.getcalls("pytest_deselected")
        assert passed + skipped + failed == 0
        deselected_tests = dlist[0].items
        assert len(deselected_tests) == 1

    def test_no_match_directories_outside_the_suite(self, pytester: Pytester) -> None:
        """`-k` should not match against directories containing the test suite (#7040)."""
        test_contents = """
            def test_aaa(): pass
            def test_ddd(): pass
        """
        pytester.makepyfile(
            **{"ddd/tests/__init__.py": "", "ddd/tests/test_foo.py": test_contents}
        )

        def get_collected_names(*args):
            _, rec = pytester.inline_genitems(*args)
            calls = rec.getcalls("pytest_collection_finish")
            assert len(calls) == 1
            return [x.name for x in calls[0].session.items]

        # sanity check: collect both tests in normal runs
        assert get_collected_names() == ["test_aaa", "test_ddd"]

        # do not collect anything based on names outside the collection tree
        assert get_collected_names("-k", pytester._name) == []

        # "-k ddd" should only collect "test_ddd", but not
        # 'test_aaa' just because one of its parent directories is named "ddd";
        # this was matched previously because Package.name would contain the full path
        # to the package
        assert get_collected_names("-k", "ddd") == ["test_ddd"]


class TestMarkDecorator:
    @pytest.mark.parametrize(
        "lhs, rhs, expected",
        [
            (pytest.mark.foo(), pytest.mark.foo(), True),
            (pytest.mark.foo(), pytest.mark.bar(), False),
            (pytest.mark.foo(), "bar", False),
            ("foo", pytest.mark.bar(), False),
        ],
    )
    def test__eq__(self, lhs, rhs, expected) -> None:
        assert (lhs == rhs) == expected

    def test_aliases(self) -> None:
        md = pytest.mark.foo(1, "2", three=3)
        assert md.name == "foo"
        assert md.args == (1, "2")
        assert md.kwargs == {"three": 3}


@pytest.mark.parametrize("mark", [None, "", "skip", "xfail"])
def test_parameterset_for_parametrize_marks(
    pytester: Pytester, mark: Optional[str]
) -> None:
    if mark is not None:
        pytester.makeini(
            """
        [pytest]
        {}={}
        """.format(
                EMPTY_PARAMETERSET_OPTION, mark
            )
        )

    config = pytester.parseconfig()
    from _pytest.mark import pytest_configure, get_empty_parameterset_mark

    pytest_configure(config)
    result_mark = get_empty_parameterset_mark(config, ["a"], all)
    if mark in (None, ""):
        # normalize to the requested name
        mark = "skip"
    assert result_mark.name == mark
    assert result_mark.kwargs["reason"].startswith("got empty parameter set ")
    if mark == "xfail":
        assert result_mark.kwargs.get("run") is False


def test_parameterset_for_fail_at_collect(pytester: Pytester) -> None:
    pytester.makeini(
        """
    [pytest]
    {}=fail_at_collect
    """.format(
            EMPTY_PARAMETERSET_OPTION
        )
    )

    config = pytester.parseconfig()
    from _pytest.mark import pytest_configure, get_empty_parameterset_mark

    pytest_configure(config)

    with pytest.raises(
        Collector.CollectError,
        match=r"Empty parameter set in 'pytest_configure' at line \d\d+",
    ):
        get_empty_parameterset_mark(config, ["a"], pytest_configure)

    p1 = pytester.makepyfile(
        """
        import pytest

        @pytest.mark.parametrize("empty", [])
        def test():
            pass
        """
    )
    result = pytester.runpytest(str(p1))
    result.stdout.fnmatch_lines(
        [
            "collected 0 items / 1 error",
            "* ERROR collecting test_parameterset_for_fail_at_collect.py *",
            "Empty parameter set in 'test' at line 3",
            "*= 1 error in *",
        ]
    )
    assert result.ret == ExitCode.INTERRUPTED


def test_parameterset_for_parametrize_bad_markname(pytester: Pytester) -> None:
    with pytest.raises(pytest.UsageError):
        test_parameterset_for_parametrize_marks(pytester, "bad")


def test_mark_expressions_no_smear(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest

        class BaseTests(object):
            def test_something(self):
                pass

        @pytest.mark.FOO
        class TestFooClass(BaseTests):
            pass

        @pytest.mark.BAR
        class TestBarClass(BaseTests):
            pass
    """
    )

    reprec = pytester.inline_run("-m", "FOO")
    passed, skipped, failed = reprec.countoutcomes()
    dlist = reprec.getcalls("pytest_deselected")
    assert passed == 1
    assert skipped == failed == 0
    deselected_tests = dlist[0].items
    assert len(deselected_tests) == 1

    # todo: fixed
    # keywords smear - expected behaviour
    # reprec_keywords = pytester.inline_run("-k", "FOO")
    # passed_k, skipped_k, failed_k = reprec_keywords.countoutcomes()
    # assert passed_k == 2
    # assert skipped_k == failed_k == 0


def test_addmarker_order(pytester) -> None:
    session = mock.Mock()
    session.own_markers = []
    session.parent = None
    session.nodeid = ""
    session.path = pytester.path
    node = Node.from_parent(session, name="Test")
    node.add_marker("foo")
    node.add_marker("bar")
    node.add_marker("baz", append=False)
    extracted = [x.name for x in node.iter_markers()]
    assert extracted == ["baz", "foo", "bar"]


@pytest.mark.filterwarnings("ignore")
def test_markers_from_parametrize(pytester: Pytester) -> None:
    """#3605"""
    pytester.makepyfile(
        """
        import pytest

        first_custom_mark = pytest.mark.custom_marker
        custom_mark = pytest.mark.custom_mark
        @pytest.fixture(autouse=True)
        def trigger(request):
            custom_mark = list(request.node.iter_markers('custom_mark'))
            print("Custom mark %s" % custom_mark)

        @custom_mark("custom mark non parametrized")
        def test_custom_mark_non_parametrized():
            print("Hey from test")

        @pytest.mark.parametrize(
            "obj_type",
            [
                first_custom_mark("first custom mark")("template"),
                pytest.param( # Think this should be recommended way?
                    "disk",
                    marks=custom_mark('custom mark1')
                ),
                custom_mark("custom mark2")("vm"),  # Tried also this
            ]
        )
        def test_custom_mark_parametrized(obj_type):
            print("obj_type is:", obj_type)
    """
    )

    result = pytester.runpytest()
    result.assert_outcomes(passed=4)


def test_pytest_param_id_requires_string() -> None:
    with pytest.raises(TypeError) as excinfo:
        pytest.param(id=True)  # type: ignore[arg-type]
    (msg,) = excinfo.value.args
    assert msg == "Expected id to be a string, got <class 'bool'>: True"


@pytest.mark.parametrize("s", (None, "hello world"))
def test_pytest_param_id_allows_none_or_string(s) -> None:
    assert pytest.param(id=s)


@pytest.mark.parametrize("expr", ("NOT internal_err", "NOT (internal_err)", "bogus="))
def test_marker_expr_eval_failure_handling(pytester: Pytester, expr) -> None:
    foo = pytester.makepyfile(
        """
        import pytest

        @pytest.mark.internal_err
        def test_foo():
            pass
        """
    )
    expected = f"ERROR: Wrong expression passed to '-m': {expr}: *"
    result = pytester.runpytest(foo, "-m", expr)
    result.stderr.fnmatch_lines([expected])
    assert result.ret == ExitCode.USAGE_ERROR
