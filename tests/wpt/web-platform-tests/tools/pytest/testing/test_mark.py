import os

import py, pytest
from _pytest.mark import MarkGenerator as Mark

class TestMark:
    def test_markinfo_repr(self):
        from _pytest.mark import MarkInfo
        m = MarkInfo("hello", (1,2), {})
        repr(m)

    def test_pytest_exists_in_namespace_all(self):
        assert 'mark' in py.test.__all__
        assert 'mark' in pytest.__all__

    def test_pytest_mark_notcallable(self):
        mark = Mark()
        pytest.raises((AttributeError, TypeError), mark)

    def test_pytest_mark_name_starts_with_underscore(self):
        mark = Mark()
        pytest.raises(AttributeError, getattr, mark, '_some_name')

    def test_pytest_mark_bare(self):
        mark = Mark()
        def f():
            pass
        mark.hello(f)
        assert f.hello

    def test_pytest_mark_keywords(self):
        mark = Mark()
        def f():
            pass
        mark.world(x=3, y=4)(f)
        assert f.world
        assert f.world.kwargs['x'] == 3
        assert f.world.kwargs['y'] == 4

    def test_apply_multiple_and_merge(self):
        mark = Mark()
        def f():
            pass
        mark.world
        mark.world(x=3)(f)
        assert f.world.kwargs['x'] == 3
        mark.world(y=4)(f)
        assert f.world.kwargs['x'] == 3
        assert f.world.kwargs['y'] == 4
        mark.world(y=1)(f)
        assert f.world.kwargs['y'] == 1
        assert len(f.world.args) == 0

    def test_pytest_mark_positional(self):
        mark = Mark()
        def f():
            pass
        mark.world("hello")(f)
        assert f.world.args[0] == "hello"
        mark.world("world")(f)

    def test_pytest_mark_positional_func_and_keyword(self):
        mark = Mark()
        def f():
            raise Exception
        m = mark.world(f, omega="hello")
        def g():
            pass
        assert m(g) == g
        assert g.world.args[0] is f
        assert g.world.kwargs["omega"] == "hello"

    def test_pytest_mark_reuse(self):
        mark = Mark()
        def f():
            pass
        w = mark.some
        w("hello", reason="123")(f)
        assert f.some.args[0] == "hello"
        assert f.some.kwargs['reason'] == "123"
        def g():
            pass
        w("world", reason2="456")(g)
        assert g.some.args[0] == "world"
        assert 'reason' not in g.some.kwargs
        assert g.some.kwargs['reason2'] == "456"


def test_marked_class_run_twice(testdir, request):
    """Test fails file is run twice that contains marked class.
    See issue#683.
    """
    py_file = testdir.makepyfile("""
    import pytest
    @pytest.mark.parametrize('abc', [1, 2, 3])
    class Test1(object):
        def test_1(self, abc):
            assert abc in [1, 2, 3]
    """)
    file_name = os.path.basename(py_file.strpath)
    rec = testdir.inline_run(file_name, file_name)
    rec.assertoutcome(passed=6)


def test_ini_markers(testdir):
    testdir.makeini("""
        [pytest]
        markers =
            a1: this is a webtest marker
            a2: this is a smoke marker
    """)
    testdir.makepyfile("""
        def test_markers(pytestconfig):
            markers = pytestconfig.getini("markers")
            print (markers)
            assert len(markers) >= 2
            assert markers[0].startswith("a1:")
            assert markers[1].startswith("a2:")
    """)
    rec = testdir.inline_run()
    rec.assertoutcome(passed=1)

def test_markers_option(testdir):
    testdir.makeini("""
        [pytest]
        markers =
            a1: this is a webtest marker
            a1some: another marker
    """)
    result = testdir.runpytest("--markers", )
    result.stdout.fnmatch_lines([
        "*a1*this is a webtest*",
        "*a1some*another marker",
    ])

