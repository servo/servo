# -*- coding: utf-8 -*-
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import glob
import os
import py_compile
import stat
import sys
import textwrap
import zipfile

import py
import six

import _pytest._code
import pytest
from _pytest.assertion import util
from _pytest.assertion.rewrite import AssertionRewritingHook
from _pytest.assertion.rewrite import PYTEST_TAG
from _pytest.assertion.rewrite import rewrite_asserts
from _pytest.main import EXIT_NOTESTSCOLLECTED

ast = pytest.importorskip("ast")
if sys.platform.startswith("java"):
    # XXX should be xfail
    pytest.skip("assert rewrite does currently not work on jython")


def setup_module(mod):
    mod._old_reprcompare = util._reprcompare
    _pytest._code._reprcompare = None


def teardown_module(mod):
    util._reprcompare = mod._old_reprcompare
    del mod._old_reprcompare


def rewrite(src):
    tree = ast.parse(src)
    rewrite_asserts(tree)
    return tree


def getmsg(f, extra_ns=None, must_pass=False):
    """Rewrite the assertions in f, run it, and get the failure message."""
    src = "\n".join(_pytest._code.Code(f).source().lines)
    mod = rewrite(src)
    code = compile(mod, "<test>", "exec")
    ns = {}
    if extra_ns is not None:
        ns.update(extra_ns)
    exec(code, ns)
    func = ns[f.__name__]
    try:
        func()
    except AssertionError:
        if must_pass:
            pytest.fail("shouldn't have raised")
        s = six.text_type(sys.exc_info()[1])
        if not s.startswith("assert"):
            return "AssertionError: " + s
        return s
    else:
        if not must_pass:
            pytest.fail("function didn't raise at all")


