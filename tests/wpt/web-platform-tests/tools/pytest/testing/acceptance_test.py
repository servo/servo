import sys

import _pytest._code
import py
import pytest
from _pytest.main import EXIT_NOTESTSCOLLECTED, EXIT_USAGEERROR


class TestGeneralUsage:
    def test_config_error(self, testdir):
        testdir.makeconftest("""
            def pytest_configure(config):
                import pytest
                raise pytest.UsageError("hello")
        """)
        result = testdir.runpytest(testdir.tmpdir)
        assert result.ret != 0
        result.stderr.fnmatch_lines([
            '*ERROR: hello'
        ])

    def test_root_conftest_syntax_error(self, testdir):
        testdir.makepyfile(conftest="raise SyntaxError\n")
        result = testdir.runpytest()
        result.stderr.fnmatch_lines(["*raise SyntaxError*"])
        assert result.ret != 0

    def test_early_hook_error_issue38_1(self, testdir):
        testdir.makeconftest("""
            def pytest_sessionstart():
                0 / 0
        """)
        result = testdir.runpytest(testdir.tmpdir)
        assert result.ret != 0
        # tracestyle is native by default for hook failures
        result.stdout.fnmatch_lines([
            '*INTERNALERROR*File*conftest.py*line 2*',
            '*0 / 0*',
        ])
        result = testdir.runpytest(testdir.tmpdir, "--fulltrace")
        assert result.ret != 0
        # tracestyle is native by default for hook failures
        result.stdout.fnmatch_lines([
            '*INTERNALERROR*def pytest_sessionstart():*',
            '*INTERNALERROR*0 / 0*',
        ])

    def test_early_hook_configure_error_issue38(self, testdir):
        testdir.makeconftest("""
            def pytest_configure():
                0 / 0
        """)
        result = testdir.runpytest(testdir.tmpdir)
        assert result.ret != 0
        # here we get it on stderr
        result.stderr.fnmatch_lines([
            '*INTERNALERROR*File*conftest.py*line 2*',
            '*0 / 0*',
        ])

    def test_file_not_found(self, testdir):
        result = testdir.runpytest("asd")
        assert result.ret != 0
        result.stderr.fnmatch_lines(["ERROR: file not found*asd"])

    def test_file_not_found_unconfigure_issue143(self, testdir):
        testdir.makeconftest("""
            def pytest_configure():
                print("---configure")
            def pytest_unconfigure():
                print("---unconfigure")
        """)
        result = testdir.runpytest("-s", "asd")
        assert result.ret == 4 # EXIT_USAGEERROR
        result.stderr.fnmatch_lines(["ERROR: file not found*asd"])
        result.stdout.fnmatch_lines([
            "*---configure",
            "*---unconfigure",
        ])


    def test_config_preparse_plugin_option(self, testdir):
        testdir.makepyfile(pytest_xyz="""
            def pytest_addoption(parser):
                parser.addoption("--xyz", dest="xyz", action="store")
        """)
        testdir.makepyfile(test_one="""
            def test_option(pytestconfig):
                assert pytestconfig.option.xyz == "123"
        """)
        result = testdir.runpytest("-p", "pytest_xyz", "--xyz=123", syspathinsert=True)
        assert result.ret == 0
        result.stdout.fnmatch_lines([
            '*1 passed*',
        ])

    def test_assertion_magic(self, testdir):
        p = testdir.makepyfile("""
            def test_this():
                x = 0
                assert x
        """)
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines([
            ">       assert x",
            "E       assert 0",
        ])
        assert result.ret == 1

    def test_nested_import_error(self, testdir):
        p = testdir.makepyfile("""
                import import_fails
                def test_this():
                    assert import_fails.a == 1
        """)
        testdir.makepyfile(import_fails="import does_not_work")
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines([
            #XXX on jython this fails:  ">   import import_fails",
            "E   ImportError: No module named *does_not_work*",
        ])
        assert result.ret == 1

    def test_not_collectable_arguments(self, testdir):
        p1 = testdir.makepyfile("")
        p2 = testdir.makefile(".pyc", "123")
        result = testdir.runpytest(p1, p2)
        assert result.ret
        result.stderr.fnmatch_lines([
            "*ERROR: not found:*%s" %(p2.basename,)
        ])

    def test_issue486_better_reporting_on_conftest_load_failure(self, testdir):
        testdir.makepyfile("")
        testdir.makeconftest("import qwerty")
        result = testdir.runpytest("--help")
        result.stdout.fnmatch_lines("""
            *--version*
            *warning*conftest.py*
        """)
        result = testdir.runpytest()
        result.stderr.fnmatch_lines("""
            *ERROR*could not load*conftest.py*
        """)


    def test_early_skip(self, testdir):
        testdir.mkdir("xyz")
        testdir.makeconftest("""
            import pytest
            def pytest_collect_directory():
                pytest.skip("early")
        """)
        result = testdir.runpytest()
        assert result.ret == EXIT_NOTESTSCOLLECTED
        result.stdout.fnmatch_lines([
            "*1 skip*"
        ])

    def test_issue88_initial_file_multinodes(self, testdir):
        testdir.makeconftest("""
            import pytest
            class MyFile(pytest.File):
                def collect(self):
                    return [MyItem("hello", parent=self)]
            def pytest_collect_file(path, parent):
                return MyFile(path, parent)
            class MyItem(pytest.Item):
                pass
        """)
        p = testdir.makepyfile("def test_hello(): pass")
        result = testdir.runpytest(p, "--collect-only")
        result.stdout.fnmatch_lines([
            "*MyFile*test_issue88*",
            "*Module*test_issue88*",
        ])

    def test_issue93_initialnode_importing_capturing(self, testdir):
        testdir.makeconftest("""
            import sys
            print ("should not be seen")
            sys.stderr.write("stder42\\n")
        """)
        result = testdir.runpytest()
        assert result.ret == EXIT_NOTESTSCOLLECTED
        assert "should not be seen" not in result.stdout.str()
        assert "stderr42" not in result.stderr.str()

    def test_conftest_printing_shows_if_error(self, testdir):
        testdir.makeconftest("""
            print ("should be seen")
            assert 0
        """)
        result = testdir.runpytest()
        assert result.ret != 0
        assert "should be seen" in result.stdout.str()

    @pytest.mark.skipif(not hasattr(py.path.local, 'mksymlinkto'),
                        reason="symlink not available on this platform")
    def test_chdir(self, testdir):
        testdir.tmpdir.join("py").mksymlinkto(py._pydir)
        p = testdir.tmpdir.join("main.py")
        p.write(_pytest._code.Source("""
            import sys, os
            sys.path.insert(0, '')
            import py
            print (py.__file__)
            print (py.__path__)
            os.chdir(os.path.dirname(os.getcwd()))
            print (py.log)
        """))
        result = testdir.runpython(p)
        assert not result.ret

    def test_issue109_sibling_conftests_not_loaded(self, testdir):
        sub1 = testdir.tmpdir.mkdir("sub1")
        sub2 = testdir.tmpdir.mkdir("sub2")
        sub1.join("conftest.py").write("assert 0")
        result = testdir.runpytest(sub2)
        assert result.ret == EXIT_NOTESTSCOLLECTED
        sub2.ensure("__init__.py")
        p = sub2.ensure("test_hello.py")
        result = testdir.runpytest(p)
        assert result.ret == EXIT_NOTESTSCOLLECTED
        result = testdir.runpytest(sub1)
        assert result.ret == EXIT_USAGEERROR

    def test_directory_skipped(self, testdir):
        testdir.makeconftest("""
            import pytest
            def pytest_ignore_collect():
                pytest.skip("intentional")
        """)
        testdir.makepyfile("def test_hello(): pass")
        result = testdir.runpytest()
        assert result.ret == EXIT_NOTESTSCOLLECTED
        result.stdout.fnmatch_lines([
            "*1 skipped*"
        ])

    def test_multiple_items_per_collector_byid(self, testdir):
        c = testdir.makeconftest("""
            import pytest
            class MyItem(pytest.Item):
                def runtest(self):
                    pass
            class MyCollector(pytest.File):
                def collect(self):
                    return [MyItem(name="xyz", parent=self)]
            def pytest_collect_file(path, parent):
                if path.basename.startswith("conftest"):
                    return MyCollector(path, parent)
        """)
        result = testdir.runpytest(c.basename+"::"+"xyz")
        assert result.ret == 0
        result.stdout.fnmatch_lines([
            "*1 pass*",
        ])

    def test_skip_on_generated_funcarg_id(self, testdir):
        testdir.makeconftest("""
            import pytest
            def pytest_generate_tests(metafunc):
                metafunc.addcall({'x': 3}, id='hello-123')
            def pytest_runtest_setup(item):
                print (item.keywords)
                if 'hello-123' in item.keywords:
                    pytest.skip("hello")
                assert 0
        """)
        p = testdir.makepyfile("""def test_func(x): pass""")
        res = testdir.runpytest(p)
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 skipped*"])

    def test_direct_addressing_selects(self, testdir):
        p = testdir.makepyfile("""
            def pytest_generate_tests(metafunc):
                metafunc.addcall({'i': 1}, id="1")
                metafunc.addcall({'i': 2}, id="2")
            def test_func(i):
                pass
        """)
        res = testdir.runpytest(p.basename + "::" + "test_func[1]")
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 passed*"])

    def test_direct_addressing_notfound(self, testdir):
        p = testdir.makepyfile("""
            def test_func():
                pass
        """)
        res = testdir.runpytest(p.basename + "::" + "test_notfound")
        assert res.ret
        res.stderr.fnmatch_lines(["*ERROR*not found*"])

    def test_docstring_on_hookspec(self):
        from _pytest import hookspec
        for name, value in vars(hookspec).items():
            if name.startswith("pytest_"):
                assert value.__doc__, "no docstring for %s" % name

    def test_initialization_error_issue49(self, testdir):
        testdir.makeconftest("""
            def pytest_configure():
                x
        """)
        result = testdir.runpytest()
        assert result.ret == 3 # internal error
        result.stderr.fnmatch_lines([
            "INTERNAL*pytest_configure*",
            "INTERNAL*x*",
        ])
        assert 'sessionstarttime' not in result.stderr.str()

    @pytest.mark.parametrize('lookfor', ['test_fun.py', 'test_fun.py::test_a'])
    def test_issue134_report_syntaxerror_when_collecting_member(self, testdir, lookfor):
        testdir.makepyfile(test_fun="""
            def test_a():
                pass
            def""")
        result = testdir.runpytest(lookfor)
        result.stdout.fnmatch_lines(['*SyntaxError*'])
        if '::' in lookfor:
            result.stderr.fnmatch_lines([
                '*ERROR*',
            ])
            assert result.ret == 4  # usage error only if item not found

    def test_report_all_failed_collections_initargs(self, testdir):
        testdir.makepyfile(test_a="def", test_b="def")
        result = testdir.runpytest("test_a.py::a", "test_b.py::b")
        result.stderr.fnmatch_lines([
            "*ERROR*test_a.py::a*",
            "*ERROR*test_b.py::b*",
        ])

    def test_namespace_import_doesnt_confuse_import_hook(self, testdir):
        # Ref #383. Python 3.3's namespace package messed with our import hooks
        # Importing a module that didn't exist, even if the ImportError was
        # gracefully handled, would make our test crash.
        testdir.mkdir('not_a_package')
        p = testdir.makepyfile("""
            try:
                from not_a_package import doesnt_exist
            except ImportError:
                # We handle the import error gracefully here
                pass

            def test_whatever():
                pass
        """)
        res = testdir.runpytest(p.basename)
        assert res.ret == 0

    def test_unknown_option(self, testdir):
        result = testdir.runpytest("--qwlkej")
        result.stderr.fnmatch_lines("""
            *unrecognized*
        """)

    def test_getsourcelines_error_issue553(self, testdir, monkeypatch):
        monkeypatch.setattr("inspect.getsourcelines", None)
        p = testdir.makepyfile("""
            def raise_error(obj):
                raise IOError('source code not available')

            import inspect
            inspect.getsourcelines = raise_error

            def test_foo(invalid_fixture):
                pass
        """)
        res = testdir.runpytest(p)
        res.stdout.fnmatch_lines([
            "*source code not available*",
            "*fixture 'invalid_fixture' not found",
        ])

    def test_plugins_given_as_strings(self, tmpdir, monkeypatch):
        """test that str values passed to main() as `plugins` arg
        are interpreted as module names to be imported and registered.
        #855.
        """
        with pytest.raises(ImportError) as excinfo:
            pytest.main([str(tmpdir)], plugins=['invalid.module'])
        assert 'invalid' in str(excinfo.value)

        p = tmpdir.join('test_test_plugins_given_as_strings.py')
        p.write('def test_foo(): pass')
        mod = py.std.types.ModuleType("myplugin")
        monkeypatch.setitem(sys.modules, 'myplugin', mod)
        assert pytest.main(args=[str(tmpdir)], plugins=['myplugin']) == 0

    def test_parameterized_with_bytes_regex(self, testdir):
        p = testdir.makepyfile("""
            import re
            import pytest
            @pytest.mark.parametrize('r', [re.compile(b'foo')])
            def test_stuff(r):
                pass
        """
        )
        res = testdir.runpytest(p)
        res.stdout.fnmatch_lines([
            '*1 passed*'
        ])


