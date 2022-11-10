from typing import Any

import pytest
from _pytest import runner
from _pytest._code import getfslineno
from _pytest.fixtures import getfixturemarker
from _pytest.pytester import Pytester
from _pytest.python import Function


class TestOEJSKITSpecials:
    def test_funcarg_non_pycollectobj(
        self, pytester: Pytester, recwarn
    ) -> None:  # rough jstests usage
        pytester.makeconftest(
            """
            import pytest
            def pytest_pycollect_makeitem(collector, name, obj):
                if name == "MyClass":
                    return MyCollector.from_parent(collector, name=name)
            class MyCollector(pytest.Collector):
                def reportinfo(self):
                    return self.path, 3, "xyz"
        """
        )
        modcol = pytester.getmodulecol(
            """
            import pytest
            @pytest.fixture
            def arg1(request):
                return 42
            class MyClass(object):
                pass
        """
        )
        # this hook finds funcarg factories
        rep = runner.collect_one_node(collector=modcol)
        # TODO: Don't treat as Any.
        clscol: Any = rep.result[0]
        clscol.obj = lambda arg1: None
        clscol.funcargs = {}
        pytest._fillfuncargs(clscol)
        assert clscol.funcargs["arg1"] == 42

    def test_autouse_fixture(
        self, pytester: Pytester, recwarn
    ) -> None:  # rough jstests usage
        pytester.makeconftest(
            """
            import pytest
            def pytest_pycollect_makeitem(collector, name, obj):
                if name == "MyClass":
                    return MyCollector.from_parent(collector, name=name)
            class MyCollector(pytest.Collector):
                def reportinfo(self):
                    return self.path, 3, "xyz"
        """
        )
        modcol = pytester.getmodulecol(
            """
            import pytest
            @pytest.fixture(autouse=True)
            def hello():
                pass
            @pytest.fixture
            def arg1(request):
                return 42
            class MyClass(object):
                pass
        """
        )
        # this hook finds funcarg factories
        rep = runner.collect_one_node(modcol)
        # TODO: Don't treat as Any.
        clscol: Any = rep.result[0]
        clscol.obj = lambda: None
        clscol.funcargs = {}
        pytest._fillfuncargs(clscol)
        assert not clscol.funcargs


def test_wrapped_getfslineno() -> None:
    def func():
        pass

    def wrap(f):
        func.__wrapped__ = f  # type: ignore
        func.patchings = ["qwe"]  # type: ignore
        return func

    @wrap
    def wrapped_func(x, y, z):
        pass

    fs, lineno = getfslineno(wrapped_func)
    fs2, lineno2 = getfslineno(wrap)
    assert lineno > lineno2, "getfslineno does not unwrap correctly"


