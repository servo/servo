# -*- coding: utf-8 -*-
import re
import sys
import textwrap

import attr
import hypothesis
import six
from hypothesis import strategies

import pytest
from _pytest import fixtures
from _pytest import python
from _pytest.warnings import SHOW_PYTEST_WARNINGS_ARG

PY3 = sys.version_info >= (3, 0)


class TestMetafunc(object):
    def Metafunc(self, func, config=None):
        # the unit tests of this class check if things work correctly
        # on the funcarg level, so we don't need a full blown
        # initiliazation
        class FixtureInfo(object):
            name2fixturedefs = None

            def __init__(self, names):
                self.names_closure = names

        @attr.s
        class DefinitionMock(object):
            obj = attr.ib()

        names = fixtures.getfuncargnames(func)
        fixtureinfo = FixtureInfo(names)
        definition = DefinitionMock(func)
        return python.Metafunc(definition, fixtureinfo, config)

    def test_no_funcargs(self, testdir):
        def function():
            pass

        metafunc = self.Metafunc(function)
        assert not metafunc.fixturenames
        repr(metafunc._calls)

    def test_function_basic(self):
        def func(arg1, arg2="qwe"):
            pass

        metafunc = self.Metafunc(func)
        assert len(metafunc.fixturenames) == 1
        assert "arg1" in metafunc.fixturenames
        assert metafunc.function is func
        assert metafunc.cls is None

    def test_parametrize_error(self):
        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x", [1, 2])
        pytest.raises(ValueError, lambda: metafunc.parametrize("x", [5, 6]))
        pytest.raises(ValueError, lambda: metafunc.parametrize("x", [5, 6]))
        metafunc.parametrize("y", [1, 2])
        pytest.raises(ValueError, lambda: metafunc.parametrize("y", [5, 6]))
        pytest.raises(ValueError, lambda: metafunc.parametrize("y", [5, 6]))

    def test_parametrize_bad_scope(self, testdir):
        def func(x):
            pass

        metafunc = self.Metafunc(func)
        with pytest.raises(
            pytest.fail.Exception,
            match=r"parametrize\(\) call in func got an unexpected scope value 'doggy'",
        ):
            metafunc.parametrize("x", [1], scope="doggy")

    def test_find_parametrized_scope(self):
        """unittest for _find_parametrized_scope (#3941)"""
        from _pytest.python import _find_parametrized_scope

        @attr.s
        class DummyFixtureDef(object):
            scope = attr.ib()

        fixtures_defs = dict(
            session_fix=[DummyFixtureDef("session")],
            package_fix=[DummyFixtureDef("package")],
            module_fix=[DummyFixtureDef("module")],
            class_fix=[DummyFixtureDef("class")],
            func_fix=[DummyFixtureDef("function")],
        )

        # use arguments to determine narrow scope; the cause of the bug is that it would look on all
        # fixture defs given to the method
        def find_scope(argnames, indirect):
            return _find_parametrized_scope(argnames, fixtures_defs, indirect=indirect)

        assert find_scope(["func_fix"], indirect=True) == "function"
        assert find_scope(["class_fix"], indirect=True) == "class"
        assert find_scope(["module_fix"], indirect=True) == "module"
        assert find_scope(["package_fix"], indirect=True) == "package"
        assert find_scope(["session_fix"], indirect=True) == "session"

        assert find_scope(["class_fix", "func_fix"], indirect=True) == "function"
        assert find_scope(["func_fix", "session_fix"], indirect=True) == "function"
        assert find_scope(["session_fix", "class_fix"], indirect=True) == "class"
        assert find_scope(["package_fix", "session_fix"], indirect=True) == "package"
        assert find_scope(["module_fix", "session_fix"], indirect=True) == "module"

        # when indirect is False or is not for all scopes, always use function
        assert find_scope(["session_fix", "module_fix"], indirect=False) == "function"
        assert (
            find_scope(["session_fix", "module_fix"], indirect=["module_fix"])
            == "function"
        )
        assert (
            find_scope(
                ["session_fix", "module_fix"], indirect=["session_fix", "module_fix"]
            )
            == "module"
        )

    def test_parametrize_and_id(self):
        def func(x, y):
            pass

        metafunc = self.Metafunc(func)

        metafunc.parametrize("x", [1, 2], ids=["basic", "advanced"])
        metafunc.parametrize("y", ["abc", "def"])
        ids = [x.id for x in metafunc._calls]
        assert ids == ["basic-abc", "basic-def", "advanced-abc", "advanced-def"]

    def test_parametrize_and_id_unicode(self):
        """Allow unicode strings for "ids" parameter in Python 2 (##1905)"""

        def func(x):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x", [1, 2], ids=[u"basic", u"advanced"])
        ids = [x.id for x in metafunc._calls]
        assert ids == [u"basic", u"advanced"]

    def test_parametrize_with_wrong_number_of_ids(self, testdir):
        def func(x, y):
            pass

        metafunc = self.Metafunc(func)

        with pytest.raises(pytest.fail.Exception):
            metafunc.parametrize("x", [1, 2], ids=["basic"])

        with pytest.raises(pytest.fail.Exception):
            metafunc.parametrize(
                ("x", "y"), [("abc", "def"), ("ghi", "jkl")], ids=["one"]
            )

    def test_parametrize_empty_list(self):
        """#510"""

        def func(y):
            pass

        class MockConfig(object):
            def getini(self, name):
                return ""

            @property
            def hook(self):
                return self

            def pytest_make_parametrize_id(self, **kw):
                pass

        metafunc = self.Metafunc(func, MockConfig())
        metafunc.parametrize("y", [])
        assert "skip" == metafunc._calls[0].marks[0].name

    def test_parametrize_with_userobjects(self):
        def func(x, y):
            pass

        metafunc = self.Metafunc(func)

        class A(object):
            pass

        metafunc.parametrize("x", [A(), A()])
        metafunc.parametrize("y", list("ab"))
        assert metafunc._calls[0].id == "x0-a"
        assert metafunc._calls[1].id == "x0-b"
        assert metafunc._calls[2].id == "x1-a"
        assert metafunc._calls[3].id == "x1-b"

    @hypothesis.given(strategies.text() | strategies.binary())
    @hypothesis.settings(
        deadline=400.0
    )  # very close to std deadline and CI boxes are not reliable in CPU power
    def test_idval_hypothesis(self, value):
        from _pytest.python import _idval

        escaped = _idval(value, "a", 6, None, item=None, config=None)
        assert isinstance(escaped, six.text_type)
        escaped.encode("ascii")

    def test_unicode_idval(self):
        """This tests that Unicode strings outside the ASCII character set get
        escaped, using byte escapes if they're in that range or unicode
        escapes if they're not.

        """
        from _pytest.python import _idval

        values = [
            (u"", ""),
            (u"ascii", "ascii"),
            (u"ação", "a\\xe7\\xe3o"),
            (u"josé@blah.com", "jos\\xe9@blah.com"),
            (
                u"δοκ.ιμή@παράδειγμα.δοκιμή",
                "\\u03b4\\u03bf\\u03ba.\\u03b9\\u03bc\\u03ae@\\u03c0\\u03b1\\u03c1\\u03ac\\u03b4\\u03b5\\u03b9\\u03b3"
                "\\u03bc\\u03b1.\\u03b4\\u03bf\\u03ba\\u03b9\\u03bc\\u03ae",
            ),
        ]
        for val, expected in values:
            assert _idval(val, "a", 6, None, item=None, config=None) == expected

    def test_unicode_idval_with_config(self):
        """unittest for expected behavior to obtain ids with
        disable_test_id_escaping_and_forfeit_all_rights_to_community_support
        option. (#5294)
        """
        from _pytest.python import _idval

        class MockConfig(object):
            def __init__(self, config):
                self.config = config

            @property
            def hook(self):
                return self

            def pytest_make_parametrize_id(self, **kw):
                pass

            def getini(self, name):
                return self.config[name]

        option = "disable_test_id_escaping_and_forfeit_all_rights_to_community_support"

        values = [
            (u"ação", MockConfig({option: True}), u"ação"),
            (u"ação", MockConfig({option: False}), "a\\xe7\\xe3o"),
        ]
        for val, config, expected in values:
            assert _idval(val, "a", 6, None, item=None, config=config) == expected

    def test_bytes_idval(self):
        """unittest for the expected behavior to obtain ids for parametrized
        bytes values:
        - python2: non-ascii strings are considered bytes and formatted using
        "binary escape", where any byte < 127 is escaped into its hex form.
        - python3: bytes objects are always escaped using "binary escape".
        """
        from _pytest.python import _idval

        values = [
            (b"", ""),
            (b"\xc3\xb4\xff\xe4", "\\xc3\\xb4\\xff\\xe4"),
            (b"ascii", "ascii"),
            (u"αρά".encode("utf-8"), "\\xce\\xb1\\xcf\\x81\\xce\\xac"),
        ]
        for val, expected in values:
            assert _idval(val, "a", 6, idfn=None, item=None, config=None) == expected

    def test_class_or_function_idval(self):
        """unittest for the expected behavior to obtain ids for parametrized
        values that are classes or functions: their __name__.
        """
        from _pytest.python import _idval

        class TestClass(object):
            pass

        def test_function():
            pass

        values = [(TestClass, "TestClass"), (test_function, "test_function")]
        for val, expected in values:
            assert _idval(val, "a", 6, None, item=None, config=None) == expected

    def test_idmaker_autoname(self):
        """#250"""
        from _pytest.python import idmaker

        result = idmaker(
            ("a", "b"), [pytest.param("string", 1.0), pytest.param("st-ring", 2.0)]
        )
        assert result == ["string-1.0", "st-ring-2.0"]

        result = idmaker(
            ("a", "b"), [pytest.param(object(), 1.0), pytest.param(object(), object())]
        )
        assert result == ["a0-1.0", "a1-b1"]
        # unicode mixing, issue250
        result = idmaker((u"a", "b"), [pytest.param({}, b"\xc3\xb4")])
        assert result == ["a0-\\xc3\\xb4"]

    def test_idmaker_with_bytes_regex(self):
        from _pytest.python import idmaker

        result = idmaker(("a"), [pytest.param(re.compile(b"foo"), 1.0)])
        assert result == ["foo"]

    def test_idmaker_native_strings(self):
        from _pytest.python import idmaker

        result = idmaker(
            ("a", "b"),
            [
                pytest.param(1.0, -1.1),
                pytest.param(2, -202),
                pytest.param("three", "three hundred"),
                pytest.param(True, False),
                pytest.param(None, None),
                pytest.param(re.compile("foo"), re.compile("bar")),
                pytest.param(str, int),
                pytest.param(list("six"), [66, 66]),
                pytest.param({7}, set("seven")),
                pytest.param(tuple("eight"), (8, -8, 8)),
                pytest.param(b"\xc3\xb4", b"name"),
                pytest.param(b"\xc3\xb4", u"other"),
            ],
        )
        assert result == [
            "1.0--1.1",
            "2--202",
            "three-three hundred",
            "True-False",
            "None-None",
            "foo-bar",
            "str-int",
            "a7-b7",
            "a8-b8",
            "a9-b9",
            "\\xc3\\xb4-name",
            "\\xc3\\xb4-other",
        ]

    def test_idmaker_non_printable_characters(self):
        from _pytest.python import idmaker

        result = idmaker(
            ("s", "n"),
            [
                pytest.param("\x00", 1),
                pytest.param("\x05", 2),
                pytest.param(b"\x00", 3),
                pytest.param(b"\x05", 4),
                pytest.param("\t", 5),
                pytest.param(b"\t", 6),
            ],
        )
        assert result == ["\\x00-1", "\\x05-2", "\\x00-3", "\\x05-4", "\\t-5", "\\t-6"]

    def test_idmaker_manual_ids_must_be_printable(self):
        from _pytest.python import idmaker

        result = idmaker(
            ("s",),
            [
                pytest.param("x00", id="hello \x00"),
                pytest.param("x05", id="hello \x05"),
            ],
        )
        assert result == ["hello \\x00", "hello \\x05"]

    def test_idmaker_enum(self):
        from _pytest.python import idmaker

        enum = pytest.importorskip("enum")
        e = enum.Enum("Foo", "one, two")
        result = idmaker(("a", "b"), [pytest.param(e.one, e.two)])
        assert result == ["Foo.one-Foo.two"]

    def test_idmaker_idfn(self):
        """#351"""
        from _pytest.python import idmaker

        def ids(val):
            if isinstance(val, Exception):
                return repr(val)

        result = idmaker(
            ("a", "b"),
            [
                pytest.param(10.0, IndexError()),
                pytest.param(20, KeyError()),
                pytest.param("three", [1, 2, 3]),
            ],
            idfn=ids,
        )
        assert result == ["10.0-IndexError()", "20-KeyError()", "three-b2"]

    def test_idmaker_idfn_unique_names(self):
        """#351"""
        from _pytest.python import idmaker

        def ids(val):
            return "a"

        result = idmaker(
            ("a", "b"),
            [
                pytest.param(10.0, IndexError()),
                pytest.param(20, KeyError()),
                pytest.param("three", [1, 2, 3]),
            ],
            idfn=ids,
        )
        assert result == ["a-a0", "a-a1", "a-a2"]

    def test_idmaker_with_idfn_and_config(self):
        """unittest for expected behavior to create ids with idfn and
        disable_test_id_escaping_and_forfeit_all_rights_to_community_support
        option. (#5294)
        """
        from _pytest.python import idmaker

        class MockConfig(object):
            def __init__(self, config):
                self.config = config

            @property
            def hook(self):
                return self

            def pytest_make_parametrize_id(self, **kw):
                pass

            def getini(self, name):
                return self.config[name]

        option = "disable_test_id_escaping_and_forfeit_all_rights_to_community_support"

        values = [
            (MockConfig({option: True}), u"ação"),
            (MockConfig({option: False}), "a\\xe7\\xe3o"),
        ]
        for config, expected in values:
            result = idmaker(
                ("a",), [pytest.param("string")], idfn=lambda _: u"ação", config=config
            )
            assert result == [expected]

    def test_idmaker_with_ids_and_config(self):
        """unittest for expected behavior to create ids with ids and
        disable_test_id_escaping_and_forfeit_all_rights_to_community_support
        option. (#5294)
        """
        from _pytest.python import idmaker

        class MockConfig(object):
            def __init__(self, config):
                self.config = config

            @property
            def hook(self):
                return self

            def pytest_make_parametrize_id(self, **kw):
                pass

            def getini(self, name):
                return self.config[name]

        option = "disable_test_id_escaping_and_forfeit_all_rights_to_community_support"

        values = [
            (MockConfig({option: True}), u"ação"),
            (MockConfig({option: False}), "a\\xe7\\xe3o"),
        ]
        for config, expected in values:
            result = idmaker(
                ("a",), [pytest.param("string")], ids=[u"ação"], config=config
            )
            assert result == [expected]

    def test_parametrize_ids_exception(self, testdir):
        """
        :param testdir: the instance of Testdir class, a temporary
        test directory.
        """
        testdir.makepyfile(
            """
                import pytest

                def ids(arg):
                    raise Exception("bad ids")

                @pytest.mark.parametrize("arg", ["a", "b"], ids=ids)
                def test_foo(arg):
                    pass
            """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*test_foo: error raised while trying to determine id of parameter 'arg' at position 0",
                "*Exception: bad ids",
            ]
        )

    def test_parametrize_ids_returns_non_string(self, testdir):
        testdir.makepyfile(
            """\
            import pytest

            def ids(d):
                return d

            @pytest.mark.parametrize("arg", ({1: 2}, {3, 4}), ids=ids)
            def test(arg):
                assert arg
            """
        )
        assert testdir.runpytest().ret == 0

    def test_idmaker_with_ids(self):
        from _pytest.python import idmaker

        result = idmaker(
            ("a", "b"), [pytest.param(1, 2), pytest.param(3, 4)], ids=["a", None]
        )
        assert result == ["a", "3-4"]

    def test_idmaker_with_paramset_id(self):
        from _pytest.python import idmaker

        result = idmaker(
            ("a", "b"),
            [pytest.param(1, 2, id="me"), pytest.param(3, 4, id="you")],
            ids=["a", None],
        )
        assert result == ["me", "you"]

    def test_idmaker_with_ids_unique_names(self):
        from _pytest.python import idmaker

        result = idmaker(
            ("a"), map(pytest.param, [1, 2, 3, 4, 5]), ids=["a", "a", "b", "c", "b"]
        )
        assert result == ["a0", "a1", "b0", "c", "b1"]

    def test_parametrize_indirect(self):
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x", [1], indirect=True)
        metafunc.parametrize("y", [2, 3], indirect=True)
        assert len(metafunc._calls) == 2
        assert metafunc._calls[0].funcargs == {}
        assert metafunc._calls[1].funcargs == {}
        assert metafunc._calls[0].params == dict(x=1, y=2)
        assert metafunc._calls[1].params == dict(x=1, y=3)

    def test_parametrize_indirect_list(self):
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x, y", [("a", "b")], indirect=["x"])
        assert metafunc._calls[0].funcargs == dict(y="b")
        assert metafunc._calls[0].params == dict(x="a")

    def test_parametrize_indirect_list_all(self):
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x, y", [("a", "b")], indirect=["x", "y"])
        assert metafunc._calls[0].funcargs == {}
        assert metafunc._calls[0].params == dict(x="a", y="b")

    def test_parametrize_indirect_list_empty(self):
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x, y", [("a", "b")], indirect=[])
        assert metafunc._calls[0].funcargs == dict(x="a", y="b")
        assert metafunc._calls[0].params == {}

    def test_parametrize_indirect_list_functional(self, testdir):
        """
        #714
        Test parametrization with 'indirect' parameter applied on
        particular arguments. As y is is direct, its value should
        be used directly rather than being passed to the fixture
        y.

        :param testdir: the instance of Testdir class, a temporary
        test directory.
        """
        testdir.makepyfile(
            """
            import pytest
            @pytest.fixture(scope='function')
            def x(request):
                return request.param * 3
            @pytest.fixture(scope='function')
            def y(request):
                return request.param * 2
            @pytest.mark.parametrize('x, y', [('a', 'b')], indirect=['x'])
            def test_simple(x,y):
                assert len(x) == 3
                assert len(y) == 1
        """
        )
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines(["*test_simple*a-b*", "*1 passed*"])

    def test_parametrize_indirect_list_error(self, testdir):
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        with pytest.raises(pytest.fail.Exception):
            metafunc.parametrize("x, y", [("a", "b")], indirect=["x", "z"])

    def test_parametrize_uses_no_fixture_error_indirect_false(self, testdir):
        """The 'uses no fixture' error tells the user at collection time
        that the parametrize data they've set up doesn't correspond to the
        fixtures in their test function, rather than silently ignoring this
        and letting the test potentially pass.

        #714
        """
        testdir.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize('x, y', [('a', 'b')], indirect=False)
            def test_simple(x):
                assert len(x) == 3
        """
        )
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no argument 'y'*"])

    def test_parametrize_uses_no_fixture_error_indirect_true(self, testdir):
        """#714"""
        testdir.makepyfile(
            """
            import pytest
            @pytest.fixture(scope='function')
            def x(request):
                return request.param * 3
            @pytest.fixture(scope='function')
            def y(request):
                return request.param * 2

            @pytest.mark.parametrize('x, y', [('a', 'b')], indirect=True)
            def test_simple(x):
                assert len(x) == 3
        """
        )
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no fixture 'y'*"])

    def test_parametrize_indirect_uses_no_fixture_error_indirect_string(self, testdir):
        """#714"""
        testdir.makepyfile(
            """
            import pytest
            @pytest.fixture(scope='function')
            def x(request):
                return request.param * 3

            @pytest.mark.parametrize('x, y', [('a', 'b')], indirect='y')
            def test_simple(x):
                assert len(x) == 3
        """
        )
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no fixture 'y'*"])

    def test_parametrize_indirect_uses_no_fixture_error_indirect_list(self, testdir):
        """#714"""
        testdir.makepyfile(
            """
            import pytest
            @pytest.fixture(scope='function')
            def x(request):
                return request.param * 3

            @pytest.mark.parametrize('x, y', [('a', 'b')], indirect=['y'])
            def test_simple(x):
                assert len(x) == 3
        """
        )
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no fixture 'y'*"])

    def test_parametrize_argument_not_in_indirect_list(self, testdir):
        """#714"""
        testdir.makepyfile(
            """
            import pytest
            @pytest.fixture(scope='function')
            def x(request):
                return request.param * 3

            @pytest.mark.parametrize('x, y', [('a', 'b')], indirect=['x'])
            def test_simple(x):
                assert len(x) == 3
        """
        )
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no argument 'y'*"])

    def test_parametrize_gives_indicative_error_on_function_with_default_argument(
        self, testdir
    ):
        testdir.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize('x, y', [('a', 'b')])
            def test_simple(x, y=1):
                assert len(x) == 1
        """
        )
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines(
            ["*already takes an argument 'y' with a default value"]
        )

    def test_parametrize_functional(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('x', [1,2], indirect=True)
                metafunc.parametrize('y', [2])
            @pytest.fixture
            def x(request):
                return request.param * 10

            def test_simple(x,y):
                assert x in (10,20)
                assert y == 2
        """
        )
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines(
            ["*test_simple*1-2*", "*test_simple*2-2*", "*2 passed*"]
        )

    def test_parametrize_onearg(self):
        metafunc = self.Metafunc(lambda x: None)
        metafunc.parametrize("x", [1, 2])
        assert len(metafunc._calls) == 2
        assert metafunc._calls[0].funcargs == dict(x=1)
        assert metafunc._calls[0].id == "1"
        assert metafunc._calls[1].funcargs == dict(x=2)
        assert metafunc._calls[1].id == "2"

    def test_parametrize_onearg_indirect(self):
        metafunc = self.Metafunc(lambda x: None)
        metafunc.parametrize("x", [1, 2], indirect=True)
        assert metafunc._calls[0].params == dict(x=1)
        assert metafunc._calls[0].id == "1"
        assert metafunc._calls[1].params == dict(x=2)
        assert metafunc._calls[1].id == "2"

    def test_parametrize_twoargs(self):
        metafunc = self.Metafunc(lambda x, y: None)
        metafunc.parametrize(("x", "y"), [(1, 2), (3, 4)])
        assert len(metafunc._calls) == 2
        assert metafunc._calls[0].funcargs == dict(x=1, y=2)
        assert metafunc._calls[0].id == "1-2"
        assert metafunc._calls[1].funcargs == dict(x=3, y=4)
        assert metafunc._calls[1].id == "3-4"

    def test_parametrize_multiple_times(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            pytestmark = pytest.mark.parametrize("x", [1,2])
            def test_func(x):
                assert 0, x
            class TestClass(object):
                pytestmark = pytest.mark.parametrize("y", [3,4])
                def test_meth(self, x, y):
                    assert 0, x
        """
        )
        result = testdir.runpytest()
        assert result.ret == 1
        result.assert_outcomes(failed=6)

    def test_parametrize_CSV(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.parametrize("x, y,", [(1,2), (2,3)])
            def test_func(x, y):
                assert x+1 == y
        """
        )
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    def test_parametrize_class_scenarios(self, testdir):
        testdir.makepyfile(
            """
        # same as doc/en/example/parametrize scenario example
        def pytest_generate_tests(metafunc):
            idlist = []
            argvalues = []
            for scenario in metafunc.cls.scenarios:
                idlist.append(scenario[0])
                items = scenario[1].items()
                argnames = [x[0] for x in items]
                argvalues.append(([x[1] for x in items]))
            metafunc.parametrize(argnames, argvalues, ids=idlist, scope="class")

        class Test(object):
               scenarios = [['1', {'arg': {1: 2}, "arg2": "value2"}],
                            ['2', {'arg':'value2', "arg2": "value2"}]]

               def test_1(self, arg, arg2):
                  pass

               def test_2(self, arg2, arg):
                  pass

               def test_3(self, arg, arg2):
                  pass
        """
        )
        result = testdir.runpytest("-v")
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            """
            *test_1*1*
            *test_2*1*
            *test_3*1*
            *test_1*2*
            *test_2*2*
            *test_3*2*
            *6 passed*
        """
        )

    def test_format_args(self):
        def function1():
            pass

        assert fixtures._format_args(function1) == "()"

        def function2(arg1):
            pass

        assert fixtures._format_args(function2) == "(arg1)"

        def function3(arg1, arg2="qwe"):
            pass

        assert fixtures._format_args(function3) == "(arg1, arg2='qwe')"

        def function4(arg1, *args, **kwargs):
            pass

        assert fixtures._format_args(function4) == "(arg1, *args, **kwargs)"