class TestInvocationVariants:
    def test_earlyinit(self, testdir):
        p = testdir.makepyfile("""
            import pytest
            assert hasattr(pytest, 'mark')
        """)
        result = testdir.runpython(p)
        assert result.ret == 0

    @pytest.mark.xfail("sys.platform.startswith('java')")
    def test_pydoc(self, testdir):
        for name in ('py.test', 'pytest'):
            result = testdir.runpython_c("import %s;help(%s)" % (name, name))
            assert result.ret == 0
            s = result.stdout.str()
            assert 'MarkGenerator' in s

    def test_import_star_py_dot_test(self, testdir):
        p = testdir.makepyfile("""
            from py.test import *
            #collect
            #cmdline
            #Item
            #assert collect.Item is Item
            #assert collect.Collector is Collector
            main
            skip
            xfail
        """)
        result = testdir.runpython(p)
        assert result.ret == 0

    def test_import_star_pytest(self, testdir):
        p = testdir.makepyfile("""
            from pytest import *
            #Item
            #File
            main
            skip
            xfail
        """)
        result = testdir.runpython(p)
        assert result.ret == 0

    def test_double_pytestcmdline(self, testdir):
        p = testdir.makepyfile(run="""
            import pytest
            pytest.main()
            pytest.main()
        """)
        testdir.makepyfile("""
            def test_hello():
                pass
        """)
        result = testdir.runpython(p)
        result.stdout.fnmatch_lines([
            "*1 passed*",
            "*1 passed*",
        ])

    def test_python_minus_m_invocation_ok(self, testdir):
        p1 = testdir.makepyfile("def test_hello(): pass")
        res = testdir.run(py.std.sys.executable, "-m", "pytest", str(p1))
        assert res.ret == 0

    def test_python_minus_m_invocation_fail(self, testdir):
        p1 = testdir.makepyfile("def test_fail(): 0/0")
        res = testdir.run(py.std.sys.executable, "-m", "pytest", str(p1))
        assert res.ret == 1

    def test_python_pytest_package(self, testdir):
        p1 = testdir.makepyfile("def test_pass(): pass")
        res = testdir.run(py.std.sys.executable, "-m", "pytest", str(p1))
        assert res.ret == 0
        res.stdout.fnmatch_lines(["*1 passed*"])

    def test_equivalence_pytest_pytest(self):
        assert pytest.main == py.test.cmdline.main

    def test_invoke_with_string(self, capsys):
        retcode = pytest.main("-h")
        assert not retcode
        out, err = capsys.readouterr()
        assert "--help" in out
        pytest.raises(ValueError, lambda: pytest.main(0))

    def test_invoke_with_path(self, tmpdir, capsys):
        retcode = pytest.main(tmpdir)
        assert retcode == EXIT_NOTESTSCOLLECTED
        out, err = capsys.readouterr()

    def test_invoke_plugin_api(self, testdir, capsys):
        class MyPlugin:
            def pytest_addoption(self, parser):
                parser.addoption("--myopt")

        pytest.main(["-h"], plugins=[MyPlugin()])
        out, err = capsys.readouterr()
        assert "--myopt" in out

    def test_pyargs_importerror(self, testdir, monkeypatch):
        monkeypatch.delenv('PYTHONDONTWRITEBYTECODE', False)
        path = testdir.mkpydir("tpkg")
        path.join("test_hello.py").write('raise ImportError')

        result = testdir.runpytest("--pyargs", "tpkg.test_hello")
        assert result.ret != 0
        # FIXME: It would be more natural to match NOT
        # "ERROR*file*or*package*not*found*".
        result.stdout.fnmatch_lines([
            "*collected 0 items*"
        ])

    def test_cmdline_python_package(self, testdir, monkeypatch):
        monkeypatch.delenv('PYTHONDONTWRITEBYTECODE', False)
        path = testdir.mkpydir("tpkg")
        path.join("test_hello.py").write("def test_hello(): pass")
        path.join("test_world.py").write("def test_world(): pass")
        result = testdir.runpytest("--pyargs", "tpkg")
        assert result.ret == 0
        result.stdout.fnmatch_lines([
            "*2 passed*"
        ])
        result = testdir.runpytest("--pyargs", "tpkg.test_hello")
        assert result.ret == 0
        result.stdout.fnmatch_lines([
            "*1 passed*"
        ])

        def join_pythonpath(what):
            cur = py.std.os.environ.get('PYTHONPATH')
            if cur:
                return str(what) + ':' + cur
            return what
        empty_package = testdir.mkpydir("empty_package")
        monkeypatch.setenv('PYTHONPATH', join_pythonpath(empty_package))
        result = testdir.runpytest("--pyargs", ".")
        assert result.ret == 0
        result.stdout.fnmatch_lines([
            "*2 passed*"
        ])

        monkeypatch.setenv('PYTHONPATH', join_pythonpath(testdir))
        path.join('test_hello.py').remove()
        result = testdir.runpytest("--pyargs", "tpkg.test_hello")
        assert result.ret != 0
        result.stderr.fnmatch_lines([
            "*not*found*test_hello*",
        ])

    def test_cmdline_python_package_not_exists(self, testdir):
        result = testdir.runpytest("--pyargs", "tpkgwhatv")
        assert result.ret
        result.stderr.fnmatch_lines([
            "ERROR*file*or*package*not*found*",
        ])

    @pytest.mark.xfail(reason="decide: feature or bug")
    def test_noclass_discovery_if_not_testcase(self, testdir):
        testpath = testdir.makepyfile("""
            import unittest
            class TestHello(object):
                def test_hello(self):
                    assert self.attr

            class RealTest(unittest.TestCase, TestHello):
                attr = 42
        """)
        reprec = testdir.inline_run(testpath)
        reprec.assertoutcome(passed=1)

    def test_doctest_id(self, testdir):
        testdir.makefile('.txt', """
            >>> x=3
            >>> x
            4
        """)
        result = testdir.runpytest("-rf")
        lines = result.stdout.str().splitlines()
        for line in lines:
            if line.startswith("FAIL "):
                testid = line[5:].strip()
                break
        result = testdir.runpytest(testid, '-rf')
        result.stdout.fnmatch_lines([
            line,
            "*1 failed*",
        ])

    def test_core_backward_compatibility(self):
        """Test backward compatibility for get_plugin_manager function. See #787."""
        import _pytest.config
        assert type(_pytest.config.get_plugin_manager()) is _pytest.config.PytestPluginManager


    def test_has_plugin(self, request):
        """Test hasplugin function of the plugin manager (#932)."""
        assert request.config.pluginmanager.hasplugin('python')


