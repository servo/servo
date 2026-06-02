# mypy: allow-untyped-defs
import ast
import errno
from functools import partial
import glob
import importlib
import marshal
import os
from pathlib import Path
import py_compile
import stat
import sys
import textwrap
from typing import cast
from typing import Dict
from typing import Generator
from typing import List
from typing import Mapping
from typing import Optional
from typing import Set
from unittest import mock
import zipfile

import _pytest._code
from _pytest._io.saferepr import DEFAULT_REPR_MAX_SIZE
from _pytest.assertion import util
from _pytest.assertion.rewrite import _get_assertion_exprs
from _pytest.assertion.rewrite import _get_maxsize_for_saferepr
from _pytest.assertion.rewrite import AssertionRewritingHook
from _pytest.assertion.rewrite import get_cache_dir
from _pytest.assertion.rewrite import PYC_TAIL
from _pytest.assertion.rewrite import PYTEST_TAG
from _pytest.assertion.rewrite import rewrite_asserts
from _pytest.config import Config
from _pytest.config import ExitCode
from _pytest.pathlib import make_numbered_dir
from _pytest.pytester import Pytester
import pytest


def rewrite(src: str) -> ast.Module:
    tree = ast.parse(src)
    rewrite_asserts(tree, src.encode())
    return tree


def getmsg(
    f, extra_ns: Optional[Mapping[str, object]] = None, *, must_pass: bool = False
) -> Optional[str]:
    """Rewrite the assertions in f, run it, and get the failure message."""
    src = "\n".join(_pytest._code.Code.from_function(f).source().lines)
    mod = rewrite(src)
    code = compile(mod, "<test>", "exec")
    ns: Dict[str, object] = {}
    if extra_ns is not None:
        ns.update(extra_ns)
    exec(code, ns)
    func = ns[f.__name__]
    try:
        func()  # type: ignore[operator]
    except AssertionError:
        if must_pass:
            pytest.fail("shouldn't have raised")
        s = str(sys.exc_info()[1])
        if not s.startswith("assert"):
            return "AssertionError: " + s
        return s
    else:
        if not must_pass:
            pytest.fail("function didn't raise at all")
        return None


