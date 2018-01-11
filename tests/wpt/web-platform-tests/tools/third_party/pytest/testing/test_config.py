from __future__ import absolute_import, division, print_function
import sys
import py
import pytest

import _pytest._code
from _pytest.config import getcfg, get_common_ancestor, determine_setup, _iter_rewritable_modules
from _pytest.main import EXIT_NOTESTSCOLLECTED


class TestParseIni(object):

    @pytest.mark.parametrize('section, filename',
                             [('pytest', 'pytest.ini'), ('tool:pytest', 'setup.cfg')])
    def test_getcfg_and_config(self, testdir, tmpdir, section, filename):
        sub = tmpdir.mkdir("sub")
        sub.chdir()
        tmpdir.join(filename).write(_pytest._code.Source("""
            [{section}]
            name = value
        """.format(section=section)))
        rootdir, inifile, cfg = getcfg([sub])
        assert cfg['name'] == "value"
        config = testdir.parseconfigure(sub)
        assert config.inicfg['name'] == 'value'

    def test_getcfg_empty_path(self):
        """correctly handle zero length arguments (a la pytest '')"""
        getcfg([''])

    def test_append_parse_args(self, testdir, tmpdir, monkeypatch):
        monkeypatch.setenv('PYTEST_ADDOPTS', '--color no -rs --tb="short"')
        tmpdir.join("pytest.ini").write(_pytest._code.Source("""
            [pytest]
            addopts = --verbose
        """))
        config = testdir.parseconfig(tmpdir)
        assert config.option.color == 'no'
        assert config.option.reportchars == 's'
        assert config.option.tbstyle == 'short'
        assert config.option.verbose

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

    @pytest.mark.parametrize("section, name", [
        ('tool:pytest', 'setup.cfg'),
        ('pytest', 'tox.ini'),
        ('pytest', 'pytest.ini')],
    )
    def test_ini_names(self, testdir, name, section):
        testdir.tmpdir.join(name).write(py.std.textwrap.dedent("""
            [{section}]
            minversion = 1.0
        """.format(section=section)))
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


class TestConfigCmdlineParsing(object):
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
        testdir.makefile(".cfg", custom="""
            [pytest]
            custom = 1
        """)
        config = testdir.parseconfig("-c", "custom.cfg")
        assert config.getini("custom") == "1"

    def test_absolute_win32_path(self, testdir):
        temp_cfg_file = testdir.makefile(".cfg", custom="""
            [pytest]
            addopts = --version
        """)
        from os.path import normpath
        temp_cfg_file = normpath(str(temp_cfg_file))
        ret = pytest.main("-c " + temp_cfg_file)
        assert ret == _pytest.main.EXIT_OK


