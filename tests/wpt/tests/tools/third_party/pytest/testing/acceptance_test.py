# mypy: allow-untyped-defs
import dataclasses
import importlib.metadata
import os
from pathlib import Path
import subprocess
import sys
import types

from _pytest.config import ExitCode
from _pytest.pathlib import symlink_or_skip
from _pytest.pytester import Pytester
import pytest


def prepend_pythonpath(*dirs) -> str:
    cur = os.getenv("PYTHONPATH")
    if cur:
        dirs += (cur,)
    return os.pathsep.join(str(p) for p in dirs)


class TestGeneralUsage:
    def test_config_error(self, pytester: Pytester) -> None:
        pytester.copy_example("conftest_usageerror/conftest.py")
        result = pytester.runpytest(pytester.path)
        assert result.ret == ExitCode.USAGE_ERROR
        result.stderr.fnmatch_lines(["*ERROR: hello"])
        result.stdout.fnmatch_lines(["*pytest_unconfigure_called"])

    def test_root_conftest_syntax_error(self, pytester: Pytester) -> None:
        pytester.makepyfile(conftest="raise SyntaxError\n")
        result = pytester.runpytest()
        result.stderr.fnmatch_lines(["*raise SyntaxError*"])
        assert result.ret != 0

    def test_early_hook_error_issue38_1(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_sessionstart():
                0 / 0
        """
        )
        result = pytester.runpytest(pytester.path)
        assert result.ret != 0
        # tracestyle is native by default for hook failures
        result.stdout.fnmatch_lines(
            ["*INTERNALERROR*File*conftest.py*line 2*", "*0 / 0*"]
        )
        result = pytester.runpytest(pytester.path, "--fulltrace")
        assert result.ret != 0
        # tracestyle is native by default for hook failures
        result.stdout.fnmatch_lines(
            ["*INTERNALERROR*def pytest_sessionstart():*", "*INTERNALERROR*0 / 0*"]
        )

    def test_early_hook_configure_error_issue38(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_configure():
                0 / 0
        """
        )
        result = pytester.runpytest(pytester.path)
        assert result.ret != 0
        # here we get it on stderr
        result.stderr.fnmatch_lines(
            ["*INTERNALERROR*File*conftest.py*line 2*", "*0 / 0*"]
        )

    def test_file_not_found(self, pytester: Pytester) -> None:
        result = pytester.runpytest("asd")
        assert result.ret != 0
        result.stderr.fnmatch_lines(["ERROR: file or directory not found: asd"])

    def test_file_not_found_unconfigure_issue143(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_configure():
                print("---configure")
            def pytest_unconfigure():
                print("---unconfigure")
        """
        )
        result = pytester.runpytest("-s", "asd")
        assert result.ret == ExitCode.USAGE_ERROR
        result.stderr.fnmatch_lines(["ERROR: file or directory not found: asd"])
        result.stdout.fnmatch_lines(["*---configure", "*---unconfigure"])

    def test_config_preparse_plugin_option(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            pytest_xyz="""
            def pytest_addoption(parser):
                parser.addoption("--xyz", dest="xyz", action="store")
        """
        )
        pytester.makepyfile(
            test_one="""
            def test_option(pytestconfig):
                assert pytestconfig.option.xyz == "123"
        """
        )
        result = pytester.runpytest("-p", "pytest_xyz", "--xyz=123", syspathinsert=True)
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

    @pytest.mark.parametrize("load_cov_early", [True, False])
    def test_early_load_setuptools_name(
        self, pytester: Pytester, monkeypatch, load_cov_early
    ) -> None:
        monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD")

        pytester.makepyfile(mytestplugin1_module="")
        pytester.makepyfile(mytestplugin2_module="")
        pytester.makepyfile(mycov_module="")
        pytester.syspathinsert()

        loaded = []

        @dataclasses.dataclass
        class DummyEntryPoint:
            name: str
            module: str
            group: str = "pytest11"

            def load(self):
                __import__(self.module)
                loaded.append(self.name)
                return sys.modules[self.module]

        entry_points = [
            DummyEntryPoint("myplugin1", "mytestplugin1_module"),
            DummyEntryPoint("myplugin2", "mytestplugin2_module"),
            DummyEntryPoint("mycov", "mycov_module"),
        ]

        @dataclasses.dataclass
        class DummyDist:
            entry_points: object
            files: object = ()

        def my_dists():
            return (DummyDist(entry_points),)

        monkeypatch.setattr(importlib.metadata, "distributions", my_dists)
        params = ("-p", "mycov") if load_cov_early else ()
        pytester.runpytest_inprocess(*params)
        if load_cov_early:
            assert loaded == ["mycov", "myplugin1", "myplugin2"]
        else:
            assert loaded == ["myplugin1", "myplugin2", "mycov"]

    @pytest.mark.parametrize("import_mode", ["prepend", "append", "importlib"])
    def test_assertion_rewrite(self, pytester: Pytester, import_mode) -> None:
        p = pytester.makepyfile(
            """
            def test_this():
                x = 0
                assert x
        """
        )
        result = pytester.runpytest(p, f"--import-mode={import_mode}")
        result.stdout.fnmatch_lines([">       assert x", "E       assert 0"])
        assert result.ret == 1

    def test_nested_import_error(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
                import import_fails
                def test_this():
                    assert import_fails.a == 1
        """
        )
        pytester.makepyfile(import_fails="import does_not_work")
        result = pytester.runpytest(p)
        result.stdout.fnmatch_lines(
            [
                "ImportError while importing test module*",
                "*No module named *does_not_work*",
            ]
        )
        assert result.ret == 2

    def test_not_collectable_arguments(self, pytester: Pytester) -> None:
        p1 = pytester.makepyfile("")
        p2 = pytester.makefile(".pyc", "123")
        result = pytester.runpytest(p1, p2)
        assert result.ret == ExitCode.USAGE_ERROR
        result.stderr.fnmatch_lines(
            [
                f"ERROR: not found: {p2}",
                "(no match in any of *)",
                "",
            ]
        )

    @pytest.mark.filterwarnings("default")
    def test_better_reporting_on_conftest_load_failure(
        self, pytester: Pytester
    ) -> None:
        """Show a user-friendly traceback on conftest import failures (#486, #3332)"""
        pytester.makepyfile("")
        conftest = pytester.makeconftest(
            """
            def foo():
                import qwerty
            foo()
        """
        )
        result = pytester.runpytest("--help")
        result.stdout.fnmatch_lines(
            """
            *--version*
            *warning*conftest.py*
        """
        )
        result = pytester.runpytest()
        assert result.stdout.lines == []
        assert result.stderr.lines == [
            f"ImportError while loading conftest '{conftest}'.",
            "conftest.py:3: in <module>",
            "    foo()",
            "conftest.py:2: in foo",
            "    import qwerty",
            "E   ModuleNotFoundError: No module named 'qwerty'",
        ]

    def test_early_skip(self, pytester: Pytester) -> None:
        pytester.mkdir("xyz")
        pytester.makeconftest(
            """
            import pytest
            def pytest_collect_file():
                pytest.skip("early")
        """
        )
        result = pytester.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.fnmatch_lines(["*1 skip*"])

    def test_issue88_initial_file_multinodes(self, pytester: Pytester) -> None:
        pytester.copy_example("issue88_initial_file_multinodes")
        p = pytester.makepyfile("def test_hello(): pass")
        result = pytester.runpytest(p, "--collect-only")
        result.stdout.fnmatch_lines(["*MyFile*test_issue88*", "*Module*test_issue88*"])

    def test_issue93_initialnode_importing_capturing(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import sys
            print("should not be seen")
            sys.stderr.write("stder42\\n")
        """
        )
        result = pytester.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.no_fnmatch_line("*should not be seen*")
        assert "stderr42" not in result.stderr.str()

    def test_conftest_printing_shows_if_error(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            print("should be seen")
            assert 0
        """
        )
        result = pytester.runpytest()
        assert result.ret != 0
        assert "should be seen" in result.stdout.str()

    def test_issue109_sibling_conftests_not_loaded(self, pytester: Pytester) -> None:
        sub1 = pytester.mkdir("sub1")
        sub2 = pytester.mkdir("sub2")
        sub1.joinpath("conftest.py").write_text("assert 0", encoding="utf-8")
        result = pytester.runpytest(sub2)
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        sub2.joinpath("__init__.py").touch()
        p = sub2.joinpath("test_hello.py")
        p.touch()
        result = pytester.runpytest(p)
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result = pytester.runpytest(sub1)
        assert result.ret == ExitCode.USAGE_ERROR

    def test_directory_skipped(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            def pytest_ignore_collect():
                pytest.skip("intentional")
        """
        )
        pytester.makepyfile("def test_hello(): pass")
        result = pytester.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.fnmatch_lines(["*1 skipped*"])

    def test_multiple_items_per_collector_byid(self, pytester: Pytester) -> None:
        c = pytester.makeconftest(
            """
            import pytest
            class MyItem(pytest.Item):
                def runtest(self):
                    pass
            class MyCollector(pytest.File):
                def collect(self):
                    return [MyItem.from_parent(name="xyz", parent=self)]
            def pytest_collect_file(file_path, parent):
                if file_path.name.startswith("conftest"):
                    return MyCollector.from_parent(path=file_path, parent=parent)
        """
        )
        result = pytester.runpytest(c.name + "::" + "xyz")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 pass*"])

    def test_skip_on_generated_funcarg_id(self, pytester: Pytester) -> None:
        pytester.makeconftest(
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
        p = pytester.makepyfile("""def test_func(x): pass""")
        res = pytester.runpytest(p)
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 skipped*"])

    def test_direct_addressing_selects(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            def pytest_generate_tests(metafunc):
                metafunc.parametrize('i', [1, 2], ids=["1", "2"])
            def test_func(i):
                pass
        """
        )
        res = pytester.runpytest(p.name + "::" + "test_func[1]")
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 passed*"])

    def test_direct_addressing_selects_duplicates(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize("a", [1, 2, 10, 11, 2, 1, 12, 11])
            def test_func(a):
                pass
            """
        )
        result = pytester.runpytest(p)
        result.assert_outcomes(failed=0, passed=8)

    def test_direct_addressing_selects_duplicates_1(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize("a", [1, 2, 10, 11, 2, 1, 12, 1_1,2_1])
            def test_func(a):
                pass
            """
        )
        result = pytester.runpytest(p)
        result.assert_outcomes(failed=0, passed=9)

    def test_direct_addressing_selects_duplicates_2(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            @pytest.mark.parametrize("a", ["a","b","c","a","a1"])
            def test_func(a):
                pass
            """
        )
        result = pytester.runpytest(p)
        result.assert_outcomes(failed=0, passed=5)

    def test_direct_addressing_notfound(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            def test_func():
                pass
        """
        )
        res = pytester.runpytest(p.name + "::" + "test_notfound")
        assert res.ret
        res.stderr.fnmatch_lines(["*ERROR*not found*"])

    def test_docstring_on_hookspec(self) -> None:
        from _pytest import hookspec

        for name, value in vars(hookspec).items():
            if name.startswith("pytest_"):
                assert value.__doc__, "no docstring for %s" % name

    def test_initialization_error_issue49(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_configure():
                x
        """
        )
        result = pytester.runpytest()
        assert result.ret == 3  # internal error
        result.stderr.fnmatch_lines(["INTERNAL*pytest_configure*", "INTERNAL*x*"])
        assert "sessionstarttime" not in result.stderr.str()

    @pytest.mark.parametrize("lookfor", ["test_fun.py::test_a"])
    def test_issue134_report_error_when_collecting_member(
        self, pytester: Pytester, lookfor
    ) -> None:
        pytester.makepyfile(
            test_fun="""
            def test_a():
                pass
            def"""
        )
        result = pytester.runpytest(lookfor)
        result.stdout.fnmatch_lines(["*SyntaxError*"])
        if "::" in lookfor:
            result.stderr.fnmatch_lines(["*ERROR*"])
            assert result.ret == 4  # usage error only if item not found

    def test_report_all_failed_collections_initargs(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            from _pytest.config import ExitCode

            def pytest_sessionfinish(exitstatus):
                assert exitstatus == ExitCode.USAGE_ERROR
                print("pytest_sessionfinish_called")
            """
        )
        pytester.makepyfile(test_a="def", test_b="def")
        result = pytester.runpytest("test_a.py::a", "test_b.py::b")
        result.stderr.fnmatch_lines(["*ERROR*test_a.py::a*", "*ERROR*test_b.py::b*"])
        result.stdout.fnmatch_lines(["pytest_sessionfinish_called"])
        assert result.ret == ExitCode.USAGE_ERROR

    def test_namespace_import_doesnt_confuse_import_hook(
        self, pytester: Pytester
    ) -> None:
        """Ref #383.

        Python 3.3's namespace package messed with our import hooks.
        Importing a module that didn't exist, even if the ImportError was
        gracefully handled, would make our test crash.
        """
        pytester.mkdir("not_a_package")
        p = pytester.makepyfile(
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
        res = pytester.runpytest(p.name)
        assert res.ret == 0

    def test_unknown_option(self, pytester: Pytester) -> None:
        result = pytester.runpytest("--qwlkej")
        result.stderr.fnmatch_lines(
            """
            *unrecognized*
        """
        )

    def test_getsourcelines_error_issue553(
        self, pytester: Pytester, monkeypatch
    ) -> None:
        monkeypatch.setattr("inspect.getsourcelines", None)
        p = pytester.makepyfile(
            """
            def raise_error(obj):
                raise OSError('source code not available')

            import inspect
            inspect.getsourcelines = raise_error

            def test_foo(invalid_fixture):
                pass
        """
        )
        res = pytester.runpytest(p)
        res.stdout.fnmatch_lines(
            ["*source code not available*", "E*fixture 'invalid_fixture' not found"]
        )

    def test_plugins_given_as_strings(
        self, pytester: Pytester, monkeypatch, _sys_snapshot
    ) -> None:
        """Test that str values passed to main() as `plugins` arg are
        interpreted as module names to be imported and registered (#855)."""
        with pytest.raises(ImportError) as excinfo:
            pytest.main([str(pytester.path)], plugins=["invalid.module"])
        assert "invalid" in str(excinfo.value)

        p = pytester.path.joinpath("test_test_plugins_given_as_strings.py")
        p.write_text("def test_foo(): pass", encoding="utf-8")
        mod = types.ModuleType("myplugin")
        monkeypatch.setitem(sys.modules, "myplugin", mod)
        assert pytest.main(args=[str(pytester.path)], plugins=["myplugin"]) == 0

    def test_parametrized_with_bytes_regex(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import re
            import pytest
            @pytest.mark.parametrize('r', [re.compile(b'foo')])
            def test_stuff(r):
                pass
        """
        )
        res = pytester.runpytest(p)
        res.stdout.fnmatch_lines(["*1 passed*"])

    def test_parametrized_with_null_bytes(self, pytester: Pytester) -> None:
        """Test parametrization with values that contain null bytes and unicode characters (#2644, #2957)"""
        p = pytester.makepyfile(
            """\
            import pytest

            @pytest.mark.parametrize("data", [b"\\x00", "\\x00", 'ação'])
            def test_foo(data):
                assert data
            """
        )
        res = pytester.runpytest(p)
        res.assert_outcomes(passed=3)

    # Warning ignore because of:
    # https://github.com/python/cpython/issues/85308
    # Can be removed once Python<3.12 support is dropped.
    @pytest.mark.filterwarnings("ignore:'encoding' argument not specified")
    def test_command_line_args_from_file(
        self, pytester: Pytester, tmp_path: Path
    ) -> None:
        pytester.makepyfile(
            test_file="""
            import pytest

            class TestClass:
                @pytest.mark.parametrize("a", ["x","y"])
                def test_func(self, a):
                    pass
            """
        )
        tests = [
            "test_file.py::TestClass::test_func[x]",
            "test_file.py::TestClass::test_func[y]",
            "-q",
        ]
        args_file = pytester.maketxtfile(tests="\n".join(tests))
        result = pytester.runpytest(f"@{args_file}")
        result.assert_outcomes(failed=0, passed=2)


class TestInvocationVariants:
    def test_earlyinit(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest
            assert hasattr(pytest, 'mark')
        """
        )
        result = pytester.runpython(p)
        assert result.ret == 0

    def test_pydoc(self, pytester: Pytester) -> None:
        result = pytester.runpython_c("import pytest;help(pytest)")
        assert result.ret == 0
        s = result.stdout.str()
        assert "MarkGenerator" in s

    def test_import_star_pytest(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            from pytest import *
            #Item
            #File
            main
            skip
            xfail
        """
        )
        result = pytester.runpython(p)
        assert result.ret == 0

    def test_double_pytestcmdline(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            run="""
            import pytest
            pytest.main()
            pytest.main()
        """
        )
        pytester.makepyfile(
            """
            def test_hello():
                pass
        """
        )
        result = pytester.runpython(p)
        result.stdout.fnmatch_lines(["*1 passed*", "*1 passed*"])

    def test_python_minus_m_invocation_ok(self, pytester: Pytester) -> None:
        p1 = pytester.makepyfile("def test_hello(): pass")
        res = pytester.run(sys.executable, "-m", "pytest", str(p1))
        assert res.ret == 0

    def test_python_minus_m_invocation_fail(self, pytester: Pytester) -> None:
        p1 = pytester.makepyfile("def test_fail(): 0/0")
        res = pytester.run(sys.executable, "-m", "pytest", str(p1))
        assert res.ret == 1

    def test_python_pytest_package(self, pytester: Pytester) -> None:
        p1 = pytester.makepyfile("def test_pass(): pass")
        res = pytester.run(sys.executable, "-m", "pytest", str(p1))
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 passed*"])

    def test_invoke_with_invalid_type(self) -> None:
        with pytest.raises(
            TypeError, match="expected to be a list of strings, got: '-h'"
        ):
            pytest.main("-h")  # type: ignore[arg-type]

    def test_invoke_with_path(self, pytester: Pytester, capsys) -> None:
        retcode = pytest.main([str(pytester.path)])
        assert retcode == ExitCode.NO_TESTS_COLLECTED
        out, err = capsys.readouterr()

    def test_invoke_plugin_api(self, capsys) -> None:
        class MyPlugin:
            def pytest_addoption(self, parser):
                parser.addoption("--myopt")

        pytest.main(["-h"], plugins=[MyPlugin()])
        out, err = capsys.readouterr()
        assert "--myopt" in out

    def test_pyargs_importerror(self, pytester: Pytester, monkeypatch) -> None:
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", False)
        path = pytester.mkpydir("tpkg")
        path.joinpath("test_hello.py").write_text("raise ImportError", encoding="utf-8")

        result = pytester.runpytest("--pyargs", "tpkg.test_hello", syspathinsert=True)
        assert result.ret != 0

        result.stdout.fnmatch_lines(["collected*0*items*/*1*error"])

    def test_pyargs_only_imported_once(self, pytester: Pytester) -> None:
        pkg = pytester.mkpydir("foo")
        pkg.joinpath("test_foo.py").write_text(
            "print('hello from test_foo')\ndef test(): pass", encoding="utf-8"
        )
        pkg.joinpath("conftest.py").write_text(
            "def pytest_configure(config): print('configuring')", encoding="utf-8"
        )

        result = pytester.runpytest(
            "--pyargs", "foo.test_foo", "-s", syspathinsert=True
        )
        # should only import once
        assert result.outlines.count("hello from test_foo") == 1
        # should only configure once
        assert result.outlines.count("configuring") == 1

    def test_pyargs_filename_looks_like_module(self, pytester: Pytester) -> None:
        pytester.path.joinpath("conftest.py").touch()
        pytester.path.joinpath("t.py").write_text("def test(): pass", encoding="utf-8")
        result = pytester.runpytest("--pyargs", "t.py")
        assert result.ret == ExitCode.OK

    def test_cmdline_python_package(self, pytester: Pytester, monkeypatch) -> None:
        import warnings

        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", False)
        path = pytester.mkpydir("tpkg")
        path.joinpath("test_hello.py").write_text(
            "def test_hello(): pass", encoding="utf-8"
        )
        path.joinpath("test_world.py").write_text(
            "def test_world(): pass", encoding="utf-8"
        )
        result = pytester.runpytest("--pyargs", "tpkg")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*2 passed*"])
        result = pytester.runpytest("--pyargs", "tpkg.test_hello", syspathinsert=True)
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

        empty_package = pytester.mkpydir("empty_package")
        monkeypatch.setenv("PYTHONPATH", str(empty_package), prepend=os.pathsep)
        # the path which is not a package raises a warning on pypy;
        # no idea why only pypy and not normal python warn about it here
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", ImportWarning)
            result = pytester.runpytest("--pyargs", ".")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*2 passed*"])

        monkeypatch.setenv("PYTHONPATH", str(pytester), prepend=os.pathsep)
        result = pytester.runpytest("--pyargs", "tpkg.test_missing", syspathinsert=True)
        assert result.ret != 0
        result.stderr.fnmatch_lines(["*not*found*test_missing*"])

    def test_cmdline_python_namespace_package(
        self, pytester: Pytester, monkeypatch
    ) -> None:
        """Test --pyargs option with namespace packages (#1567).

        Ref: https://packaging.python.org/guides/packaging-namespace-packages/
        """
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", raising=False)

        search_path = []
        for dirname in "hello", "world":
            d = pytester.mkdir(dirname)
            search_path.append(d)
            ns = d.joinpath("ns_pkg")
            ns.mkdir()
            ns.joinpath("__init__.py").write_text(
                "__import__('pkg_resources').declare_namespace(__name__)",
                encoding="utf-8",
            )
            lib = ns.joinpath(dirname)
            lib.mkdir()
            lib.joinpath("__init__.py").touch()
            lib.joinpath(f"test_{dirname}.py").write_text(
                f"def test_{dirname}(): pass\ndef test_other():pass",
                encoding="utf-8",
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

        # pgk_resources.declare_namespace has been deprecated in favor of implicit namespace packages.
        # pgk_resources has been deprecated entirely.
        # While we could change the test to use implicit namespace packages, seems better
        # to still ensure the old declaration via declare_namespace still works.
        ignore_w = (
            r"-Wignore:Deprecated call to `pkg_resources.declare_namespace",
            r"-Wignore:pkg_resources is deprecated",
        )
        result = pytester.runpytest(
            "--pyargs", "-v", "ns_pkg.hello", "ns_pkg/world", *ignore_w
        )
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
        pytester.chdir()
        result = pytester.runpytest(
            "--pyargs", "-v", "ns_pkg.world.test_world::test_other"
        )
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            ["*test_world.py::test_other*PASSED*", "*1 passed*"]
        )

    def test_invoke_test_and_doctestmodules(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            def test():
                pass
        """
        )
        result = pytester.runpytest(str(p) + "::test", "--doctest-modules")
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_cmdline_python_package_symlink(
        self, pytester: Pytester, monkeypatch
    ) -> None:
        """
        --pyargs with packages with path containing symlink can have conftest.py in
        their package (#2985)
        """
        monkeypatch.delenv("PYTHONDONTWRITEBYTECODE", raising=False)

        dirname = "lib"
        d = pytester.mkdir(dirname)
        foo = d.joinpath("foo")
        foo.mkdir()
        foo.joinpath("__init__.py").touch()
        lib = foo.joinpath("bar")
        lib.mkdir()
        lib.joinpath("__init__.py").touch()
        lib.joinpath("test_bar.py").write_text(
            "def test_bar(): pass\ndef test_other(a_fixture):pass", encoding="utf-8"
        )
        lib.joinpath("conftest.py").write_text(
            "import pytest\n@pytest.fixture\ndef a_fixture():pass", encoding="utf-8"
        )

        d_local = pytester.mkdir("symlink_root")
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
        result = pytester.runpytest("--pyargs", "-v", "foo.bar")
        pytester.chdir()
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            [
                "symlink_root/lib/foo/bar/test_bar.py::test_bar PASSED*",
                "symlink_root/lib/foo/bar/test_bar.py::test_other PASSED*",
                "*2 passed*",
            ]
        )

    def test_cmdline_python_package_not_exists(self, pytester: Pytester) -> None:
        result = pytester.runpytest("--pyargs", "tpkgwhatv")
        assert result.ret
        result.stderr.fnmatch_lines(["ERROR*module*or*package*not*found*"])

    @pytest.mark.xfail(reason="decide: feature or bug")
    def test_noclass_discovery_if_not_testcase(self, pytester: Pytester) -> None:
        testpath = pytester.makepyfile(
            """
            import unittest
            class TestHello(object):
                def test_hello(self):
                    assert self.attr

            class RealTest(unittest.TestCase, TestHello):
                attr = 42
        """
        )
        reprec = pytester.inline_run(testpath)
        reprec.assertoutcome(passed=1)

    def test_doctest_id(self, pytester: Pytester) -> None:
        pytester.makefile(
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
        result = pytester.runpytest(testid, "-rf", "--tb=short")
        result.stdout.fnmatch_lines(expected_lines)

        # Ensure that re-running it will still handle it as
        # doctest.DocTestFailure, which was not the case before when
        # re-importing doctest, but not creating a new RUNNER_CLASS.
        result = pytester.runpytest(testid, "-rf", "--tb=short")
        result.stdout.fnmatch_lines(expected_lines)

    def test_core_backward_compatibility(self) -> None:
        """Test backward compatibility for get_plugin_manager function. See #787."""
        import _pytest.config

        assert (
            type(_pytest.config.get_plugin_manager())
            is _pytest.config.PytestPluginManager
        )

    def test_has_plugin(self, request) -> None:
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

    def test_calls(self, pytester: Pytester, mock_timing) -> None:
        pytester.makepyfile(self.source)
        result = pytester.runpytest_inprocess("--durations=10")
        assert result.ret == 0

        result.stdout.fnmatch_lines_random(
            ["*durations*", "*call*test_3*", "*call*test_2*"]
        )

        result.stdout.fnmatch_lines(
            ["(8 durations < 0.005s hidden.  Use -vv to show these durations.)"]
        )

    def test_calls_show_2(self, pytester: Pytester, mock_timing) -> None:
        pytester.makepyfile(self.source)
        result = pytester.runpytest_inprocess("--durations=2")
        assert result.ret == 0

        lines = result.stdout.get_lines_after("*slowest*durations*")
        assert "4 passed" in lines[2]

    def test_calls_showall(self, pytester: Pytester, mock_timing) -> None:
        pytester.makepyfile(self.source)
        result = pytester.runpytest_inprocess("--durations=0")
        assert result.ret == 0

        tested = "3"
        for x in tested:
            for y in ("call",):  # 'setup', 'call', 'teardown':
                for line in result.stdout.lines:
                    if ("test_%s" % x) in line and y in line:
                        break
                else:
                    raise AssertionError(f"not found {x} {y}")

    def test_calls_showall_verbose(self, pytester: Pytester, mock_timing) -> None:
        pytester.makepyfile(self.source)
        result = pytester.runpytest_inprocess("--durations=0", "-vv")
        assert result.ret == 0

        for x in "123":
            for y in ("call",):  # 'setup', 'call', 'teardown':
                for line in result.stdout.lines:
                    if ("test_%s" % x) in line and y in line:
                        break
                else:
                    raise AssertionError(f"not found {x} {y}")

    def test_with_deselected(self, pytester: Pytester, mock_timing) -> None:
        pytester.makepyfile(self.source)
        result = pytester.runpytest_inprocess("--durations=2", "-k test_3")
        assert result.ret == 0

        result.stdout.fnmatch_lines(["*durations*", "*call*test_3*"])

    def test_with_failing_collection(self, pytester: Pytester, mock_timing) -> None:
        pytester.makepyfile(self.source)
        pytester.makepyfile(test_collecterror="""xyz""")
        result = pytester.runpytest_inprocess("--durations=2", "-k test_1")
        assert result.ret == 2

        result.stdout.fnmatch_lines(["*Interrupted: 1 error during collection*"])
        # Collection errors abort test execution, therefore no duration is
        # output
        result.stdout.no_fnmatch_line("*duration*")

    def test_with_not(self, pytester: Pytester, mock_timing) -> None:
        pytester.makepyfile(self.source)
        result = pytester.runpytest_inprocess("-k not 1")
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

    def test_setup_function(self, pytester: Pytester, mock_timing) -> None:
        pytester.makepyfile(self.source)
        result = pytester.runpytest_inprocess("--durations=10")
        assert result.ret == 0

        result.stdout.fnmatch_lines_random(
            """
            *durations*
            5.00s call *test_1*
            2.00s setup *test_1*
        """
        )


def test_zipimport_hook(pytester: Pytester) -> None:
    """Test package loader is being used correctly (see #1837)."""
    zipapp = pytest.importorskip("zipapp")
    pytester.path.joinpath("app").mkdir()
    pytester.makepyfile(
        **{
            "app/foo.py": """
            import pytest
            def main():
                pytest.main(['--pyargs', 'foo'])
        """
        }
    )
    target = pytester.path.joinpath("foo.zip")
    zipapp.create_archive(
        str(pytester.path.joinpath("app")), str(target), main="foo:main"
    )
    result = pytester.runpython(target)
    assert result.ret == 0
    result.stderr.fnmatch_lines(["*not found*foo*"])
    result.stdout.no_fnmatch_line("*INTERNALERROR>*")


def test_import_plugin_unicode_name(pytester: Pytester) -> None:
    pytester.makepyfile(myplugin="")
    pytester.makepyfile("def test(): pass")
    pytester.makeconftest("pytest_plugins = ['myplugin']")
    r = pytester.runpytest()
    assert r.ret == 0


def test_pytest_plugins_as_module(pytester: Pytester) -> None:
    """Do not raise an error if pytest_plugins attribute is a module (#3899)"""
    pytester.makepyfile(
        **{
            "__init__.py": "",
            "pytest_plugins.py": "",
            "conftest.py": "from . import pytest_plugins",
            "test_foo.py": "def test(): pass",
        }
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["* 1 passed in *"])


def test_deferred_hook_checking(pytester: Pytester) -> None:
    """Check hooks as late as possible (#1821)."""
    pytester.syspathinsert()
    pytester.makepyfile(
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
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["* 1 passed *"])


def test_fixture_values_leak(pytester: Pytester) -> None:
    """Ensure that fixture objects are properly destroyed by the garbage collector at the end of their expected
    life-times (#2981).
    """
    pytester.makepyfile(
        """
        import dataclasses
        import gc
        import pytest
        import weakref

        @dataclasses.dataclass
        class SomeObj:
            name: str

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
    result = pytester.runpytest_subprocess()
    result.stdout.fnmatch_lines(["* 2 passed *"])


def test_fixture_order_respects_scope(pytester: Pytester) -> None:
    """Ensure that fixtures are created according to scope order (#2405)."""
    pytester.makepyfile(
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
    result = pytester.runpytest()
    assert result.ret == 0


def test_frame_leak_on_failing_test(pytester: Pytester) -> None:
    """Pytest would leak garbage referencing the frames of tests that failed
    that could never be reclaimed (#2798).

    Unfortunately it was not possible to remove the actual circles because most of them
    are made of traceback objects which cannot be weakly referenced. Those objects at least
    can be eventually claimed by the garbage collector.
    """
    pytester.makepyfile(
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
    result = pytester.runpytest_subprocess()
    result.stdout.fnmatch_lines(["*1 failed, 1 passed in*"])


def test_fixture_mock_integration(pytester: Pytester) -> None:
    """Test that decorators applied to fixture are left working (#3774)"""
    p = pytester.copy_example("acceptance/fixture_mock_integration.py")
    result = pytester.runpytest(p)
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_usage_error_code(pytester: Pytester) -> None:
    result = pytester.runpytest("-unknown-option-")
    assert result.ret == ExitCode.USAGE_ERROR


def test_warn_on_async_function(pytester: Pytester) -> None:
    # In the below we .close() the coroutine only to avoid
    # "RuntimeWarning: coroutine 'test_2' was never awaited"
    # which messes with other tests.
    pytester.makepyfile(
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
    result = pytester.runpytest("-Wdefault")
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


def test_warn_on_async_gen_function(pytester: Pytester) -> None:
    pytester.makepyfile(
        test_async="""
        async def test_1():
            yield
        async def test_2():
            yield
        def test_3():
            return test_2()
    """
    )
    result = pytester.runpytest("-Wdefault")
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


def test_pdb_can_be_rewritten(pytester: Pytester) -> None:
    pytester.makepyfile(
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
    result = pytester.runpytest_subprocess("-p", "no:debugging", "-vv")
    result.stdout.fnmatch_lines(
        [
            "    def check():",
            ">       assert 1 == 2",
            "E       assert 1 == 2",
            "",
            "pdb.py:2: AssertionError",
            "*= 1 failed in *",
        ]
    )
    assert result.ret == 1


def test_tee_stdio_captures_and_live_prints(pytester: Pytester) -> None:
    testpath = pytester.makepyfile(
        """
        import sys
        def test_simple():
            print ("@this is stdout@")
            print ("@this is stderr@", file=sys.stderr)
    """
    )
    result = pytester.runpytest_subprocess(
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
    fullXml = pytester.path.joinpath("output.xml").read_text(encoding="utf-8")
    assert "@this is stdout@\n" in fullXml
    assert "@this is stderr@\n" in fullXml


@pytest.mark.skipif(
    sys.platform == "win32",
    reason="Windows raises `OSError: [Errno 22] Invalid argument` instead",
)
def test_no_brokenpipeerror_message(pytester: Pytester) -> None:
    """Ensure that the broken pipe error message is suppressed.

    In some Python versions, it reaches sys.unraisablehook, in others
    a BrokenPipeError exception is propagated, but either way it prints
    to stderr on shutdown, so checking nothing is printed is enough.
    """
    popen = pytester.popen((*pytester._getpytestargs(), "--help"))
    popen.stdout.close()
    ret = popen.wait()
    assert popen.stderr.read() == b""
    assert ret == 1

    # Cleanup.
    popen.stderr.close()


def test_function_return_non_none_warning(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        def test_stuff():
            return "something"
    """
    )
    res = pytester.runpytest()
    res.stdout.fnmatch_lines(["*Did you mean to use `assert` instead of `return`?*"])


def test_doctest_and_normal_imports_with_importlib(pytester: Pytester) -> None:
    """
    Regression test for #10811: previously import_path with ImportMode.importlib would
    not return a module if already in sys.modules, resulting in modules being imported
    multiple times, which causes problems with modules that have import side effects.
    """
    # Uses the exact reproducer form #10811, given it is very minimal
    # and illustrates the problem well.
    pytester.makepyfile(
        **{
            "pmxbot/commands.py": "from . import logging",
            "pmxbot/logging.py": "",
            "tests/__init__.py": "",
            "tests/test_commands.py": """
                import importlib
                from pmxbot import logging

                class TestCommands:
                    def test_boo(self):
                        assert importlib.import_module('pmxbot.logging') is logging
                """,
        }
    )
    pytester.makeini(
        """
        [pytest]
        addopts=
            --doctest-modules
            --import-mode importlib
        """
    )
    result = pytester.runpytest_subprocess()
    result.stdout.fnmatch_lines("*1 passed*")


@pytest.mark.skip(reason="Test is not isolated")
def test_issue_9765(pytester: Pytester) -> None:
    """Reproducer for issue #9765 on Windows

    https://github.com/pytest-dev/pytest/issues/9765
    """
    pytester.makepyprojecttoml(
        """
        [tool.pytest.ini_options]
        addopts = "-p my_package.plugin.my_plugin"
        """
    )
    pytester.makepyfile(
        **{
            "setup.py": (
                """
                from setuptools import setup

                if __name__ == '__main__':
                    setup(name='my_package', packages=['my_package', 'my_package.plugin'])
                """
            ),
            "my_package/__init__.py": "",
            "my_package/conftest.py": "",
            "my_package/test_foo.py": "def test(): pass",
            "my_package/plugin/__init__.py": "",
            "my_package/plugin/my_plugin.py": (
                """
                import pytest

                def pytest_configure(config):

                    class SimplePlugin:
                        @pytest.fixture(params=[1, 2, 3])
                        def my_fixture(self, request):
                            yield request.param

                    config.pluginmanager.register(SimplePlugin())
                """
            ),
        }
    )

    subprocess.run([sys.executable, "setup.py", "develop"], check=True)
    try:
        # We are using subprocess.run rather than pytester.run on purpose.
        # pytester.run is adding the current directory to PYTHONPATH which avoids
        # the bug. We also use pytest rather than python -m pytest for the same
        # PYTHONPATH reason.
        subprocess.run(
            ["pytest", "my_package"], capture_output=True, check=True, text=True
        )
    except subprocess.CalledProcessError as exc:
        raise AssertionError(
            f"pytest command failed:\n{exc.stdout=!s}\n{exc.stderr=!s}"
        ) from exc
