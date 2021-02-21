import os
import sys
import types

import attr
import py

import pytest
from _pytest.compat import importlib_metadata
from _pytest.config import ExitCode
from _pytest.pathlib import symlink_or_skip
from _pytest.pytester import Testdir


def prepend_pythonpath(*dirs):
    cur = os.getenv("PYTHONPATH")
    if cur:
        dirs += (cur,)
    return os.pathsep.join(str(p) for p in dirs)


class TestGeneralUsage:
    def test_config_error(self, testdir):
        testdir.copy_example("conftest_usageerror/conftest.py")
        result = testdir.runpytest(testdir.tmpdir)
        assert result.ret == ExitCode.USAGE_ERROR
        result.stderr.fnmatch_lines(["*ERROR: hello"])
        result.stdout.fnmatch_lines(["*pytest_unconfigure_called"])

    def test_root_conftest_syntax_error(self, testdir):
        testdir.makepyfile(conftest="raise SyntaxError\n")
        result = testdir.runpytest()
        result.stderr.fnmatch_lines(["*raise SyntaxError*"])
        assert result.ret != 0

    def test_early_hook_error_issue38_1(self, testdir):
        testdir.makeconftest(
            """
            def pytest_sessionstart():
                0 / 0
        """
        )
        result = testdir.runpytest(testdir.tmpdir)
        assert result.ret != 0
        # tracestyle is native by default for hook failures
        result.stdout.fnmatch_lines(
            ["*INTERNALERROR*File*conftest.py*line 2*", "*0 / 0*"]
        )
        result = testdir.runpytest(testdir.tmpdir, "--fulltrace")
        assert result.ret != 0
        # tracestyle is native by default for hook failures
        result.stdout.fnmatch_lines(
            ["*INTERNALERROR*def pytest_sessionstart():*", "*INTERNALERROR*0 / 0*"]
        )

    def test_early_hook_configure_error_issue38(self, testdir):
        testdir.makeconftest(
            """
            def pytest_configure():
                0 / 0
        """
        )
        result = testdir.runpytest(testdir.tmpdir)
        assert result.ret != 0
        # here we get it on stderr
        result.stderr.fnmatch_lines(
            ["*INTERNALERROR*File*conftest.py*line 2*", "*0 / 0*"]
        )

    def test_file_not_found(self, testdir):
        result = testdir.runpytest("asd")
        assert result.ret != 0
        result.stderr.fnmatch_lines(["ERROR: file or directory not found: asd"])

    def test_file_not_found_unconfigure_issue143(self, testdir):
        testdir.makeconftest(
            """
            def pytest_configure():
                print("---configure")
            def pytest_unconfigure():
                print("---unconfigure")
        """
        )
        result = testdir.runpytest("-s", "asd")
        assert result.ret == ExitCode.USAGE_ERROR
        result.stderr.fnmatch_lines(["ERROR: file or directory not found: asd"])
        result.stdout.fnmatch_lines(["*---configure", "*---unconfigure"])

    def test_config_preparse_plugin_option(self, testdir):
        testdir.makepyfile(
            pytest_xyz="""
            def pytest_addoption(parser):
                parser.addoption("--xyz", dest="xyz", action="store")
        """
        )
        testdir.makepyfile(
            test_one="""
            def test_option(pytestconfig):
                assert pytestconfig.option.xyz == "123"
        """
        )
        result = testdir.runpytest("-p", "pytest_xyz", "--xyz=123", syspathinsert=True)
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

    @pytest.mark.parametrize("load_cov_early", [True, False])
    def test_early_load_setuptools_name(self, testdir, monkeypatch, load_cov_early):
        monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD")

        testdir.makepyfile(mytestplugin1_module="")
        testdir.makepyfile(mytestplugin2_module="")
        testdir.makepyfile(mycov_module="")
        testdir.syspathinsert()

        loaded = []

        @attr.s
        class DummyEntryPoint:
            name = attr.ib()
            module = attr.ib()
            group = "pytest11"

            def load(self):
                __import__(self.module)
                loaded.append(self.name)
                return sys.modules[self.module]

        entry_points = [
            DummyEntryPoint("myplugin1", "mytestplugin1_module"),
            DummyEntryPoint("myplugin2", "mytestplugin2_module"),
            DummyEntryPoint("mycov", "mycov_module"),
        ]

        @attr.s
        class DummyDist:
            entry_points = attr.ib()
            files = ()

        def my_dists():
            return (DummyDist(entry_points),)

        monkeypatch.setattr(importlib_metadata, "distributions", my_dists)
        params = ("-p", "mycov") if load_cov_early else ()
        testdir.runpytest_inprocess(*params)
        if load_cov_early:
            assert loaded == ["mycov", "myplugin1", "myplugin2"]
        else:
            assert loaded == ["myplugin1", "myplugin2", "mycov"]

    @pytest.mark.parametrize("import_mode", ["prepend", "append", "importlib"])
    def test_assertion_rewrite(self, testdir, import_mode):
        p = testdir.makepyfile(
            """
            def test_this():
                x = 0
                assert x
        """
        )
        result = testdir.runpytest(p, "--import-mode={}".format(import_mode))
        result.stdout.fnmatch_lines([">       assert x", "E       assert 0"])
        assert result.ret == 1

    def test_nested_import_error(self, testdir):
        p = testdir.makepyfile(
            """
                import import_fails
                def test_this():
                    assert import_fails.a == 1
        """
        )
        testdir.makepyfile(import_fails="import does_not_work")
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(
            [
                "ImportError while importing test module*",
                "*No module named *does_not_work*",
            ]
        )
        assert result.ret == 2

    def test_not_collectable_arguments(self, testdir):
        p1 = testdir.makepyfile("")
        p2 = testdir.makefile(".pyc", "123")
        result = testdir.runpytest(p1, p2)
        assert result.ret == ExitCode.USAGE_ERROR
        result.stderr.fnmatch_lines(
            [
                "ERROR: not found: {}".format(p2),
                "(no name {!r} in any of [[][]])".format(str(p2)),
                "",
            ]
        )

    @pytest.mark.filterwarnings("default")
    def test_better_reporting_on_conftest_load_failure(self, testdir):
        """Show a user-friendly traceback on conftest import failures (#486, #3332)"""
        testdir.makepyfile("")
        conftest = testdir.makeconftest(
            """
            def foo():
                import qwerty
            foo()
        """
        )
        result = testdir.runpytest("--help")
        result.stdout.fnmatch_lines(
            """
            *--version*
            *warning*conftest.py*
        """
        )
        result = testdir.runpytest()
        exc_name = (
            "ModuleNotFoundError" if sys.version_info >= (3, 6) else "ImportError"
        )
        assert result.stdout.lines == []
        assert result.stderr.lines == [
            "ImportError while loading conftest '{}'.".format(conftest),
            "conftest.py:3: in <module>",
            "    foo()",
            "conftest.py:2: in foo",
            "    import qwerty",
            "E   {}: No module named 'qwerty'".format(exc_name),
        ]

    def test_early_skip(self, testdir):
        testdir.mkdir("xyz")
        testdir.makeconftest(
            """
            import pytest
            def pytest_collect_file():
                pytest.skip("early")
        """
        )
        result = testdir.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.fnmatch_lines(["*1 skip*"])

    def test_issue88_initial_file_multinodes(self, testdir):
        testdir.copy_example("issue88_initial_file_multinodes")
        p = testdir.makepyfile("def test_hello(): pass")
        result = testdir.runpytest(p, "--collect-only")
        result.stdout.fnmatch_lines(["*MyFile*test_issue88*", "*Module*test_issue88*"])

    def test_issue93_initialnode_importing_capturing(self, testdir):
        testdir.makeconftest(
            """
            import sys
            print("should not be seen")
            sys.stderr.write("stder42\\n")
        """
        )
        result = testdir.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.no_fnmatch_line("*should not be seen*")
        assert "stderr42" not in result.stderr.str()

    def test_conftest_printing_shows_if_error(self, testdir):
        testdir.makeconftest(
            """
            print("should be seen")
            assert 0
        """
        )
        result = testdir.runpytest()
        assert result.ret != 0
        assert "should be seen" in result.stdout.str()

    def test_issue109_sibling_conftests_not_loaded(self, testdir):
        sub1 = testdir.mkdir("sub1")
        sub2 = testdir.mkdir("sub2")
        sub1.join("conftest.py").write("assert 0")
        result = testdir.runpytest(sub2)
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        sub2.ensure("__init__.py")
        p = sub2.ensure("test_hello.py")
        result = testdir.runpytest(p)
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result = testdir.runpytest(sub1)
        assert result.ret == ExitCode.USAGE_ERROR

    def test_directory_skipped(self, testdir):
        testdir.makeconftest(
            """
            import pytest
            def pytest_ignore_collect():
                pytest.skip("intentional")
        """
        )
        testdir.makepyfile("def test_hello(): pass")
        result = testdir.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.fnmatch_lines(["*1 skipped*"])

    def test_multiple_items_per_collector_byid(self, testdir):
        c = testdir.makeconftest(
            """
            import pytest
            class MyItem(pytest.Item):
                def runtest(self):
                    pass
            class MyCollector(pytest.File):
                def collect(self):
                    return [MyItem.from_parent(name="xyz", parent=self)]
            def pytest_collect_file(path, parent):
                if path.basename.startswith("conftest"):
                    return MyCollector.from_parent(fspath=path, parent=parent)
        """
        )
        result = testdir.runpytest(c.basename + "::" + "xyz")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 pass*"])

    def test_skip_on_generated_funcarg_id(self, testdir):
        testdir.makeconftest(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('x', [3], ids=['hello-123'])
            def pytest_runtest_setup(item):
                print(item.keywords)
                if 'hello-123' in item.keywords:
                    pytest.skip("hello")
                assert 0
        """
        )
        p = testdir.makepyfile("""def test_func(x): pass""")
        res = testdir.runpytest(p)
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 skipped*"])

    def test_direct_addressing_selects(self, testdir):
        p = testdir.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('i', [1, 2], ids=["1", "2"])
            def test_func(i):
                pass
        """
        )
        res = testdir.runpytest(p.basename + "::" + "test_func[1]")
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 passed*"])

    def test_direct_addressing_notfound(self, testdir):
        p = testdir.makepyfile(
            """
            def test_func():
                pass
        """
        )
        res = testdir.runpytest(p.basename + "::" + "test_notfound")
        assert res.ret
        res.stderr.fnmatch_lines(["*ERROR*not found*"])

    def test_docstring_on_hookspec(self):
        from _pytest import hookspec

        for name, value in vars(hookspec).items():
            if name.startswith("pytest_"):
                assert value.__doc__, "no docstring for %s" % name

    def test_initialization_error_issue49(self, testdir):
        testdir.makeconftest(
            """
            def pytest_configure():
                x
        """
        )
        result = testdir.runpytest()
        assert result.ret == 3  # internal error
        result.stderr.fnmatch_lines(["INTERNAL*pytest_configure*", "INTERNAL*x*"])
        assert "sessionstarttime" not in result.stderr.str()

    @pytest.mark.parametrize("lookfor", ["test_fun.py::test_a"])
    def test_issue134_report_error_when_collecting_member(self, testdir, lookfor):
        testdir.makepyfile(
            test_fun="""
            def test_a():
                pass
            def"""
        )
        result = testdir.runpytest(lookfor)
        result.stdout.fnmatch_lines(["*SyntaxError*"])
        if "::" in lookfor:
            result.stderr.fnmatch_lines(["*ERROR*"])
            assert result.ret == 4  # usage error only if item not found

    def test_report_all_failed_collections_initargs(self, testdir):
        testdir.makeconftest(
            """
            from _pytest.config import ExitCode

            def pytest_sessionfinish(exitstatus):
                assert exitstatus == ExitCode.USAGE_ERROR
                print("pytest_sessionfinish_called")
            """
        )
        testdir.makepyfile(test_a="def", test_b="def")
        result = testdir.runpytest("test_a.py::a", "test_b.py::b")
        result.stderr.fnmatch_lines(["*ERROR*test_a.py::a*", "*ERROR*test_b.py::b*"])
        result.stdout.fnmatch_lines(["pytest_sessionfinish_called"])
        assert result.ret == ExitCode.USAGE_ERROR

    def test_namespace_import_doesnt_confuse_import_hook(self, testdir):
        """Ref #383.

        Python 3.3's namespace package messed with our import hooks.
        Importing a module that didn't exist, even if the ImportError was
        gracefully handled, would make our test crash.
        """
        testdir.mkdir("not_a_package")
        p = testdir.makepyfile(
            """
            try:
                from not_a_package import doesnt_exist
            except ImportError:
                # We handle the import error gracefully here
                pass

            def test_whatever():
                pass
        """
        )
        res = testdir.runpytest(p.basename)
        assert res.ret == 0

    def test_unknown_option(self, testdir):
        result = testdir.runpytest("--qwlkej")
        result.stderr.fnmatch_lines(
            """
            *unrecognized*
        """
        )

    def test_getsourcelines_error_issue553(self, testdir, monkeypatch):
        monkeypatch.setattr("inspect.getsourcelines", None)
        p = testdir.makepyfile(
            """
            def raise_error(obj):
                raise OSError('source code not available')

            import inspect
            inspect.getsourcelines = raise_error

            def test_foo(invalid_fixture):
                pass
        """
        )
        res = testdir.runpytest(p)
        res.stdout.fnmatch_lines(
            ["*source code not available*", "E*fixture 'invalid_fixture' not found"]
        )

    def test_plugins_given_as_strings(self, tmpdir, monkeypatch, _sys_snapshot):
        """Test that str values passed to main() as `plugins` arg are
        interpreted as module names to be imported and registered (#855)."""
        with pytest.raises(ImportError) as excinfo:
            pytest.main([str(tmpdir)], plugins=["invalid.module"])
        assert "invalid" in str(excinfo.value)

        p = tmpdir.join("test_test_plugins_given_as_strings.py")
        p.write("def test_foo(): pass")
        mod = types.ModuleType("myplugin")
        monkeypatch.setitem(sys.modules, "myplugin", mod)
        assert pytest.main(args=[str(tmpdir)], plugins=["myplugin"]) == 0

    def test_parametrized_with_bytes_regex(self, testdir):
        p = testdir.makepyfile(
            """
            import re
            import pytest
            @pytest.mark.parametrize('r', [re.compile(b'foo')])
            def test_stuff(r):
                pass
        """
        )
        res = testdir.runpytest(p)
        res.stdout.fnmatch_lines(["*1 passed*"])

    def test_parametrized_with_null_bytes(self, testdir):
        """Test parametrization with values that contain null bytes and unicode characters (#2644, #2957)"""
        p = testdir.makepyfile(
            """\
            import pytest

            @pytest.mark.parametrize("data", [b"\\x00", "\\x00", 'ação'])
            def test_foo(data):
                assert data
            """
        )
        res = testdir.runpytest(p)
        res.assert_outcomes(passed=3)


