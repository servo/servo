import py, pytest

import _pytest._code
from _pytest.config import getcfg, get_common_ancestor, determine_setup
from _pytest.main import EXIT_NOTESTSCOLLECTED

class TestParseIni:
    def test_getcfg_and_config(self, testdir, tmpdir):
        sub = tmpdir.mkdir("sub")
        sub.chdir()
        tmpdir.join("setup.cfg").write(_pytest._code.Source("""
            [pytest]
            name = value
        """))
        rootdir, inifile, cfg = getcfg([sub], ["setup.cfg"])
        assert cfg['name'] == "value"
        config = testdir.parseconfigure(sub)
        assert config.inicfg['name'] == 'value'

    def test_getcfg_empty_path(self, tmpdir):
        getcfg([''], ['setup.cfg']) #happens on py.test ""

    def test_append_parse_args(self, testdir, tmpdir, monkeypatch):
        monkeypatch.setenv('PYTEST_ADDOPTS', '--color no -rs --tb="short"')
        tmpdir.join("setup.cfg").write(_pytest._code.Source("""
            [pytest]
            addopts = --verbose
        """))
        config = testdir.parseconfig(tmpdir)
        assert config.option.color == 'no'
        assert config.option.reportchars == 's'
        assert config.option.tbstyle == 'short'
        assert config.option.verbose
        #config = testdir.Config()
        #args = [tmpdir,]
        #config._preparse(args, addopts=False)
        #assert len(args) == 1

    def test_tox_ini_wrong_version(self, testdir):
        testdir.makefile('.ini', tox="""
            [pytest]
            minversion=9.0
        """)
        result = testdir.runpytest()
        assert result.ret != 0
        result.stderr.fnmatch_lines([
            "*tox.ini:2*requires*9.0*actual*"
        ])

    @pytest.mark.parametrize("name", "setup.cfg tox.ini pytest.ini".split())
    def test_ini_names(self, testdir, name):
        testdir.tmpdir.join(name).write(py.std.textwrap.dedent("""
            [pytest]
            minversion = 1.0
        """))
        config = testdir.parseconfig()
        assert config.getini("minversion") == "1.0"

    def test_toxini_before_lower_pytestini(self, testdir):
        sub = testdir.tmpdir.mkdir("sub")
        sub.join("tox.ini").write(py.std.textwrap.dedent("""
            [pytest]
            minversion = 2.0
        """))
        testdir.tmpdir.join("pytest.ini").write(py.std.textwrap.dedent("""
            [pytest]
            minversion = 1.5
        """))
        config = testdir.parseconfigure(sub)
        assert config.getini("minversion") == "2.0"

    @pytest.mark.xfail(reason="probably not needed")
    def test_confcutdir(self, testdir):
        sub = testdir.mkdir("sub")
        sub.chdir()
        testdir.makeini("""
            [pytest]
            addopts = --qwe
        """)
        result = testdir.inline_run("--confcutdir=.")
        assert result.ret == 0