class TestDurations:
    source = """
        import time
        frag = 0.002
        def test_something():
            pass
        def test_2():
            time.sleep(frag*5)
        def test_1():
            time.sleep(frag)
        def test_3():
            time.sleep(frag*10)
    """

    def test_calls(self, testdir):
        testdir.makepyfile(self.source)
        result = testdir.runpytest("--durations=10")
        assert result.ret == 0
        result.stdout.fnmatch_lines_random([
            "*durations*",
            "*call*test_3*",
            "*call*test_2*",
            "*call*test_1*",
        ])

    def test_calls_show_2(self, testdir):
        testdir.makepyfile(self.source)
        result = testdir.runpytest("--durations=2")
        assert result.ret == 0
        lines = result.stdout.get_lines_after("*slowest*durations*")
        assert "4 passed" in lines[2]

    def test_calls_showall(self, testdir):
        testdir.makepyfile(self.source)
        result = testdir.runpytest("--durations=0")
        assert result.ret == 0
        for x in "123":
            for y in 'call',: #'setup', 'call', 'teardown':
                for line in result.stdout.lines:
                    if ("test_%s" % x) in line and y in line:
                        break
                else:
                    raise AssertionError("not found %s %s" % (x,y))

    def test_with_deselected(self, testdir):
        testdir.makepyfile(self.source)
        result = testdir.runpytest("--durations=2", "-k test_1")
        assert result.ret == 0
        result.stdout.fnmatch_lines([
            "*durations*",
            "*call*test_1*",
        ])

    def test_with_failing_collection(self, testdir):
        testdir.makepyfile(self.source)
        testdir.makepyfile(test_collecterror="""xyz""")
        result = testdir.runpytest("--durations=2", "-k test_1")
        assert result.ret != 0
        result.stdout.fnmatch_lines([
            "*durations*",
            "*call*test_1*",
        ])


class TestDurationWithFixture:
    source = """
        import time
        frag = 0.001
        def setup_function(func):
            time.sleep(frag * 3)
        def test_1():
            time.sleep(frag*2)
        def test_2():
            time.sleep(frag)
    """
    def test_setup_function(self, testdir):
        testdir.makepyfile(self.source)
        result = testdir.runpytest("--durations=10")
        assert result.ret == 0

        result.stdout.fnmatch_lines_random("""
            *durations*
            * setup *test_1*
            * call *test_1*
        """)