class TestInvocationVariants:
    def test_earlyinit(self, testdir):
        p = testdir.makepyfile(
            """
            import pytest
            assert hasattr(pytest, 'mark')
        """
        )
        result = testdir.runpython(p)
        assert result.ret == 0

    def test_pydoc(self, testdir):
        for name in ("py.test", "pytest"):
            result = testdir.runpython_c("import {};help({})".format(name, name))
            assert result.ret == 0
            s = result.stdout.str()
            assert "MarkGenerator" in s

    def test_import_star_py_dot_test(self, testdir):
        p = testdir.makepyfile(
            """
            from py.test import *
            #collect
            #cmdline
            #Item
            # assert collect.Item is Item
            # assert collect.Collector is Collector
            main
            skip
            xfail
        """
        )
        result = testdir.runpython(p)
        assert result.ret == 0

    def test_import_star_pytest(self, testdir):
        p = testdir.makepyfile(
            """
            from pytest import *
            #Item
            #File
            main
            skip
            xfail
        """
        )
        result = testdir.runpython(p)
        assert result.ret == 0

    def test_double_pytestcmdline(self, testdir):
        p = testdir.makepyfile(
            run="""
            import pytest
            pytest.main()
            pytest.main()
        """
        )
        testdir.makepyfile(
            """
            def test_hello():
                pass
        """
        )
        result = testdir.runpython(p)
        result.stdout.fnmatch_lines(["*1 passed*", "*1 passed*"])

    def test_python_minus_m_invocation_ok(self, testdir):
        p1 = testdir.makepyfile("def test_hello(): pass")
        res = testdir.run(sys.executable, "-m", "pytest", str(p1))
        assert res.ret == 0

    def test_python_minus_m_invocation_fail(self, testdir):
        p1 = testdir.makepyfile("def test_fail(): 0/0")
        res = testdir.run(sys.executable, "-m", "pytest", str(p1))
        assert res.ret == 1

    def test_python_pytest_package(self, testdir):
        p1 = testdir.makepyfile("def test_pass(): pass")
        res = testdir.run(sys.executable, "-m", "pytest", str(p1))
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 passed*"])

    def test_equivalence_pytest_pydottest(self) -> None:
        # Type ignored because `py.test` is not and will not be typed.
        assert pytest.main == py.test.cmdline.main  # type: ignore[attr-defined]

    def test_invoke_with_invalid_type(self) -> None:
        with pytest.raises(
            TypeError, match="expected to be a list of strings, got: '-h'"
        ):
            pytest.main("-h")  # type: ignore[arg-type]

    def test_invoke_with_path(self, tmpdir: py.path.local, capsys) -> None:
        retcode = pytest.main(tmpdir)
        assert retcode == ExitCode.NO_TESTS_COLLECTED
        out, err = capsys.readouterr()

    def test_invoke_plugin_api(self, capsys):
        class MyPlugin:
            def pytest_addoption(self, parser):
                parser.addoption("--myopt")

        pytest.main(["-h"], plugins=[MyPlugin()])
        out, err = capsys.readouterr()
        assert "--myopt" in out

    def test_pyargs_importerror(self, testdir, monkeypatch):
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", False)
        path = testdir.mkpydir("tpkg")
        path.join("test_hello.py").write("raise ImportError")

        result = testdir.runpytest("--pyargs", "tpkg.test_hello", syspathinsert=True)
        assert result.ret != 0

        result.stdout.fnmatch_lines(["collected*0*items*/*1*error"])

    def test_pyargs_only_imported_once(self, testdir):
        pkg = testdir.mkpydir("foo")
        pkg.join("test_foo.py").write("print('hello from test_foo')\ndef test(): pass")
        pkg.join("conftest.py").write(
            "def pytest_configure(config): print('configuring')"
        )

        result = testdir.runpytest("--pyargs", "foo.test_foo", "-s", syspathinsert=True)
        # should only import once
        assert result.outlines.count("hello from test_foo") == 1
        # should only configure once
        assert result.outlines.count("configuring") == 1

    def test_pyargs_filename_looks_like_module(self, testdir):
        testdir.tmpdir.join("conftest.py").ensure()
        testdir.tmpdir.join("t.py").write("def test(): pass")
        result = testdir.runpytest("--pyargs", "t.py")
        assert result.ret == ExitCode.OK

    def test_cmdline_python_package(self, testdir, monkeypatch):
        import warnings

        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", False)
        path = testdir.mkpydir("tpkg")
        path.join("test_hello.py").write("def test_hello(): pass")
        path.join("test_world.py").write("def test_world(): pass")
        result = testdir.runpytest("--pyargs", "tpkg")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*2 passed*"])
        result = testdir.runpytest("--pyargs", "tpkg.test_hello", syspathinsert=True)
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

        empty_package = testdir.mkpydir("empty_package")
        monkeypatch.setenv("PYTHONPATH", str(empty_package), prepend=os.pathsep)
        # the path which is not a package raises a warning on pypy;
        # no idea why only pypy and not normal python warn about it here
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", ImportWarning)
            result = testdir.runpytest("--pyargs", ".")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*2 passed*"])

        monkeypatch.setenv("PYTHONPATH", str(testdir), prepend=os.pathsep)
        result = testdir.runpytest("--pyargs", "tpkg.test_missing", syspathinsert=True)
        assert result.ret != 0
        result.stderr.fnmatch_lines(["*not*found*test_missing*"])

    def test_cmdline_python_namespace_package(self, testdir, monkeypatch):
        """Test --pyargs option with namespace packages (#1567).

        Ref: https://packaging.python.org/guides/packaging-namespace-packages/
        """
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", raising=False)

        search_path = []
        for dirname in "hello", "world":
            d = testdir.mkdir(dirname)
            search_path.append(d)
            ns = d.mkdir("ns_pkg")
            ns.join("__init__.py").write(
                "__import__('pkg_resources').declare_namespace(__name__)"
            )
            lib = ns.mkdir(dirname)
            lib.ensure("__init__.py")
            lib.join("test_{}.py".format(dirname)).write(
                "def test_{}(): pass\ndef test_other():pass".format(dirname)
            )

        # The structure of the test directory is now:
        # .
        # ├── hello
        # │   └── ns_pkg
        # │       ├── __init__.py
        # │       └── hello
        # │           ├── __init__.py
        # │           └── test_hello.py
        # └── world
        #     └── ns_pkg
        #         ├── __init__.py
        #         └── world
        #             ├── __init__.py
        #             └── test_world.py

        # NOTE: the different/reversed ordering is intentional here.
        monkeypatch.setenv("PYTHONPATH", prepend_pythonpath(*search_path))
        for p in search_path:
            monkeypatch.syspath_prepend(p)

        # mixed module and filenames:
        monkeypatch.chdir("world")
        result = testdir.runpytest("--pyargs", "-v", "ns_pkg.hello", "ns_pkg/world")
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            [
                "test_hello.py::test_hello*PASSED*",
                "test_hello.py::test_other*PASSED*",
                "ns_pkg/world/test_world.py::test_world*PASSED*",
                "ns_pkg/world/test_world.py::test_other*PASSED*",
                "*4 passed in*",
            ]
        )

        # specify tests within a module
        testdir.chdir()
        result = testdir.runpytest(
            "--pyargs", "-v", "ns_pkg.world.test_world::test_other"
        )
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            ["*test_world.py::test_other*PASSED*", "*1 passed*"]
        )

    def test_invoke_test_and_doctestmodules(self, testdir):
        p = testdir.makepyfile(
            """
            def test():
                pass
        """
        )
        result = testdir.runpytest(str(p) + "::test", "--doctest-modules")
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_cmdline_python_package_symlink(self, testdir, monkeypatch):
        """
        --pyargs with packages with path containing symlink can have conftest.py in
        their package (#2985)
        """
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", raising=False)

        dirname = "lib"
        d = testdir.mkdir(dirname)
        foo = d.mkdir("foo")
        foo.ensure("__init__.py")
        lib = foo.mkdir("bar")
        lib.ensure("__init__.py")
        lib.join("test_bar.py").write(
            "def test_bar(): pass\ndef test_other(a_fixture):pass"
        )
        lib.join("conftest.py").write(
            "import pytest\n@pytest.fixture\ndef a_fixture():pass"
        )

        d_local = testdir.mkdir("symlink_root")
        symlink_location = d_local / "lib"
        symlink_or_skip(d, symlink_location, target_is_directory=True)

        # The structure of the test directory is now:
        # .
        # ├── symlink_root
        # │   └── lib -> ../lib
        # └── lib
        #     └── foo
        #         ├── __init__.py
        #         └── bar
        #             ├── __init__.py
        #             ├── conftest.py
        #             └── test_bar.py

        # NOTE: the different/reversed ordering is intentional here.
        search_path = ["lib", os.path.join("symlink_root", "lib")]
        monkeypatch.setenv("PYTHONPATH", prepend_pythonpath(*search_path))
        for p in search_path:
            monkeypatch.syspath_prepend(p)

        # module picked up in symlink-ed directory:
        # It picks up symlink_root/lib/foo/bar (symlink) via sys.path.
        result = testdir.runpytest("--pyargs", "-v", "foo.bar")
        testdir.chdir()
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            [
                "symlink_root/lib/foo/bar/test_bar.py::test_bar PASSED*",
                "symlink_root/lib/foo/bar/test_bar.py::test_other PASSED*",
                "*2 passed*",
            ]
        )

    def test_cmdline_python_package_not_exists(self, testdir):
        result = testdir.runpytest("--pyargs", "tpkgwhatv")
        assert result.ret
        result.stderr.fnmatch_lines(["ERROR*module*or*package*not*found*"])

    @pytest.mark.xfail(reason="decide: feature or bug")
    def test_noclass_discovery_if_not_testcase(self, testdir):
        testpath = testdir.makepyfile(
            """
            import unittest
            class TestHello(object):
                def test_hello(self):
                    assert self.attr

            class RealTest(unittest.TestCase, TestHello):
                attr = 42
        """
        )
        reprec = testdir.inline_run(testpath)
        reprec.assertoutcome(passed=1)

    def test_doctest_id(self, testdir):
        testdir.makefile(
            ".txt",
            """
            >>> x=3
            >>> x
            4
        """,
        )
        testid = "test_doctest_id.txt::test_doctest_id.txt"
        expected_lines = [
            "*= FAILURES =*",
            "*_ ?doctest? test_doctest_id.txt _*",
            "FAILED test_doctest_id.txt::test_doctest_id.txt",
            "*= 1 failed in*",
        ]
        result = testdir.runpytest(testid, "-rf", "--tb=short")
        result.stdout.fnmatch_lines(expected_lines)

        # Ensure that re-running it will still handle it as
        # doctest.DocTestFailure, which was not the case before when
        # re-importing doctest, but not creating a new RUNNER_CLASS.
        result = testdir.runpytest(testid, "-rf", "--tb=short")
        result.stdout.fnmatch_lines(expected_lines)

    def test_core_backward_compatibility(self):
        """Test backward compatibility for get_plugin_manager function. See #787."""
        import _pytest.config

        assert (
            type(_pytest.config.get_plugin_manager())
            is _pytest.config.PytestPluginManager
        )

    def test_has_plugin(self, request):
        """Test hasplugin function of the plugin manager (#932)."""
        assert request.config.pluginmanager.hasplugin("python")