class TestConfigCmdlineParsing:
    def test_parsing_again_fails(self, testdir):
        config = testdir.parseconfig()
        pytest.raises(AssertionError, lambda: config.parse([]))

    def test_explicitly_specified_config_file_is_loaded(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("custom", "")
        """)
        testdir.makeini("""
            [pytest]
            custom = 0
        """)
        testdir.makefile(".cfg", custom = """
            [pytest]
            custom = 1
        """)
        config = testdir.parseconfig("-c", "custom.cfg")
        assert config.getini("custom") == "1"

class TestConfigAPI:
    def test_config_trace(self, testdir):
        config = testdir.parseconfig()
        l = []
        config.trace.root.setwriter(l.append)
        config.trace("hello")
        assert len(l) == 1
        assert l[0] == "hello [config]\n"

    def test_config_getoption(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addoption("--hello", "-X", dest="hello")
        """)
        config = testdir.parseconfig("--hello=this")
        for x in ("hello", "--hello", "-X"):
            assert config.getoption(x) == "this"
        pytest.raises(ValueError, "config.getoption('qweqwe')")

    @pytest.mark.skipif('sys.version_info[:2] not in [(2, 6), (2, 7)]')
    def test_config_getoption_unicode(self, testdir):
        testdir.makeconftest("""
            from __future__ import unicode_literals

            def pytest_addoption(parser):
                parser.addoption('--hello', type='string')
        """)
        config = testdir.parseconfig('--hello=this')
        assert config.getoption('hello') == 'this'

    def test_config_getvalueorskip(self, testdir):
        config = testdir.parseconfig()
        pytest.raises(pytest.skip.Exception,
            "config.getvalueorskip('hello')")
        verbose = config.getvalueorskip("verbose")
        assert verbose == config.option.verbose

    def test_config_getvalueorskip_None(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addoption("--hello")
        """)
        config = testdir.parseconfig()
        with pytest.raises(pytest.skip.Exception):
            config.getvalueorskip('hello')

    def test_getoption(self, testdir):
        config = testdir.parseconfig()
        with pytest.raises(ValueError):
            config.getvalue('x')
        assert config.getoption("x", 1) == 1

    def test_getconftest_pathlist(self, testdir, tmpdir):
        somepath = tmpdir.join("x", "y", "z")
        p = tmpdir.join("conftest.py")
        p.write("pathlist = ['.', %r]" % str(somepath))
        config = testdir.parseconfigure(p)
        assert config._getconftest_pathlist('notexist', path=tmpdir) is None
        pl = config._getconftest_pathlist('pathlist', path=tmpdir)
        print(pl)
        assert len(pl) == 2
        assert pl[0] == tmpdir
        assert pl[1] == somepath

    def test_addini(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("myname", "my new ini value")
        """)
        testdir.makeini("""
            [pytest]
            myname=hello
        """)
        config = testdir.parseconfig()
        val = config.getini("myname")
        assert val == "hello"
        pytest.raises(ValueError, config.getini, 'other')

    def test_addini_pathlist(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("paths", "my new ini value", type="pathlist")
                parser.addini("abc", "abc value")
        """)
        p = testdir.makeini("""
            [pytest]
            paths=hello world/sub.py
        """)
        config = testdir.parseconfig()
        l = config.getini("paths")
        assert len(l) == 2
        assert l[0] == p.dirpath('hello')
        assert l[1] == p.dirpath('world/sub.py')
        pytest.raises(ValueError, config.getini, 'other')

    def test_addini_args(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("args", "new args", type="args")
                parser.addini("a2", "", "args", default="1 2 3".split())
        """)
        testdir.makeini("""
            [pytest]
            args=123 "123 hello" "this"
        """)
        config = testdir.parseconfig()
        l = config.getini("args")
        assert len(l) == 3
        assert l == ["123", "123 hello", "this"]
        l = config.getini("a2")
        assert l == list("123")

    def test_addini_linelist(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("xy", "", type="linelist")
                parser.addini("a2", "", "linelist")
        """)
        testdir.makeini("""
            [pytest]
            xy= 123 345
                second line
        """)
        config = testdir.parseconfig()
        l = config.getini("xy")
        assert len(l) == 2
        assert l == ["123 345", "second line"]
        l = config.getini("a2")
        assert l == []

    @pytest.mark.parametrize('str_val, bool_val',
                             [('True', True), ('no', False), ('no-ini', True)])
    def test_addini_bool(self, testdir, str_val, bool_val):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("strip", "", type="bool", default=True)
        """)
        if str_val != 'no-ini':
            testdir.makeini("""
                [pytest]
                strip=%s
            """ % str_val)
        config = testdir.parseconfig()
        assert config.getini("strip") is bool_val

    def test_addinivalue_line_existing(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("xy", "", type="linelist")
        """)
        testdir.makeini("""
            [pytest]
            xy= 123
        """)
        config = testdir.parseconfig()
        l = config.getini("xy")
        assert len(l) == 1
        assert l == ["123"]
        config.addinivalue_line("xy", "456")
        l = config.getini("xy")
        assert len(l) == 2
        assert l == ["123", "456"]

    def test_addinivalue_line_new(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("xy", "", type="linelist")
        """)
        config = testdir.parseconfig()
        assert not config.getini("xy")
        config.addinivalue_line("xy", "456")
        l = config.getini("xy")
        assert len(l) == 1
        assert l == ["456"]
        config.addinivalue_line("xy", "123")
        l = config.getini("xy")
        assert len(l) == 2
        assert l == ["456", "123"]