class TestMetafuncFunctional(object):
    def test_attributes(self, testdir):
        p = testdir.makepyfile(
            """
            # assumes that generate/provide runs in the same process
            import sys, pytest, six
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('metafunc', [metafunc])

            @pytest.fixture
            def metafunc(request):
                return request.param

            def test_function(metafunc, pytestconfig):
                assert metafunc.config == pytestconfig
                assert metafunc.module.__name__ == __name__
                assert metafunc.function == test_function
                assert metafunc.cls is None

            class TestClass(object):
                def test_method(self, metafunc, pytestconfig):
                    assert metafunc.config == pytestconfig
                    assert metafunc.module.__name__ == __name__
                    unbound = six.get_unbound_function(TestClass.test_method)
                    assert metafunc.function == unbound
                    assert metafunc.cls == TestClass
        """
        )
        result = testdir.runpytest(p, "-v", SHOW_PYTEST_WARNINGS_ARG)
        result.assert_outcomes(passed=2)

    def test_two_functions(self, testdir):
        p = testdir.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('arg1', [10, 20], ids=['0', '1'])

            def test_func1(arg1):
                assert arg1 == 10

            def test_func2(arg1):
                assert arg1 in (10, 20)
        """
        )
        result = testdir.runpytest("-v", p, SHOW_PYTEST_WARNINGS_ARG)
        result.stdout.fnmatch_lines(
            [
                "*test_func1*0*PASS*",
                "*test_func1*1*FAIL*",
                "*test_func2*PASS*",
                "*test_func2*PASS*",
                "*1 failed, 3 passed*",
            ]
        )

    def test_noself_in_method(self, testdir):
        p = testdir.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                assert 'xyz' not in metafunc.fixturenames

            class TestHello(object):
                def test_hello(xyz):
                    pass
        """
        )
        result = testdir.runpytest(p)
        result.assert_outcomes(passed=1)

    def test_generate_tests_in_class(self, testdir):
        p = testdir.makepyfile(
            """
            class TestClass(object):
                def pytest_generate_tests(self, metafunc):
                    metafunc.parametrize('hello', ['world'], ids=['hellow'])

                def test_myfunc(self, hello):
                    assert hello == "world"
        """
        )
        result = testdir.runpytest("-v", p, SHOW_PYTEST_WARNINGS_ARG)
        result.stdout.fnmatch_lines(["*test_myfunc*hello*PASS*", "*1 passed*"])

    def test_two_functions_not_same_instance(self, testdir):
        p = testdir.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('arg1', [10, 20], ids=["0", "1"])

            class TestClass(object):
                def test_func(self, arg1):
                    assert not hasattr(self, 'x')
                    self.x = 1
        """
        )
        result = testdir.runpytest("-v", p, SHOW_PYTEST_WARNINGS_ARG)
        result.stdout.fnmatch_lines(
            ["*test_func*0*PASS*", "*test_func*1*PASS*", "*2 pass*"]
        )

    def test_issue28_setup_method_in_generate_tests(self, testdir):
        p = testdir.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('arg1', [1])

            class TestClass(object):
                def test_method(self, arg1):
                    assert arg1 == self.val
                def setup_method(self, func):
                    self.val = 1
            """
        )
        result = testdir.runpytest(p, SHOW_PYTEST_WARNINGS_ARG)
        result.assert_outcomes(passed=1)

    def test_parametrize_functional2(self, testdir):
        testdir.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize("arg1", [1,2])
                metafunc.parametrize("arg2", [4,5])
            def test_hello(arg1, arg2):
                assert 0, (arg1, arg2)
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            ["*(1, 4)*", "*(1, 5)*", "*(2, 4)*", "*(2, 5)*", "*4 failed*"]
        )

    def test_parametrize_and_inner_getfixturevalue(self, testdir):
        p = testdir.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize("arg1", [1], indirect=True)
                metafunc.parametrize("arg2", [10], indirect=True)

            import pytest
            @pytest.fixture
            def arg1(request):
                x = request.getfixturevalue("arg2")
                return x + request.param

            @pytest.fixture
            def arg2(request):
                return request.param

            def test_func1(arg1, arg2):
                assert arg1 == 11
        """
        )
        result = testdir.runpytest("-v", p)
        result.stdout.fnmatch_lines(["*test_func1*1*PASS*", "*1 passed*"])

    def test_parametrize_on_setup_arg(self, testdir):
        p = testdir.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                assert "arg1" in metafunc.fixturenames
                metafunc.parametrize("arg1", [1], indirect=True)

            import pytest
            @pytest.fixture
            def arg1(request):
                return request.param

            @pytest.fixture
            def arg2(request, arg1):
                return 10 * arg1

            def test_func(arg2):
                assert arg2 == 10
        """
        )
        result = testdir.runpytest("-v", p)
        result.stdout.fnmatch_lines(["*test_func*1*PASS*", "*1 passed*"])

    def test_parametrize_with_ids(self, testdir):
        testdir.makeini(
            """
            [pytest]
            console_output_style=classic
        """
        )
        testdir.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize(("a", "b"), [(1,1), (1,2)],
                                     ids=["basic", "advanced"])

            def test_function(a, b):
                assert a == b
        """
        )
        result = testdir.runpytest("-v")
        assert result.ret == 1
        result.stdout.fnmatch_lines_random(
            ["*test_function*basic*PASSED", "*test_function*advanced*FAILED"]
        )

    def test_parametrize_without_ids(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize(("a", "b"),
                                     [(1,object()), (1.3,object())])

            def test_function(a, b):
                assert 1
        """
        )
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines(
            """
            *test_function*1-b0*
            *test_function*1.3-b1*
        """
        )

    def test_parametrize_with_None_in_ids(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize(("a", "b"), [(1,1), (1,1), (1,2)],
                                     ids=["basic", None, "advanced"])

            def test_function(a, b):
                assert a == b
        """
        )
        result = testdir.runpytest("-v")
        assert result.ret == 1
        result.stdout.fnmatch_lines_random(
            [
                "*test_function*basic*PASSED*",
                "*test_function*1-1*PASSED*",
                "*test_function*advanced*FAILED*",
            ]
        )

    def test_fixture_parametrized_empty_ids(self, testdir):
        """Fixtures parametrized with empty ids cause an internal error (#1849)."""
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture(scope="module", ids=[], params=[])
            def temp(request):
               return request.param

            def test_temp(temp):
                 pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 1 skipped *"])

    def test_parametrized_empty_ids(self, testdir):
        """Tests parametrized with empty ids cause an internal error (#1849)."""
        testdir.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize('temp', [], ids=list())
            def test_temp(temp):
                 pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 1 skipped *"])

    def test_parametrized_ids_invalid_type(self, testdir):
        """Tests parametrized with ids as non-strings (#1857)."""
        testdir.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize("x, expected", [(10, 20), (40, 80)], ids=(None, 2))
            def test_ids_numbers(x,expected):
                assert x * 2 == expected
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*In test_ids_numbers: ids must be list of strings, found: 2 (type: *'int'>)*"
            ]
        )

    def test_parametrize_with_identical_ids_get_unique_names(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize(("a", "b"), [(1,1), (1,2)],
                                     ids=["a", "a"])

            def test_function(a, b):
                assert a == b
        """
        )
        result = testdir.runpytest("-v")
        assert result.ret == 1
        result.stdout.fnmatch_lines_random(
            ["*test_function*a0*PASSED*", "*test_function*a1*FAILED*"]
        )

    @pytest.mark.parametrize(("scope", "length"), [("module", 2), ("function", 4)])
    def test_parametrize_scope_overrides(self, testdir, scope, length):
        testdir.makepyfile(
            """
            import pytest
            values = []
            def pytest_generate_tests(metafunc):
                if "arg" in metafunc.funcargnames:
                    metafunc.parametrize("arg", [1,2], indirect=True,
                                         scope=%r)
            @pytest.fixture
            def arg(request):
                values.append(request.param)
                return request.param
            def test_hello(arg):
                assert arg in (1,2)
            def test_world(arg):
                assert arg in (1,2)
            def test_checklength():
                assert len(values) == %d
        """
            % (scope, length)
        )
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=5)

    def test_parametrize_issue323(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture(scope='module', params=range(966))
            def foo(request):
                return request.param

            def test_it(foo):
                pass
            def test_it2(foo):
                pass
        """
        )
        reprec = testdir.inline_run("--collect-only")
        assert not reprec.getcalls("pytest_internalerror")

    def test_usefixtures_seen_in_generate_tests(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                assert "abc" in metafunc.fixturenames
                metafunc.parametrize("abc", [1])

            @pytest.mark.usefixtures("abc")
            def test_function():
                pass
        """
        )
        reprec = testdir.runpytest()
        reprec.assert_outcomes(passed=1)

    def test_generate_tests_only_done_in_subdir(self, testdir):
        sub1 = testdir.mkpydir("sub1")
        sub2 = testdir.mkpydir("sub2")
        sub1.join("conftest.py").write(
            textwrap.dedent(
                """\
                def pytest_generate_tests(metafunc):
                    assert metafunc.function.__name__ == "test_1"
                """
            )
        )
        sub2.join("conftest.py").write(
            textwrap.dedent(
                """\
                def pytest_generate_tests(metafunc):
                    assert metafunc.function.__name__ == "test_2"
                """
            )
        )
        sub1.join("test_in_sub1.py").write("def test_1(): pass")
        sub2.join("test_in_sub2.py").write("def test_2(): pass")
        result = testdir.runpytest("--keep-duplicates", "-v", "-s", sub1, sub2, sub1)
        result.assert_outcomes(passed=3)

    def test_generate_same_function_names_issue403(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            def make_tests():
                @pytest.mark.parametrize("x", range(2))
                def test_foo(x):
                    pass
                return test_foo

            test_x = make_tests()
            test_y = make_tests()
        """
        )
        reprec = testdir.runpytest()
        reprec.assert_outcomes(passed=4)

    @pytest.mark.parametrize("attr", ["parametrise", "parameterize", "parameterise"])
    def test_parametrize_misspelling(self, testdir, attr):
        """#463"""
        testdir.makepyfile(
            """
            import pytest

            @pytest.mark.{}("x", range(2))
            def test_foo(x):
                pass
        """.format(
                attr
            )
        )
        result = testdir.runpytest("--collectonly")
        result.stdout.fnmatch_lines(
            [
                "test_foo has '{}' mark, spelling should be 'parametrize'".format(attr),
                "*1 error in*",
            ]
        )