class TestDurations:
    source = """
        from _pytest import timing
        def test_something():
            pass
        def test_2():
            timing.sleep(0.010)
        def test_1():
            timing.sleep(0.002)
        def test_3():
            timing.sleep(0.020)
    """

    def test_calls(self, testdir, mock_timing):
        testdir.makepyfile(self.source)
        result = testdir.runpytest_inprocess("--durations=10")
        assert result.ret == 0

        result.stdout.fnmatch_lines_random(
            ["*durations*", "*call*test_3*", "*call*test_2*"]
        )

        result.stdout.fnmatch_lines(
            ["(8 durations < 0.005s hidden.  Use -vv to show these durations.)"]
        )

    def test_calls_show_2(self, testdir, mock_timing):

        testdir.makepyfile(self.source)
        result = testdir.runpytest_inprocess("--durations=2")
        assert result.ret == 0

        lines = result.stdout.get_lines_after("*slowest*durations*")
        assert "4 passed" in lines[2]

    def test_calls_showall(self, testdir, mock_timing):
        testdir.makepyfile(self.source)
        result = testdir.runpytest_inprocess("--durations=0")
        assert result.ret == 0

        tested = "3"
        for x in tested:
            for y in ("call",):  # 'setup', 'call', 'teardown':
                for line in result.stdout.lines:
                    if ("test_%s" % x) in line and y in line:
                        break
                else:
                    raise AssertionError("not found {} {}".format(x, y))

    def test_calls_showall_verbose(self, testdir, mock_timing):
        testdir.makepyfile(self.source)
        result = testdir.runpytest_inprocess("--durations=0", "-vv")
        assert result.ret == 0

        for x in "123":
            for y in ("call",):  # 'setup', 'call', 'teardown':
                for line in result.stdout.lines:
                    if ("test_%s" % x) in line and y in line:
                        break
                else:
                    raise AssertionError("not found {} {}".format(x, y))

    def test_with_deselected(self, testdir, mock_timing):
        testdir.makepyfile(self.source)
        result = testdir.runpytest_inprocess("--durations=2", "-k test_3")
        assert result.ret == 0

        result.stdout.fnmatch_lines(["*durations*", "*call*test_3*"])

    def test_with_failing_collection(self, testdir, mock_timing):
        testdir.makepyfile(self.source)
        testdir.makepyfile(test_collecterror="""xyz""")
        result = testdir.runpytest_inprocess("--durations=2", "-k test_1")
        assert result.ret == 2

        result.stdout.fnmatch_lines(["*Interrupted: 1 error during collection*"])
        # Collection errors abort test execution, therefore no duration is
        # output
        result.stdout.no_fnmatch_line("*duration*")

    def test_with_not(self, testdir, mock_timing):
        testdir.makepyfile(self.source)
        result = testdir.runpytest_inprocess("-k not 1")
        assert result.ret == 0


