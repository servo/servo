# flake8: noqa
# disable flake check on this file because some constructs are strange
# or redundant on purpose and can't be disable on a line-by-line basis
import ast
import inspect
import linecache
import sys
import textwrap
from pathlib import Path
from types import CodeType
from typing import Any
from typing import Dict
from typing import Optional

import pytest
from _pytest._code import Code
from _pytest._code import Frame
from _pytest._code import getfslineno
from _pytest._code import Source
from _pytest.pathlib import import_path


def test_source_str_function() -> None:
    x = Source("3")
    assert str(x) == "3"

    x = Source("   3")
    assert str(x) == "3"

    x = Source(
        """
        3
        """
    )
    assert str(x) == "\n3"


def test_source_from_function() -> None:
    source = Source(test_source_str_function)
    assert str(source).startswith("def test_source_str_function() -> None:")


def test_source_from_method() -> None:
    class TestClass:
        def test_method(self):
            pass

    source = Source(TestClass().test_method)
    assert source.lines == ["def test_method(self):", "    pass"]


def test_source_from_lines() -> None:
    lines = ["a \n", "b\n", "c"]
    source = Source(lines)
    assert source.lines == ["a ", "b", "c"]


def test_source_from_inner_function() -> None:
    def f():
        raise NotImplementedError()

    source = Source(f)
    assert str(source).startswith("def f():")


def test_source_strips() -> None:
    source = Source("")
    assert source == Source()
    assert str(source) == ""
    assert source.strip() == source


def test_source_strip_multiline() -> None:
    source = Source()
    source.lines = ["", " hello", "  "]
    source2 = source.strip()
    assert source2.lines == [" hello"]


class TestAccesses:
    def setup_class(self) -> None:
        self.source = Source(
            """\
            def f(x):
                pass
            def g(x):
                pass
        """
        )

    def test_getrange(self) -> None:
        x = self.source[0:2]
        assert len(x.lines) == 2
        assert str(x) == "def f(x):\n    pass"

    def test_getrange_step_not_supported(self) -> None:
        with pytest.raises(IndexError, match=r"step"):
            self.source[::2]

    def test_getline(self) -> None:
        x = self.source[0]
        assert x == "def f(x):"

    def test_len(self) -> None:
        assert len(self.source) == 4

    def test_iter(self) -> None:
        values = [x for x in self.source]
        assert len(values) == 4


class TestSourceParsing:
    def setup_class(self) -> None:
        self.source = Source(
            """\
            def f(x):
                assert (x ==
                        3 +
                        4)
        """
        ).strip()

    def test_getstatement(self) -> None:
        # print str(self.source)
        ass = str(self.source[1:])
        for i in range(1, 4):
            # print "trying start in line %r" % self.source[i]
            s = self.source.getstatement(i)
            # x = s.deindent()
            assert str(s) == ass

    def test_getstatementrange_triple_quoted(self) -> None:
        # print str(self.source)
        source = Source(
            """hello('''
        ''')"""
        )
        s = source.getstatement(0)
        assert s == source
        s = source.getstatement(1)
        assert s == source

    def test_getstatementrange_within_constructs(self) -> None:
        source = Source(
            """\
            try:
                try:
                    raise ValueError
                except SomeThing:
                    pass
            finally:
                42
        """
        )
        assert len(source) == 7
        # check all lineno's that could occur in a traceback
        # assert source.getstatementrange(0) == (0, 7)
        # assert source.getstatementrange(1) == (1, 5)
        assert source.getstatementrange(2) == (2, 3)
        assert source.getstatementrange(3) == (3, 4)
        assert source.getstatementrange(4) == (4, 5)
        # assert source.getstatementrange(5) == (0, 7)
        assert source.getstatementrange(6) == (6, 7)

    def test_getstatementrange_bug(self) -> None:
        source = Source(
            """\
            try:
                x = (
                   y +
                   z)
            except:
                pass
        """
        )
        assert len(source) == 6
        assert source.getstatementrange(2) == (1, 4)

    def test_getstatementrange_bug2(self) -> None:
        source = Source(
            """\
            assert (
                33
                ==
                [
                  X(3,
                      b=1, c=2
                   ),
                ]
              )
        """
        )
        assert len(source) == 9
        assert source.getstatementrange(5) == (0, 9)

    def test_getstatementrange_ast_issue58(self) -> None:
        source = Source(
            """\

            def test_some():
                for a in [a for a in
                    CAUSE_ERROR]: pass

            x = 3
        """
        )
        assert getstatement(2, source).lines == source.lines[2:3]
        assert getstatement(3, source).lines == source.lines[3:4]

    def test_getstatementrange_out_of_bounds_py3(self) -> None:
        source = Source("if xxx:\n   from .collections import something")
        r = source.getstatementrange(1)
        assert r == (1, 2)

    def test_getstatementrange_with_syntaxerror_issue7(self) -> None:
        source = Source(":")
        pytest.raises(SyntaxError, lambda: source.getstatementrange(0))


def test_getstartingblock_singleline() -> None:
    class A:
        def __init__(self, *args) -> None:
            frame = sys._getframe(1)
            self.source = Frame(frame).statement

    x = A("x", "y")

    values = [i for i in x.source.lines if i.strip()]
    assert len(values) == 1