class TestMockDecoration:
    def test_wrapped_getfuncargnames(self) -> None:
        from _pytest.compat import getfuncargnames

        def wrap(f):
            def func():
                pass

            func.__wrapped__ = f  # type: ignore
            return func

        @wrap
        def f(x):
            pass

        values = getfuncargnames(f)
        assert values == ("x",)

    def test_getfuncargnames_patching(self):
        from _pytest.compat import getfuncargnames
        from unittest.mock import patch

        class T:
            def original(self, x, y, z):
                pass

        @patch.object(T, "original")
        def f(x, y, z):
            pass

        values = getfuncargnames(f)
        assert values == ("y", "z")

    def test_unittest_mock(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import unittest.mock
            class T(unittest.TestCase):
                @unittest.mock.patch("os.path.abspath")
                def test_hello(self, abspath):
                    import os
                    os.path.abspath("hello")
                    abspath.assert_any_call("hello")
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_unittest_mock_and_fixture(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import os.path
            import unittest.mock
            import pytest

            @pytest.fixture
            def inject_me():
                pass

            @unittest.mock.patch.object(os.path, "abspath",
                                        new=unittest.mock.MagicMock)
            def test_hello(inject_me):
                import os
                os.path.abspath("hello")
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_unittest_mock_and_pypi_mock(self, pytester: Pytester) -> None:
        pytest.importorskip("mock", "1.0.1")
        pytester.makepyfile(
            """
            import mock
            import unittest.mock
            class TestBoth(object):
                @unittest.mock.patch("os.path.abspath")
                def test_hello(self, abspath):
                    import os
                    os.path.abspath("hello")
                    abspath.assert_any_call("hello")

                @mock.patch("os.path.abspath")
                def test_hello_mock(self, abspath):
                    import os
                    os.path.abspath("hello")
                    abspath.assert_any_call("hello")
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)

    def test_mock_sentinel_check_against_numpy_like(self, pytester: Pytester) -> None:
        """Ensure our function that detects mock arguments compares against sentinels using
        identity to circumvent objects which can't be compared with equality against others
        in a truth context, like with numpy arrays (#5606).
        """
        pytester.makepyfile(
            dummy="""
            class NumpyLike:
                def __init__(self, value):
                    self.value = value
                def __eq__(self, other):
                    raise ValueError("like numpy, cannot compare against others for truth")
            FOO = NumpyLike(10)
        """
        )
        pytester.makepyfile(
            """
            from unittest.mock import patch
            import dummy
            class Test(object):
                @patch("dummy.FOO", new=dummy.NumpyLike(50))
                def test_hello(self):
                    assert dummy.FOO.value == 50
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)

    def test_mock(self, pytester: Pytester) -> None:
        pytest.importorskip("mock", "1.0.1")
        pytester.makepyfile(
            """
            import os
            import unittest
            import mock

            class T(unittest.TestCase):
                @mock.patch("os.path.abspath")
                def test_hello(self, abspath):
                    os.path.abspath("hello")
                    abspath.assert_any_call("hello")
            def mock_basename(path):
                return "mock_basename"
            @mock.patch("os.path.abspath")
            @mock.patch("os.path.normpath")
            @mock.patch("os.path.basename", new=mock_basename)
            def test_someting(normpath, abspath, tmp_path):
                abspath.return_value = "this"
                os.path.normpath(os.path.abspath("hello"))
                normpath.assert_any_call("this")
                assert os.path.basename("123") == "mock_basename"
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=2)
        calls = reprec.getcalls("pytest_runtest_logreport")
        funcnames = [
            call.report.location[2] for call in calls if call.report.when == "call"
        ]
        assert funcnames == ["T.test_hello", "test_someting"]

    def test_mock_sorting(self, pytester: Pytester) -> None:
        pytest.importorskip("mock", "1.0.1")
        pytester.makepyfile(
            """
            import os
            import mock

            @mock.patch("os.path.abspath")
            def test_one(abspath):
                pass
            @mock.patch("os.path.abspath")
            def test_two(abspath):
                pass
            @mock.patch("os.path.abspath")
            def test_three(abspath):
                pass
        """
        )
        reprec = pytester.inline_run()
        calls = reprec.getreports("pytest_runtest_logreport")
        calls = [x for x in calls if x.when == "call"]
        names = [x.nodeid.split("::")[-1] for x in calls]
        assert names == ["test_one", "test_two", "test_three"]

    def test_mock_double_patch_issue473(self, pytester: Pytester) -> None:
        pytest.importorskip("mock", "1.0.1")
        pytester.makepyfile(
            """
            from mock import patch
            from pytest import mark

            @patch('os.getcwd')
            @patch('os.path')
            @mark.slow
            class TestSimple(object):
                def test_simple_thing(self, mock_path, mock_getcwd):
                    pass
        """
        )
        reprec = pytester.inline_run()
        reprec.assertoutcome(passed=1)


class TestReRunTests:
    def test_rerun(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            from _pytest.runner import runtestprotocol
            def pytest_runtest_protocol(item, nextitem):
                runtestprotocol(item, log=False, nextitem=nextitem)
                runtestprotocol(item, log=True, nextitem=nextitem)
        """
        )
        pytester.makepyfile(
            """
            import pytest
            count = 0
            req = None
            @pytest.fixture
            def fix(request):
                global count, req
                assert request != req
                req = request
                print("fix count %s" % count)
                count += 1
            def test_fix(fix):
                pass
        """
        )
        result = pytester.runpytest("-s")
        result.stdout.fnmatch_lines(
            """
            *fix count 0*
            *fix count 1*
        """
        )
        result.stdout.fnmatch_lines(
            """
            *2 passed*
        """
        )


def test_pytestconfig_is_session_scoped() -> None:
    from _pytest.fixtures import pytestconfig

    marker = getfixturemarker(pytestconfig)
    assert marker is not None
    assert marker.scope == "session"


class TestNoselikeTestAttribute:
    def test_module_with_global_test(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            __test__ = False
            def test_hello():
                pass
        """
        )
        reprec = pytester.inline_run()
        assert not reprec.getfailedcollections()
        calls = reprec.getreports("pytest_runtest_logreport")
        assert not calls

    def test_class_and_method(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            __test__ = True
            def test_func():
                pass
            test_func.__test__ = False

            class TestSome(object):
                __test__ = False
                def test_method(self):
                    pass
        """
        )
        reprec = pytester.inline_run()
        assert not reprec.getfailedcollections()
        calls = reprec.getreports("pytest_runtest_logreport")
        assert not calls

    def test_unittest_class(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import unittest
            class TC(unittest.TestCase):
                def test_1(self):
                    pass
            class TC2(unittest.TestCase):
                __test__ = False
                def test_2(self):
                    pass
        """
        )
        reprec = pytester.inline_run()
        assert not reprec.getfailedcollections()
        call = reprec.getcalls("pytest_collection_modifyitems")[0]
        assert len(call.items) == 1
        assert call.items[0].cls.__name__ == "TC"

    def test_class_with_nasty_getattr(self, pytester: Pytester) -> None:
        """Make sure we handle classes with a custom nasty __getattr__ right.

        With a custom __getattr__ which e.g. returns a function (like with a
        RPC wrapper), we shouldn't assume this meant "__test__ = True".
        """
        # https://github.com/pytest-dev/pytest/issues/1204
        pytester.makepyfile(
            """
            class MetaModel(type):

                def __getattr__(cls, key):
                    return lambda: None


            BaseModel = MetaModel('Model', (), {})


            class Model(BaseModel):

                __metaclass__ = MetaModel

                def test_blah(self):
                    pass
        """
        )
        reprec = pytester.inline_run()
        assert not reprec.getfailedcollections()
        call = reprec.getcalls("pytest_collection_modifyitems")[0]
        assert not call.items


class TestParameterize:
    """#351"""

    def test_idfn_marker(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            def idfn(param):
                if param == 0:
                    return 'spam'
                elif param == 1:
                    return 'ham'
                else:
                    return None

            @pytest.mark.parametrize('a,b', [(0, 2), (1, 2)], ids=idfn)
            def test_params(a, b):
                pass
        """
        )
        res = pytester.runpytest("--collect-only")
        res.stdout.fnmatch_lines(["*spam-2*", "*ham-2*"])

    def test_idfn_fixture(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            import pytest

            def idfn(param):
                if param == 0:
                    return 'spam'
                elif param == 1:
                    return 'ham'
                else:
                    return None

            @pytest.fixture(params=[0, 1], ids=idfn)
            def a(request):
                return request.param

            @pytest.fixture(params=[1, 2], ids=idfn)
            def b(request):
                return request.param

            def test_params(a, b):
                pass
        """
        )
        res = pytester.runpytest("--collect-only")
        res.stdout.fnmatch_lines(["*spam-2*", "*ham-2*"])


def test_function_instance(pytester: Pytester) -> None:
    items = pytester.getitems(
        """
        def test_func(): pass
        class TestIt:
            def test_method(self): pass
            @classmethod
            def test_class(cls): pass
            @staticmethod
            def test_static(): pass
        """
    )
    assert len(items) == 3
    assert isinstance(items[0], Function)
    assert items[0].name == "test_func"
    assert items[0].instance is None
    assert isinstance(items[1], Function)
    assert items[1].name == "test_method"
    assert items[1].instance is not None
    assert items[1].instance.__class__.__name__ == "TestIt"
    assert isinstance(items[2], Function)
    assert items[2].name == "test_static"
    assert items[2].instance is None