class TestDurationsWithFixture:
    source = """
        import pytest
        from _pytest import timing

        @pytest.fixture
        def setup_fixt():
            timing.sleep(2)

        def test_1(setup_fixt):
            timing.sleep(5)
    """

    def test_setup_function(self, testdir, mock_timing):
        testdir.makepyfile(self.source)
        result = testdir.runpytest_inprocess("--durations=10")
        assert result.ret == 0

        result.stdout.fnmatch_lines_random(
            """
            *durations*
            5.00s call *test_1*
            2.00s setup *test_1*
        """
        )


def test_zipimport_hook(testdir, tmpdir):
    """Test package loader is being used correctly (see #1837)."""
    zipapp = pytest.importorskip("zipapp")
    testdir.tmpdir.join("app").ensure(dir=1)
    testdir.makepyfile(
        **{
            "app/foo.py": """
            import pytest
            def main():
                pytest.main(['--pyargs', 'foo'])
        """
        }
    )
    target = tmpdir.join("foo.zip")
    zipapp.create_archive(str(testdir.tmpdir.join("app")), str(target), main="foo:main")
    result = testdir.runpython(target)
    assert result.ret == 0
    result.stderr.fnmatch_lines(["*not found*foo*"])
    result.stdout.no_fnmatch_line("*INTERNALERROR>*")