class TestAssertionRewrite(object):
    def test_place_initial_imports(self):
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

    def test_dont_rewrite(self):
        s = """'PYTEST_DONT_REWRITE'\nassert 14"""
        m = rewrite(s)
        assert len(m.body) == 2
        assert m.body[1].msg is None

    def test_dont_rewrite_plugin(self, testdir):
        contents = {
            "conftest.py": "pytest_plugins = 'plugin'; import plugin",
            "plugin.py": "'PYTEST_DONT_REWRITE'",
            "test_foo.py": "def test_foo(): pass",
        }
        testdir.makepyfile(**contents)
        result = testdir.runpytest_subprocess()
        assert "warnings" not in "".join(result.outlines)

    def test_name(self, request):
        def f():
            assert False

        assert getmsg(f) == "assert False"

        def f():
            f = False
            assert f

        assert getmsg(f) == "assert False"

        def f():
            assert a_global  # noqa

        assert getmsg(f, {"a_global": False}) == "assert False"

        def f():
            assert sys == 42

        verbose = request.config.getoption("verbose")
        msg = getmsg(f, {"sys": sys})
        if verbose > 0:
            assert msg == (
                "assert <module 'sys' (built-in)> == 42\n"
                "  -<module 'sys' (built-in)>\n"
                "  +42"
            )
        else:
            assert msg == "assert sys == 42"

        def f():
            assert cls == 42  # noqa: F821

        class X(object):
            pass

        msg = getmsg(f, {"cls": X}).splitlines()
        if verbose > 0:
            if six.PY2:
                assert msg == [
                    "assert <class 'test_assertrewrite.X'> == 42",
                    "  -<class 'test_assertrewrite.X'>",
                    "  +42",
                ]
            else:
                assert msg == [
                    "assert <class 'test_...e.<locals>.X'> == 42",
                    "  -<class 'test_assertrewrite.TestAssertionRewrite.test_name.<locals>.X'>",
                    "  +42",
                ]
        else:
            assert msg == ["assert cls == 42"]

    def test_dont_rewrite_if_hasattr_fails(self, request):
        class Y(object):
            """ A class whos getattr fails, but not with `AttributeError` """

            def __getattr__(self, attribute_name):
                raise KeyError()

            def __repr__(self):
                return "Y"

            def __init__(self):
                self.foo = 3

        def f():
            assert cls().foo == 2  # noqa

        # XXX: looks like the "where" should also be there in verbose mode?!
        message = getmsg(f, {"cls": Y}).splitlines()
        if request.config.getoption("verbose") > 0:
            assert message == ["assert 3 == 2", "  -3", "  +2"]
        else:
            assert message == [
                "assert 3 == 2",
                " +  where 3 = Y.foo",
                " +    where Y = cls()",
            ]

    def test_assert_already_has_message(self):
        def f():
            assert False, "something bad!"

        assert getmsg(f) == "AssertionError: something bad!\nassert False"

    def test_assertion_message(self, testdir):
        testdir.makepyfile(
            """
            def test_foo():
                assert 1 == 2, "The failure message"
        """
        )
        result = testdir.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(
            ["*AssertionError*The failure message*", "*assert 1 == 2*"]
        )

    def test_assertion_message_multiline(self, testdir):
        testdir.makepyfile(
            """
            def test_foo():
                assert 1 == 2, "A multiline\\nfailure message"
        """
        )
        result = testdir.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(
            ["*AssertionError*A multiline*", "*failure message*", "*assert 1 == 2*"]
        )

    def test_assertion_message_tuple(self, testdir):
        testdir.makepyfile(
            """
            def test_foo():
                assert 1 == 2, (1, 2)
        """
        )
        result = testdir.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(
            ["*AssertionError*%s*" % repr((1, 2)), "*assert 1 == 2*"]
        )

    def test_assertion_message_expr(self, testdir):
        testdir.makepyfile(
            """
            def test_foo():
                assert 1 == 2, 1 + 2
        """
        )
        result = testdir.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*AssertionError*3*", "*assert 1 == 2*"])

    def test_assertion_message_escape(self, testdir):
        testdir.makepyfile(
            """
            def test_foo():
                assert 1 == 2, 'To be escaped: %'
        """
        )
        result = testdir.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(
            ["*AssertionError: To be escaped: %", "*assert 1 == 2"]
        )

    @pytest.mark.skipif(
        sys.version_info < (3,), reason="bytes is a string type in python 2"
    )
    def test_assertion_messages_bytes(self, testdir):
        testdir.makepyfile("def test_bytes_assertion():\n    assert False, b'ohai!'\n")
        result = testdir.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*AssertionError: b'ohai!'", "*assert False"])

    def test_boolop(self):
        def f():
            f = g = False
            assert f and g

        assert getmsg(f) == "assert (False)"

        def f():
            f = True
            g = False
            assert f and g

        assert getmsg(f) == "assert (True and False)"

        def f():
            f = False
            g = True
            assert f and g

        assert getmsg(f) == "assert (False)"

        def f():
            f = g = False
            assert f or g

        assert getmsg(f) == "assert (False or False)"

        def f():
            f = g = False
            assert not f and not g

        getmsg(f, must_pass=True)

        def x():
            return False

        def f():
            assert x() and x()

        assert (
            getmsg(f, {"x": x})
            == """assert (False)
 +  where False = x()"""
        )

        def f():
            assert False or x()

        assert (
            getmsg(f, {"x": x})
            == """assert (False or False)
 +  where False = x()"""
        )

        def f():
            assert 1 in {} and 2 in {}

        assert getmsg(f) == "assert (1 in {})"

        def f():
            x = 1
            y = 2
            assert x in {1: None} and y in {}

        assert getmsg(f) == "assert (1 in {1: None} and 2 in {})"

        def f():
            f = True
            g = False
            assert f or g

        getmsg(f, must_pass=True)

        def f():
            f = g = h = lambda: True
            assert f() and g() and h()

        getmsg(f, must_pass=True)

    def test_short_circuit_evaluation(self):
        def f():
            assert True or explode  # noqa

        getmsg(f, must_pass=True)

        def f():
            x = 1
            assert x == 1 or x == 2

        getmsg(f, must_pass=True)

    def test_unary_op(self):
        def f():
            x = True
            assert not x

        assert getmsg(f) == "assert not True"

        def f():
            x = 0
            assert ~x + 1

        assert getmsg(f) == "assert (~0 + 1)"

        def f():
            x = 3
            assert -x + x

        assert getmsg(f) == "assert (-3 + 3)"

        def f():
            x = 0
            assert +x + x

        assert getmsg(f) == "assert (+0 + 0)"

    def test_binary_op(self):
        def f():
            x = 1
            y = -1
            assert x + y

        assert getmsg(f) == "assert (1 + -1)"

        def f():
            assert not 5 % 4

        assert getmsg(f) == "assert not (5 % 4)"

    def test_boolop_percent(self):
        def f():
            assert 3 % 2 and False

        assert getmsg(f) == "assert ((3 % 2) and False)"

        def f():
            assert False or 4 % 2

        assert getmsg(f) == "assert (False or (4 % 2))"

    @pytest.mark.skipif("sys.version_info < (3,5)")
    def test_at_operator_issue1290(self, testdir):
        testdir.makepyfile(
            """
            class Matrix(object):
                def __init__(self, num):
                    self.num = num
                def __matmul__(self, other):
                    return self.num * other.num

            def test_multmat_operator():
                assert Matrix(2) @ Matrix(3) == 6"""
        )
        testdir.runpytest().assert_outcomes(passed=1)

    @pytest.mark.skipif("sys.version_info < (3,5)")
    def test_starred_with_side_effect(self, testdir):
        """See #4412"""
        testdir.makepyfile(
            """\
            def test():
                f = lambda x: x
                x = iter([1, 2, 3])
                assert 2 * next(x) == f(*[next(x)])
            """
        )
        testdir.runpytest().assert_outcomes(passed=1)

    def test_call(self):
        def g(a=42, *args, **kwargs):
            return False

        ns = {"g": g}

        def f():
            assert g()

        assert (
            getmsg(f, ns)
            == """assert False
 +  where False = g()"""
        )

        def f():
            assert g(1)

        assert (
            getmsg(f, ns)
            == """assert False
 +  where False = g(1)"""
        )

        def f():
            assert g(1, 2)

        assert (
            getmsg(f, ns)
            == """assert False
 +  where False = g(1, 2)"""
        )

        def f():
            assert g(1, g=42)

        assert (
            getmsg(f, ns)
            == """assert False
 +  where False = g(1, g=42)"""
        )

        def f():
            assert g(1, 3, g=23)

        assert (
            getmsg(f, ns)
            == """assert False
 +  where False = g(1, 3, g=23)"""
        )

        def f():
            seq = [1, 2, 3]
            assert g(*seq)

        assert (
            getmsg(f, ns)
            == """assert False
 +  where False = g(*[1, 2, 3])"""
        )

        def f():
            x = "a"
            assert g(**{x: 2})

        assert (
            getmsg(f, ns)
            == """assert False
 +  where False = g(**{'a': 2})"""
        )

    def test_attribute(self):
        class X(object):
            g = 3

        ns = {"x": X}

        def f():
            assert not x.g  # noqa

        assert (
            getmsg(f, ns)
            == """assert not 3
 +  where 3 = x.g"""
        )

        def f():
            x.a = False  # noqa
            assert x.a  # noqa

        assert (
            getmsg(f, ns)
            == """assert False
 +  where False = x.a"""
        )

    def test_comparisons(self):
        def f():
            a, b = range(2)
            assert b < a

        assert getmsg(f) == """assert 1 < 0"""

        def f():
            a, b, c = range(3)
            assert a > b > c

        assert getmsg(f) == """assert 0 > 1"""

        def f():
            a, b, c = range(3)
            assert a < b > c

        assert getmsg(f) == """assert 1 > 2"""

        def f():
            a, b, c = range(3)
            assert a < b <= c

        getmsg(f, must_pass=True)

        def f():
            a, b, c = range(3)
            assert a < b
            assert b < c

        getmsg(f, must_pass=True)

    def test_len(self, request):
        def f():
            values = list(range(10))
            assert len(values) == 11

        msg = getmsg(f)
        if request.config.getoption("verbose") > 0:
            assert msg == "assert 10 == 11\n  -10\n  +11"
        else:
            assert msg == "assert 10 == 11\n +  where 10 = len([0, 1, 2, 3, 4, 5, ...])"

    def test_custom_reprcompare(self, monkeypatch):
        def my_reprcompare(op, left, right):
            return "42"

        monkeypatch.setattr(util, "_reprcompare", my_reprcompare)

        def f():
            assert 42 < 3

        assert getmsg(f) == "assert 42"

        def my_reprcompare(op, left, right):
            return "{} {} {}".format(left, op, right)

        monkeypatch.setattr(util, "_reprcompare", my_reprcompare)

        def f():
            assert 1 < 3 < 5 <= 4 < 7

        assert getmsg(f) == "assert 5 <= 4"

    def test_assert_raising_nonzero_in_comparison(self):
        def f():
            class A(object):
                def __nonzero__(self):
                    raise ValueError(42)

                def __lt__(self, other):
                    return A()

                def __repr__(self):
                    return "<MY42 object>"

            def myany(x):
                return False

            assert myany(A() < 0)

        assert "<MY42 object> < 0" in getmsg(f)

    def test_formatchar(self):
        def f():
            assert "%test" == "test"

        assert getmsg(f).startswith("assert '%test' == 'test'")

    def test_custom_repr(self, request):
        def f():
            class Foo(object):
                a = 1

                def __repr__(self):
                    return "\n{ \n~ \n}"

            f = Foo()
            assert 0 == f.a

        lines = util._format_lines([getmsg(f)])
        if request.config.getoption("verbose") > 0:
            assert lines == ["assert 0 == 1\n  -0\n  +1"]
        else:
            assert lines == ["assert 0 == 1\n +  where 1 = \\n{ \\n~ \\n}.a"]

    def test_custom_repr_non_ascii(self):
        def f():
            class A(object):
                name = u"ä"

                def __repr__(self):
                    return self.name.encode("UTF-8")  # only legal in python2

            a = A()
            assert not a.name

        msg = getmsg(f)
        assert "UnicodeDecodeError" not in msg
        assert "UnicodeEncodeError" not in msg