class TestConfigFromdictargs:
    def test_basic_behavior(self):
        from _pytest.config import Config
        option_dict = {
            'verbose': 444,
            'foo': 'bar',
            'capture': 'no',
        }
        args = ['a', 'b']

        config = Config.fromdictargs(option_dict, args)
        with pytest.raises(AssertionError):
            config.parse(['should refuse to parse again'])
        assert config.option.verbose == 444
        assert config.option.foo == 'bar'
        assert config.option.capture == 'no'
        assert config.args == args

    def test_origargs(self):
        """Show that fromdictargs can handle args in their "orig" format"""
        from _pytest.config import Config
        option_dict = {}
        args = ['-vvvv', '-s', 'a', 'b']

        config = Config.fromdictargs(option_dict, args)
        assert config.args == ['a', 'b']
        assert config._origargs == args
        assert config.option.verbose == 4
        assert config.option.capture == 'no'

    def test_inifilename(self, tmpdir):
        tmpdir.join("foo/bar.ini").ensure().write(_pytest._code.Source("""
            [pytest]
            name = value
        """))

        from _pytest.config import Config
        inifile = '../../foo/bar.ini'
        option_dict = {
            'inifilename': inifile,
            'capture': 'no',
        }

        cwd = tmpdir.join('a/b')
        cwd.join('pytest.ini').ensure().write(_pytest._code.Source("""
            [pytest]
            name = wrong-value
            should_not_be_set = true
        """))
        with cwd.ensure(dir=True).as_cwd():
            config = Config.fromdictargs(option_dict, ())

        assert config.args == [str(cwd)]
        assert config.option.inifilename == inifile
        assert config.option.capture == 'no'

        # this indicates this is the file used for getting configuration values
        assert config.inifile == inifile
        assert config.inicfg.get('name') == 'value'
        assert config.inicfg.get('should_not_be_set') is None


def test_options_on_small_file_do_not_blow_up(testdir):
    def runfiletest(opts):
        reprec = testdir.inline_run(*opts)
        passed, skipped, failed = reprec.countoutcomes()
        assert failed == 2
        assert skipped == passed == 0
    path = testdir.makepyfile("""
        def test_f1(): assert 0
        def test_f2(): assert 0
    """)

    for opts in ([], ['-l'], ['-s'], ['--tb=no'], ['--tb=short'],
                 ['--tb=long'], ['--fulltrace'], ['--nomagic'],
                 ['--traceconfig'], ['-v'], ['-v', '-v']):
        runfiletest(opts + [path])

def test_preparse_ordering_with_setuptools(testdir, monkeypatch):
    pkg_resources = pytest.importorskip("pkg_resources")
    def my_iter(name):
        assert name == "pytest11"
        class EntryPoint:
            name = "mytestplugin"
            class dist:
                pass
            def load(self):
                class PseudoPlugin:
                    x = 42
                return PseudoPlugin()
        return iter([EntryPoint()])
    monkeypatch.setattr(pkg_resources, 'iter_entry_points', my_iter)
    testdir.makeconftest("""
        pytest_plugins = "mytestplugin",
    """)
    monkeypatch.setenv("PYTEST_PLUGINS", "mytestplugin")
    config = testdir.parseconfig()
    plugin = config.pluginmanager.getplugin("mytestplugin")
    assert plugin.x == 42

def test_plugin_preparse_prevents_setuptools_loading(testdir, monkeypatch):
    pkg_resources = pytest.importorskip("pkg_resources")
    def my_iter(name):
        assert name == "pytest11"
        class EntryPoint:
            name = "mytestplugin"
            def load(self):
                assert 0, "should not arrive here"
        return iter([EntryPoint()])
    monkeypatch.setattr(pkg_resources, 'iter_entry_points', my_iter)
    config = testdir.parseconfig("-p", "no:mytestplugin")
    plugin = config.pluginmanager.getplugin("mytestplugin")
    assert plugin is None

def test_cmdline_processargs_simple(testdir):
    testdir.makeconftest("""
        def pytest_cmdline_preparse(args):
            args.append("-h")
    """)
    result = testdir.runpytest()
    result.stdout.fnmatch_lines([
        "*pytest*",
        "*-h*",
    ])

def test_invalid_options_show_extra_information(testdir):
    """display extra information when pytest exits due to unrecognized
    options in the command-line"""
    testdir.makeini("""
        [pytest]
        addopts = --invalid-option
    """)
    result = testdir.runpytest()
    result.stderr.fnmatch_lines([
        "*error: unrecognized arguments: --invalid-option*",
        "*  inifile: %s*" % testdir.tmpdir.join('tox.ini'),
        "*  rootdir: %s*" % testdir.tmpdir,
    ])


@pytest.mark.parametrize('args', [
    ['dir1', 'dir2', '-v'],
    ['dir1', '-v', 'dir2'],
    ['dir2', '-v', 'dir1'],
    ['-v', 'dir2', 'dir1'],
])
def test_consider_args_after_options_for_rootdir_and_inifile(testdir, args):
    """
    Consider all arguments in the command-line for rootdir and inifile
    discovery, even if they happen to occur after an option. #949
    """
    # replace "dir1" and "dir2" from "args" into their real directory
    root = testdir.tmpdir.mkdir('myroot')
    d1 = root.mkdir('dir1')
    d2 = root.mkdir('dir2')
    for i, arg in enumerate(args):
        if arg == 'dir1':
            args[i] = d1
        elif arg == 'dir2':
            args[i] = d2
    result = testdir.runpytest(*args)
    result.stdout.fnmatch_lines(['*rootdir: *myroot, inifile: '])