def test_getline_finally() -> None:
    def c() -> None:
        pass

    with pytest.raises(TypeError) as excinfo:
        teardown = None
        try:
            c(1)  # type: ignore
        finally:
            if teardown:
                teardown()  # type: ignore[unreachable]
    source = excinfo.traceback[-1].statement
    assert str(source).strip() == "c(1)  # type: ignore"


def test_getfuncsource_dynamic() -> None:
    def f():
        raise NotImplementedError()

    def g():
        pass  # pragma: no cover

    f_source = Source(f)
    g_source = Source(g)
    assert str(f_source).strip() == "def f():\n    raise NotImplementedError()"
    assert str(g_source).strip() == "def g():\n    pass  # pragma: no cover"


def test_getfuncsource_with_multine_string() -> None:
    def f():
        c = """while True:
    pass
"""

    expected = '''\
    def f():
        c = """while True:
    pass
"""
'''
    assert str(Source(f)) == expected.rstrip()


def test_deindent() -> None:
    from _pytest._code.source import deindent as deindent

    assert deindent(["\tfoo", "\tbar"]) == ["foo", "bar"]

    source = """\
        def f():
            def g():
                pass
    """
    lines = deindent(source.splitlines())
    assert lines == ["def f():", "    def g():", "        pass"]


def test_source_of_class_at_eof_without_newline(_sys_snapshot, tmp_path: Path) -> None:
    # this test fails because the implicit inspect.getsource(A) below
    # does not return the "x = 1" last line.
    source = Source(
        """
        class A:
            def method(self):
                x = 1
    """
    )
    path = tmp_path.joinpath("a.py")
    path.write_text(str(source))
    mod: Any = import_path(path, root=tmp_path)
    s2 = Source(mod.A)
    assert str(source).strip() == str(s2).strip()


if True:

    def x():
        pass


def test_source_fallback() -> None:
    src = Source(x)
    expected = """def x():
    pass"""
    assert str(src) == expected


def test_findsource_fallback() -> None:
    from _pytest._code.source import findsource

    src, lineno = findsource(x)
    assert src is not None
    assert "test_findsource_simple" in str(src)
    assert src[lineno] == "    def x():"


def test_findsource(monkeypatch) -> None:
    from _pytest._code.source import findsource

    filename = "<pytest-test_findsource>"
    lines = ["if 1:\n", "    def x():\n", "          pass\n"]
    co = compile("".join(lines), filename, "exec")

    # Type ignored because linecache.cache is private.
    monkeypatch.setitem(linecache.cache, filename, (1, None, lines, filename))  # type: ignore[attr-defined]

    src, lineno = findsource(co)
    assert src is not None
    assert "if 1:" in str(src)

    d: Dict[str, Any] = {}
    eval(co, d)
    src, lineno = findsource(d["x"])
    assert src is not None
    assert "if 1:" in str(src)
    assert src[lineno] == "    def x():"


def test_getfslineno() -> None:
    def f(x) -> None:
        raise NotImplementedError()

    fspath, lineno = getfslineno(f)

    assert isinstance(fspath, Path)
    assert fspath.name == "test_source.py"
    assert lineno == f.__code__.co_firstlineno - 1  # see findsource

    class A:
        pass

    fspath, lineno = getfslineno(A)

    _, A_lineno = inspect.findsource(A)
    assert isinstance(fspath, Path)
    assert fspath.name == "test_source.py"
    assert lineno == A_lineno

    assert getfslineno(3) == ("", -1)

    class B:
        pass

    B.__name__ = B.__qualname__ = "B2"
    assert getfslineno(B)[1] == -1


def test_code_of_object_instance_with_call() -> None:
    class A:
        pass

    pytest.raises(TypeError, lambda: Source(A()))

    class WithCall:
        def __call__(self) -> None:
            pass

    code = Code.from_function(WithCall())
    assert "pass" in str(code.source())

    class Hello:
        def __call__(self) -> None:
            pass

    pytest.raises(TypeError, lambda: Code.from_function(Hello))


def getstatement(lineno: int, source) -> Source:
    from _pytest._code.source import getstatementrange_ast

    src = Source(source)
    ast, start, end = getstatementrange_ast(lineno, src)
    return src[start:end]


def test_oneline() -> None:
    source = getstatement(0, "raise ValueError")
    assert str(source) == "raise ValueError"


def test_comment_and_no_newline_at_end() -> None:
    from _pytest._code.source import getstatementrange_ast

    source = Source(
        [
            "def test_basic_complex():",
            "    assert 1 == 2",
            "# vim: filetype=pyopencl:fdm=marker",
        ]
    )
    ast, start, end = getstatementrange_ast(1, source)
    assert end == 2


def test_oneline_and_comment() -> None:
    source = getstatement(0, "raise ValueError\n#hello")
    assert str(source) == "raise ValueError"