class TestConfigAPI(object):
    def test_config_trace(self, testdir):
        config = testdir.parseconfig()
        values = []
        config.trace.root.setwriter(values.append)
        config.trace("hello")
        assert len(values) == 1
        assert values[0] == "hello [config]\n"

    def test_config_getoption(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addoption("--hello", "-X", dest="hello")
        """)
        config = testdir.parseconfig("--hello=this")
        for x in ("hello", "--hello", "-X"):
            assert config.getoption(x) == "this"
        pytest.raises(ValueError, "config.getoption('qweqwe')")

    @pytest.mark.skipif('sys.version_info[0] < 3')
    def test_config_getoption_unicode(self, testdir):
        testdir.makeconftest("""
            from __future__ import unicode_literals

            def pytest_addoption(parser):
                parser.addoption('--hello', type=str)
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
        values = config.getini("paths")
        assert len(values) == 2
        assert values[0] == p.dirpath('hello')
        assert values[1] == p.dirpath('world/sub.py')
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
        values = config.getini("args")
        assert len(values) == 3
        assert values == ["123", "123 hello", "this"]
        values = config.getini("a2")
        assert values == list("123")

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
        values = config.getini("xy")
        assert len(values) == 2
        assert values == ["123 345", "second line"]
        values = config.getini("a2")
        assert values == []

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
        values = config.getini("xy")
        assert len(values) == 1
        assert values == ["123"]
        config.addinivalue_line("xy", "456")
        values = config.getini("xy")
        assert len(values) == 2
        assert values == ["123", "456"]

    def test_addinivalue_line_new(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("xy", "", type="linelist")
        """)
        config = testdir.parseconfig()
        assert not config.getini("xy")
        config.addinivalue_line("xy", "456")
        values = config.getini("xy")
        assert len(values) == 1
        assert values == ["456"]
        config.addinivalue_line("xy", "123")
        values = config.getini("xy")
        assert len(values) == 2
        assert values == ["456", "123"]

    def test_confcutdir_check_isdir(self, testdir):
        """Give an error if --confcutdir is not a valid directory (#2078)"""
        with pytest.raises(pytest.UsageError):
            testdir.parseconfig('--confcutdir', testdir.tmpdir.join('file').ensure(file=1))
        with pytest.raises(pytest.UsageError):
            testdir.parseconfig('--confcutdir', testdir.tmpdir.join('inexistant'))
        config = testdir.parseconfig('--confcutdir', testdir.tmpdir.join('dir').ensure(dir=1))
        assert config.getoption('confcutdir') == str(testdir.tmpdir.join('dir'))

    @pytest.mark.parametrize('names, expected', [
        (['bar.py'], ['bar']),
        (['foo', 'bar.py'], []),
        (['foo', 'bar.pyc'], []),
        (['foo', '__init__.py'], ['foo']),
        (['foo', 'bar', '__init__.py'], []),
    ])
    def test_iter_rewritable_modules(self, names, expected):
        assert list(_iter_rewritable_modules(['/'.join(names)])) == expected


class TestConfigFromdictargs(object):
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
                 ['--tb=long'], ['--fulltrace'],
                 ['--traceconfig'], ['-v'], ['-v', '-v']):
        runfiletest(opts + [path])


def test_preparse_ordering_with_setuptools(testdir, monkeypatch):
    pkg_resources = pytest.importorskip("pkg_resources")

    def my_iter(name):
        assert name == "pytest11"

        class Dist(object):
            project_name = 'spam'
            version = '1.0'

            def _get_metadata(self, name):
                return ['foo.txt,sha256=abc,123']

        class EntryPoint(object):
            name = "mytestplugin"
            dist = Dist()

            def load(self):
                class PseudoPlugin(object):
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


def test_setuptools_importerror_issue1479(testdir, monkeypatch):
    pkg_resources = pytest.importorskip("pkg_resources")

    def my_iter(name):
        assert name == "pytest11"

        class Dist(object):
            project_name = 'spam'
            version = '1.0'

            def _get_metadata(self, name):
                return ['foo.txt,sha256=abc,123']

        class EntryPoint(object):
            name = "mytestplugin"
            dist = Dist()

            def load(self):
                raise ImportError("Don't hide me!")

        return iter([EntryPoint()])

    monkeypatch.setattr(pkg_resources, 'iter_entry_points', my_iter)
    with pytest.raises(ImportError):
        testdir.parseconfig()


@pytest.mark.parametrize('block_it', [True, False])
def test_plugin_preparse_prevents_setuptools_loading(testdir, monkeypatch, block_it):
    pkg_resources = pytest.importorskip("pkg_resources")

    plugin_module_placeholder = object()

    def my_iter(name):
        assert name == "pytest11"

        class Dist(object):
            project_name = 'spam'
            version = '1.0'

            def _get_metadata(self, name):
                return ['foo.txt,sha256=abc,123']

        class EntryPoint(object):
            name = "mytestplugin"
            dist = Dist()

            def load(self):
                return plugin_module_placeholder

        return iter([EntryPoint()])

    monkeypatch.setattr(pkg_resources, 'iter_entry_points', my_iter)
    args = ("-p", "no:mytestplugin") if block_it else ()
    config = testdir.parseconfig(*args)
    config.pluginmanager.import_plugin("mytestplugin")
    if block_it:
        assert "mytestplugin" not in sys.modules
        assert config.pluginmanager.get_plugin('mytestplugin') is None
    else:
        assert config.pluginmanager.get_plugin('mytestplugin') is plugin_module_placeholder


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
    with root.as_cwd():
        result = testdir.runpytest(*args)
    result.stdout.fnmatch_lines(['*rootdir: *myroot, inifile:'])


@pytest.mark.skipif("sys.platform == 'win32'")
def test_toolongargs_issue224(testdir):
    result = testdir.runpytest("-m", "hello" * 500)
    assert result.ret == EXIT_NOTESTSCOLLECTED


def test_config_in_subdirectory_colon_command_line_issue2148(testdir):
    conftest_source = '''
        def pytest_addoption(parser):
            parser.addini('foo', 'foo')
    '''

    testdir.makefile('.ini', **{
        'pytest': '[pytest]\nfoo = root',
        'subdir/pytest': '[pytest]\nfoo = subdir',
    })

    testdir.makepyfile(**{
        'conftest': conftest_source,
        'subdir/conftest': conftest_source,
        'subdir/test_foo': '''
            def test_foo(pytestconfig):
                assert pytestconfig.getini('foo') == 'subdir'
        '''})

    result = testdir.runpytest('subdir/test_foo.py::test_foo')
    assert result.ret == 0


def test_notify_exception(testdir, capfd):
    config = testdir.parseconfig()
    excinfo = pytest.raises(ValueError, "raise ValueError(1)")
    config.notify_exception(excinfo)
    out, err = capfd.readouterr()
    assert "ValueError" in err

    class A(object):
        def pytest_internalerror(self, excrepr):
            return True

    config.pluginmanager.register(A())
    config.notify_exception(excinfo)
    out, err = capfd.readouterr()
    assert not err


def test_load_initial_conftest_last_ordering(testdir):
    from _pytest.config import get_config
    pm = get_config().pluginmanager

    class My(object):
        def pytest_load_initial_conftests(self):
            pass

    m = My()
    pm.register(m)
    hc = pm.hook.pytest_load_initial_conftests
    values = hc._nonwrappers + hc._wrappers
    expected = [
        "_pytest.config",
        'test_config',
        '_pytest.capture',
    ]
    assert [x.function.__module__ for x in values] == expected


def test_get_plugin_specs_as_list():
    from _pytest.config import _get_plugin_specs_as_list
    with pytest.raises(pytest.UsageError):
        _get_plugin_specs_as_list(set(['foo']))
    with pytest.raises(pytest.UsageError):
        _get_plugin_specs_as_list(dict())

    assert _get_plugin_specs_as_list(None) == []
    assert _get_plugin_specs_as_list('') == []
    assert _get_plugin_specs_as_list('foo') == ['foo']
    assert _get_plugin_specs_as_list('foo,bar') == ['foo', 'bar']
    assert _get_plugin_specs_as_list(['foo', 'bar']) == ['foo', 'bar']
    assert _get_plugin_specs_as_list(('foo', 'bar')) == ['foo', 'bar']


class TestWarning(object):
    def test_warn_config(self, testdir):
        testdir.makeconftest("""
            values = []
            def pytest_configure(config):
                config.warn("C1", "hello")
            def pytest_logwarning(code, message):
                if message == "hello" and code == "C1":
                    values.append(1)
        """)
        testdir.makepyfile("""
            def test_proper(pytestconfig):
                import conftest
                assert conftest.values == [1]
        """)
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)

    def test_warn_on_test_item_from_request(self, testdir, request):
        testdir.makepyfile("""
            import pytest

            @pytest.fixture
            def fix(request):
                request.node.warn("T1", "hello")

            def test_hello(fix):
                pass
        """)
        result = testdir.runpytest("--disable-pytest-warnings")
        assert result.parseoutcomes()["warnings"] > 0
        assert "hello" not in result.stdout.str()

        result = testdir.runpytest()
        result.stdout.fnmatch_lines("""
            ===*warnings summary*===
            *test_warn_on_test_item_from_request.py::test_hello*
            *hello*
        """)


class TestRootdir(object):
    def test_simple_noini(self, tmpdir):
        assert get_common_ancestor([tmpdir]) == tmpdir
        a = tmpdir.mkdir("a")
        assert get_common_ancestor([a, tmpdir]) == tmpdir
        assert get_common_ancestor([tmpdir, a]) == tmpdir
        with tmpdir.as_cwd():
            assert get_common_ancestor([]) == tmpdir
            no_path = tmpdir.join('does-not-exist')
            assert get_common_ancestor([no_path]) == tmpdir
            assert get_common_ancestor([no_path.join('a')]) == tmpdir

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
        rootdir, inifile, inicfg = determine_setup(None, [b, a])
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

    def test_nothing(self, tmpdir, monkeypatch):
        monkeypatch.chdir(str(tmpdir))
        rootdir, inifile, inicfg = determine_setup(None, [tmpdir])
        assert rootdir == tmpdir
        assert inifile is None
        assert inicfg == {}

    def test_with_specific_inifile(self, tmpdir):
        inifile = tmpdir.ensure("pytest.ini")
        rootdir, inifile, inicfg = determine_setup(inifile, [tmpdir])
        assert rootdir == tmpdir


class TestOverrideIniArgs(object):
    @pytest.mark.parametrize("name", "setup.cfg tox.ini pytest.ini".split())
    def test_override_ini_names(self, testdir, name):
        testdir.tmpdir.join(name).write(py.std.textwrap.dedent("""
            [pytest]
            custom = 1.0"""))
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("custom", "")""")
        testdir.makepyfile("""
            def test_pass(pytestconfig):
                ini_val = pytestconfig.getini("custom")
                print('\\ncustom_option:%s\\n' % ini_val)""")

        result = testdir.runpytest("--override-ini", "custom=2.0", "-s")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["custom_option:2.0"])

        result = testdir.runpytest("--override-ini", "custom=2.0",
                                   "--override-ini=custom=3.0", "-s")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["custom_option:3.0"])

    def test_override_ini_pathlist(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                parser.addini("paths", "my new ini value", type="pathlist")""")
        testdir.makeini("""
            [pytest]
            paths=blah.py""")
        testdir.makepyfile("""
            import py.path
            def test_pathlist(pytestconfig):
                config_paths = pytestconfig.getini("paths")
                print(config_paths)
                for cpf in config_paths:
                    print('\\nuser_path:%s' % cpf.basename)""")
        result = testdir.runpytest("--override-ini",
                                   'paths=foo/bar1.py foo/bar2.py', "-s")
        result.stdout.fnmatch_lines(["user_path:bar1.py",
                                     "user_path:bar2.py"])

    def test_override_multiple_and_default(self, testdir):
        testdir.makeconftest("""
            def pytest_addoption(parser):
                addini = parser.addini
                addini("custom_option_1", "", default="o1")
                addini("custom_option_2", "", default="o2")
                addini("custom_option_3", "", default=False, type="bool")
                addini("custom_option_4", "", default=True, type="bool")""")
        testdir.makeini("""
            [pytest]
            custom_option_1=custom_option_1
            custom_option_2=custom_option_2""")
        testdir.makepyfile("""
            def test_multiple_options(pytestconfig):
                prefix = "custom_option"
                for x in range(1, 5):
                    ini_value=pytestconfig.getini("%s_%d" % (prefix, x))
                    print('\\nini%d:%s' % (x, ini_value))""")
        result = testdir.runpytest(
            "--override-ini", 'custom_option_1=fulldir=/tmp/user1',
            'custom_option_2=url=/tmp/user2?a=b&d=e',
            "-o", 'custom_option_3=True',
            "-o", 'custom_option_4=no', "-s")
        result.stdout.fnmatch_lines(["ini1:fulldir=/tmp/user1",
                                     "ini2:url=/tmp/user2?a=b&d=e",
                                     "ini3:True",
                                     "ini4:False"])

    def test_override_ini_usage_error_bad_style(self, testdir):
        testdir.makeini("""
            [pytest]
            xdist_strict=False
        """)
        result = testdir.runpytest("--override-ini", 'xdist_strict True', "-s")
        result.stderr.fnmatch_lines(["*ERROR* *expects option=value*"])

    @pytest.mark.parametrize('with_ini', [True, False])
    def test_override_ini_handled_asap(self, testdir, with_ini):
        """-o should be handled as soon as possible and always override what's in ini files (#2238)"""
        if with_ini:
            testdir.makeini("""
                [pytest]
                python_files=test_*.py
            """)
        testdir.makepyfile(unittest_ini_handle="""
            def test():
                pass
        """)
        result = testdir.runpytest("--override-ini", 'python_files=unittest_*.py')
        result.stdout.fnmatch_lines(["*1 passed in*"])

    def test_with_arg_outside_cwd_without_inifile(self, tmpdir, monkeypatch):
        monkeypatch.chdir(str(tmpdir))
        a = tmpdir.mkdir("a")
        b = tmpdir.mkdir("b")
        rootdir, inifile, inicfg = determine_setup(None, [a, b])
        assert rootdir == tmpdir
        assert inifile is None

    def test_with_arg_outside_cwd_with_inifile(self, tmpdir):
        a = tmpdir.mkdir("a")
        b = tmpdir.mkdir("b")
        inifile = a.ensure("pytest.ini")
        rootdir, parsed_inifile, inicfg = determine_setup(None, [a, b])
        assert rootdir == a
        assert inifile == parsed_inifile

    @pytest.mark.parametrize('dirs', ([], ['does-not-exist'],
                                      ['a/does-not-exist']))
    def test_with_non_dir_arg(self, dirs, tmpdir):
        with tmpdir.ensure(dir=True).as_cwd():
            rootdir, inifile, inicfg = determine_setup(None, dirs)
            assert rootdir == tmpdir
            assert inifile is None

    def test_with_existing_file_in_subdir(self, tmpdir):
        a = tmpdir.mkdir("a")
        a.ensure("exist")
        with tmpdir.as_cwd():
            rootdir, inifile, inicfg = determine_setup(None, ['a/exist'])
            assert rootdir == tmpdir
            assert inifile is None

    def test_addopts_before_initini(self, testdir, tmpdir, monkeypatch):
        cache_dir = '.custom_cache'
        monkeypatch.setenv('PYTEST_ADDOPTS', '-o cache_dir=%s' % cache_dir)
        from _pytest.config import get_config
        config = get_config()
        config._preparse([], addopts=True)
        assert config._override_ini == [['cache_dir=%s' % cache_dir]]