class TestAssertionRewrite:
    def test_place_initial_imports(self) -> None:
        s = """'Doc string'\nother = stuff"""
        m = rewrite(s)
        assert isinstance(m.body[0], ast.Expr)
        for imp in m.body[1:3]:
            assert isinstance(imp, ast.Import)
            assert imp.lineno == 2
            assert imp.col_offset == 0
        assert isinstance(m.body[3], ast.Assign)
        s = """from __future__ import division\nother_stuff"""
        m = rewrite(s)
        assert isinstance(m.body[0], ast.ImportFrom)
        for imp in m.body[1:3]:
            assert isinstance(imp, ast.Import)
            assert imp.lineno == 2
            assert imp.col_offset == 0
        assert isinstance(m.body[3], ast.Expr)
        s = """'doc string'\nfrom __future__ import division"""
        m = rewrite(s)
        assert isinstance(m.body[0], ast.Expr)
        assert isinstance(m.body[1], ast.ImportFrom)
        for imp in m.body[2:4]:
            assert isinstance(imp, ast.Import)
            assert imp.lineno == 2
            assert imp.col_offset == 0
        s = """'doc string'\nfrom __future__ import division\nother"""
        m = rewrite(s)
        assert isinstance(m.body[0], ast.Expr)
        assert isinstance(m.body[1], ast.ImportFrom)
        for imp in m.body[2:4]:
            assert isinstance(imp, ast.Import)
            assert imp.lineno == 3
            assert imp.col_offset == 0
        assert isinstance(m.body[4], ast.Expr)
        s = """from . import relative\nother_stuff"""
        m = rewrite(s)
        for imp in m.body[:2]:
            assert isinstance(imp, ast.Import)
            assert imp.lineno == 1
            assert imp.col_offset == 0
        assert isinstance(m.body[3], ast.Expr)

    def test_location_is_set(self) -> None:
        s = textwrap.dedent(
            """

        assert False, (

            "Ouch"
          )

        """
        )
        m = rewrite(s)
        for node in m.body:
            if isinstance(node, ast.Import):
                continue
            for n in [node, *ast.iter_child_nodes(node)]:
                assert n.lineno == 3
                assert n.col_offset == 0
                assert n.end_lineno == 6
                assert n.end_col_offset == 3

    def test_dont_rewrite(self) -> None:
        s = """'PYTEST_DONT_REWRITE'\nassert 14"""
        m = rewrite(s)
        assert len(m.body) == 2
        assert isinstance(m.body[1], ast.Assert)
        assert m.body[1].msg is None

    def test_dont_rewrite_plugin(self, pytester: Pytester) -> None:
        contents = {
            "conftest.py": "pytest_plugins = 'plugin'; import plugin",
            "plugin.py": "'PYTEST_DONT_REWRITE'",
            "test_foo.py": "def test_foo(): pass",
        }
        pytester.makepyfile(**contents)
        result = pytester.runpytest_subprocess()
        assert "warning" not in "".join(result.outlines)

    def test_rewrites_plugin_as_a_package(self, pytester: Pytester) -> None:
        pkgdir = pytester.mkpydir("plugin")
        pkgdir.joinpath("__init__.py").write_text(
            "import pytest\n"
            "@pytest.fixture\n"
            "def special_asserter():\n"
            "    def special_assert(x, y):\n"
            "        assert x == y\n"
            "    return special_assert\n",
            encoding="utf-8",
        )
        pytester.makeconftest('pytest_plugins = ["plugin"]')
        pytester.makepyfile("def test(special_asserter): special_asserter(1, 2)\n")
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*assert 1 == 2*"])

    def test_honors_pep_235(self, pytester: Pytester, monkeypatch) -> None:
        # note: couldn't make it fail on macos with a single `sys.path` entry
        # note: these modules are named `test_*` to trigger rewriting
        pytester.makepyfile(test_y="x = 1")
        xdir = pytester.mkdir("x")
        pytester.mkpydir(str(xdir.joinpath("test_Y")))
        xdir.joinpath("test_Y").joinpath("__init__.py").write_text(
            "x = 2", encoding="utf-8"
        )
        pytester.makepyfile(
            "import test_y\n"
            "import test_Y\n"
            "def test():\n"
            "    assert test_y.x == 1\n"
            "    assert test_Y.x == 2\n"
        )
        monkeypatch.syspath_prepend(str(xdir))
        pytester.runpytest().assert_outcomes(passed=1)

    def test_name(self, request) -> None:
        def f1() -> None:
            assert False

        assert getmsg(f1) == "assert False"

        def f2() -> None:
            f = False
            assert f

        assert getmsg(f2) == "assert False"

        def f3() -> None:
            assert a_global  # type: ignore[name-defined] # noqa: F821

        assert getmsg(f3, {"a_global": False}) == "assert False"

        def f4() -> None:
            assert sys == 42  # type: ignore[comparison-overlap]

        msg = getmsg(f4, {"sys": sys})
        assert msg == "assert sys == 42"

        def f5() -> None:
            assert cls == 42  # type: ignore[name-defined]  # noqa: F821

        class X:
            pass

        msg = getmsg(f5, {"cls": X})
        assert msg is not None
        lines = msg.splitlines()
        assert lines == ["assert cls == 42"]

    def test_assertrepr_compare_same_width(self, request) -> None:
        """Should use same width/truncation with same initial width."""

        def f() -> None:
            assert "1234567890" * 5 + "A" == "1234567890" * 5 + "B"

        msg = getmsg(f)
        assert msg is not None
        line = msg.splitlines()[0]
        if request.config.getoption("verbose") > 1:
            assert line == (
                "assert '12345678901234567890123456789012345678901234567890A' "
                "== '12345678901234567890123456789012345678901234567890B'"
            )
        else:
            assert line == (
                "assert '123456789012...901234567890A' "
                "== '123456789012...901234567890B'"
            )

    def test_dont_rewrite_if_hasattr_fails(self, request) -> None:
        class Y:
            """A class whose getattr fails, but not with `AttributeError`."""

            def __getattr__(self, attribute_name):
                raise KeyError()

            def __repr__(self) -> str:
                return "Y"

            def __init__(self) -> None:
                self.foo = 3

        def f() -> None:
            assert cls().foo == 2  # type: ignore[name-defined] # noqa: F821

        # XXX: looks like the "where" should also be there in verbose mode?!
        msg = getmsg(f, {"cls": Y})
        assert msg is not None
        lines = msg.splitlines()
        assert lines == [
            "assert 3 == 2",
            " +  where 3 = Y.foo",
            " +    where Y = cls()",
        ]

    def test_assert_already_has_message(self) -> None:
        def f():
            assert False, "something bad!"

        assert getmsg(f) == "AssertionError: something bad!\nassert False"

    def test_assertion_message(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_foo():
                assert 1 == 2, "The failure message"
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(
            ["*AssertionError*The failure message*", "*assert 1 == 2*"]
        )

    def test_assertion_message_multiline(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_foo():
                assert 1 == 2, "A multiline\\nfailure message"
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(
            ["*AssertionError*A multiline*", "*failure message*", "*assert 1 == 2*"]
        )

    def test_assertion_message_tuple(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_foo():
                assert 1 == 2, (1, 2)
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(
            ["*AssertionError*%s*" % repr((1, 2)), "*assert 1 == 2*"]
        )

    def test_assertion_message_expr(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_foo():
                assert 1 == 2, 1 + 2
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*AssertionError*3*", "*assert 1 == 2*"])

    def test_assertion_message_escape(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_foo():
                assert 1 == 2, 'To be escaped: %'
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(
            ["*AssertionError: To be escaped: %", "*assert 1 == 2"]
        )

    def test_assertion_messages_bytes(self, pytester: Pytester) -> None:
        pytester.makepyfile("def test_bytes_assertion():\n    assert False, b'ohai!'\n")
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*AssertionError: b'ohai!'", "*assert False"])

    def test_boolop(self) -> None:
        def f1() -> None:
            f = g = False
            assert f and g

        assert getmsg(f1) == "assert (False)"

        def f2() -> None:
            f = True
            g = False
            assert f and g

        assert getmsg(f2) == "assert (True and False)"

        def f3() -> None:
            f = False
            g = True
            assert f and g

        assert getmsg(f3) == "assert (False)"

        def f4() -> None:
            f = g = False
            assert f or g

        assert getmsg(f4) == "assert (False or False)"

        def f5() -> None:
            f = g = False
            assert not f and not g

        getmsg(f5, must_pass=True)

        def x() -> bool:
            return False

        def f6() -> None:
            assert x() and x()

        assert (
            getmsg(f6, {"x": x})
            == """assert (False)
 +  where False = x()"""
        )

        def f7() -> None:
            assert False or x()

        assert (
            getmsg(f7, {"x": x})
            == """assert (False or False)
 +  where False = x()"""
        )

        def f8() -> None:
            assert 1 in {} and 2 in {}

        assert getmsg(f8) == "assert (1 in {})"

        def f9() -> None:
            x = 1
            y = 2
            assert x in {1: None} and y in {}

        assert getmsg(f9) == "assert (1 in {1: None} and 2 in {})"

        def f10() -> None:
            f = True
            g = False
            assert f or g

        getmsg(f10, must_pass=True)

        def f11() -> None:
            f = g = h = lambda: True
            assert f() and g() and h()

        getmsg(f11, must_pass=True)

    def test_short_circuit_evaluation(self) -> None:
        def f1() -> None:
            assert True or explode  # type: ignore[name-defined,unreachable] # noqa: F821

        getmsg(f1, must_pass=True)

        def f2() -> None:
            x = 1
            assert x == 1 or x == 2  # noqa: PLR1714

        getmsg(f2, must_pass=True)

    def test_unary_op(self) -> None:
        def f1() -> None:
            x = True
            assert not x

        assert getmsg(f1) == "assert not True"

        def f2() -> None:
            x = 0
            assert ~x + 1

        assert getmsg(f2) == "assert (~0 + 1)"

        def f3() -> None:
            x = 3
            assert -x + x

        assert getmsg(f3) == "assert (-3 + 3)"

        def f4() -> None:
            x = 0
            assert +x + x

        assert getmsg(f4) == "assert (+0 + 0)"

    def test_binary_op(self) -> None:
        def f1() -> None:
            x = 1
            y = -1
            assert x + y

        assert getmsg(f1) == "assert (1 + -1)"

        def f2() -> None:
            assert not 5 % 4

        assert getmsg(f2) == "assert not (5 % 4)"

    def test_boolop_percent(self) -> None:
        def f1() -> None:
            assert 3 % 2 and False

        assert getmsg(f1) == "assert ((3 % 2) and False)"

        def f2() -> None:
            assert False or 4 % 2

        assert getmsg(f2) == "assert (False or (4 % 2))"

    def test_at_operator_issue1290(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            class Matrix(object):
                def __init__(self, num):
                    self.num = num
                def __matmul__(self, other):
                    return self.num * other.num

            def test_multmat_operator():
                assert Matrix(2) @ Matrix(3) == 6"""
        )
        pytester.runpytest().assert_outcomes(passed=1)

    def test_starred_with_side_effect(self, pytester: Pytester) -> None:
        """See #4412"""
        pytester.makepyfile(
            """\
            def test():
                f = lambda x: x
                x = iter([1, 2, 3])
                assert 2 * next(x) == f(*[next(x)])
            """
        )
        pytester.runpytest().assert_outcomes(passed=1)

    def test_call(self) -> None:
        def g(a=42, *args, **kwargs) -> bool:
            return False

        ns = {"g": g}

        def f1() -> None:
            assert g()

        assert (
            getmsg(f1, ns)
            == """assert False
 +  where False = g()"""
        )

        def f2() -> None:
            assert g(1)

        assert (
            getmsg(f2, ns)
            == """assert False
 +  where False = g(1)"""
        )

        def f3() -> None:
            assert g(1, 2)

        assert (
            getmsg(f3, ns)
            == """assert False
 +  where False = g(1, 2)"""
        )

        def f4() -> None:
            assert g(1, g=42)

        assert (
            getmsg(f4, ns)
            == """assert False
 +  where False = g(1, g=42)"""
        )

        def f5() -> None:
            assert g(1, 3, g=23)

        assert (
            getmsg(f5, ns)
            == """assert False
 +  where False = g(1, 3, g=23)"""
        )

        def f6() -> None:
            seq = [1, 2, 3]
            assert g(*seq)

        assert (
            getmsg(f6, ns)
            == """assert False
 +  where False = g(*[1, 2, 3])"""
        )

        def f7() -> None:
            x = "a"
            assert g(**{x: 2})

        assert (
            getmsg(f7, ns)
            == """assert False
 +  where False = g(**{'a': 2})"""
        )

    def test_attribute(self) -> None:
        class X:
            g = 3

        ns = {"x": X}

        def f1() -> None:
            assert not x.g  # type: ignore[name-defined] # noqa: F821

        assert (
            getmsg(f1, ns)
            == """assert not 3
 +  where 3 = x.g"""
        )

        def f2() -> None:
            x.a = False  # type: ignore[name-defined] # noqa: F821
            assert x.a  # type: ignore[name-defined] # noqa: F821

        assert (
            getmsg(f2, ns)
            == """assert False
 +  where False = x.a"""
        )

    def test_comparisons(self) -> None:
        def f1() -> None:
            a, b = range(2)
            assert b < a

        assert getmsg(f1) == """assert 1 < 0"""

        def f2() -> None:
            a, b, c = range(3)
            assert a > b > c

        assert getmsg(f2) == """assert 0 > 1"""

        def f3() -> None:
            a, b, c = range(3)
            assert a < b > c

        assert getmsg(f3) == """assert 1 > 2"""

        def f4() -> None:
            a, b, c = range(3)
            assert a < b <= c

        getmsg(f4, must_pass=True)

        def f5() -> None:
            a, b, c = range(3)
            assert a < b
            assert b < c

        getmsg(f5, must_pass=True)

    def test_len(self, request) -> None:
        def f():
            values = list(range(10))
            assert len(values) == 11

        msg = getmsg(f)
        assert msg == "assert 10 == 11\n +  where 10 = len([0, 1, 2, 3, 4, 5, ...])"

    def test_custom_reprcompare(self, monkeypatch) -> None:
        def my_reprcompare1(op, left, right) -> str:
            return "42"

        monkeypatch.setattr(util, "_reprcompare", my_reprcompare1)

        def f1() -> None:
            assert 42 < 3

        assert getmsg(f1) == "assert 42"

        def my_reprcompare2(op, left, right) -> str:
            return f"{left} {op} {right}"

        monkeypatch.setattr(util, "_reprcompare", my_reprcompare2)

        def f2() -> None:
            assert 1 < 3 < 5 <= 4 < 7

        assert getmsg(f2) == "assert 5 <= 4"

    def test_assert_raising__bool__in_comparison(self) -> None:
        def f() -> None:
            class A:
                def __bool__(self):
                    raise ValueError(42)

                def __lt__(self, other):
                    return A()

                def __repr__(self):
                    return "<MY42 object>"

            def myany(x) -> bool:
                return False

            assert myany(A() < 0)

        msg = getmsg(f)
        assert msg is not None
        assert "<MY42 object> < 0" in msg

    def test_assert_handling_raise_in__iter__(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """\
            class A:
                def __iter__(self):
                    raise ValueError()

                def __eq__(self, o: object) -> bool:
                    return self is o

                def __repr__(self):
                    return "<A object>"

            assert A() == A()
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*E*assert <A object> == <A object>"])

    def test_formatchar(self) -> None:
        def f() -> None:
            assert "%test" == "test"  # type: ignore[comparison-overlap]

        msg = getmsg(f)
        assert msg is not None
        assert msg.startswith("assert '%test' == 'test'")

    def test_custom_repr(self, request) -> None:
        def f() -> None:
            class Foo:
                a = 1

                def __repr__(self):
                    return "\n{ \n~ \n}"

            f = Foo()
            assert 0 == f.a

        msg = getmsg(f)
        assert msg is not None
        lines = util._format_lines([msg])
        assert lines == ["assert 0 == 1\n +  where 1 = \\n{ \\n~ \\n}.a"]

    def test_custom_repr_non_ascii(self) -> None:
        def f() -> None:
            class A:
                name = "ä"

                def __repr__(self):
                    return self.name.encode("UTF-8")  # only legal in python2

            a = A()
            assert not a.name

        msg = getmsg(f)
        assert msg is not None
        assert "UnicodeDecodeError" not in msg
        assert "UnicodeEncodeError" not in msg


class TestRewriteOnImport:
    def test_pycache_is_a_file(self, pytester: Pytester) -> None:
        pytester.path.joinpath("__pycache__").write_text("Hello", encoding="utf-8")
        pytester.makepyfile(
            """
            def test_rewritten():
                assert "@py_builtins" in globals()"""
        )
        assert pytester.runpytest().ret == 0

    def test_pycache_is_readonly(self, pytester: Pytester) -> None:
        cache = pytester.mkdir("__pycache__")
        old_mode = cache.stat().st_mode
        cache.chmod(old_mode ^ stat.S_IWRITE)
        pytester.makepyfile(
            """
            def test_rewritten():
                assert "@py_builtins" in globals()"""
        )
        try:
            assert pytester.runpytest().ret == 0
        finally:
            cache.chmod(old_mode)

    def test_zipfile(self, pytester: Pytester) -> None:
        z = pytester.path.joinpath("myzip.zip")
        z_fn = str(z)
        f = zipfile.ZipFile(z_fn, "w")
        try:
            f.writestr("test_gum/__init__.py", "")
            f.writestr("test_gum/test_lizard.py", "")
        finally:
            f.close()
        z.chmod(256)
        pytester.makepyfile(
            f"""
            import sys
            sys.path.append({z_fn!r})
            import test_gum.test_lizard"""
        )
        assert pytester.runpytest().ret == ExitCode.NO_TESTS_COLLECTED

    @pytest.mark.skipif(
        sys.version_info < (3, 9),
        reason="importlib.resources.files was introduced in 3.9",
    )
    def test_load_resource_via_files_with_rewrite(self, pytester: Pytester) -> None:
        example = pytester.path.joinpath("demo") / "example"
        init = pytester.path.joinpath("demo") / "__init__.py"
        pytester.makepyfile(
            **{
                "demo/__init__.py": """
                from importlib.resources import files

                def load():
                    return files(__name__)
                """,
                "test_load": f"""
                pytest_plugins = ["demo"]

                def test_load():
                    from demo import load
                    found = {{str(i) for i in load().iterdir() if i.name != "__pycache__"}}
                    assert found == {{{str(example)!r}, {str(init)!r}}}
                """,
            }
        )
        example.mkdir()

        assert pytester.runpytest("-vv").ret == ExitCode.OK

    def test_readonly(self, pytester: Pytester) -> None:
        sub = pytester.mkdir("testing")
        sub.joinpath("test_readonly.py").write_bytes(
            b"""
def test_rewritten():
    assert "@py_builtins" in globals()
            """,
        )
        old_mode = sub.stat().st_mode
        sub.chmod(320)
        try:
            assert pytester.runpytest().ret == 0
        finally:
            sub.chmod(old_mode)

    def test_dont_write_bytecode(self, pytester: Pytester, monkeypatch) -> None:
        monkeypatch.delenv("PYTHONPYCACHEPREFIX", raising=False)

        pytester.makepyfile(
            """
            import os
            def test_no_bytecode():
                assert "__pycache__" in __cached__
                assert not os.path.exists(__cached__)
                assert not os.path.exists(os.path.dirname(__cached__))"""
        )
        monkeypatch.setenv("PYTHONDONTWRITEBYTECODE", "1")
        assert pytester.runpytest_subprocess().ret == 0

    def test_orphaned_pyc_file(self, pytester: Pytester, monkeypatch) -> None:
        monkeypatch.delenv("PYTHONPYCACHEPREFIX", raising=False)
        monkeypatch.setattr(sys, "pycache_prefix", None, raising=False)

        pytester.makepyfile(
            """
            import orphan
            def test_it():
                assert orphan.value == 17
            """
        )
        pytester.makepyfile(
            orphan="""
            value = 17
            """
        )
        py_compile.compile("orphan.py")
        os.remove("orphan.py")

        # Python 3 puts the .pyc files in a __pycache__ directory, and will
        # not import from there without source.  It will import a .pyc from
        # the source location though.
        if not os.path.exists("orphan.pyc"):
            pycs = glob.glob("__pycache__/orphan.*.pyc")
            assert len(pycs) == 1
            os.rename(pycs[0], "orphan.pyc")

        assert pytester.runpytest().ret == 0

    def test_cached_pyc_includes_pytest_version(
        self, pytester: Pytester, monkeypatch
    ) -> None:
        """Avoid stale caches (#1671)"""
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", raising=False)
        monkeypatch.delenv("PYTHONPYCACHEPREFIX", raising=False)
        pytester.makepyfile(
            test_foo="""
            def test_foo():
                assert True
            """
        )
        result = pytester.runpytest_subprocess()
        assert result.ret == 0
        found_names = glob.glob(f"__pycache__/*-pytest-{pytest.__version__}.pyc")
        assert found_names, "pyc with expected tag not found in names: {}".format(
            glob.glob("__pycache__/*.pyc")
        )

    @pytest.mark.skipif('"__pypy__" in sys.modules')
    def test_pyc_vs_pyo(
        self,
        pytester: Pytester,
        monkeypatch: pytest.MonkeyPatch,
    ) -> None:
        pytester.makepyfile(
            """
            import pytest
            def test_optimized():
                "hello"
                assert test_optimized.__doc__ is None"""
        )
        p = make_numbered_dir(root=Path(pytester.path), prefix="runpytest-")
        tmp = "--basetemp=%s" % p
        with monkeypatch.context() as mp:
            mp.setenv("PYTHONOPTIMIZE", "2")
            mp.delenv("PYTHONDONTWRITEBYTECODE", raising=False)
            mp.delenv("PYTHONPYCACHEPREFIX", raising=False)
            assert pytester.runpytest_subprocess(tmp).ret == 0
            tagged = "test_pyc_vs_pyo." + PYTEST_TAG
            assert tagged + ".pyo" in os.listdir("__pycache__")
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", raising=False)
        monkeypatch.delenv("PYTHONPYCACHEPREFIX", raising=False)
        assert pytester.runpytest_subprocess(tmp).ret == 1
        assert tagged + ".pyc" in os.listdir("__pycache__")

    def test_package(self, pytester: Pytester) -> None:
        pkg = pytester.path.joinpath("pkg")
        pkg.mkdir()
        pkg.joinpath("__init__.py")
        pkg.joinpath("test_blah.py").write_text(
            """
def test_rewritten():
    assert "@py_builtins" in globals()""",
            encoding="utf-8",
        )
        assert pytester.runpytest().ret == 0

    def test_translate_newlines(self, pytester: Pytester) -> None:
        content = "def test_rewritten():\r\n assert '@py_builtins' in globals()"
        b = content.encode("utf-8")
        pytester.path.joinpath("test_newlines.py").write_bytes(b)
        assert pytester.runpytest().ret == 0

    def test_package_without__init__py(self, pytester: Pytester) -> None:
        pkg = pytester.mkdir("a_package_without_init_py")
        pkg.joinpath("module.py").touch()
        pytester.makepyfile("import a_package_without_init_py.module")
        assert pytester.runpytest().ret == ExitCode.NO_TESTS_COLLECTED

    def test_rewrite_warning(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            pytest.register_assert_rewrite("_pytest")
        """
        )
        # needs to be a subprocess because pytester explicitly disables this warning
        result = pytester.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*Module already imported*: _pytest"])

    def test_rewrite_module_imported_from_conftest(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import test_rewrite_module_imported
        """
        )
        pytester.makepyfile(
            test_rewrite_module_imported="""
            def test_rewritten():
                assert "@py_builtins" in globals()
        """
        )
        assert pytester.runpytest_subprocess().ret == 0

    def test_remember_rewritten_modules(
        self, pytestconfig, pytester: Pytester, monkeypatch
    ) -> None:
        """`AssertionRewriteHook` should remember rewritten modules so it
        doesn't give false positives (#2005)."""
        monkeypatch.syspath_prepend(pytester.path)
        pytester.makepyfile(test_remember_rewritten_modules="")
        warnings = []
        hook = AssertionRewritingHook(pytestconfig)
        monkeypatch.setattr(
            hook, "_warn_already_imported", lambda code, msg: warnings.append(msg)
        )
        spec = hook.find_spec("test_remember_rewritten_modules")
        assert spec is not None
        module = importlib.util.module_from_spec(spec)
        hook.exec_module(module)
        hook.mark_rewrite("test_remember_rewritten_modules")
        hook.mark_rewrite("test_remember_rewritten_modules")
        assert warnings == []

    def test_rewrite_warning_using_pytest_plugins(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            **{
                "conftest.py": "pytest_plugins = ['core', 'gui', 'sci']",
                "core.py": "",
                "gui.py": "pytest_plugins = ['core', 'sci']",
                "sci.py": "pytest_plugins = ['core']",
                "test_rewrite_warning_pytest_plugins.py": "def test(): pass",
            }
        )
        pytester.chdir()
        result = pytester.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*= 1 passed in *=*"])
        result.stdout.no_fnmatch_line("*pytest-warning summary*")

    def test_rewrite_warning_using_pytest_plugins_env_var(
        self, pytester: Pytester, monkeypatch
    ) -> None:
        monkeypatch.setenv("PYTEST_PLUGINS", "plugin")
        pytester.makepyfile(
            **{
                "plugin.py": "",
                "test_rewrite_warning_using_pytest_plugins_env_var.py": """
                import plugin
                pytest_plugins = ['plugin']
                def test():
                    pass
            """,
            }
        )
        pytester.chdir()
        result = pytester.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*= 1 passed in *=*"])
        result.stdout.no_fnmatch_line("*pytest-warning summary*")


class TestAssertionRewriteHookDetails:
    def test_sys_meta_path_munged(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_meta_path():
                import sys; sys.meta_path = []"""
        )
        assert pytester.runpytest().ret == 0

    def test_write_pyc(self, pytester: Pytester, tmp_path) -> None:
        from _pytest.assertion import AssertionState
        from _pytest.assertion.rewrite import _write_pyc

        config = pytester.parseconfig()
        state = AssertionState(config, "rewrite")
        tmp_path.joinpath("source.py").touch()
        source_path = str(tmp_path)
        pycpath = tmp_path.joinpath("pyc")
        co = compile("1", "f.py", "single")
        assert _write_pyc(state, co, os.stat(source_path), pycpath)

        with mock.patch.object(os, "replace", side_effect=OSError):
            assert not _write_pyc(state, co, os.stat(source_path), pycpath)

    def test_resources_provider_for_loader(self, pytester: Pytester) -> None:
        """
        Attempts to load resources from a package should succeed normally,
        even when the AssertionRewriteHook is used to load the modules.

        See #366 for details.
        """
        pytest.importorskip("pkg_resources")

        pytester.mkpydir("testpkg")
        contents = {
            "testpkg/test_pkg": """
                import pkg_resources

                import pytest
                from _pytest.assertion.rewrite import AssertionRewritingHook

                def test_load_resource():
                    assert isinstance(__loader__, AssertionRewritingHook)
                    res = pkg_resources.resource_string(__name__, 'resource.txt')
                    res = res.decode('ascii')
                    assert res == 'Load me please.'
                """
        }
        pytester.makepyfile(**contents)
        pytester.maketxtfile(**{"testpkg/resource": "Load me please."})

        result = pytester.runpytest_subprocess()
        result.assert_outcomes(passed=1)

    def test_read_pyc(self, tmp_path: Path) -> None:
        """
        Ensure that the `_read_pyc` can properly deal with corrupted pyc files.
        In those circumstances it should just give up instead of generating
        an exception that is propagated to the caller.
        """
        import py_compile

        from _pytest.assertion.rewrite import _read_pyc

        source = tmp_path / "source.py"
        pyc = Path(str(source) + "c")

        source.write_text("def test(): pass", encoding="utf-8")
        py_compile.compile(str(source), str(pyc))

        contents = pyc.read_bytes()
        strip_bytes = 20  # header is around 16 bytes, strip a little more
        assert len(contents) > strip_bytes
        pyc.write_bytes(contents[:strip_bytes])

        assert _read_pyc(source, pyc) is None  # no error

    def test_read_pyc_success(self, tmp_path: Path, pytester: Pytester) -> None:
        """
        Ensure that the _rewrite_test() -> _write_pyc() produces a pyc file
        that can be properly read with _read_pyc()
        """
        from _pytest.assertion import AssertionState
        from _pytest.assertion.rewrite import _read_pyc
        from _pytest.assertion.rewrite import _rewrite_test
        from _pytest.assertion.rewrite import _write_pyc

        config = pytester.parseconfig()
        state = AssertionState(config, "rewrite")

        fn = tmp_path / "source.py"
        pyc = Path(str(fn) + "c")

        fn.write_text("def test(): assert True", encoding="utf-8")

        source_stat, co = _rewrite_test(fn, config)
        _write_pyc(state, co, source_stat, pyc)
        assert _read_pyc(fn, pyc, state.trace) is not None

    def test_read_pyc_more_invalid(self, tmp_path: Path) -> None:
        from _pytest.assertion.rewrite import _read_pyc

        source = tmp_path / "source.py"
        pyc = tmp_path / "source.pyc"

        source_bytes = b"def test(): pass\n"
        source.write_bytes(source_bytes)

        magic = importlib.util.MAGIC_NUMBER

        flags = b"\x00\x00\x00\x00"

        mtime = b"\x58\x3c\xb0\x5f"
        mtime_int = int.from_bytes(mtime, "little")
        os.utime(source, (mtime_int, mtime_int))

        size = len(source_bytes).to_bytes(4, "little")

        code = marshal.dumps(compile(source_bytes, str(source), "exec"))

        # Good header.
        pyc.write_bytes(magic + flags + mtime + size + code)
        assert _read_pyc(source, pyc, print) is not None

        # Too short.
        pyc.write_bytes(magic + flags + mtime)
        assert _read_pyc(source, pyc, print) is None

        # Bad magic.
        pyc.write_bytes(b"\x12\x34\x56\x78" + flags + mtime + size + code)
        assert _read_pyc(source, pyc, print) is None

        # Unsupported flags.
        pyc.write_bytes(magic + b"\x00\xff\x00\x00" + mtime + size + code)
        assert _read_pyc(source, pyc, print) is None

        # Bad mtime.
        pyc.write_bytes(magic + flags + b"\x58\x3d\xb0\x5f" + size + code)
        assert _read_pyc(source, pyc, print) is None

        # Bad size.
        pyc.write_bytes(magic + flags + mtime + b"\x99\x00\x00\x00" + code)
        assert _read_pyc(source, pyc, print) is None

    def test_reload_is_same_and_reloads(self, pytester: Pytester) -> None:
        """Reloading a (collected) module after change picks up the change."""
        pytester.makeini(
            """
            [pytest]
            python_files = *.py
            """
        )
        pytester.makepyfile(
            file="""
            def reloaded():
                return False

            def rewrite_self():
                with open(__file__, 'w', encoding='utf-8') as self:
                    self.write('def reloaded(): return True')
            """,
            test_fun="""
            import sys
            from importlib import reload

            def test_loader():
                import file
                assert not file.reloaded()
                file.rewrite_self()
                assert sys.modules["file"] is reload(file)
                assert file.reloaded()
            """,
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 1 passed*"])

    def test_get_data_support(self, pytester: Pytester) -> None:
        """Implement optional PEP302 api (#808)."""
        path = pytester.mkpydir("foo")
        path.joinpath("test_foo.py").write_text(
            textwrap.dedent(
                """\
                class Test(object):
                    def test_foo(self):
                        import pkgutil
                        data = pkgutil.get_data('foo.test_foo', 'data.txt')
                        assert data == b'Hey'
                """
            ),
            encoding="utf-8",
        )
        path.joinpath("data.txt").write_text("Hey", encoding="utf-8")
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])


def test_issue731(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
    class LongReprWithBraces(object):
        def __repr__(self):
           return 'LongReprWithBraces({' + ('a' * 80) + '}' + ('a' * 120) + ')'

        def some_method(self):
            return False

    def test_long_repr():
        obj = LongReprWithBraces()
        assert obj.some_method()
    """
    )
    result = pytester.runpytest()
    result.stdout.no_fnmatch_line("*unbalanced braces*")


class TestIssue925:
    def test_simple_case(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
        def test_ternary_display():
            assert (False == False) == False
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*E*assert (False == False) == False"])

    def test_long_case(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
        def test_ternary_display():
             assert False == (False == True) == True
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*E*assert (False == True) == True"])

    def test_many_brackets(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_ternary_display():
                 assert True == ((False == True) == True)
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*E*assert True == ((False == True) == True)"])


class TestIssue2121:
    def test_rewrite_python_files_contain_subdirs(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            **{
                "tests/file.py": """
                def test_simple_failure():
                    assert 1 + 1 == 3
                """
            }
        )
        pytester.makeini(
            """
                [pytest]
                python_files = tests/**.py
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*E*assert (1 + 1) == 3"])


class TestIssue10743:
    def test_assertion_walrus_operator(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def my_func(before, after):
                return before == after

            def change_value(value):
                return value.lower()

            def test_walrus_conversion():
                a = "Hello"
                assert not my_func(a, a := change_value(a))
                assert a == "hello"
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_walrus_operator_dont_rewrite(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            'PYTEST_DONT_REWRITE'
            def my_func(before, after):
                return before == after

            def change_value(value):
                return value.lower()

            def test_walrus_conversion_dont_rewrite():
                a = "Hello"
                assert not my_func(a, a := change_value(a))
                assert a == "hello"
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_inline_walrus_operator(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def my_func(before, after):
                return before == after

            def test_walrus_conversion_inline():
                a = "Hello"
                assert not my_func(a, a := a.lower())
                assert a == "hello"
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_inline_walrus_operator_reverse(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def my_func(before, after):
                return before == after

            def test_walrus_conversion_reverse():
                a = "Hello"
                assert my_func(a := a.lower(), a)
                assert a == 'hello'
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_walrus_no_variable_name_conflict(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def test_walrus_conversion_no_conflict():
                a = "Hello"
                assert a == (b := a.lower())
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*AssertionError: assert 'Hello' == 'hello'"])

    def test_assertion_walrus_operator_true_assertion_and_changes_variable_value(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def test_walrus_conversion_succeed():
                a = "Hello"
                assert a != (a := a.lower())
                assert a == 'hello'
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_walrus_operator_fail_assertion(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_walrus_conversion_fails():
                a = "Hello"
                assert a == (a := a.lower())
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*AssertionError: assert 'Hello' == 'hello'"])

    def test_assertion_walrus_operator_boolean_composite(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def test_walrus_operator_change_boolean_value():
                a = True
                assert a and True and ((a := False) is False) and (a is False) and ((a := None) is None)
                assert a is None
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_walrus_operator_compare_boolean_fails(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def test_walrus_operator_change_boolean_value():
                a = True
                assert not (a and ((a := False) is False))
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*assert not (True and False is False)"])

    def test_assertion_walrus_operator_boolean_none_fails(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def test_walrus_operator_change_boolean_value():
                a = True
                assert not (a and ((a := None) is None))
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*assert not (True and None is None)"])

    def test_assertion_walrus_operator_value_changes_cleared_after_each_test(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def test_walrus_operator_change_value():
                a = True
                assert (a := None) is None

            def test_walrus_operator_not_override_value():
                a = True
                assert a is True
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0


class TestIssue11028:
    def test_assertion_walrus_operator_in_operand(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_in_string():
              assert (obj := "foo") in obj
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_walrus_operator_in_operand_json_dumps(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            import json

            def test_json_encoder():
                assert (obj := "foo") in json.dumps(obj)
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_walrus_operator_equals_operand_function(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def f(a):
                return a

            def test_call_other_function_arg():
              assert (obj := "foo") == f(obj)
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_walrus_operator_equals_operand_function_keyword_arg(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def f(a='test'):
                return a

            def test_call_other_function_k_arg():
              assert (obj := "foo") == f(a=obj)
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_walrus_operator_equals_operand_function_arg_as_function(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def f(a='test'):
                return a

            def test_function_of_function():
              assert (obj := "foo") == f(f(obj))
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0

    def test_assertion_walrus_operator_gt_operand_function(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            """
            def add_one(a):
                return a + 1

            def test_gt():
              assert (obj := 4) > add_one(obj)
        """
        )
        result = pytester.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*assert 4 > 5", "*where 5 = add_one(4)"])


class TestIssue11239:
    def test_assertion_walrus_different_test_cases(self, pytester: Pytester) -> None:
        """Regression for (#11239)

        Walrus operator rewriting would leak to separate test cases if they used the same variables.
        """
        pytester.makepyfile(
            """
            def test_1():
                state = {"x": 2}.get("x")
                assert state is not None

            def test_2():
                db = {"x": 2}
                assert (state := db.get("x")) is not None
        """
        )
        result = pytester.runpytest()
        assert result.ret == 0


@pytest.mark.skipif(
    sys.maxsize <= (2**31 - 1), reason="Causes OverflowError on 32bit systems"
)
@pytest.mark.parametrize("offset", [-1, +1])
def test_source_mtime_long_long(pytester: Pytester, offset) -> None:
    """Support modification dates after 2038 in rewritten files (#4903).

    pytest would crash with:

            fp.write(struct.pack("<ll", mtime, size))
        E   struct.error: argument out of range
    """
    p = pytester.makepyfile(
        """
        def test(): pass
    """
    )
    # use unsigned long timestamp which overflows signed long,
    # which was the cause of the bug
    # +1 offset also tests masking of 0xFFFFFFFF
    timestamp = 2**32 + offset
    os.utime(str(p), (timestamp, timestamp))
    result = pytester.runpytest()
    assert result.ret == 0


def test_rewrite_infinite_recursion(
    pytester: Pytester, pytestconfig, monkeypatch
) -> None:
    """Fix infinite recursion when writing pyc files: if an import happens to be triggered when writing the pyc
    file, this would cause another call to the hook, which would trigger another pyc writing, which could
    trigger another import, and so on. (#3506)"""
    from _pytest.assertion import rewrite as rewritemod

    pytester.syspathinsert()
    pytester.makepyfile(test_foo="def test_foo(): pass")
    pytester.makepyfile(test_bar="def test_bar(): pass")

    original_write_pyc = rewritemod._write_pyc

    write_pyc_called = []

    def spy_write_pyc(*args, **kwargs):
        # make a note that we have called _write_pyc
        write_pyc_called.append(True)
        # try to import a module at this point: we should not try to rewrite this module
        assert hook.find_spec("test_bar") is None
        return original_write_pyc(*args, **kwargs)

    monkeypatch.setattr(rewritemod, "_write_pyc", spy_write_pyc)
    monkeypatch.setattr(sys, "dont_write_bytecode", False)

    hook = AssertionRewritingHook(pytestconfig)
    spec = hook.find_spec("test_foo")
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    hook.exec_module(module)
    assert len(write_pyc_called) == 1


class TestEarlyRewriteBailout:
    @pytest.fixture
    def hook(
        self, pytestconfig, monkeypatch, pytester: Pytester
    ) -> Generator[AssertionRewritingHook, None, None]:
        """Returns a patched AssertionRewritingHook instance so we can configure its initial paths and track
        if PathFinder.find_spec has been called.
        """
        import importlib.machinery

        self.find_spec_calls: List[str] = []
        self.initial_paths: Set[Path] = set()

        class StubSession:
            _initialpaths = self.initial_paths

            def isinitpath(self, p):
                return p in self._initialpaths

        def spy_find_spec(name, path):
            self.find_spec_calls.append(name)
            return importlib.machinery.PathFinder.find_spec(name, path)

        hook = AssertionRewritingHook(pytestconfig)
        # use default patterns, otherwise we inherit pytest's testing config
        with mock.patch.object(hook, "fnpats", ["test_*.py", "*_test.py"]):
            monkeypatch.setattr(hook, "_find_spec", spy_find_spec)
            hook.set_session(StubSession())  # type: ignore[arg-type]
            pytester.syspathinsert()
            yield hook

    def test_basic(self, pytester: Pytester, hook: AssertionRewritingHook) -> None:
        """
        Ensure we avoid calling PathFinder.find_spec when we know for sure a certain
        module will not be rewritten to optimize assertion rewriting (#3918).
        """
        pytester.makeconftest(
            """
            import pytest
            @pytest.fixture
            def fix(): return 1
        """
        )
        pytester.makepyfile(test_foo="def test_foo(): pass")
        pytester.makepyfile(bar="def bar(): pass")
        foobar_path = pytester.makepyfile(foobar="def foobar(): pass")
        self.initial_paths.add(foobar_path)

        # conftest files should always be rewritten
        assert hook.find_spec("conftest") is not None
        assert self.find_spec_calls == ["conftest"]

        # files matching "python_files" mask should always be rewritten
        assert hook.find_spec("test_foo") is not None
        assert self.find_spec_calls == ["conftest", "test_foo"]

        # file does not match "python_files": early bailout
        assert hook.find_spec("bar") is None
        assert self.find_spec_calls == ["conftest", "test_foo"]

        # file is an initial path (passed on the command-line): should be rewritten
        assert hook.find_spec("foobar") is not None
        assert self.find_spec_calls == ["conftest", "test_foo", "foobar"]

    def test_pattern_contains_subdirectories(
        self, pytester: Pytester, hook: AssertionRewritingHook
    ) -> None:
        """If one of the python_files patterns contain subdirectories ("tests/**.py") we can't bailout early
        because we need to match with the full path, which can only be found by calling PathFinder.find_spec
        """
        pytester.makepyfile(
            **{
                "tests/file.py": """\
                    def test_simple_failure():
                        assert 1 + 1 == 3
                """
            }
        )
        pytester.syspathinsert("tests")
        with mock.patch.object(hook, "fnpats", ["tests/**.py"]):
            assert hook.find_spec("file") is not None
            assert self.find_spec_calls == ["file"]

    @pytest.mark.skipif(
        sys.platform.startswith("win32"), reason="cannot remove cwd on Windows"
    )
    @pytest.mark.skipif(
        sys.platform.startswith("sunos5"), reason="cannot remove cwd on Solaris"
    )
    def test_cwd_changed(self, pytester: Pytester, monkeypatch) -> None:
        # Setup conditions for py's fspath trying to import pathlib on py34
        # always (previously triggered via xdist only).
        # Ref: https://github.com/pytest-dev/py/pull/207
        monkeypatch.syspath_prepend("")
        monkeypatch.delitem(sys.modules, "pathlib", raising=False)

        pytester.makepyfile(
            **{
                "test_setup_nonexisting_cwd.py": """\
                    import os
                    import tempfile

                    with tempfile.TemporaryDirectory() as newpath:
                        os.chdir(newpath)
                """,
                "test_test.py": """\
                    def test():
                        pass
                """,
            }
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["* 1 passed in *"])


class TestAssertionPass:
    def test_option_default(self, pytester: Pytester) -> None:
        config = pytester.parseconfig()
        assert config.getini("enable_assertion_pass_hook") is False

    @pytest.fixture
    def flag_on(self, pytester: Pytester):
        pytester.makeini("[pytest]\nenable_assertion_pass_hook = True\n")

    @pytest.fixture
    def hook_on(self, pytester: Pytester):
        pytester.makeconftest(
            """\
            def pytest_assertion_pass(item, lineno, orig, expl):
                raise Exception("Assertion Passed: {} {} at line {}".format(orig, expl, lineno))
            """
        )

    def test_hook_call(self, pytester: Pytester, flag_on, hook_on) -> None:
        pytester.makepyfile(
            """\
            def test_simple():
                a=1
                b=2
                c=3
                d=0

                assert a+b == c+d

            # cover failing assertions with a message
            def test_fails():
                assert False, "assert with message"
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(
            "*Assertion Passed: a+b == c+d (1 + 2) == (3 + 0) at line 7*"
        )

    def test_hook_call_with_parens(self, pytester: Pytester, flag_on, hook_on) -> None:
        pytester.makepyfile(
            """\
            def f(): return 1
            def test():
                assert f()
            """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines("*Assertion Passed: f() 1")

    def test_hook_not_called_without_hookimpl(
        self, pytester: Pytester, monkeypatch, flag_on
    ) -> None:
        """Assertion pass should not be called (and hence formatting should
        not occur) if there is no hook declared for pytest_assertion_pass"""

        def raise_on_assertionpass(*_, **__):
            raise Exception("Assertion passed called when it shouldn't!")

        monkeypatch.setattr(
            _pytest.assertion.rewrite, "_call_assertion_pass", raise_on_assertionpass
        )

        pytester.makepyfile(
            """\
            def test_simple():
                a=1
                b=2
                c=3
                d=0

                assert a+b == c+d
            """
        )
        result = pytester.runpytest()
        result.assert_outcomes(passed=1)

    def test_hook_not_called_without_cmd_option(
        self, pytester: Pytester, monkeypatch
    ) -> None:
        """Assertion pass should not be called (and hence formatting should
        not occur) if there is no hook declared for pytest_assertion_pass"""

        def raise_on_assertionpass(*_, **__):
            raise Exception("Assertion passed called when it shouldn't!")

        monkeypatch.setattr(
            _pytest.assertion.rewrite, "_call_assertion_pass", raise_on_assertionpass
        )

        pytester.makeconftest(
            """\
            def pytest_assertion_pass(item, lineno, orig, expl):
                raise Exception("Assertion Passed: {} {} at line {}".format(orig, expl, lineno))
            """
        )

        pytester.makepyfile(
            """\
            def test_simple():
                a=1
                b=2
                c=3
                d=0

                assert a+b == c+d
            """
        )
        result = pytester.runpytest()
        result.assert_outcomes(passed=1)


# fmt: off
@pytest.mark.parametrize(
    ("src", "expected"),
    (
        pytest.param(b"", {}, id="trivial"),
        pytest.param(
            b"def x(): assert 1\n",
            {1: "1"},
            id="assert statement not on own line",
        ),
        pytest.param(
            b"def x():\n"
            b"    assert 1\n"
            b"    assert 1+2\n",
            {2: "1", 3: "1+2"},
            id="multiple assertions",
        ),
        pytest.param(
            # changes in encoding cause the byte offsets to be different
            "# -*- coding: latin1\n"
            "def ÀÀÀÀÀ(): assert 1\n".encode("latin1"),
            {2: "1"},
            id="latin1 encoded on first line\n",
        ),
        pytest.param(
            # using the default utf-8 encoding
            "def ÀÀÀÀÀ(): assert 1\n".encode(),
            {1: "1"},
            id="utf-8 encoded on first line",
        ),
        pytest.param(
            b"def x():\n"
            b"    assert (\n"
            b"        1 + 2  # comment\n"
            b"    )\n",
            {2: "(\n        1 + 2  # comment\n    )"},
            id="multi-line assertion",
        ),
        pytest.param(
            b"def x():\n"
            b"    assert y == [\n"
            b"        1, 2, 3\n"
            b"    ]\n",
            {2: "y == [\n        1, 2, 3\n    ]"},
            id="multi line assert with list continuation",
        ),
        pytest.param(
            b"def x():\n"
            b"    assert 1 + \\\n"
            b"        2\n",
            {2: "1 + \\\n        2"},
            id="backslash continuation",
        ),
        pytest.param(
            b"def x():\n"
            b"    assert x, y\n",
            {2: "x"},
            id="assertion with message",
        ),
        pytest.param(
            b"def x():\n"
            b"    assert (\n"
            b"        f(1, 2, 3)\n"
            b"    ),  'f did not work!'\n",
            {2: "(\n        f(1, 2, 3)\n    )"},
            id="assertion with message, test spanning multiple lines",
        ),
        pytest.param(
            b"def x():\n"
            b"    assert \\\n"
            b"        x\\\n"
            b"        , 'failure message'\n",
            {2: "x"},
            id="escaped newlines plus message",
        ),
        pytest.param(
            b"def x(): assert 5",
            {1: "5"},
            id="no newline at end of file",
        ),
    ),
)
# fmt: on
def test_get_assertion_exprs(src, expected) -> None:
    assert _get_assertion_exprs(src) == expected


def test_try_makedirs(monkeypatch, tmp_path: Path) -> None:
    from _pytest.assertion.rewrite import try_makedirs

    p = tmp_path / "foo"

    # create
    assert try_makedirs(p)
    assert p.is_dir()

    # already exist
    assert try_makedirs(p)

    # monkeypatch to simulate all error situations
    def fake_mkdir(p, exist_ok=False, *, exc):
        assert isinstance(p, Path)
        raise exc

    monkeypatch.setattr(os, "makedirs", partial(fake_mkdir, exc=FileNotFoundError()))
    assert not try_makedirs(p)

    monkeypatch.setattr(os, "makedirs", partial(fake_mkdir, exc=NotADirectoryError()))
    assert not try_makedirs(p)

    monkeypatch.setattr(os, "makedirs", partial(fake_mkdir, exc=PermissionError()))
    assert not try_makedirs(p)

    err = OSError()
    err.errno = errno.EROFS
    monkeypatch.setattr(os, "makedirs", partial(fake_mkdir, exc=err))
    assert not try_makedirs(p)

    err = OSError()
    err.errno = errno.ENOSYS
    monkeypatch.setattr(os, "makedirs", partial(fake_mkdir, exc=err))
    assert not try_makedirs(p)

    # unhandled OSError should raise
    err = OSError()
    err.errno = errno.ECHILD
    monkeypatch.setattr(os, "makedirs", partial(fake_mkdir, exc=err))
    with pytest.raises(OSError) as exc_info:
        try_makedirs(p)
    assert exc_info.value.errno == errno.ECHILD


class TestPyCacheDir:
    @pytest.mark.parametrize(
        "prefix, source, expected",
        [
            ("c:/tmp/pycs", "d:/projects/src/foo.py", "c:/tmp/pycs/projects/src"),
            (None, "d:/projects/src/foo.py", "d:/projects/src/__pycache__"),
            ("/tmp/pycs", "/home/projects/src/foo.py", "/tmp/pycs/home/projects/src"),
            (None, "/home/projects/src/foo.py", "/home/projects/src/__pycache__"),
        ],
    )
    def test_get_cache_dir(self, monkeypatch, prefix, source, expected) -> None:
        monkeypatch.delenv("PYTHONPYCACHEPREFIX", raising=False)
        monkeypatch.setattr(sys, "pycache_prefix", prefix, raising=False)

        assert get_cache_dir(Path(source)) == Path(expected)

    @pytest.mark.skipif(
        sys.version_info[:2] == (3, 9) and sys.platform.startswith("win"),
        reason="#9298",
    )
    def test_sys_pycache_prefix_integration(
        self, tmp_path, monkeypatch, pytester: Pytester
    ) -> None:
        """Integration test for sys.pycache_prefix (#4730)."""
        pycache_prefix = tmp_path / "my/pycs"
        monkeypatch.setattr(sys, "pycache_prefix", str(pycache_prefix))
        monkeypatch.setattr(sys, "dont_write_bytecode", False)

        pytester.makepyfile(
            **{
                "src/test_foo.py": """
                import bar
                def test_foo():
                    pass
            """,
                "src/bar/__init__.py": "",
            }
        )
        result = pytester.runpytest()
        assert result.ret == 0

        test_foo = pytester.path.joinpath("src/test_foo.py")
        bar_init = pytester.path.joinpath("src/bar/__init__.py")
        assert test_foo.is_file()
        assert bar_init.is_file()

        # test file: rewritten, custom pytest cache tag
        test_foo_pyc = get_cache_dir(test_foo) / ("test_foo" + PYC_TAIL)
        assert test_foo_pyc.is_file()

        # normal file: not touched by pytest, normal cache tag
        bar_init_pyc = get_cache_dir(bar_init) / f"__init__.{sys.implementation.cache_tag}.pyc"
        assert bar_init_pyc.is_file()


class TestReprSizeVerbosity:
    """
    Check that verbosity also controls the string length threshold to shorten it using
    ellipsis.
    """

    @pytest.mark.parametrize(
        "verbose, expected_size",
        [
            (0, DEFAULT_REPR_MAX_SIZE),
            (1, DEFAULT_REPR_MAX_SIZE * 10),
            (2, None),
            (3, None),
        ],
    )
    def test_get_maxsize_for_saferepr(self, verbose: int, expected_size) -> None:
        class FakeConfig:
            def get_verbosity(self, verbosity_type: Optional[str] = None) -> int:
                return verbose

        config = FakeConfig()
        assert _get_maxsize_for_saferepr(cast(Config, config)) == expected_size

    def test_get_maxsize_for_saferepr_no_config(self) -> None:
        assert _get_maxsize_for_saferepr(None) == DEFAULT_REPR_MAX_SIZE

    def create_test_file(self, pytester: Pytester, size: int) -> None:
        pytester.makepyfile(
            f"""
            def test_very_long_string():
                text = "x" * {size}
                assert "hello world" in text
            """
        )

    def test_default_verbosity(self, pytester: Pytester) -> None:
        self.create_test_file(pytester, DEFAULT_REPR_MAX_SIZE)
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*xxx...xxx*"])

    def test_increased_verbosity(self, pytester: Pytester) -> None:
        self.create_test_file(pytester, DEFAULT_REPR_MAX_SIZE)
        result = pytester.runpytest("-v")
        result.stdout.no_fnmatch_line("*xxx...xxx*")

    def test_max_increased_verbosity(self, pytester: Pytester) -> None:
        self.create_test_file(pytester, DEFAULT_REPR_MAX_SIZE * 10)
        result = pytester.runpytest("-vv")
        result.stdout.no_fnmatch_line("*xxx...xxx*")


class TestIssue11140:
    def test_constant_not_picked_as_module_docstring(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """\
            0

            def test_foo():
                pass
            """
        )
        result = pytester.runpytest()
        assert result.ret == 0