class TestMetafuncFunctionalAuto(object):
    """
    Tests related to automatically find out the correct scope for parametrized tests (#1832).
    """

    def test_parametrize_auto_scope(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture(scope='session', autouse=True)
            def fixture():
                return 1

            @pytest.mark.parametrize('animal', ["dog", "cat"])
            def test_1(animal):
                assert animal in ('dog', 'cat')

            @pytest.mark.parametrize('animal', ['fish'])
            def test_2(animal):
                assert animal == 'fish'

        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 3 passed *"])

    def test_parametrize_auto_scope_indirect(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture(scope='session')
            def echo(request):
                return request.param

            @pytest.mark.parametrize('animal, echo', [("dog", 1), ("cat", 2)], indirect=['echo'])
            def test_1(animal, echo):
                assert animal in ('dog', 'cat')
                assert echo in (1, 2, 3)

            @pytest.mark.parametrize('animal, echo', [('fish', 3)], indirect=['echo'])
            def test_2(animal, echo):
                assert animal == 'fish'
                assert echo in (1, 2, 3)
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 3 passed *"])

    def test_parametrize_auto_scope_override_fixture(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture(scope='session', autouse=True)
            def animal():
                return 'fox'

            @pytest.mark.parametrize('animal', ["dog", "cat"])
            def test_1(animal):
                assert animal in ('dog', 'cat')
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed *"])

    def test_parametrize_all_indirects(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture()
            def animal(request):
                return request.param

            @pytest.fixture(scope='session')
            def echo(request):
                return request.param

            @pytest.mark.parametrize('animal, echo', [("dog", 1), ("cat", 2)], indirect=True)
            def test_1(animal, echo):
                assert animal in ('dog', 'cat')
                assert echo in (1, 2, 3)

            @pytest.mark.parametrize('animal, echo', [("fish", 3)], indirect=True)
            def test_2(animal, echo):
                assert animal == 'fish'
                assert echo in (1, 2, 3)
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 3 passed *"])

    def test_parametrize_some_arguments_auto_scope(self, testdir, monkeypatch):
        """Integration test for (#3941)"""
        class_fix_setup = []
        monkeypatch.setattr(sys, "class_fix_setup", class_fix_setup, raising=False)
        func_fix_setup = []
        monkeypatch.setattr(sys, "func_fix_setup", func_fix_setup, raising=False)

        testdir.makepyfile(
            """
            import pytest
            import sys

            @pytest.fixture(scope='class', autouse=True)
            def class_fix(request):
                sys.class_fix_setup.append(request.param)

            @pytest.fixture(autouse=True)
            def func_fix():
                sys.func_fix_setup.append(True)

            @pytest.mark.parametrize('class_fix', [10, 20], indirect=True)
            class Test:
                def test_foo(self):
                    pass
                def test_bar(self):
                    pass
            """
        )
        result = testdir.runpytest_inprocess()
        result.stdout.fnmatch_lines(["* 4 passed in *"])
        assert func_fix_setup == [True] * 4
        assert class_fix_setup == [10, 20]

    def test_parametrize_issue634(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture(scope='module')
            def foo(request):
                print('preparing foo-%d' % request.param)
                return 'foo-%d' % request.param

            def test_one(foo):
                pass

            def test_two(foo):
                pass

            test_two.test_with = (2, 3)

            def pytest_generate_tests(metafunc):
                params = (1, 2, 3, 4)
                if not 'foo' in metafunc.fixturenames:
                    return

                test_with = getattr(metafunc.function, 'test_with', None)
                if test_with:
                    params = test_with
                metafunc.parametrize('foo', params, indirect=True)
        """
        )
        result = testdir.runpytest("-s")
        output = result.stdout.str()
        assert output.count("preparing foo-2") == 1
        assert output.count("preparing foo-3") == 1


class TestMarkersWithParametrization(object):
    """#308"""

    def test_simple_mark(self, testdir):
        s = """
            import pytest

            @pytest.mark.foo
            @pytest.mark.parametrize(("n", "expected"), [
                (1, 2),
                pytest.param(1, 3, marks=pytest.mark.bar),
                (2, 3),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """
        items = testdir.getitems(s)
        assert len(items) == 3
        for item in items:
            assert "foo" in item.keywords
        assert "bar" not in items[0].keywords
        assert "bar" in items[1].keywords
        assert "bar" not in items[2].keywords

    def test_select_based_on_mark(self, testdir):
        s = """
            import pytest

            @pytest.mark.parametrize(("n", "expected"), [
                (1, 2),
                pytest.param(2, 3, marks=pytest.mark.foo),
                (3, 4),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """
        testdir.makepyfile(s)
        rec = testdir.inline_run("-m", "foo", SHOW_PYTEST_WARNINGS_ARG)
        passed, skipped, fail = rec.listoutcomes()
        assert len(passed) == 1
        assert len(skipped) == 0
        assert len(fail) == 0

    @pytest.mark.xfail(reason="is this important to support??")
    def test_nested_marks(self, testdir):
        s = """
            import pytest
            mastermark = pytest.mark.foo(pytest.mark.bar)

            @pytest.mark.parametrize(("n", "expected"), [
                (1, 2),
                mastermark((1, 3)),
                (2, 3),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """
        items = testdir.getitems(s)
        assert len(items) == 3
        for mark in ["foo", "bar"]:
            assert mark not in items[0].keywords
            assert mark in items[1].keywords
            assert mark not in items[2].keywords

    def test_simple_xfail(self, testdir):
        s = """
            import pytest

            @pytest.mark.parametrize(("n", "expected"), [
                (1, 2),
                pytest.param(1, 3, marks=pytest.mark.xfail),
                (2, 3),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """
        testdir.makepyfile(s)
        reprec = testdir.inline_run(SHOW_PYTEST_WARNINGS_ARG)
        # xfail is skip??
        reprec.assertoutcome(passed=2, skipped=1)

    def test_simple_xfail_single_argname(self, testdir):
        s = """
            import pytest

            @pytest.mark.parametrize("n", [
                2,
                pytest.param(3, marks=pytest.mark.xfail),
                4,
            ])
            def test_isEven(n):
                assert n % 2 == 0
        """
        testdir.makepyfile(s)
        reprec = testdir.inline_run(SHOW_PYTEST_WARNINGS_ARG)
        reprec.assertoutcome(passed=2, skipped=1)

    def test_xfail_with_arg(self, testdir):
        s = """
            import pytest

            @pytest.mark.parametrize(("n", "expected"), [
                (1, 2),
                pytest.param(1, 3, marks=pytest.mark.xfail("True")),
                (2, 3),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """
        testdir.makepyfile(s)
        reprec = testdir.inline_run(SHOW_PYTEST_WARNINGS_ARG)
        reprec.assertoutcome(passed=2, skipped=1)

    def test_xfail_with_kwarg(self, testdir):
        s = """
            import pytest

            @pytest.mark.parametrize(("n", "expected"), [
                (1, 2),
                pytest.param(1, 3, marks=pytest.mark.xfail(reason="some bug")),
                (2, 3),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """
        testdir.makepyfile(s)
        reprec = testdir.inline_run(SHOW_PYTEST_WARNINGS_ARG)
        reprec.assertoutcome(passed=2, skipped=1)

    def test_xfail_with_arg_and_kwarg(self, testdir):
        s = """
            import pytest

            @pytest.mark.parametrize(("n", "expected"), [
                (1, 2),
                pytest.param(1, 3, marks=pytest.mark.xfail("True", reason="some bug")),
                (2, 3),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """
        testdir.makepyfile(s)
        reprec = testdir.inline_run(SHOW_PYTEST_WARNINGS_ARG)
        reprec.assertoutcome(passed=2, skipped=1)

    @pytest.mark.parametrize("strict", [True, False])
    def test_xfail_passing_is_xpass(self, testdir, strict):
        s = """
            import pytest

            m = pytest.mark.xfail("sys.version_info > (0, 0, 0)", reason="some bug", strict={strict})

            @pytest.mark.parametrize(("n", "expected"), [
                (1, 2),
                pytest.param(2, 3, marks=m),
                (3, 4),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """.format(
            strict=strict
        )
        testdir.makepyfile(s)
        reprec = testdir.inline_run(SHOW_PYTEST_WARNINGS_ARG)
        passed, failed = (2, 1) if strict else (3, 0)
        reprec.assertoutcome(passed=passed, failed=failed)

    def test_parametrize_called_in_generate_tests(self, testdir):
        s = """
            import pytest


            def pytest_generate_tests(metafunc):
                passingTestData = [(1, 2),
                                   (2, 3)]
                failingTestData = [(1, 3),
                                   (2, 2)]

                testData = passingTestData + [pytest.param(*d, marks=pytest.mark.xfail)
                                  for d in failingTestData]
                metafunc.parametrize(("n", "expected"), testData)


            def test_increment(n, expected):
                assert n + 1 == expected
        """
        testdir.makepyfile(s)
        reprec = testdir.inline_run(SHOW_PYTEST_WARNINGS_ARG)
        reprec.assertoutcome(passed=2, skipped=2)

    def test_parametrize_ID_generation_string_int_works(self, testdir):
        """#290"""
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture
            def myfixture():
                return 'example'
            @pytest.mark.parametrize(
                'limit', (0, '0'))
            def test_limit(limit, myfixture):
                return
        """
        )
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=2)

    @pytest.mark.parametrize("strict", [True, False])
    def test_parametrize_marked_value(self, testdir, strict):
        s = """
            import pytest

            @pytest.mark.parametrize(("n", "expected"), [
                pytest.param(
                    2,3,
                    marks=pytest.mark.xfail("sys.version_info > (0, 0, 0)", reason="some bug", strict={strict}),
                ),
                pytest.param(
                    2,3,
                    marks=[pytest.mark.xfail("sys.version_info > (0, 0, 0)", reason="some bug", strict={strict})],
                ),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """.format(
            strict=strict
        )
        testdir.makepyfile(s)
        reprec = testdir.inline_run()
        passed, failed = (0, 2) if strict else (2, 0)
        reprec.assertoutcome(passed=passed, failed=failed)

    def test_pytest_make_parametrize_id(self, testdir):
        testdir.makeconftest(
            """
            def pytest_make_parametrize_id(config, val):
                return str(val * 2)
        """
        )
        testdir.makepyfile(
            """
                import pytest

                @pytest.mark.parametrize("x", range(2))
                def test_func(x):
                    pass
                """
        )
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines(["*test_func*0*PASS*", "*test_func*2*PASS*"])

    def test_pytest_make_parametrize_id_with_argname(self, testdir):
        testdir.makeconftest(
            """
            def pytest_make_parametrize_id(config, val, argname):
                return str(val * 2 if argname == 'x' else val * 10)
        """
        )
        testdir.makepyfile(
            """
                import pytest

                @pytest.mark.parametrize("x", range(2))
                def test_func_a(x):
                    pass

                @pytest.mark.parametrize("y", [1])
                def test_func_b(y):
                    pass
                """
        )
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines(
            ["*test_func_a*0*PASS*", "*test_func_a*2*PASS*", "*test_func_b*10*PASS*"]
        )

    def test_parametrize_positional_args(self, testdir):
        testdir.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize("a", [1], False)
            def test_foo(a):
                pass
        """
        )
        result = testdir.runpytest()
        result.assert_outcomes(passed=1)