def test_import_plugin_unicode_name(testdir):
    testdir.makepyfile(myplugin="")
    testdir.makepyfile("def test(): pass")
    testdir.makeconftest("pytest_plugins = ['myplugin']")
    r = testdir.runpytest()
    assert r.ret == 0


def test_pytest_plugins_as_module(testdir):
    """Do not raise an error if pytest_plugins attribute is a module (#3899)"""
    testdir.makepyfile(
        **{
            "__init__.py": "",
            "pytest_plugins.py": "",
            "conftest.py": "from . import pytest_plugins",
            "test_foo.py": "def test(): pass",
        }
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["* 1 passed in *"])


def test_deferred_hook_checking(testdir):
    """Check hooks as late as possible (#1821)."""
    testdir.syspathinsert()
    testdir.makepyfile(
        **{
            "plugin.py": """
        class Hooks(object):
            def pytest_my_hook(self, config):
                pass

        def pytest_configure(config):
            config.pluginmanager.add_hookspecs(Hooks)
        """,
            "conftest.py": """
            pytest_plugins = ['plugin']
            def pytest_my_hook(config):
                return 40
        """,
            "test_foo.py": """
            def test(request):
                assert request.config.hook.pytest_my_hook(config=request.config) == [40]
        """,
        }
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["* 1 passed *"])


