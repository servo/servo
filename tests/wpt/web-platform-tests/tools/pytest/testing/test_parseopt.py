from __future__ import with_statement
import sys
import os
import py, pytest
from _pytest import config as parseopt

@pytest.fixture
def parser():
    return parseopt.Parser()

class TestParser:
    def test_no_help_by_default(self, capsys):
        parser = parseopt.Parser(usage="xyz")
        pytest.raises(SystemExit, lambda: parser.parse(["-h"]))
        out, err = capsys.readouterr()
        assert err.find("error: unrecognized arguments") != -1

    def test_argument(self):
        with pytest.raises(parseopt.ArgumentError):
            # need a short or long option
            argument = parseopt.Argument()
        argument = parseopt.Argument('-t')
        assert argument._short_opts == ['-t']
        assert argument._long_opts == []
        assert argument.dest == 't'
        argument = parseopt.Argument('-t', '--test')
        assert argument._short_opts == ['-t']
        assert argument._long_opts == ['--test']
        assert argument.dest == 'test'
        argument = parseopt.Argument('-t', '--test', dest='abc')
        assert argument.dest == 'abc'

    def test_argument_type(self):
        argument = parseopt.Argument('-t', dest='abc', type='int')
        assert argument.type is int
        argument = parseopt.Argument('-t', dest='abc', type='string')
        assert argument.type is str
        argument = parseopt.Argument('-t', dest='abc', type=float)
        assert argument.type is float
        with pytest.raises(KeyError):
            argument = parseopt.Argument('-t', dest='abc', type='choice')
        argument = parseopt.Argument('-t', dest='abc', type='choice',
                                     choices=['red', 'blue'])
        assert argument.type is str

    def test_argument_processopt(self):
        argument = parseopt.Argument('-t', type=int)
        argument.default = 42
        argument.dest = 'abc'
        res = argument.attrs()
        assert res['default'] == 42
        assert res['dest'] == 'abc'

    def test_group_add_and_get(self, parser):
        group = parser.getgroup("hello", description="desc")
        assert group.name == "hello"
        assert group.description == "desc"

    def test_getgroup_simple(self, parser):
        group = parser.getgroup("hello", description="desc")
        assert group.name == "hello"
        assert group.description == "desc"
        group2 = parser.getgroup("hello")
        assert group2 is group

    def test_group_ordering(self, parser):
        parser.getgroup("1")
        parser.getgroup("2")
        parser.getgroup("3", after="1")
        groups = parser._groups
        groups_names = [x.name for x in groups]
        assert groups_names == list("132")

    def test_group_addoption(self):
        group = parseopt.OptionGroup("hello")
        group.addoption("--option1", action="store_true")
        assert len(group.options) == 1
        assert isinstance(group.options[0], parseopt.Argument)

    def test_group_shortopt_lowercase(self, parser):
        group = parser.getgroup("hello")
        pytest.raises(ValueError, """
            group.addoption("-x", action="store_true")
        """)
        assert len(group.options) == 0
        group._addoption("-x", action="store_true")
        assert len(group.options) == 1

    def test_parser_addoption(self, parser):
        group = parser.getgroup("custom options")
        assert len(group.options) == 0
        group.addoption("--option1", action="store_true")
        assert len(group.options) == 1

    def test_parse(self, parser):
        parser.addoption("--hello", dest="hello", action="store")
        args = parser.parse(['--hello', 'world'])
        assert args.hello == "world"
        assert not getattr(args, parseopt.FILE_OR_DIR)

    def test_parse2(self, parser):
        args = parser.parse([py.path.local()])
        assert getattr(args, parseopt.FILE_OR_DIR)[0] == py.path.local()

    def test_parse_known_args(self, parser):
        parser.parse_known_args([py.path.local()])
        parser.addoption("--hello", action="store_true")
        ns = parser.parse_known_args(["x", "--y", "--hello", "this"])
        assert ns.hello
        assert ns.file_or_dir == ['x']

    def test_parse_known_and_unknown_args(self, parser):
        parser.addoption("--hello", action="store_true")
        ns, unknown = parser.parse_known_and_unknown_args(["x", "--y",
                                                           "--hello", "this"])
        assert ns.hello
        assert ns.file_or_dir == ['x']
        assert unknown == ['--y', 'this']

    def test_parse_will_set_default(self, parser):
        parser.addoption("--hello", dest="hello", default="x", action="store")
        option = parser.parse([])
        assert option.hello == "x"
        del option.hello
        parser.parse_setoption([], option)
        assert option.hello == "x"

    def test_parse_setoption(self, parser):
        parser.addoption("--hello", dest="hello", action="store")
        parser.addoption("--world", dest="world", default=42)
        class A: pass
        option = A()
        args = parser.parse_setoption(['--hello', 'world'], option)
        assert option.hello == "world"
        assert option.world == 42
        assert not args

    def test_parse_special_destination(self, parser):
        parser.addoption("--ultimate-answer", type=int)
        args = parser.parse(['--ultimate-answer', '42'])
        assert args.ultimate_answer == 42

    def test_parse_split_positional_arguments(self, parser):
        parser.addoption("-R", action='store_true')
        parser.addoption("-S", action='store_false')
        args = parser.parse(['-R', '4', '2', '-S'])
        assert getattr(args, parseopt.FILE_OR_DIR) == ['4', '2']
        args = parser.parse(['-R', '-S', '4', '2', '-R'])
        assert getattr(args, parseopt.FILE_OR_DIR) == ['4', '2']
        assert args.R == True
        assert args.S == False
        args = parser.parse(['-R', '4', '-S', '2'])
        assert getattr(args, parseopt.FILE_OR_DIR) == ['4', '2']
        assert args.R == True
        assert args.S == False

    def test_parse_defaultgetter(self):
        def defaultget(option):
            if not hasattr(option, 'type'):
                return
            if option.type is int:
                option.default = 42
            elif option.type is str:
                option.default = "world"
        parser = parseopt.Parser(processopt=defaultget)
        parser.addoption("--this", dest="this", type="int", action="store")
        parser.addoption("--hello", dest="hello", type="string", action="store")
        parser.addoption("--no", dest="no", action="store_true")
        option = parser.parse([])
        assert option.hello == "world"
        assert option.this == 42
        assert option.no is False

    def test_drop_short_helper(self):
        parser = py.std.argparse.ArgumentParser(formatter_class=parseopt.DropShorterLongHelpFormatter)
        parser.add_argument('-t', '--twoword', '--duo', '--two-word', '--two',
                             help='foo').map_long_option = {'two': 'two-word'}
        # throws error on --deux only!
        parser.add_argument('-d', '--deuxmots', '--deux-mots',
                            action='store_true', help='foo').map_long_option = {'deux': 'deux-mots'}
        parser.add_argument('-s', action='store_true', help='single short')
        parser.add_argument('--abc', '-a',
                            action='store_true', help='bar')
        parser.add_argument('--klm', '-k', '--kl-m',
                            action='store_true', help='bar')
        parser.add_argument('-P', '--pq-r', '-p', '--pqr',
                            action='store_true', help='bar')
        parser.add_argument('--zwei-wort', '--zweiwort', '--zweiwort',
                            action='store_true', help='bar')
        parser.add_argument('-x', '--exit-on-first', '--exitfirst',
                            action='store_true', help='spam').map_long_option = {'exitfirst': 'exit-on-first'}
        parser.add_argument('files_and_dirs', nargs='*')
        args = parser.parse_args(['-k', '--duo', 'hallo', '--exitfirst'])
        assert args.twoword == 'hallo'
        assert args.klm is True
        assert args.zwei_wort is False
        assert args.exit_on_first is True
        assert args.s is False
        args = parser.parse_args(['--deux-mots'])
        with pytest.raises(AttributeError):
            assert args.deux_mots is True
        assert args.deuxmots is True
        args = parser.parse_args(['file', 'dir'])
        assert '|'.join(args.files_and_dirs) == 'file|dir'

    def test_drop_short_0(self, parser):
        parser.addoption('--funcarg', '--func-arg', action='store_true')
        parser.addoption('--abc-def', '--abc-def', action='store_true')
        parser.addoption('--klm-hij', action='store_true')
        args = parser.parse(['--funcarg', '--k'])
        assert args.funcarg is True
        assert args.abc_def is False
        assert args.klm_hij is True

    def test_drop_short_2(self, parser):
        parser.addoption('--func-arg', '--doit', action='store_true')
        args = parser.parse(['--doit'])
        assert args.func_arg is True

    def test_drop_short_3(self, parser):
        parser.addoption('--func-arg', '--funcarg', '--doit', action='store_true')
        args = parser.parse(['abcd'])
        assert args.func_arg is False
        assert args.file_or_dir == ['abcd']

    def test_drop_short_help0(self, parser, capsys):
        parser.addoption('--func-args', '--doit', help = 'foo',
                         action='store_true')
        parser.parse([])
        help = parser.optparser.format_help()
        assert '--func-args, --doit  foo' in  help

    # testing would be more helpful with all help generated
    def test_drop_short_help1(self, parser, capsys):
        group = parser.getgroup("general")
        group.addoption('--doit', '--func-args', action='store_true', help='foo')
        group._addoption("-h", "--help", action="store_true", dest="help",
                help="show help message and configuration info")
        parser.parse(['-h'])
        help = parser.optparser.format_help()
        assert '-doit, --func-args  foo' in  help