def test_comments() -> None:
    source = '''def test():
    "comment 1"
    x = 1
      # comment 2
    # comment 3

    assert False

"""
comment 4
"""
'''
    for line in range(2, 6):
        assert str(getstatement(line, source)) == "    x = 1"
    if sys.version_info >= (3, 8) or hasattr(sys, "pypy_version_info"):
        tqs_start = 8
    else:
        tqs_start = 10
        assert str(getstatement(10, source)) == '"""'
    for line in range(6, tqs_start):
        assert str(getstatement(line, source)) == "    assert False"
    for line in range(tqs_start, 10):
        assert str(getstatement(line, source)) == '"""\ncomment 4\n"""'


def test_comment_in_statement() -> None:
    source = """test(foo=1,
    # comment 1
    bar=2)
"""
    for line in range(1, 3):
        assert (
            str(getstatement(line, source))
            == "test(foo=1,\n    # comment 1\n    bar=2)"
        )


def test_source_with_decorator() -> None:
    """Test behavior with Source / Code().source with regard to decorators."""
    from _pytest.compat import get_real_func

    @pytest.mark.foo
    def deco_mark():
        assert False

    src = inspect.getsource(deco_mark)
    assert textwrap.indent(str(Source(deco_mark)), "    ") + "\n" == src
    assert src.startswith("    @pytest.mark.foo")

    @pytest.fixture
    def deco_fixture():
        assert False

    src = inspect.getsource(deco_fixture)
    assert src == "    @pytest.fixture\n    def deco_fixture():\n        assert False\n"
    # currently Source does not unwrap decorators, testing the
    # existing behavior here for explicitness, but perhaps we should revisit/change this
    # in the future
    assert str(Source(deco_fixture)).startswith("@functools.wraps(function)")
    assert (
        textwrap.indent(str(Source(get_real_func(deco_fixture))), "    ") + "\n" == src
    )


def test_single_line_else() -> None:
    source = getstatement(1, "if False: 2\nelse: 3")
    assert str(source) == "else: 3"


def test_single_line_finally() -> None:
    source = getstatement(1, "try: 1\nfinally: 3")
    assert str(source) == "finally: 3"


def test_issue55() -> None:
    source = (
        "def round_trip(dinp):\n  assert 1 == dinp\n"
        'def test_rt():\n  round_trip("""\n""")\n'
    )
    s = getstatement(3, source)
    assert str(s) == '  round_trip("""\n""")'


def test_multiline() -> None:
    source = getstatement(
        0,
        """\
raise ValueError(
    23
)
x = 3
""",
    )
    assert str(source) == "raise ValueError(\n    23\n)"


class TestTry:
    def setup_class(self) -> None:
        self.source = """\
try:
    raise ValueError
except Something:
    raise IndexError(1)
else:
    raise KeyError()
"""

    def test_body(self) -> None:
        source = getstatement(1, self.source)
        assert str(source) == "    raise ValueError"

    def test_except_line(self) -> None:
        source = getstatement(2, self.source)
        assert str(source) == "except Something:"

    def test_except_body(self) -> None:
        source = getstatement(3, self.source)
        assert str(source) == "    raise IndexError(1)"

    def test_else(self) -> None:
        source = getstatement(5, self.source)
        assert str(source) == "    raise KeyError()"


class TestTryFinally:
    def setup_class(self) -> None:
        self.source = """\
try:
    raise ValueError
finally:
    raise IndexError(1)
"""

    def test_body(self) -> None:
        source = getstatement(1, self.source)
        assert str(source) == "    raise ValueError"

    def test_finally(self) -> None:
        source = getstatement(3, self.source)
        assert str(source) == "    raise IndexError(1)"


class TestIf:
    def setup_class(self) -> None:
        self.source = """\
if 1:
    y = 3
elif False:
    y = 5
else:
    y = 7
"""

    def test_body(self) -> None:
        source = getstatement(1, self.source)
        assert str(source) == "    y = 3"

    def test_elif_clause(self) -> None:
        source = getstatement(2, self.source)
        assert str(source) == "elif False:"

    def test_elif(self) -> None:
        source = getstatement(3, self.source)
        assert str(source) == "    y = 5"

    def test_else(self) -> None:
        source = getstatement(5, self.source)
        assert str(source) == "    y = 7"


def test_semicolon() -> None:
    s = """\
hello ; pytest.skip()
"""
    source = getstatement(0, s)
    assert str(source) == s.strip()


def test_def_online() -> None:
    s = """\
def func(): raise ValueError(42)

def something():
    pass
"""
    source = getstatement(0, s)
    assert str(source) == "def func(): raise ValueError(42)"


def test_decorator() -> None:
    s = """\
def foo(f):
    pass

@foo
def bar():
    pass
    """
    source = getstatement(3, s)
    assert "@foo" in str(source)


def XXX_test_expression_multiline() -> None:
    source = """\
something
'''
'''"""
    result = getstatement(1, source)
    assert str(result) == "'''\n'''"


def test_getstartingblock_multiline() -> None:
    class A:
        def __init__(self, *args):
            frame = sys._getframe(1)
            self.source = Frame(frame).statement

    # fmt: off
    x = A('x',
          'y'
          ,
          'z')
    # fmt: on
    values = [i for i in x.source.lines if i.strip()]
    assert len(values) == 4