def test_fixture_values_leak(testdir):
    """Ensure that fixture objects are properly destroyed by the garbage collector at the end of their expected
    life-times (#2981).
    """
    testdir.makepyfile(
        """
        import attr
        import gc
        import pytest
        import weakref

        @attr.s
        class SomeObj(object):
            name = attr.ib()

        fix_of_test1_ref = None
        session_ref = None

        @pytest.fixture(scope='session')
        def session_fix():
            global session_ref
            obj = SomeObj(name='session-fixture')
            session_ref = weakref.ref(obj)
            return obj

        @pytest.fixture
        def fix(session_fix):
            global fix_of_test1_ref
            obj = SomeObj(name='local-fixture')
            fix_of_test1_ref = weakref.ref(obj)
            return obj

        def test1(fix):
            assert fix_of_test1_ref() is fix

        def test2():
            gc.collect()
            # fixture "fix" created during test1 must have been destroyed by now
            assert fix_of_test1_ref() is None
    """
    )
    # Running on subprocess does not activate the HookRecorder
    # which holds itself a reference to objects in case of the
    # pytest_assert_reprcompare hook
    result = testdir.runpytest_subprocess()
    result.stdout.fnmatch_lines(["* 2 passed *"])


def test_fixture_order_respects_scope(testdir):
    """Ensure that fixtures are created according to scope order (#2405)."""
    testdir.makepyfile(
        """
        import pytest

        data = {}

        @pytest.fixture(scope='module')
        def clean_data():
            data.clear()

        @pytest.fixture(autouse=True)
        def add_data():
            data.update(value=True)

        @pytest.mark.usefixtures('clean_data')
        def test_value():
            assert data.get('value')
    """
    )
    result = testdir.runpytest()
    assert result.ret == 0