def test_markers_option_with_plugin_in_current_dir(testdir):
    testdir.makeconftest('pytest_plugins = "flip_flop"')
    testdir.makepyfile(flip_flop="""\
        def pytest_configure(config):
            config.addinivalue_line("markers", "flip:flop")

        def pytest_generate_tests(metafunc):
            try:
                mark = metafunc.function.flipper
            except AttributeError:
                return
            metafunc.parametrize("x", (10, 20))""")
    testdir.makepyfile("""\
        import pytest
        @pytest.mark.flipper
        def test_example(x):
            assert x""")

    result = testdir.runpytest("--markers")
    result.stdout.fnmatch_lines(["*flip*flop*"])


def test_mark_on_pseudo_function(testdir):
    testdir.makepyfile("""
        import pytest

        @pytest.mark.r(lambda x: 0/0)
        def test_hello():
            pass
    """)
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)

def test_strict_prohibits_unregistered_markers(testdir):
    testdir.makepyfile("""
        import pytest
        @pytest.mark.unregisteredmark
        def test_hello():
            pass
    """)
    result = testdir.runpytest("--strict")
    assert result.ret != 0
    result.stdout.fnmatch_lines([
        "*unregisteredmark*not*registered*",
    ])

@pytest.mark.parametrize("spec", [
        ("xyz", ("test_one",)),
        ("xyz and xyz2", ()),
        ("xyz2", ("test_two",)),
        ("xyz or xyz2", ("test_one", "test_two"),)
])
def test_mark_option(spec, testdir):
    testdir.makepyfile("""
        import pytest
        @pytest.mark.xyz
        def test_one():
            pass
        @pytest.mark.xyz2
        def test_two():
            pass
    """)
    opt, passed_result = spec
    rec = testdir.inline_run("-m", opt)
    passed, skipped, fail = rec.listoutcomes()
    passed = [x.nodeid.split("::")[-1] for x in passed]
    assert len(passed) == len(passed_result)
    assert list(passed) == list(passed_result)

@pytest.mark.parametrize("spec", [
        ("interface", ("test_interface",)),
        ("not interface", ("test_nointer",)),
])
def test_mark_option_custom(spec, testdir):
    testdir.makeconftest("""
        import pytest
        def pytest_collection_modifyitems(items):
            for item in items:
                if "interface" in item.nodeid:
                    item.keywords["interface"] = pytest.mark.interface
    """)
    testdir.makepyfile("""
        def test_interface():
            pass
        def test_nointer():
            pass
    """)
    opt, passed_result = spec
    rec = testdir.inline_run("-m", opt)
    passed, skipped, fail = rec.listoutcomes()
    passed = [x.nodeid.split("::")[-1] for x in passed]
    assert len(passed) == len(passed_result)
    assert list(passed) == list(passed_result)

@pytest.mark.parametrize("spec", [
        ("interface", ("test_interface",)),
        ("not interface", ("test_nointer", "test_pass")),
        ("pass", ("test_pass",)),
        ("not pass", ("test_interface", "test_nointer")),
])
def test_keyword_option_custom(spec, testdir):
    testdir.makepyfile("""
        def test_interface():
            pass
        def test_nointer():
            pass
        def test_pass():
            pass
    """)
    opt, passed_result = spec
    rec = testdir.inline_run("-k", opt)
    passed, skipped, fail = rec.listoutcomes()
    passed = [x.nodeid.split("::")[-1] for x in passed]
    assert len(passed) == len(passed_result)
    assert list(passed) == list(passed_result)


@pytest.mark.parametrize("spec", [
        ("None", ("test_func[None]",)),
        ("1.3", ("test_func[1.3]",)),
        ("2-3", ("test_func[2-3]",))
])
def test_keyword_option_parametrize(spec, testdir):
    testdir.makepyfile("""
        import pytest
        @pytest.mark.parametrize("arg", [None, 1.3, "2-3"])
        def test_func(arg):
            pass
    """)
    opt, passed_result = spec
    rec = testdir.inline_run("-k", opt)
    passed, skipped, fail = rec.listoutcomes()
    passed = [x.nodeid.split("::")[-1] for x in passed]
    assert len(passed) == len(passed_result)
    assert list(passed) == list(passed_result)


