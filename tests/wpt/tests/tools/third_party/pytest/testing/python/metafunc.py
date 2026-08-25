# mypy: allow-untyped-defs
import dataclasses
import itertools
import re
import sys
import textwrap
from typing import Any
from typing import cast
from typing import Dict
from typing import Iterator
from typing import List
from typing import Optional
from typing import Sequence
from typing import Tuple
from typing import Union

import hypothesis
from hypothesis import strategies

from _pytest import fixtures
from _pytest import python
from _pytest.compat import getfuncargnames
from _pytest.compat import NOTSET
from _pytest.outcomes import fail
from _pytest.pytester import Pytester
from _pytest.python import Function
from _pytest.python import IdMaker
from _pytest.scope import Scope
import pytest


class TestMetafunc:
    def Metafunc(self, func, config=None) -> python.Metafunc:
        # The unit tests of this class check if things work correctly
        # on the funcarg level, so we don't need a full blown
        # initialization.
        class FuncFixtureInfoMock:
            name2fixturedefs: Dict[str, List[fixtures.FixtureDef[object]]] = {}

            def __init__(self, names):
                self.names_closure = names

        @dataclasses.dataclass
        class FixtureManagerMock:
            config: Any

        @dataclasses.dataclass
        class SessionMock:
            _fixturemanager: FixtureManagerMock

        @dataclasses.dataclass
        class DefinitionMock(python.FunctionDefinition):
            _nodeid: str
            obj: object

        names = getfuncargnames(func)
        fixtureinfo: Any = FuncFixtureInfoMock(names)
        definition: Any = DefinitionMock._create(obj=func, _nodeid="mock::nodeid")
        definition._fixtureinfo = fixtureinfo
        definition.session = SessionMock(FixtureManagerMock({}))
        return python.Metafunc(definition, fixtureinfo, config, _ispytest=True)

    def test_no_funcargs(self) -> None:
        def function():
            pass

        metafunc = self.Metafunc(function)
        assert not metafunc.fixturenames
        repr(metafunc._calls)

    def test_function_basic(self) -> None:
        def func(arg1, arg2="qwe"):
            pass

        metafunc = self.Metafunc(func)
        assert len(metafunc.fixturenames) == 1
        assert "arg1" in metafunc.fixturenames
        assert metafunc.function is func
        assert metafunc.cls is None

    def test_parametrize_error(self) -> None:
        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x", [1, 2])
        pytest.raises(ValueError, lambda: metafunc.parametrize("x", [5, 6]))
        pytest.raises(ValueError, lambda: metafunc.parametrize("x", [5, 6]))
        metafunc.parametrize("y", [1, 2])
        pytest.raises(ValueError, lambda: metafunc.parametrize("y", [5, 6]))
        pytest.raises(ValueError, lambda: metafunc.parametrize("y", [5, 6]))

        with pytest.raises(TypeError, match="^ids must be a callable or an iterable$"):
            metafunc.parametrize("y", [5, 6], ids=42)  # type: ignore[arg-type]

    def test_parametrize_error_iterator(self) -> None:
        def func(x):
            raise NotImplementedError()

        class Exc(Exception):
            def __repr__(self):
                return "Exc(from_gen)"

        def gen() -> Iterator[Union[int, None, Exc]]:
            yield 0
            yield None
            yield Exc()

        metafunc = self.Metafunc(func)
        # When the input is an iterator, only len(args) are taken,
        # so the bad Exc isn't reached.
        metafunc.parametrize("x", [1, 2], ids=gen())
        assert [(x.params, x.id) for x in metafunc._calls] == [
            ({"x": 1}, "0"),
            ({"x": 2}, "2"),
        ]
        with pytest.raises(
            fail.Exception,
            match=(
                r"In func: ids contains unsupported value Exc\(from_gen\) \(type: <class .*Exc'>\) at index 2. "
                r"Supported types are: .*"
            ),
        ):
            metafunc.parametrize("x", [1, 2, 3], ids=gen())

    def test_parametrize_bad_scope(self) -> None:
        def func(x):
            pass

        metafunc = self.Metafunc(func)
        with pytest.raises(
            fail.Exception,
            match=r"parametrize\(\) call in func got an unexpected scope value 'doggy'",
        ):
            metafunc.parametrize("x", [1], scope="doggy")  # type: ignore[arg-type]

    def test_parametrize_request_name(self, pytester: Pytester) -> None:
        """Show proper error  when 'request' is used as a parameter name in parametrize (#6183)"""

        def func(request):
            raise NotImplementedError()

        metafunc = self.Metafunc(func)
        with pytest.raises(
            fail.Exception,
            match=r"'request' is a reserved name and cannot be used in @pytest.mark.parametrize",
        ):
            metafunc.parametrize("request", [1])

    def test_find_parametrized_scope(self) -> None:
        """Unit test for _find_parametrized_scope (#3941)."""
        from _pytest.python import _find_parametrized_scope

        @dataclasses.dataclass
        class DummyFixtureDef:
            _scope: Scope

        fixtures_defs = cast(
            Dict[str, Sequence[fixtures.FixtureDef[object]]],
            dict(
                session_fix=[DummyFixtureDef(Scope.Session)],
                package_fix=[DummyFixtureDef(Scope.Package)],
                module_fix=[DummyFixtureDef(Scope.Module)],
                class_fix=[DummyFixtureDef(Scope.Class)],
                func_fix=[DummyFixtureDef(Scope.Function)],
                mixed_fix=[DummyFixtureDef(Scope.Module), DummyFixtureDef(Scope.Class)],
            ),
        )

        # use arguments to determine narrow scope; the cause of the bug is that it would look on all
        # fixture defs given to the method
        def find_scope(argnames, indirect):
            return _find_parametrized_scope(argnames, fixtures_defs, indirect=indirect)

        assert find_scope(["func_fix"], indirect=True) == Scope.Function
        assert find_scope(["class_fix"], indirect=True) == Scope.Class
        assert find_scope(["module_fix"], indirect=True) == Scope.Module
        assert find_scope(["package_fix"], indirect=True) == Scope.Package
        assert find_scope(["session_fix"], indirect=True) == Scope.Session

        assert find_scope(["class_fix", "func_fix"], indirect=True) == Scope.Function
        assert find_scope(["func_fix", "session_fix"], indirect=True) == Scope.Function
        assert find_scope(["session_fix", "class_fix"], indirect=True) == Scope.Class
        assert (
            find_scope(["package_fix", "session_fix"], indirect=True) == Scope.Package
        )
        assert find_scope(["module_fix", "session_fix"], indirect=True) == Scope.Module

        # when indirect is False or is not for all scopes, always use function
        assert (
            find_scope(["session_fix", "module_fix"], indirect=False) == Scope.Function
        )
        assert (
            find_scope(["session_fix", "module_fix"], indirect=["module_fix"])
            == Scope.Function
        )
        assert (
            find_scope(
                ["session_fix", "module_fix"], indirect=["session_fix", "module_fix"]
            )
            == Scope.Module
        )
        assert find_scope(["mixed_fix"], indirect=True) == Scope.Class

    def test_parametrize_and_id(self) -> None:
        def func(x, y):
            pass

        metafunc = self.Metafunc(func)

        metafunc.parametrize("x", [1, 2], ids=["basic", "advanced"])
        metafunc.parametrize("y", ["abc", "def"])
        ids = [x.id for x in metafunc._calls]
        assert ids == ["basic-abc", "basic-def", "advanced-abc", "advanced-def"]

    def test_parametrize_and_id_unicode(self) -> None:
        """Allow unicode strings for "ids" parameter in Python 2 (##1905)"""

        def func(x):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x", [1, 2], ids=["basic", "advanced"])
        ids = [x.id for x in metafunc._calls]
        assert ids == ["basic", "advanced"]

    def test_parametrize_with_wrong_number_of_ids(self) -> None:
        def func(x, y):
            pass

        metafunc = self.Metafunc(func)

        with pytest.raises(fail.Exception):
            metafunc.parametrize("x", [1, 2], ids=["basic"])

        with pytest.raises(fail.Exception):
            metafunc.parametrize(
                ("x", "y"), [("abc", "def"), ("ghi", "jkl")], ids=["one"]
            )

    def test_parametrize_ids_iterator_without_mark(self) -> None:
        def func(x, y):
            pass

        it = itertools.count()

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x", [1, 2], ids=it)
        metafunc.parametrize("y", [3, 4], ids=it)
        ids = [x.id for x in metafunc._calls]
        assert ids == ["0-2", "0-3", "1-2", "1-3"]

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x", [1, 2], ids=it)
        metafunc.parametrize("y", [3, 4], ids=it)
        ids = [x.id for x in metafunc._calls]
        assert ids == ["4-6", "4-7", "5-6", "5-7"]

    def test_parametrize_empty_list(self) -> None:
        """#510"""

        def func(y):
            pass

        class MockConfig:
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

    def test_parametrize_with_userobjects(self) -> None:
        def func(x, y):
            pass

        metafunc = self.Metafunc(func)

        class A:
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
    def test_idval_hypothesis(self, value) -> None:
        escaped = IdMaker([], [], None, None, None, None, None)._idval(value, "a", 6)
        assert isinstance(escaped, str)
        escaped.encode("ascii")

    def test_unicode_idval(self) -> None:
        """Test that Unicode strings outside the ASCII character set get
        escaped, using byte escapes if they're in that range or unicode
        escapes if they're not.

        """
        values = [
            ("", r""),
            ("ascii", r"ascii"),
            ("ação", r"a\xe7\xe3o"),
            ("josé@blah.com", r"jos\xe9@blah.com"),
            (
                r"δοκ.ιμή@παράδειγμα.δοκιμή",
                r"\u03b4\u03bf\u03ba.\u03b9\u03bc\u03ae@\u03c0\u03b1\u03c1\u03ac\u03b4\u03b5\u03b9\u03b3"
                r"\u03bc\u03b1.\u03b4\u03bf\u03ba\u03b9\u03bc\u03ae",
            ),
        ]
        for val, expected in values:
            assert (
                IdMaker([], [], None, None, None, None, None)._idval(val, "a", 6)
                == expected
            )

    def test_unicode_idval_with_config(self) -> None:
        """Unit test for expected behavior to obtain ids with
        disable_test_id_escaping_and_forfeit_all_rights_to_community_support
        option (#5294)."""

        class MockConfig:
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

        values: List[Tuple[str, Any, str]] = [
            ("ação", MockConfig({option: True}), "ação"),
            ("ação", MockConfig({option: False}), "a\\xe7\\xe3o"),
        ]
        for val, config, expected in values:
            actual = IdMaker([], [], None, None, config, None, None)._idval(val, "a", 6)
            assert actual == expected

    def test_bytes_idval(self) -> None:
        """Unit test for the expected behavior to obtain ids for parametrized
        bytes values: bytes objects are always escaped using "binary escape"."""
        values = [
            (b"", r""),
            (b"\xc3\xb4\xff\xe4", r"\xc3\xb4\xff\xe4"),
            (b"ascii", r"ascii"),
            ("αρά".encode(), r"\xce\xb1\xcf\x81\xce\xac"),
        ]
        for val, expected in values:
            assert (
                IdMaker([], [], None, None, None, None, None)._idval(val, "a", 6)
                == expected
            )

    def test_class_or_function_idval(self) -> None:
        """Unit test for the expected behavior to obtain ids for parametrized
        values that are classes or functions: their __name__."""

        class TestClass:
            pass

        def test_function():
            pass

        values = [(TestClass, "TestClass"), (test_function, "test_function")]
        for val, expected in values:
            assert (
                IdMaker([], [], None, None, None, None, None)._idval(val, "a", 6)
                == expected
            )

    def test_notset_idval(self) -> None:
        """Test that a NOTSET value (used by an empty parameterset) generates
        a proper ID.

        Regression test for #7686.
        """
        assert (
            IdMaker([], [], None, None, None, None, None)._idval(NOTSET, "a", 0) == "a0"
        )

    def test_idmaker_autoname(self) -> None:
        """#250"""
        result = IdMaker(
            ("a", "b"),
            [pytest.param("string", 1.0), pytest.param("st-ring", 2.0)],
            None,
            None,
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
        assert result == ["string-1.0", "st-ring-2.0"]

        result = IdMaker(
            ("a", "b"),
            [pytest.param(object(), 1.0), pytest.param(object(), object())],
            None,
            None,
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
        assert result == ["a0-1.0", "a1-b1"]
        # unicode mixing, issue250
        result = IdMaker(
            ("a", "b"), [pytest.param({}, b"\xc3\xb4")], None, None, None, None, None
        ).make_unique_parameterset_ids()
        assert result == ["a0-\\xc3\\xb4"]

    def test_idmaker_with_bytes_regex(self) -> None:
        result = IdMaker(
            ("a"), [pytest.param(re.compile(b"foo"), 1.0)], None, None, None, None, None
        ).make_unique_parameterset_ids()
        assert result == ["foo"]

    def test_idmaker_native_strings(self) -> None:
        result = IdMaker(
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
                pytest.param(b"\xc3\xb4", "other"),
                pytest.param(1.0j, -2.0j),
            ],
            None,
            None,
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
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
            "1j-(-0-2j)",
        ]

    def test_idmaker_non_printable_characters(self) -> None:
        result = IdMaker(
            ("s", "n"),
            [
                pytest.param("\x00", 1),
                pytest.param("\x05", 2),
                pytest.param(b"\x00", 3),
                pytest.param(b"\x05", 4),
                pytest.param("\t", 5),
                pytest.param(b"\t", 6),
            ],
            None,
            None,
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
        assert result == ["\\x00-1", "\\x05-2", "\\x00-3", "\\x05-4", "\\t-5", "\\t-6"]

    def test_idmaker_manual_ids_must_be_printable(self) -> None:
        result = IdMaker(
            ("s",),
            [
                pytest.param("x00", id="hello \x00"),
                pytest.param("x05", id="hello \x05"),
            ],
            None,
            None,
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
        assert result == ["hello \\x00", "hello \\x05"]

    def test_idmaker_enum(self) -> None:
        enum = pytest.importorskip("enum")
        e = enum.Enum("Foo", "one, two")
        result = IdMaker(
            ("a", "b"), [pytest.param(e.one, e.two)], None, None, None, None, None
        ).make_unique_parameterset_ids()
        assert result == ["Foo.one-Foo.two"]

    def test_idmaker_idfn(self) -> None:
        """#351"""

        def ids(val: object) -> Optional[str]:
            if isinstance(val, Exception):
                return repr(val)
            return None

        result = IdMaker(
            ("a", "b"),
            [
                pytest.param(10.0, IndexError()),
                pytest.param(20, KeyError()),
                pytest.param("three", [1, 2, 3]),
            ],
            ids,
            None,
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
        assert result == ["10.0-IndexError()", "20-KeyError()", "three-b2"]

    def test_idmaker_idfn_unique_names(self) -> None:
        """#351"""

        def ids(val: object) -> str:
            return "a"

        result = IdMaker(
            ("a", "b"),
            [
                pytest.param(10.0, IndexError()),
                pytest.param(20, KeyError()),
                pytest.param("three", [1, 2, 3]),
            ],
            ids,
            None,
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
        assert result == ["a-a0", "a-a1", "a-a2"]

    def test_idmaker_with_idfn_and_config(self) -> None:
        """Unit test for expected behavior to create ids with idfn and
        disable_test_id_escaping_and_forfeit_all_rights_to_community_support
        option (#5294).
        """

        class MockConfig:
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

        values: List[Tuple[Any, str]] = [
            (MockConfig({option: True}), "ação"),
            (MockConfig({option: False}), "a\\xe7\\xe3o"),
        ]
        for config, expected in values:
            result = IdMaker(
                ("a",),
                [pytest.param("string")],
                lambda _: "ação",
                None,
                config,
                None,
                None,
            ).make_unique_parameterset_ids()
            assert result == [expected]

    def test_idmaker_with_ids_and_config(self) -> None:
        """Unit test for expected behavior to create ids with ids and
        disable_test_id_escaping_and_forfeit_all_rights_to_community_support
        option (#5294).
        """

        class MockConfig:
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

        values: List[Tuple[Any, str]] = [
            (MockConfig({option: True}), "ação"),
            (MockConfig({option: False}), "a\\xe7\\xe3o"),
        ]
        for config, expected in values:
            result = IdMaker(
                ("a",), [pytest.param("string")], None, ["ação"], config, None, None
            ).make_unique_parameterset_ids()
            assert result == [expected]

    def test_idmaker_duplicated_empty_str(self) -> None:
        """Regression test for empty strings parametrized more than once (#11563)."""
        result = IdMaker(
            ("a",), [pytest.param(""), pytest.param("")], None, None, None, None, None
        ).make_unique_parameterset_ids()
        assert result == ["0", "1"]

    def test_parametrize_ids_exception(self, pytester: Pytester) -> None:
        """
        :param pytester: the instance of Pytester class, a temporary
        test directory.
        """
        pytester.makepyfile(
            """
                import pytest

                def ids(arg):
                    raise Exception("bad ids")

                @pytest.mark.parametrize("arg", ["a", "b"], ids=ids)
                def test_foo(arg):
                    pass
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*Exception: bad ids",
                "*test_foo: error raised while trying to determine id of parameter 'arg' at position 0",
            ]
        )

    def test_parametrize_ids_returns_non_string(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """\
            import pytest

            def ids(d):
                return d

            @pytest.mark.parametrize("arg", ({1: 2}, {3, 4}), ids=ids)
            def test(arg):
                assert arg

            @pytest.mark.parametrize("arg", (1, 2.0, True), ids=ids)
            def test_int(arg):
                assert arg
            """
        )
        result = pytester.runpytest("-vv", "-s")
        result.stdout.fnmatch_lines(
            [
                "test_parametrize_ids_returns_non_string.py::test[arg0] PASSED",
                "test_parametrize_ids_returns_non_string.py::test[arg1] PASSED",
                "test_parametrize_ids_returns_non_string.py::test_int[1] PASSED",
                "test_parametrize_ids_returns_non_string.py::test_int[2.0] PASSED",
                "test_parametrize_ids_returns_non_string.py::test_int[True] PASSED",
            ]
        )

    def test_idmaker_with_ids(self) -> None:
        result = IdMaker(
            ("a", "b"),
            [pytest.param(1, 2), pytest.param(3, 4)],
            None,
            ["a", None],
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
        assert result == ["a", "3-4"]

    def test_idmaker_with_paramset_id(self) -> None:
        result = IdMaker(
            ("a", "b"),
            [pytest.param(1, 2, id="me"), pytest.param(3, 4, id="you")],
            None,
            ["a", None],
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
        assert result == ["me", "you"]

    def test_idmaker_with_ids_unique_names(self) -> None:
        result = IdMaker(
            ("a"),
            list(map(pytest.param, [1, 2, 3, 4, 5])),
            None,
            ["a", "a", "b", "c", "b"],
            None,
            None,
            None,
        ).make_unique_parameterset_ids()
        assert result == ["a0", "a1", "b0", "c", "b1"]

    def test_parametrize_indirect(self) -> None:
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x", [1], indirect=True)
        metafunc.parametrize("y", [2, 3], indirect=True)
        assert len(metafunc._calls) == 2
        assert metafunc._calls[0].params == dict(x=1, y=2)
        assert metafunc._calls[1].params == dict(x=1, y=3)

    def test_parametrize_indirect_list(self) -> None:
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x, y", [("a", "b")], indirect=["x"])
        assert metafunc._calls[0].params == dict(x="a", y="b")
        # Since `y` is a direct parameter, its pseudo-fixture would
        # be registered.
        assert list(metafunc._arg2fixturedefs.keys()) == ["y"]

    def test_parametrize_indirect_list_all(self) -> None:
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x, y", [("a", "b")], indirect=["x", "y"])
        assert metafunc._calls[0].params == dict(x="a", y="b")
        assert list(metafunc._arg2fixturedefs.keys()) == []

    def test_parametrize_indirect_list_empty(self) -> None:
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        metafunc.parametrize("x, y", [("a", "b")], indirect=[])
        assert metafunc._calls[0].params == dict(x="a", y="b")
        assert list(metafunc._arg2fixturedefs.keys()) == ["x", "y"]

    def test_parametrize_indirect_wrong_type(self) -> None:
        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        with pytest.raises(
            fail.Exception,
            match="In func: expected Sequence or boolean for indirect, got dict",
        ):
            metafunc.parametrize("x, y", [("a", "b")], indirect={})  # type: ignore[arg-type]

    def test_parametrize_indirect_list_functional(self, pytester: Pytester) -> None:
        """
        #714
        Test parametrization with 'indirect' parameter applied on
        particular arguments. As y is direct, its value should
        be used directly rather than being passed to the fixture y.

        :param pytester: the instance of Pytester class, a temporary
        test directory.
        """
        pytester.makepyfile(
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
        result = pytester.runpytest("-v")
        result.stdout.fnmatch_lines(["*test_simple*a-b*", "*1 passed*"])

    def test_parametrize_indirect_list_error(self) -> None:
        """#714"""

        def func(x, y):
            pass

        metafunc = self.Metafunc(func)
        with pytest.raises(fail.Exception):
            metafunc.parametrize("x, y", [("a", "b")], indirect=["x", "z"])

    def test_parametrize_uses_no_fixture_error_indirect_false(
        self, pytester: Pytester
    ) -> None:
        """The 'uses no fixture' error tells the user at collection time
        that the parametrize data they've set up doesn't correspond to the
        fixtures in their test function, rather than silently ignoring this
        and letting the test potentially pass.

        #714
        """
        pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize('x, y', [('a', 'b')], indirect=False)
            def test_simple(x):
                assert len(x) == 3
        """
        )
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no argument 'y'*"])

    def test_parametrize_uses_no_fixture_error_indirect_true(
        self, pytester: Pytester
    ) -> None:
        """#714"""
        pytester.makepyfile(
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
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no fixture 'y'*"])

    def test_parametrize_indirect_uses_no_fixture_error_indirect_string(
        self, pytester: Pytester
    ) -> None:
        """#714"""
        pytester.makepyfile(
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
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no fixture 'y'*"])

    def test_parametrize_indirect_uses_no_fixture_error_indirect_list(
        self, pytester: Pytester
    ) -> None:
        """#714"""
        pytester.makepyfile(
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
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no fixture 'y'*"])

    def test_parametrize_argument_not_in_indirect_list(
        self, pytester: Pytester
    ) -> None:
        """#714"""
        pytester.makepyfile(
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
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(["*uses no argument 'y'*"])

    def test_parametrize_gives_indicative_error_on_function_with_default_argument(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize('x, y', [('a', 'b')])
            def test_simple(x, y=1):
                assert len(x) == 1
        """
        )
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(
            ["*already takes an argument 'y' with a default value"]
        )

    def test_parametrize_functional(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest("-v")
        result.stdout.fnmatch_lines(
            ["*test_simple*1-2*", "*test_simple*2-2*", "*2 passed*"]
        )

    def test_parametrize_onearg(self) -> None:
        metafunc = self.Metafunc(lambda x: None)
        metafunc.parametrize("x", [1, 2])
        assert len(metafunc._calls) == 2
        assert metafunc._calls[0].params == dict(x=1)
        assert metafunc._calls[0].id == "1"
        assert metafunc._calls[1].params == dict(x=2)
        assert metafunc._calls[1].id == "2"

    def test_parametrize_onearg_indirect(self) -> None:
        metafunc = self.Metafunc(lambda x: None)
        metafunc.parametrize("x", [1, 2], indirect=True)
        assert metafunc._calls[0].params == dict(x=1)
        assert metafunc._calls[0].id == "1"
        assert metafunc._calls[1].params == dict(x=2)
        assert metafunc._calls[1].id == "2"

    def test_parametrize_twoargs(self) -> None:
        metafunc = self.Metafunc(lambda x, y: None)
        metafunc.parametrize(("x", "y"), [(1, 2), (3, 4)])
        assert len(metafunc._calls) == 2
        assert metafunc._calls[0].params == dict(x=1, y=2)
        assert metafunc._calls[0].id == "1-2"
        assert metafunc._calls[1].params == dict(x=3, y=4)
        assert metafunc._calls[1].id == "3-4"

    def test_high_scoped_parametrize_reordering(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize("arg2", [3, 4])
            @pytest.mark.parametrize("arg1", [0, 1, 2], scope='module')
            def test1(arg1, arg2):
                pass

            def test2():
                pass

            @pytest.mark.parametrize("arg1", [0, 1, 2], scope='module')
            def test3(arg1):
                pass
        """
        )
        result = pytester.runpytest("--collect-only")
        result.stdout.re_match_lines(
            [
                r"    <Function test1\[0-3\]>",
                r"    <Function test1\[0-4\]>",
                r"    <Function test3\[0\]>",
                r"    <Function test1\[1-3\]>",
                r"    <Function test1\[1-4\]>",
                r"    <Function test3\[1\]>",
                r"    <Function test1\[2-3\]>",
                r"    <Function test1\[2-4\]>",
                r"    <Function test3\[2\]>",
                r"    <Function test2>",
            ]
        )

    def test_parametrize_multiple_times(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest()
        assert result.ret == 1
        result.assert_outcomes(failed=6)

    def test_parametrize_CSV(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            @pytest.mark.parametrize("x, y,", [(1,2), (2,3)])
            def test_func(x, y):
                assert x+1 == y
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_parametrize_class_scenarios(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest("-v")
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


class TestMetafuncFunctional:
    def test_attributes(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            # assumes that generate/provide runs in the same process
            import sys, pytest
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
                    unbound = TestClass.test_method
                    assert metafunc.function == unbound
                    assert metafunc.cls == TestClass
        """
        )
        result = pytester.runpytest(p, "-v")
        result.assert_outcomes(passed=2)

    def test_two_functions(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('arg1', [10, 20], ids=['0', '1'])

            def test_func1(arg1):
                assert arg1 == 10

            def test_func2(arg1):
                assert arg1 in (10, 20)
        """
        )
        result = pytester.runpytest("-v", p)
        result.stdout.fnmatch_lines(
            [
                "*test_func1*0*PASS*",
                "*test_func1*1*FAIL*",
                "*test_func2*PASS*",
                "*test_func2*PASS*",
                "*1 failed, 3 passed*",
            ]
        )

    def test_noself_in_method(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                assert 'xyz' not in metafunc.fixturenames

            class TestHello(object):
                def test_hello(xyz):
                    pass
        """
        )
        result = pytester.runpytest(p)
        result.assert_outcomes(passed=1)

    def test_generate_tests_in_class(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            class TestClass(object):
                def pytest_generate_tests(self, metafunc):
                    metafunc.parametrize('hello', ['world'], ids=['hellow'])

                def test_myfunc(self, hello):
                    assert hello == "world"
        """
        )
        result = pytester.runpytest("-v", p)
        result.stdout.fnmatch_lines(["*test_myfunc*hello*PASS*", "*1 passed*"])

    def test_two_functions_not_same_instance(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('arg1', [10, 20], ids=["0", "1"])

            class TestClass(object):
                def test_func(self, arg1):
                    assert not hasattr(self, 'x')
                    self.x = 1
        """
        )
        result = pytester.runpytest("-v", p)
        result.stdout.fnmatch_lines(
            ["*test_func*0*PASS*", "*test_func*1*PASS*", "*2 pass*"]
        )

    def test_issue28_setup_method_in_generate_tests(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
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
        result = pytester.runpytest(p)
        result.assert_outcomes(passed=1)

    def test_parametrize_functional2(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize("arg1", [1,2])
                metafunc.parametrize("arg2", [4,5])
            def test_hello(arg1, arg2):
                assert 0, (arg1, arg2)
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            ["*(1, 4)*", "*(1, 5)*", "*(2, 4)*", "*(2, 5)*", "*4 failed*"]
        )

    def test_parametrize_and_inner_getfixturevalue(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
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
        result = pytester.runpytest("-v", p)
        result.stdout.fnmatch_lines(["*test_func1*1*PASS*", "*1 passed*"])

    def test_parametrize_on_setup_arg(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
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
        result = pytester.runpytest("-v", p)
        result.stdout.fnmatch_lines(["*test_func*1*PASS*", "*1 passed*"])

    def test_parametrize_with_ids(self, pytester: Pytester) -> None:
        pytester.makeini(
            """
            [pytest]
            console_output_style=classic
        """
        )
        pytester.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize(("a", "b"), [(1,1), (1,2)],
                                     ids=["basic", "advanced"])

            def test_function(a, b):
                assert a == b
        """
        )
        result = pytester.runpytest("-v")
        assert result.ret == 1
        result.stdout.fnmatch_lines_random(
            ["*test_function*basic*PASSED", "*test_function*advanced*FAILED"]
        )

    def test_parametrize_without_ids(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize(("a", "b"),
                                     [(1,object()), (1.3,object())])

            def test_function(a, b):
                assert 1
        """
        )
        result = pytester.runpytest("-v")
        result.stdout.fnmatch_lines(
            """
            *test_function*1-b0*
            *test_function*1.3-b1*
        """
        )

    def test_parametrize_with_None_in_ids(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize(("a", "b"), [(1,1), (1,1), (1,2)],
                                     ids=["basic", None, "advanced"])

            def test_function(a, b):
                assert a == b
        """
        )
        result = pytester.runpytest("-v")
        assert result.ret == 1
        result.stdout.fnmatch_lines_random(
            [
                "*test_function*basic*PASSED*",
                "*test_function*1-1*PASSED*",
                "*test_function*advanced*FAILED*",
            ]
        )

    def test_fixture_parametrized_empty_ids(self, pytester: Pytester) -> None:
        """Fixtures parametrized with empty ids cause an internal error (#1849)."""
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(scope="module", ids=[], params=[])
            def temp(request):
               return request.param

            def test_temp(temp):
                 pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 1 skipped *"])

    def test_parametrized_empty_ids(self, pytester: Pytester) -> None:
        """Tests parametrized with empty ids cause an internal error (#1849)."""
        pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize('temp', [], ids=list())
            def test_temp(temp):
                 pass
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 1 skipped *"])

    def test_parametrized_ids_invalid_type(self, pytester: Pytester) -> None:
        """Test error with non-strings/non-ints, without generator (#1857)."""
        pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize("x, expected", [(1, 2), (3, 4), (5, 6)], ids=(None, 2, OSError()))
            def test_ids_numbers(x,expected):
                assert x * 2 == expected
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            [
                "In test_ids_numbers: ids contains unsupported value OSError() (type: <class 'OSError'>) at index 2. "
                "Supported types are: str, bytes, int, float, complex, bool, enum, regex or anything with a __name__."
            ]
        )

    def test_parametrize_with_identical_ids_get_unique_names(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize(("a", "b"), [(1,1), (1,2)],
                                     ids=["a", "a"])

            def test_function(a, b):
                assert a == b
        """
        )
        result = pytester.runpytest("-v")
        assert result.ret == 1
        result.stdout.fnmatch_lines_random(
            ["*test_function*a0*PASSED*", "*test_function*a1*FAILED*"]
        )

    @pytest.mark.parametrize(("scope", "length"), [("module", 2), ("function", 4)])
    def test_parametrize_scope_overrides(
        self, pytester: Pytester, scope: str, length: int
    ) -> None:
        pytester.makepyfile(
            """
            import pytest
            values = []
            def pytest_generate_tests(metafunc):
                if "arg" in metafunc.fixturenames:
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
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=5)

    def test_parametrize_issue323(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        reprec = pytester.inline_run("--collect-only")
        assert not reprec.getcalls("pytest_internalerror")

    def test_usefixtures_seen_in_generate_tests(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        reprec = pytester.runpytest()
        reprec.assert_outcomes(passed=1)

    def test_generate_tests_only_done_in_subdir(self, pytester: Pytester) -> None:
        sub1 = pytester.mkpydir("sub1")
        sub2 = pytester.mkpydir("sub2")
        sub1.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                def pytest_generate_tests(metafunc):
                    assert metafunc.function.__name__ == "test_1"
                """
            ),
            encoding="utf-8",
        )
        sub2.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                def pytest_generate_tests(metafunc):
                    assert metafunc.function.__name__ == "test_2"
                """
            ),
            encoding="utf-8",
        )
        sub1.joinpath("test_in_sub1.py").write_text(
            "def test_1(): pass", encoding="utf-8"
        )
        sub2.joinpath("test_in_sub2.py").write_text(
            "def test_2(): pass", encoding="utf-8"
        )
        result = pytester.runpytest("--keep-duplicates", "-v", "-s", sub1, sub2, sub1)
        result.assert_outcomes(passed=3)

    def test_generate_same_function_names_issue403(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        reprec = pytester.runpytest()
        reprec.assert_outcomes(passed=4)

    def test_parametrize_misspelling(self, pytester: Pytester) -> None:
        """#463"""
        pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrise("x", range(2))
            def test_foo(x):
                pass
        """
        )
        result = pytester.runpytest("--collect-only")
        result.stdout.fnmatch_lines(
            [
                "collected 0 items / 1 error",
                "",
                "*= ERRORS =*",
                "*_ ERROR collecting test_parametrize_misspelling.py _*",
                "test_parametrize_misspelling.py:3: in <module>",
                '    @pytest.mark.parametrise("x", range(2))',
                "E   Failed: Unknown 'parametrise' mark, did you mean 'parametrize'?",
                "*! Interrupted: 1 error during collection !*",
                "*= no tests collected, 1 error in *",
            ]
        )

    @pytest.mark.parametrize("scope", ["class", "package"])
    def test_parametrize_missing_scope_doesnt_crash(
        self, pytester: Pytester, scope: str
    ) -> None:
        """Doesn't crash when parametrize(scope=<scope>) is used without a
        corresponding <scope> node."""
        pytester.makepyfile(
            f"""
            import pytest

            @pytest.mark.parametrize("x", [0], scope="{scope}")
            def test_it(x): pass
            """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_parametrize_module_level_test_with_class_scope(
        self, pytester: Pytester
    ) -> None:
        """
        Test that a class-scoped parametrization without a corresponding `Class`
        gets module scope, i.e. we only create a single FixtureDef for it per module.
        """
        module = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize("x", [0, 1], scope="class")
            def test_1(x):
                pass

            @pytest.mark.parametrize("x", [1, 2], scope="module")
            def test_2(x):
                pass
        """
        )
        test_1_0, _, test_2_0, _ = pytester.genitems((pytester.getmodulecol(module),))

        assert isinstance(test_1_0, Function)
        assert test_1_0.name == "test_1[0]"
        test_1_fixture_x = test_1_0._fixtureinfo.name2fixturedefs["x"][-1]

        assert isinstance(test_2_0, Function)
        assert test_2_0.name == "test_2[1]"
        test_2_fixture_x = test_2_0._fixtureinfo.name2fixturedefs["x"][-1]

        assert test_1_fixture_x is test_2_fixture_x

    def test_reordering_with_scopeless_and_just_indirect_parametrization(
        self, pytester: Pytester
    ) -> None:
        pytester.makeconftest(
            """
            import pytest

            @pytest.fixture(scope="package")
            def fixture1():
                pass
            """
        )
        pytester.makepyfile(
            """
            import pytest

            @pytest.fixture(scope="module")
            def fixture0():
                pass

            @pytest.fixture(scope="module")
            def fixture1(fixture0):
                pass

            @pytest.mark.parametrize("fixture1", [0], indirect=True)
            def test_0(fixture1):
                pass

            @pytest.fixture(scope="module")
            def fixture():
                pass

            @pytest.mark.parametrize("fixture", [0], indirect=True)
            def test_1(fixture):
                pass

            def test_2():
                pass

            class Test:
                @pytest.fixture(scope="class")
                def fixture(self, fixture):
                    pass

                @pytest.mark.parametrize("fixture", [0], indirect=True)
                def test_3(self, fixture):
                    pass
            """
        )
        result = pytester.runpytest("-v")
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            [
                "*test_0*",
                "*test_1*",
                "*test_2*",
                "*test_3*",
            ]
        )


class TestMetafuncFunctionalAuto:
    """Tests related to automatically find out the correct scope for
    parametrized tests (#1832)."""

    def test_parametrize_auto_scope(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 3 passed *"])

    def test_parametrize_auto_scope_indirect(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 3 passed *"])

    def test_parametrize_auto_scope_override_fixture(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 2 passed *"])

    def test_parametrize_all_indirects(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 3 passed *"])

    def test_parametrize_some_arguments_auto_scope(
        self, pytester: Pytester, monkeypatch
    ) -> None:
        """Integration test for (#3941)"""
        class_fix_setup: List[object] = []
        monkeypatch.setattr(sys, "class_fix_setup", class_fix_setup, raising=False)
        func_fix_setup: List[object] = []
        monkeypatch.setattr(sys, "func_fix_setup", func_fix_setup, raising=False)

        pytester.makepyfile(
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
        result = pytester.runpytest_inprocess()
        result.stdout.fnmatch_lines(["* 4 passed in *"])
        assert func_fix_setup == [True] * 4
        assert class_fix_setup == [10, 20]

    def test_parametrize_issue634(self, pytester: Pytester) -> None:
        pytester.makepyfile(
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
        result = pytester.runpytest("-s")
        output = result.stdout.str()
        assert output.count("preparing foo-2") == 1
        assert output.count("preparing foo-3") == 1


class TestMarkersWithParametrization:
    """#308"""

    def test_simple_mark(self, pytester: Pytester) -> None:
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
        items = pytester.getitems(s)
        assert len(items) == 3
        for item in items:
            assert "foo" in item.keywords
        assert "bar" not in items[0].keywords
        assert "bar" in items[1].keywords
        assert "bar" not in items[2].keywords

    def test_select_based_on_mark(self, pytester: Pytester) -> None:
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
        pytester.makepyfile(s)
        rec = pytester.inline_run("-m", "foo")
        passed, skipped, fail = rec.listoutcomes()
        assert len(passed) == 1
        assert len(skipped) == 0
        assert len(fail) == 0

    def test_simple_xfail(self, pytester: Pytester) -> None:
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
        pytester.makepyfile(s)
        reprec = pytester.inline_run()
        # xfail is skip??
        reprec.assertoutcome(passed=2, skipped=1)

    def test_simple_xfail_single_argname(self, pytester: Pytester) -> None:
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
        pytester.makepyfile(s)
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2, skipped=1)

    def test_xfail_with_arg(self, pytester: Pytester) -> None:
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
        pytester.makepyfile(s)
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2, skipped=1)

    def test_xfail_with_kwarg(self, pytester: Pytester) -> None:
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
        pytester.makepyfile(s)
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2, skipped=1)

    def test_xfail_with_arg_and_kwarg(self, pytester: Pytester) -> None:
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
        pytester.makepyfile(s)
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2, skipped=1)

    @pytest.mark.parametrize("strict", [True, False])
    def test_xfail_passing_is_xpass(self, pytester: Pytester, strict: bool) -> None:
        s = f"""
            import pytest

            m = pytest.mark.xfail("sys.version_info > (0, 0, 0)", reason="some bug", strict={strict})

            @pytest.mark.parametrize(("n", "expected"), [
                (1, 2),
                pytest.param(2, 3, marks=m),
                (3, 4),
            ])
            def test_increment(n, expected):
                assert n + 1 == expected
        """
        pytester.makepyfile(s)
        reprec = pytester.inline_run()
        passed, failed = (2, 1) if strict else (3, 0)
        reprec.assertoutcome(passed=passed, failed=failed)

    def test_parametrize_called_in_generate_tests(self, pytester: Pytester) -> None:
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
        pytester.makepyfile(s)
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2, skipped=2)

    def test_parametrize_ID_generation_string_int_works(
        self, pytester: Pytester
    ) -> None:
        """#290"""
        pytester.makepyfile(
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
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    @pytest.mark.parametrize("strict", [True, False])
    def test_parametrize_marked_value(self, pytester: Pytester, strict: bool) -> None:
        s = f"""
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
        """
        pytester.makepyfile(s)
        reprec = pytester.inline_run()
        passed, failed = (0, 2) if strict else (2, 0)
        reprec.assertoutcome(passed=passed, failed=failed)

    def test_pytest_make_parametrize_id(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_make_parametrize_id(config, val):
                return str(val * 2)
        """
        )
        pytester.makepyfile(
            """
                import pytest

                @pytest.mark.parametrize("x", range(2))
                def test_func(x):
                    pass
                """
        )
        result = pytester.runpytest("-v")
        result.stdout.fnmatch_lines(["*test_func*0*PASS*", "*test_func*2*PASS*"])

    def test_pytest_make_parametrize_id_with_argname(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_make_parametrize_id(config, val, argname):
                return str(val * 2 if argname == 'x' else val * 10)
        """
        )
        pytester.makepyfile(
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
        result = pytester.runpytest("-v")
        result.stdout.fnmatch_lines(
            ["*test_func_a*0*PASS*", "*test_func_a*2*PASS*", "*test_func_b*10*PASS*"]
        )

    def test_parametrize_positional_args(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize("a", [1], False)
            def test_foo(a):
                pass
        """
        )
        result = pytester.runpytest()
        result.assert_outcomes(passed=1)

    def test_parametrize_iterator(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import itertools
            import pytest

            id_parametrize = pytest.mark.parametrize(
                ids=("param%d" % i for i in itertools.count())
            )

            @id_parametrize('y', ['a', 'b'])
            def test1(y):
                pass

            @id_parametrize('y', ['a', 'b'])
            def test2(y):
                pass

            @pytest.mark.parametrize("a, b", [(1, 2), (3, 4)], ids=itertools.count())
            def test_converted_to_str(a, b):
                pass
        """
        )
        result = pytester.runpytest("-vv", "-s")
        result.stdout.fnmatch_lines(
            [
                "test_parametrize_iterator.py::test1[param0] PASSED",
                "test_parametrize_iterator.py::test1[param1] PASSED",
                "test_parametrize_iterator.py::test2[param0] PASSED",
                "test_parametrize_iterator.py::test2[param1] PASSED",
                "test_parametrize_iterator.py::test_converted_to_str[0] PASSED",
                "test_parametrize_iterator.py::test_converted_to_str[1] PASSED",
                "*= 6 passed in *",
            ]
        )