def test_argcomplete(testdir, monkeypatch):
    if not py.path.local.sysfind('bash'):
        pytest.skip("bash not available")
    script = str(testdir.tmpdir.join("test_argcomplete"))
    pytest_bin = sys.argv[0]
    if "py.test" not in os.path.basename(pytest_bin):
        pytest.skip("need to be run with py.test executable, not %s" %(pytest_bin,))

    with open(str(script), 'w') as fp:
        # redirect output from argcomplete to stdin and stderr is not trivial
        # http://stackoverflow.com/q/12589419/1307905
        # so we use bash
        fp.write('COMP_WORDBREAKS="$COMP_WORDBREAKS" %s 8>&1 9>&2' % pytest_bin)
    # alternative would be exteneded Testdir.{run(),_run(),popen()} to be able
    # to handle a keyword argument env that replaces os.environ in popen or
    # extends the copy, advantage: could not forget to restore
    monkeypatch.setenv('_ARGCOMPLETE', "1")
    monkeypatch.setenv('_ARGCOMPLETE_IFS',"\x0b")
    monkeypatch.setenv('COMP_WORDBREAKS', ' \\t\\n"\\\'><=;|&(:')

    arg = '--fu'
    monkeypatch.setenv('COMP_LINE', "py.test " + arg)
    monkeypatch.setenv('COMP_POINT', str(len("py.test " + arg)))
    result = testdir.run('bash', str(script), arg)
    if result.ret == 255:
        # argcomplete not found
        pytest.skip("argcomplete not available")
    elif not result.stdout.str():
        pytest.skip("bash provided no output, argcomplete not available?")
    else:
        if py.std.sys.version_info < (2,7):
            result.stdout.lines = result.stdout.lines[0].split('\x0b')
            result.stdout.fnmatch_lines(["--funcargs", "--fulltrace"])
        else:
            result.stdout.fnmatch_lines(["--funcargs", "--fulltrace"])
    if py.std.sys.version_info < (2,7):
        return
    os.mkdir('test_argcomplete.d')
    arg = 'test_argc'
    monkeypatch.setenv('COMP_LINE', "py.test " + arg)
    monkeypatch.setenv('COMP_POINT', str(len('py.test ' + arg)))
    result = testdir.run('bash', str(script), arg)
    result.stdout.fnmatch_lines(["test_argcomplete", "test_argcomplete.d/"])