def test_frame_leak_on_failing_test(testdir):
    """Pytest would leak garbage referencing the frames of tests that failed
    that could never be reclaimed (#2798).

    Unfortunately it was not possible to remove the actual circles because most of them
    are made of traceback objects which cannot be weakly referenced. Those objects at least
    can be eventually claimed by the garbage collector.
    """
    testdir.makepyfile(
        """
        import gc
        import weakref

        class Obj:
            pass

        ref = None

        def test1():
            obj = Obj()
            global ref
            ref = weakref.ref(obj)
            assert 0

        def test2():
            gc.collect()
            assert ref() is None
    """
    )
    result = testdir.runpytest_subprocess()
    result.stdout.fnmatch_lines(["*1 failed, 1 passed in*"])


def test_fixture_mock_integration(testdir):
    """Test that decorators applied to fixture are left working (#3774)"""
    p = testdir.copy_example("acceptance/fixture_mock_integration.py")
    result = testdir.runpytest(p)
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_usage_error_code(testdir):
    result = testdir.runpytest("-unknown-option-")
    assert result.ret == ExitCode.USAGE_ERROR


@pytest.mark.filterwarnings("default")
def test_warn_on_async_function(testdir):
    # In the below we .close() the coroutine only to avoid
    # "RuntimeWarning: coroutine 'test_2' was never awaited"
    # which messes with other tests.
    testdir.makepyfile(
        test_async="""
        async def test_1():
            pass
        async def test_2():
            pass
        def test_3():
            coro = test_2()
            coro.close()
            return coro
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "test_async.py::test_1",
            "test_async.py::test_2",
            "test_async.py::test_3",
            "*async def functions are not natively supported*",
            "*3 skipped, 3 warnings in*",
        ]
    )
    # ensure our warning message appears only once
    assert (
        result.stdout.str().count("async def functions are not natively supported") == 1
    )


@pytest.mark.filterwarnings("default")
@pytest.mark.skipif(
    sys.version_info < (3, 6), reason="async gen syntax available in Python 3.6+"
)
def test_warn_on_async_gen_function(testdir):
    testdir.makepyfile(
        test_async="""
        async def test_1():
            yield
        async def test_2():
            yield
        def test_3():
            return test_2()
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "test_async.py::test_1",
            "test_async.py::test_2",
            "test_async.py::test_3",
            "*async def functions are not natively supported*",
            "*3 skipped, 3 warnings in*",
        ]
    )
    # ensure our warning message appears only once
    assert (
        result.stdout.str().count("async def functions are not natively supported") == 1
    )