@pytest.mark.skipif("sys.platform == 'win32'")
def test_toolongargs_issue224(testdir):
    result = testdir.runpytest("-m", "hello" * 500)
    assert result.ret == EXIT_NOTESTSCOLLECTED

def test_notify_exception(testdir, capfd):
    config = testdir.parseconfig()
    excinfo = pytest.raises(ValueError, "raise ValueError(1)")
    config.notify_exception(excinfo)
    out, err = capfd.readouterr()
    assert "ValueError" in err
    class A:
        def pytest_internalerror(self, excrepr):
            return True
    config.pluginmanager.register(A())
    config.notify_exception(excinfo)
    out, err = capfd.readouterr()
    assert not err


def test_load_initial_conftest_last_ordering(testdir):
    from _pytest.config  import get_config
    pm = get_config().pluginmanager
    class My:
        def pytest_load_initial_conftests(self):
            pass
    m = My()
    pm.register(m)
    hc = pm.hook.pytest_load_initial_conftests
    l = hc._nonwrappers + hc._wrappers
    assert l[-1].function.__module__ == "_pytest.capture"
    assert l[-2].function == m.pytest_load_initial_conftests
    assert l[-3].function.__module__ == "_pytest.config"

class TestWarning:
    def test_warn_config(self, testdir):
        testdir.makeconftest("""
            l = []
            def pytest_configure(config):
                config.warn("C1", "hello")
            def pytest_logwarning(code, message):
                if message == "hello" and code == "C1":
                    l.append(1)
        """)
        testdir.makepyfile("""
            def test_proper(pytestconfig):
                import conftest
                assert conftest.l == [1]
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_warn_on_test_item_from_request(self, testdir):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def fix(request):
                request.node.warn("T1", "hello")
            def test_hello(fix):
                pass
        """)
        result = testdir.runpytest()
        assert result.parseoutcomes()["pytest-warnings"] > 0
        assert "hello" not in result.stdout.str()

        result = testdir.runpytest("-rw")
        result.stdout.fnmatch_lines("""
            ===*pytest-warning summary*===
            *WT1*test_warn_on_test_item*:5*hello*
        """)

class TestRootdir:
    def test_simple_noini(self, tmpdir):
        assert get_common_ancestor([tmpdir]) == tmpdir
        assert get_common_ancestor([tmpdir.mkdir("a"), tmpdir]) == tmpdir
        assert get_common_ancestor([tmpdir, tmpdir.join("a")]) == tmpdir
        with tmpdir.as_cwd():
            assert get_common_ancestor([]) == tmpdir

    @pytest.mark.parametrize("name", "setup.cfg tox.ini pytest.ini".split())
    def test_with_ini(self, tmpdir, name):
        inifile = tmpdir.join(name)
        inifile.write("[pytest]\n")

        a = tmpdir.mkdir("a")
        b = a.mkdir("b")
        for args in ([tmpdir], [a], [b]):
            rootdir, inifile, inicfg = determine_setup(None, args)
            assert rootdir == tmpdir
            assert inifile == inifile
        rootdir, inifile, inicfg = determine_setup(None, [b,a])
        assert rootdir == tmpdir
        assert inifile == inifile

    @pytest.mark.parametrize("name", "setup.cfg tox.ini".split())
    def test_pytestini_overides_empty_other(self, tmpdir, name):
        inifile = tmpdir.ensure("pytest.ini")
        a = tmpdir.mkdir("a")
        a.ensure(name)
        rootdir, inifile, inicfg = determine_setup(None, [a])
        assert rootdir == tmpdir
        assert inifile == inifile

    def test_setuppy_fallback(self, tmpdir):
        a = tmpdir.mkdir("a")
        a.ensure("setup.cfg")
        tmpdir.ensure("setup.py")
        rootdir, inifile, inicfg = determine_setup(None, [a])
        assert rootdir == tmpdir
        assert inifile is None
        assert inicfg == {}

    def test_nothing(self, tmpdir):
        rootdir, inifile, inicfg = determine_setup(None, [tmpdir])
        assert rootdir == tmpdir
        assert inifile is None
        assert inicfg == {}

    def test_with_specific_inifile(self, tmpdir):
        inifile = tmpdir.ensure("pytest.ini")
        rootdir, inifile, inicfg = determine_setup(inifile, [tmpdir])
        assert rootdir == tmpdir