class TestRewriteOnImport(object):
    def test_pycache_is_a_file(self, testdir):
        testdir.tmpdir.join("__pycache__").write("Hello")
        testdir.makepyfile(
            """
            def test_rewritten():
                assert "@py_builtins" in globals()"""
        )
        assert testdir.runpytest().ret == 0

    def test_pycache_is_readonly(self, testdir):
        cache = testdir.tmpdir.mkdir("__pycache__")
        old_mode = cache.stat().mode
        cache.chmod(old_mode ^ stat.S_IWRITE)
        testdir.makepyfile(
            """
            def test_rewritten():
                assert "@py_builtins" in globals()"""
        )
        try:
            assert testdir.runpytest().ret == 0
        finally:
            cache.chmod(old_mode)

    def test_zipfile(self, testdir):
        z = testdir.tmpdir.join("myzip.zip")
        z_fn = str(z)
        f = zipfile.ZipFile(z_fn, "w")
        try:
            f.writestr("test_gum/__init__.py", "")
            f.writestr("test_gum/test_lizard.py", "")
        finally:
            f.close()
        z.chmod(256)
        testdir.makepyfile(
            """
            import sys
            sys.path.append(%r)
            import test_gum.test_lizard"""
            % (z_fn,)
        )
        assert testdir.runpytest().ret == EXIT_NOTESTSCOLLECTED

    def test_readonly(self, testdir):
        sub = testdir.mkdir("testing")
        sub.join("test_readonly.py").write(
            b"""
def test_rewritten():
    assert "@py_builtins" in globals()
            """,
            "wb",
        )
        old_mode = sub.stat().mode
        sub.chmod(320)
        try:
            assert testdir.runpytest().ret == 0
        finally:
            sub.chmod(old_mode)

    def test_dont_write_bytecode(self, testdir, monkeypatch):
        testdir.makepyfile(
            """
            import os
            def test_no_bytecode():
                assert "__pycache__" in __cached__
                assert not os.path.exists(__cached__)
                assert not os.path.exists(os.path.dirname(__cached__))"""
        )
        monkeypatch.setenv("PYTHONDONTWRITEBYTECODE", "1")
        assert testdir.runpytest_subprocess().ret == 0

    def test_orphaned_pyc_file(self, testdir):
        if sys.version_info < (3, 0) and hasattr(sys, "pypy_version_info"):
            pytest.skip("pypy2 doesn't run orphaned pyc files")

        testdir.makepyfile(
            """
            import orphan
            def test_it():
                assert orphan.value == 17
            """
        )
        testdir.makepyfile(
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

        assert testdir.runpytest().ret == 0

    @pytest.mark.skipif('"__pypy__" in sys.modules')
    def test_pyc_vs_pyo(self, testdir, monkeypatch):
        testdir.makepyfile(
            """
            import pytest
            def test_optimized():
                "hello"
                assert test_optimized.__doc__ is None"""
        )
        p = py.path.local.make_numbered_dir(
            prefix="runpytest-", keep=None, rootdir=testdir.tmpdir
        )
        tmp = "--basetemp=%s" % p
        monkeypatch.setenv("PYTHONOPTIMIZE", "2")
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", raising=False)
        assert testdir.runpytest_subprocess(tmp).ret == 0
        tagged = "test_pyc_vs_pyo." + PYTEST_TAG
        assert tagged + ".pyo" in os.listdir("__pycache__")
        monkeypatch.undo()
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", raising=False)
        assert testdir.runpytest_subprocess(tmp).ret == 1
        assert tagged + ".pyc" in os.listdir("__pycache__")

    def test_package(self, testdir):
        pkg = testdir.tmpdir.join("pkg")
        pkg.mkdir()
        pkg.join("__init__.py").ensure()
        pkg.join("test_blah.py").write(
            """
def test_rewritten():
    assert "@py_builtins" in globals()"""
        )
        assert testdir.runpytest().ret == 0

    def test_translate_newlines(self, testdir):
        content = "def test_rewritten():\r\n assert '@py_builtins' in globals()"
        b = content.encode("utf-8")
        testdir.tmpdir.join("test_newlines.py").write(b, "wb")
        assert testdir.runpytest().ret == 0

    @pytest.mark.skipif(
        sys.version_info < (3, 4),
        reason="packages without __init__.py not supported on python 2",
    )
    def test_package_without__init__py(self, testdir):
        pkg = testdir.mkdir("a_package_without_init_py")
        pkg.join("module.py").ensure()
        testdir.makepyfile("import a_package_without_init_py.module")
        assert testdir.runpytest().ret == EXIT_NOTESTSCOLLECTED

    def test_rewrite_warning(self, testdir):
        testdir.makeconftest(
            """
            import pytest
            pytest.register_assert_rewrite("_pytest")
        """
        )
        # needs to be a subprocess because pytester explicitly disables this warning
        result = testdir.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*Module already imported*: _pytest"])

    def test_rewrite_module_imported_from_conftest(self, testdir):
        testdir.makeconftest(
            """
            import test_rewrite_module_imported
        """
        )
        testdir.makepyfile(
            test_rewrite_module_imported="""
            def test_rewritten():
                assert "@py_builtins" in globals()
        """
        )
        assert testdir.runpytest_subprocess().ret == 0

    def test_remember_rewritten_modules(self, pytestconfig, testdir, monkeypatch):
        """
        AssertionRewriteHook should remember rewritten modules so it
        doesn't give false positives (#2005).
        """
        monkeypatch.syspath_prepend(testdir.tmpdir)
        testdir.makepyfile(test_remember_rewritten_modules="")
        warnings = []
        hook = AssertionRewritingHook(pytestconfig)
        monkeypatch.setattr(
            hook, "_warn_already_imported", lambda code, msg: warnings.append(msg)
        )
        hook.find_module("test_remember_rewritten_modules")
        hook.load_module("test_remember_rewritten_modules")
        hook.mark_rewrite("test_remember_rewritten_modules")
        hook.mark_rewrite("test_remember_rewritten_modules")
        assert warnings == []

    def test_rewrite_warning_using_pytest_plugins(self, testdir):
        testdir.makepyfile(
            **{
                "conftest.py": "pytest_plugins = ['core', 'gui', 'sci']",
                "core.py": "",
                "gui.py": "pytest_plugins = ['core', 'sci']",
                "sci.py": "pytest_plugins = ['core']",
                "test_rewrite_warning_pytest_plugins.py": "def test(): pass",
            }
        )
        testdir.chdir()
        result = testdir.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*= 1 passed in *=*"])
        assert "pytest-warning summary" not in result.stdout.str()

    def test_rewrite_warning_using_pytest_plugins_env_var(self, testdir, monkeypatch):
        monkeypatch.setenv("PYTEST_PLUGINS", "plugin")
        testdir.makepyfile(
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
        testdir.chdir()
        result = testdir.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*= 1 passed in *=*"])
        assert "pytest-warning summary" not in result.stdout.str()

    @pytest.mark.skipif(sys.version_info[0] > 2, reason="python 2 only")
    def test_rewrite_future_imports(self, testdir):
        """Test that rewritten modules don't inherit the __future__ flags
        from the assertrewrite module.

        assertion.rewrite imports __future__.division (and others), so
        ensure rewritten modules don't inherit those flags.

        The test below will fail if __future__.division is enabled
        """
        testdir.makepyfile(
            """
            def test():
                x = 1 / 2
                assert type(x) is int
        """
        )
        result = testdir.runpytest()
        assert result.ret == 0


class TestAssertionRewriteHookDetails(object):
    def test_loader_is_package_false_for_module(self, testdir):
        testdir.makepyfile(
            test_fun="""
            def test_loader():
                assert not __loader__.is_package(__name__)
            """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 1 passed*"])

    def test_loader_is_package_true_for_package(self, testdir):
        testdir.makepyfile(
            test_fun="""
            def test_loader():
                assert not __loader__.is_package(__name__)

            def test_fun():
                assert __loader__.is_package('fun')

            def test_missing():
                assert not __loader__.is_package('pytest_not_there')
            """
        )
        testdir.mkpydir("fun")
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 3 passed*"])

    @pytest.mark.skipif("sys.version_info[0] >= 3")
    @pytest.mark.xfail("hasattr(sys, 'pypy_translation_info')")
    def test_assume_ascii(self, testdir):
        content = "u'\xe2\x99\xa5\x01\xfe'"
        testdir.tmpdir.join("test_encoding.py").write(content, "wb")
        res = testdir.runpytest()
        assert res.ret != 0
        assert "SyntaxError: Non-ASCII character" in res.stdout.str()

    @pytest.mark.skipif("sys.version_info[0] >= 3")
    def test_detect_coding_cookie(self, testdir):
        testdir.makepyfile(
            test_cookie="""
            # -*- coding: utf-8 -*-
            u"St\xc3\xa4d"
            def test_rewritten():
                assert "@py_builtins" in globals()"""
        )
        assert testdir.runpytest().ret == 0

    @pytest.mark.skipif("sys.version_info[0] >= 3")
    def test_detect_coding_cookie_second_line(self, testdir):
        testdir.makepyfile(
            test_cookie="""
            # -*- coding: utf-8 -*-
            u"St\xc3\xa4d"
            def test_rewritten():
                assert "@py_builtins" in globals()"""
        )
        assert testdir.runpytest().ret == 0

    @pytest.mark.skipif("sys.version_info[0] >= 3")
    def test_detect_coding_cookie_crlf(self, testdir):
        testdir.makepyfile(
            test_cookie="""
            # -*- coding: utf-8 -*-
            u"St\xc3\xa4d"
            def test_rewritten():
                assert "@py_builtins" in globals()"""
        )
        assert testdir.runpytest().ret == 0

    def test_sys_meta_path_munged(self, testdir):
        testdir.makepyfile(
            """
            def test_meta_path():
                import sys; sys.meta_path = []"""
        )
        assert testdir.runpytest().ret == 0

    def test_write_pyc(self, testdir, tmpdir, monkeypatch):
        from _pytest.assertion.rewrite import _write_pyc
        from _pytest.assertion import AssertionState
        import atomicwrites
        from contextlib import contextmanager

        config = testdir.parseconfig([])
        state = AssertionState(config, "rewrite")
        source_path = tmpdir.ensure("source.py")
        pycpath = tmpdir.join("pyc").strpath
        assert _write_pyc(state, [1], source_path.stat(), pycpath)

        @contextmanager
        def atomic_write_failed(fn, mode="r", overwrite=False):
            e = IOError()
            e.errno = 10
            raise e
            yield

        monkeypatch.setattr(atomicwrites, "atomic_write", atomic_write_failed)
        assert not _write_pyc(state, [1], source_path.stat(), pycpath)

    def test_resources_provider_for_loader(self, testdir):
        """
        Attempts to load resources from a package should succeed normally,
        even when the AssertionRewriteHook is used to load the modules.

        See #366 for details.
        """
        pytest.importorskip("pkg_resources")

        testdir.mkpydir("testpkg")
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
        testdir.makepyfile(**contents)
        testdir.maketxtfile(**{"testpkg/resource": "Load me please."})

        result = testdir.runpytest_subprocess()
        result.assert_outcomes(passed=1)

    def test_read_pyc(self, tmpdir):
        """
        Ensure that the `_read_pyc` can properly deal with corrupted pyc files.
        In those circumstances it should just give up instead of generating
        an exception that is propagated to the caller.
        """
        import py_compile
        from _pytest.assertion.rewrite import _read_pyc

        source = tmpdir.join("source.py")
        pyc = source + "c"

        source.write("def test(): pass")
        py_compile.compile(str(source), str(pyc))

        contents = pyc.read(mode="rb")
        strip_bytes = 20  # header is around 8 bytes, strip a little more
        assert len(contents) > strip_bytes
        pyc.write(contents[:strip_bytes], mode="wb")

        assert _read_pyc(source, str(pyc)) is None  # no error

    def test_reload_is_same(self, testdir):
        # A file that will be picked up during collecting.
        testdir.tmpdir.join("file.py").ensure()
        testdir.tmpdir.join("pytest.ini").write(
            textwrap.dedent(
                """
            [pytest]
            python_files = *.py
        """
            )
        )

        testdir.makepyfile(
            test_fun="""
            import sys
            try:
                from imp import reload
            except ImportError:
                pass

            def test_loader():
                import file
                assert sys.modules["file"] is reload(file)
            """
        )
        result = testdir.runpytest("-s")
        result.stdout.fnmatch_lines(["* 1 passed*"])

    def test_reload_reloads(self, testdir):
        """Reloading a module after change picks up the change."""
        testdir.tmpdir.join("file.py").write(
            textwrap.dedent(
                """
            def reloaded():
                return False

            def rewrite_self():
                with open(__file__, 'w') as self:
                    self.write('def reloaded(): return True')
        """
            )
        )
        testdir.tmpdir.join("pytest.ini").write(
            textwrap.dedent(
                """
            [pytest]
            python_files = *.py
        """
            )
        )

        testdir.makepyfile(
            test_fun="""
            import sys
            try:
                from imp import reload
            except ImportError:
                pass

            def test_loader():
                import file
                assert not file.reloaded()
                file.rewrite_self()
                reload(file)
                assert file.reloaded()
            """
        )
        result = testdir.runpytest("-s")
        result.stdout.fnmatch_lines(["* 1 passed*"])

    def test_get_data_support(self, testdir):
        """Implement optional PEP302 api (#808).
        """
        path = testdir.mkpydir("foo")
        path.join("test_foo.py").write(
            textwrap.dedent(
                """\
                class Test(object):
                    def test_foo(self):
                        import pkgutil
                        data = pkgutil.get_data('foo.test_foo', 'data.txt')
                        assert data == b'Hey'
                """
            )
        )
        path.join("data.txt").write("Hey")
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 passed*"])


def test_issue731(testdir):
    testdir.makepyfile(
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
    result = testdir.runpytest()
    assert "unbalanced braces" not in result.stdout.str()


class TestIssue925(object):
    def test_simple_case(self, testdir):
        testdir.makepyfile(
            """
        def test_ternary_display():
            assert (False == False) == False
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*E*assert (False == False) == False"])

    def test_long_case(self, testdir):
        testdir.makepyfile(
            """
        def test_ternary_display():
             assert False == (False == True) == True
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*E*assert (False == True) == True"])

    def test_many_brackets(self, testdir):
        testdir.makepyfile(
            """
            def test_ternary_display():
                 assert True == ((False == True) == True)
            """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*E*assert True == ((False == True) == True)"])


class TestIssue2121:
    def test_rewrite_python_files_contain_subdirs(self, testdir):
        testdir.makepyfile(
            **{
                "tests/file.py": """
                def test_simple_failure():
                    assert 1 + 1 == 3
                """
            }
        )
        testdir.makeini(
            """
                [pytest]
                python_files = tests/**.py
            """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*E*assert (1 + 1) == 3"])


@pytest.mark.skipif(
    sys.maxsize <= (2 ** 31 - 1), reason="Causes OverflowError on 32bit systems"
)
@pytest.mark.parametrize("offset", [-1, +1])
def test_source_mtime_long_long(testdir, offset):
    """Support modification dates after 2038 in rewritten files (#4903).

    pytest would crash with:

            fp.write(struct.pack("<ll", mtime, size))
        E   struct.error: argument out of range
    """
    p = testdir.makepyfile(
        """
        def test(): pass
    """
    )
    # use unsigned long timestamp which overflows signed long,
    # which was the cause of the bug
    # +1 offset also tests masking of 0xFFFFFFFF
    timestamp = 2 ** 32 + offset
    os.utime(str(p), (timestamp, timestamp))
    result = testdir.runpytest()
    assert result.ret == 0


def test_rewrite_infinite_recursion(testdir, pytestconfig, monkeypatch):
    """Fix infinite recursion when writing pyc files: if an import happens to be triggered when writing the pyc
    file, this would cause another call to the hook, which would trigger another pyc writing, which could
    trigger another import, and so on. (#3506)"""
    from _pytest.assertion import rewrite

    testdir.syspathinsert()
    testdir.makepyfile(test_foo="def test_foo(): pass")
    testdir.makepyfile(test_bar="def test_bar(): pass")

    original_write_pyc = rewrite._write_pyc

    write_pyc_called = []

    def spy_write_pyc(*args, **kwargs):
        # make a note that we have called _write_pyc
        write_pyc_called.append(True)
        # try to import a module at this point: we should not try to rewrite this module
        assert hook.find_module("test_bar") is None
        return original_write_pyc(*args, **kwargs)

    monkeypatch.setattr(rewrite, "_write_pyc", spy_write_pyc)
    monkeypatch.setattr(sys, "dont_write_bytecode", False)

    hook = AssertionRewritingHook(pytestconfig)
    assert hook.find_module("test_foo") is not None
    assert len(write_pyc_called) == 1


class TestEarlyRewriteBailout(object):
    @pytest.fixture
    def hook(self, pytestconfig, monkeypatch, testdir):
        """Returns a patched AssertionRewritingHook instance so we can configure its initial paths and track
        if imp.find_module has been called.
        """
        import imp

        self.find_module_calls = []
        self.initial_paths = set()

        class StubSession(object):
            _initialpaths = self.initial_paths

            def isinitpath(self, p):
                return p in self._initialpaths

        def spy_imp_find_module(name, path):
            self.find_module_calls.append(name)
            return imp.find_module(name, path)

        hook = AssertionRewritingHook(pytestconfig)
        # use default patterns, otherwise we inherit pytest's testing config
        hook.fnpats[:] = ["test_*.py", "*_test.py"]
        monkeypatch.setattr(hook, "_imp_find_module", spy_imp_find_module)
        hook.set_session(StubSession())
        testdir.syspathinsert()
        return hook

    def test_basic(self, testdir, hook):
        """
        Ensure we avoid calling imp.find_module when we know for sure a certain module will not be rewritten
        to optimize assertion rewriting (#3918).
        """
        testdir.makeconftest(
            """
            import pytest
            @pytest.fixture
            def fix(): return 1
        """
        )
        testdir.makepyfile(test_foo="def test_foo(): pass")
        testdir.makepyfile(bar="def bar(): pass")
        foobar_path = testdir.makepyfile(foobar="def foobar(): pass")
        self.initial_paths.add(foobar_path)

        # conftest files should always be rewritten
        assert hook.find_module("conftest") is not None
        assert self.find_module_calls == ["conftest"]

        # files matching "python_files" mask should always be rewritten
        assert hook.find_module("test_foo") is not None
        assert self.find_module_calls == ["conftest", "test_foo"]

        # file does not match "python_files": early bailout
        assert hook.find_module("bar") is None
        assert self.find_module_calls == ["conftest", "test_foo"]

        # file is an initial path (passed on the command-line): should be rewritten
        assert hook.find_module("foobar") is not None
        assert self.find_module_calls == ["conftest", "test_foo", "foobar"]

    def test_pattern_contains_subdirectories(self, testdir, hook):
        """If one of the python_files patterns contain subdirectories ("tests/**.py") we can't bailout early
        because we need to match with the full path, which can only be found by calling imp.find_module.
        """
        p = testdir.makepyfile(
            **{
                "tests/file.py": """
                        def test_simple_failure():
                            assert 1 + 1 == 3
                        """
            }
        )
        testdir.syspathinsert(p.dirpath())
        hook.fnpats[:] = ["tests/**.py"]
        assert hook.find_module("file") is not None
        assert self.find_module_calls == ["file"]

    @pytest.mark.skipif(
        sys.platform.startswith("win32"), reason="cannot remove cwd on Windows"
    )
    def test_cwd_changed(self, testdir, monkeypatch):
        # Setup conditions for py's fspath trying to import pathlib on py34
        # always (previously triggered via xdist only).
        # Ref: https://github.com/pytest-dev/py/pull/207
        monkeypatch.syspath_prepend("")
        monkeypatch.delitem(sys.modules, "pathlib", raising=False)

        testdir.makepyfile(
            **{
                "test_setup_nonexisting_cwd.py": """
                import os
                import shutil
                import tempfile

                d = tempfile.mkdtemp()
                os.chdir(d)
                shutil.rmtree(d)
            """,
                "test_test.py": """
                def test():
                    pass
            """,
            }
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 1 passed in *"])