def test_pdb_can_be_rewritten(testdir):
    testdir.makepyfile(
        **{
            "conftest.py": """
                import pytest
                pytest.register_assert_rewrite("pdb")
                """,
            "__init__.py": "",
            "pdb.py": """
                def check():
                    assert 1 == 2
                """,
            "test_pdb.py": """
                def test():
                    import pdb
                    assert pdb.check()
                """,
        }
    )
    # Disable debugging plugin itself to avoid:
    # > INTERNALERROR> AttributeError: module 'pdb' has no attribute 'set_trace'
    result = testdir.runpytest_subprocess("-p", "no:debugging", "-vv")
    result.stdout.fnmatch_lines(
        [
            "    def check():",
            ">       assert 1 == 2",
            "E       assert 1 == 2",
            "E         +1",
            "E         -2",
            "",
            "pdb.py:2: AssertionError",
            "*= 1 failed in *",
        ]
    )
    assert result.ret == 1


def test_tee_stdio_captures_and_live_prints(testdir):
    testpath = testdir.makepyfile(
        """
        import sys
        def test_simple():
            print ("@this is stdout@")
            print ("@this is stderr@", file=sys.stderr)
    """
    )
    result = testdir.runpytest_subprocess(
        testpath,
        "--capture=tee-sys",
        "--junitxml=output.xml",
        "-o",
        "junit_logging=all",
    )

    # ensure stdout/stderr were 'live printed'
    result.stdout.fnmatch_lines(["*@this is stdout@*"])
    result.stderr.fnmatch_lines(["*@this is stderr@*"])

    # now ensure the output is in the junitxml
    with open(os.path.join(testdir.tmpdir.strpath, "output.xml")) as f:
        fullXml = f.read()
    assert "@this is stdout@\n" in fullXml
    assert "@this is stderr@\n" in fullXml


@pytest.mark.skipif(
    sys.platform == "win32",
    reason="Windows raises `OSError: [Errno 22] Invalid argument` instead",
)
def test_no_brokenpipeerror_message(testdir: Testdir) -> None:
    """Ensure that the broken pipe error message is supressed.

    In some Python versions, it reaches sys.unraisablehook, in others
    a BrokenPipeError exception is propagated, but either way it prints
    to stderr on shutdown, so checking nothing is printed is enough.
    """
    popen = testdir.popen((*testdir._getpytestargs(), "--help"))
    popen.stdout.close()
    ret = popen.wait()
    assert popen.stderr.read() == b""
    assert ret == 1