def test_parametrized_collected_from_command_line(testdir):
    """Parametrized test not collected if test named specified
       in command line issue#649.
    """
    py_file = testdir.makepyfile("""
        import pytest
        @pytest.mark.parametrize("arg", [None, 1.3, "2-3"])
        def test_func(arg):
            pass
    """)
    file_name = os.path.basename(py_file.strpath)
    rec = testdir.inline_run(file_name + "::" + "test_func")
    rec.assertoutcome(passed=3)


class TestFunctional:

    def test_mark_per_function(self, testdir):
        p = testdir.makepyfile("""
            import pytest
            @pytest.mark.hello
            def test_hello():
                assert hasattr(test_hello, 'hello')
        """)
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_mark_per_module(self, testdir):
        item = testdir.getitem("""
            import pytest
            pytestmark = pytest.mark.hello
            def test_func():
                pass
        """)
        keywords = item.keywords
        assert 'hello' in keywords

    def test_marklist_per_class(self, testdir):
        item = testdir.getitem("""
            import pytest
            class TestClass:
                pytestmark = [pytest.mark.hello, pytest.mark.world]
                def test_func(self):
                    assert TestClass.test_func.hello
                    assert TestClass.test_func.world
        """)
        keywords = item.keywords
        assert 'hello' in keywords

    def test_marklist_per_module(self, testdir):
        item = testdir.getitem("""
            import pytest
            pytestmark = [pytest.mark.hello, pytest.mark.world]
            class TestClass:
                def test_func(self):
                    assert TestClass.test_func.hello
                    assert TestClass.test_func.world
        """)
        keywords = item.keywords
        assert 'hello' in keywords
        assert 'world' in keywords

    def test_mark_per_class_decorator(self, testdir):
        item = testdir.getitem("""
            import pytest
            @pytest.mark.hello
            class TestClass:
                def test_func(self):
                    assert TestClass.test_func.hello
        """)
        keywords = item.keywords
        assert 'hello' in keywords

    def test_mark_per_class_decorator_plus_existing_dec(self, testdir):
        item = testdir.getitem("""
            import pytest
            @pytest.mark.hello
            class TestClass:
                pytestmark = pytest.mark.world
                def test_func(self):
                    assert TestClass.test_func.hello
                    assert TestClass.test_func.world
        """)
        keywords = item.keywords
        assert 'hello' in keywords
        assert 'world' in keywords

    def test_merging_markers(self, testdir):
        p = testdir.makepyfile("""
            import pytest
            pytestmark = pytest.mark.hello("pos1", x=1, y=2)
            class TestClass:
                # classlevel overrides module level
                pytestmark = pytest.mark.hello(x=3)
                @pytest.mark.hello("pos0", z=4)
                def test_func(self):
                    pass
        """)
        items, rec = testdir.inline_genitems(p)
        item, = items
        keywords = item.keywords
        marker = keywords['hello']
        assert marker.args == ("pos0", "pos1")
        assert marker.kwargs == {'x': 1, 'y': 2, 'z': 4}

        # test the new __iter__ interface
        l = list(marker)
        assert len(l) == 3
        assert l[0].args == ("pos0",)
        assert l[1].args == ()
        assert l[2].args == ("pos1", )

    @pytest.mark.xfail(reason='unfixed')
    def test_merging_markers_deep(self, testdir):
        # issue 199 - propagate markers into nested classes
        p = testdir.makepyfile("""
            import pytest
            class TestA:
                pytestmark = pytest.mark.a
                def test_b(self):
                    assert True
                class TestC:
                    # this one didnt get marked
                    def test_d(self):
                        assert True
        """)
        items, rec = testdir.inline_genitems(p)
        for item in items:
            print (item, item.keywords)
            assert 'a' in item.keywords

    def test_mark_decorator_subclass_does_not_propagate_to_base(self, testdir):
        p = testdir.makepyfile("""
            import pytest

            @pytest.mark.a
            class Base: pass

            @pytest.mark.b
            class Test1(Base):
                def test_foo(self): pass

            class Test2(Base):
                def test_bar(self): pass
        """)
        items, rec = testdir.inline_genitems(p)
        self.assert_markers(items, test_foo=('a', 'b'), test_bar=('a',))

    def test_mark_decorator_baseclasses_merged(self, testdir):
        p = testdir.makepyfile("""
            import pytest

            @pytest.mark.a
            class Base: pass

            @pytest.mark.b
            class Base2(Base): pass

            @pytest.mark.c
            class Test1(Base2):
                def test_foo(self): pass

            class Test2(Base2):
                @pytest.mark.d
                def test_bar(self): pass
        """)
        items, rec = testdir.inline_genitems(p)
        self.assert_markers(items, test_foo=('a', 'b', 'c'),
                            test_bar=('a', 'b', 'd'))

    def test_mark_with_wrong_marker(self, testdir):
        reprec = testdir.inline_runsource("""
                import pytest
                class pytestmark:
                    pass
                def test_func():
                    pass
        """)
        l = reprec.getfailedcollections()
        assert len(l) == 1
        assert "TypeError" in str(l[0].longrepr)

    def test_mark_dynamically_in_funcarg(self, testdir):
        testdir.makeconftest("""
            import pytest
            def pytest_funcarg__arg(request):
                request.applymarker(pytest.mark.hello)
            def pytest_terminal_summary(terminalreporter):
                l = terminalreporter.stats['passed']
                terminalreporter.writer.line("keyword: %s" % l[0].keywords)
        """)
        testdir.makepyfile("""
            def test_func(arg):
                pass
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "keyword: *hello*"
        ])

    def test_merging_markers_two_functions(self, testdir):
        p = testdir.makepyfile("""
            import pytest
            @pytest.mark.hello("pos1", z=4)
            @pytest.mark.hello("pos0", z=3)
            def test_func():
                pass
        """)
        items, rec = testdir.inline_genitems(p)
        item, = items
        keywords = item.keywords
        marker = keywords['hello']
        l = list(marker)
        assert len(l) == 2
        assert l[0].args == ("pos0",)
        assert l[1].args == ("pos1",)

    def test_no_marker_match_on_unmarked_names(self, testdir):
        p = testdir.makepyfile("""
            import pytest
            @pytest.mark.shouldmatch
            def test_marked():
                assert 1

            def test_unmarked():
                assert 1
        """)
        reprec = testdir.inline_run("-m", "test_unmarked", p)
        passed, skipped, failed = reprec.listoutcomes()
        assert len(passed) + len(skipped) + len(failed) == 0
        dlist = reprec.getcalls("pytest_deselected")
        deselected_tests = dlist[0].items
        assert len(deselected_tests) == 2

    def test_keywords_at_node_level(self, testdir):
        testdir.makepyfile("""
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
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_keyword_added_for_session(self, testdir):
        testdir.makeconftest("""
            import pytest
            def pytest_collection_modifyitems(session):
                session.add_marker("mark1")
                session.add_marker(pytest.mark.mark2)
                session.add_marker(pytest.mark.mark3)
                pytest.raises(ValueError, lambda:
                        session.add_marker(10))
        """)
        testdir.makepyfile("""
            def test_some(request):
                assert "mark1" in request.keywords
                assert "mark2" in request.keywords
                assert "mark3" in request.keywords
                assert 10 not in request.keywords
                marker = request.node.get_marker("mark1")
                assert marker.name == "mark1"
                assert marker.args == ()
                assert marker.kwargs == {}
        """)
        reprec = testdir.inline_run("-m", "mark1")
        reprec.assertoutcome(passed=1)

    def assert_markers(self, items, **expected):
        """assert that given items have expected marker names applied to them.
        expected should be a dict of (item name -> seq of expected marker names)

        .. note:: this could be moved to ``testdir`` if proven to be useful
        to other modules.
        """
        from _pytest.mark import MarkInfo
        items = dict((x.name, x) for x in items)
        for name, expected_markers in expected.items():
            markers = items[name].keywords._markers
            marker_names = set([name for (name, v) in markers.items()
                                if isinstance(v, MarkInfo)])
            assert marker_names == set(expected_markers)


class TestKeywordSelection:
    def test_select_simple(self, testdir):
        file_test = testdir.makepyfile("""
            def test_one():
                assert 0
            class TestClass(object):
                def test_method_one(self):
                    assert 42 == 43
        """)
        def check(keyword, name):
            reprec = testdir.inline_run("-s", "-k", keyword, file_test)
            passed, skipped, failed = reprec.listoutcomes()
            assert len(failed) == 1
            assert failed[0].nodeid.split("::")[-1] == name
            assert len(reprec.getcalls('pytest_deselected')) == 1

        for keyword in ['test_one', 'est_on']:
            check(keyword, 'test_one')
        check('TestClass and test', 'test_method_one')

    @pytest.mark.parametrize("keyword", [
        'xxx', 'xxx and test_2', 'TestClass', 'xxx and not test_1',
        'TestClass and test_2', 'xxx and TestClass and test_2'])
    def test_select_extra_keywords(self, testdir, keyword):
        p = testdir.makepyfile(test_select="""
            def test_1():
                pass
            class TestClass:
                def test_2(self):
                    pass
        """)
        testdir.makepyfile(conftest="""
            import pytest
            @pytest.hookimpl(hookwrapper=True)
            def pytest_pycollect_makeitem(name):
                outcome = yield
                if name == "TestClass":
                    item = outcome.get_result()
                    item.extra_keyword_matches.add("xxx")
        """)
        reprec = testdir.inline_run(p.dirpath(), '-s', '-k', keyword)
        py.builtin.print_("keyword", repr(keyword))
        passed, skipped, failed = reprec.listoutcomes()
        assert len(passed) == 1
        assert passed[0].nodeid.endswith("test_2")
        dlist = reprec.getcalls("pytest_deselected")
        assert len(dlist) == 1
        assert dlist[0].items[0].name == 'test_1'

    def test_select_starton(self, testdir):
        threepass = testdir.makepyfile(test_threepass="""
            def test_one(): assert 1
            def test_two(): assert 1
            def test_three(): assert 1
        """)
        reprec = testdir.inline_run("-k", "test_two:", threepass)
        passed, skipped, failed = reprec.listoutcomes()
        assert len(passed) == 2
        assert not failed
        dlist = reprec.getcalls("pytest_deselected")
        assert len(dlist) == 1
        item = dlist[0].items[0]
        assert item.name == "test_one"

    def test_keyword_extra(self, testdir):
        p = testdir.makepyfile("""
           def test_one():
               assert 0
           test_one.mykeyword = True
        """)
        reprec = testdir.inline_run("-k", "mykeyword", p)
        passed, skipped, failed = reprec.countoutcomes()
        assert failed == 1

    @pytest.mark.xfail
    def test_keyword_extra_dash(self, testdir):
        p = testdir.makepyfile("""
           def test_one():
               assert 0
           test_one.mykeyword = True
        """)
        # with argparse the argument to an option cannot
        # start with '-'
        reprec = testdir.inline_run("-k", "-mykeyword", p)
        passed, skipped, failed = reprec.countoutcomes()
        assert passed + skipped + failed == 0

    def test_no_magic_values(self, testdir):
        """Make sure the tests do not match on magic values,
        no double underscored values, like '__dict__',
        and no instance values, like '()'.
        """
        p = testdir.makepyfile("""
            def test_one(): assert 1
        """)
        def assert_test_is_not_selected(keyword):
            reprec = testdir.inline_run("-k", keyword, p)
            passed, skipped, failed = reprec.countoutcomes()
            dlist = reprec.getcalls("pytest_deselected")
            assert passed + skipped + failed == 0
            deselected_tests = dlist[0].items
            assert len(deselected_tests) == 1

        assert_test_is_not_selected("__")
        assert_test_is_not_selected("()")

