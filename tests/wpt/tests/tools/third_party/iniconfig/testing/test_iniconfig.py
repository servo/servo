import py
import pytest
from iniconfig import IniConfig, ParseError, __all__ as ALL
from iniconfig import iscommentline
from textwrap import dedent


check_tokens = {
    'section': (
        '[section]',
        [(0, 'section', None, None)]
    ),
    'value': (
        'value = 1',
        [(0, None, 'value', '1')]
    ),
    'value in section': (
        '[section]\nvalue=1',
        [(0, 'section', None, None), (1, 'section', 'value', '1')]
    ),
    'value with continuation': (
        'names =\n Alice\n Bob',
        [(0, None, 'names', 'Alice\nBob')]
    ),
    'value with aligned continuation': (
        'names = Alice\n'
        '        Bob',
        [(0, None, 'names', 'Alice\nBob')]
    ),
    'blank line': (
        '[section]\n\nvalue=1',
        [(0, 'section', None, None), (2, 'section', 'value', '1')]
    ),
    'comment': (
        '# comment',
        []
    ),
    'comment on value': (
        'value = 1',
        [(0, None, 'value', '1')]
    ),

    'comment on section': (
        '[section] #comment',
        [(0, 'section', None, None)]
    ),
    'comment2': (
        '; comment',
        []
    ),

    'comment2 on section': (
        '[section] ;comment',
        [(0, 'section', None, None)]
    ),
    'pseudo section syntax in value': (
        'name = value []',
        [(0, None, 'name', 'value []')]
    ),
    'assignment in value': (
        'value = x = 3',
        [(0, None, 'value', 'x = 3')]
    ),
    'use of colon for name-values': (
        'name: y',
        [(0, None, 'name', 'y')]
    ),
    'use of colon without space': (
        'value:y=5',
        [(0, None, 'value', 'y=5')]
    ),
    'equality gets precedence': (
        'value=xyz:5',
        [(0, None, 'value', 'xyz:5')]
    ),

}


@pytest.fixture(params=sorted(check_tokens))
def input_expected(request):
    return check_tokens[request.param]


@pytest.fixture
def input(input_expected):
    return input_expected[0]


@pytest.fixture
def expected(input_expected):
    return input_expected[1]


def parse(input):
    # only for testing purposes - _parse() does not use state except path
    ini = object.__new__(IniConfig)
    ini.path = "sample"
    return ini._parse(input.splitlines(True))


def parse_a_error(input):
    return py.test.raises(ParseError, parse, input)


def test_tokenize(input, expected):
    parsed = parse(input)
    assert parsed == expected


def test_parse_empty():
    parsed = parse("")
    assert not parsed
    ini = IniConfig("sample", "")
    assert not ini.sections


def test_ParseError():
    e = ParseError("filename", 0, "hello")
    assert str(e) == "filename:1: hello"


def test_continuation_needs_perceeding_token():
    excinfo = parse_a_error(' Foo')
    assert excinfo.value.lineno == 0


def test_continuation_cant_be_after_section():
    excinfo = parse_a_error('[section]\n Foo')
    assert excinfo.value.lineno == 1


def test_section_cant_be_empty():
    excinfo = parse_a_error('[]')
    assert excinfo.value.lineno == 0


@py.test.mark.parametrize('line', [
    '!!',
    ])
def test_error_on_weird_lines(line):
    parse_a_error(line)


def test_iniconfig_from_file(tmpdir):
    path = tmpdir/'test.txt'
    path.write('[metadata]\nname=1')

    config = IniConfig(path=path)
    assert list(config.sections) == ['metadata']
    config = IniConfig(path, "[diff]")
    assert list(config.sections) == ['diff']
    with pytest.raises(TypeError):
        IniConfig(data=path.read())


def test_iniconfig_section_first(tmpdir):
    with pytest.raises(ParseError) as excinfo:
        IniConfig("x", data='name=1')
    assert excinfo.value.msg == "no section header defined"


def test_iniconig_section_duplicate_fails():
    with pytest.raises(ParseError) as excinfo:
        IniConfig("x", data='[section]\n[section]')
    assert 'duplicate section' in str(excinfo.value)


def test_iniconfig_duplicate_key_fails():
    with pytest.raises(ParseError) as excinfo:
        IniConfig("x", data='[section]\nname = Alice\nname = bob')

    assert 'duplicate name' in str(excinfo.value)


def test_iniconfig_lineof():
    config = IniConfig("x.ini", data=(
        '[section]\n'
        'value = 1\n'
        '[section2]\n'
        '# comment\n'
        'value =2'
    ))

    assert config.lineof('missing') is None
    assert config.lineof('section') == 1
    assert config.lineof('section2') == 3
    assert config.lineof('section', 'value') == 2
    assert config.lineof('section2', 'value') == 5

    assert config['section'].lineof('value') == 2
    assert config['section2'].lineof('value') == 5


def test_iniconfig_get_convert():
    config = IniConfig("x", data='[section]\nint = 1\nfloat = 1.1')
    assert config.get('section', 'int') == '1'
    assert config.get('section', 'int', convert=int) == 1


def test_iniconfig_get_missing():
    config = IniConfig("x", data='[section]\nint = 1\nfloat = 1.1')
    assert config.get('section', 'missing', default=1) == 1
    assert config.get('section', 'missing') is None


def test_section_get():
    config = IniConfig("x", data='[section]\nvalue=1')
    section = config['section']
    assert section.get('value', convert=int) == 1
    assert section.get('value', 1) == "1"
    assert section.get('missing', 2) == 2


def test_missing_section():
    config = IniConfig("x", data='[section]\nvalue=1')
    with pytest.raises(KeyError):
            config["other"]


def test_section_getitem():
    config = IniConfig("x", data='[section]\nvalue=1')
    assert config['section']['value'] == '1'
    assert config['section']['value'] == '1'


def test_section_iter():
    config = IniConfig("x", data='[section]\nvalue=1')
    names = list(config['section'])
    assert names == ['value']
    items = list(config['section'].items())
    assert items == [('value', '1')]


def test_config_iter():
    config = IniConfig("x.ini", data=dedent('''
          [section1]
          value=1
          [section2]
          value=2
    '''))
    l = list(config)
    assert len(l) == 2
    assert l[0].name == 'section1'
    assert l[0]['value'] == '1'
    assert l[1].name == 'section2'
    assert l[1]['value'] == '2'


def test_config_contains():
    config = IniConfig("x.ini", data=dedent('''
          [section1]
          value=1
          [section2]
          value=2
    '''))
    assert 'xyz' not in config
    assert 'section1' in config
    assert 'section2' in config


def test_iter_file_order():
    config = IniConfig("x.ini", data="""
[section2] #cpython dict ordered before section
value = 1
value2 = 2 # dict ordered before value
[section]
a = 1
b = 2
""")
    l = list(config)
    secnames = [x.name for x in l]
    assert secnames == ['section2', 'section']
    assert list(config['section2']) == ['value', 'value2']
    assert list(config['section']) == ['a', 'b']


def test_example_pypirc():
    config = IniConfig("pypirc", data=dedent('''
        [distutils]
        index-servers =
            pypi
            other

        [pypi]
        repository: <repository-url>
        username: <username>
        password: <password>

        [other]
        repository: http://example.com/pypi
        username: <username>
        password: <password>
    '''))
    distutils, pypi, other = list(config)
    assert distutils["index-servers"] == "pypi\nother"
    assert pypi['repository'] == '<repository-url>'
    assert pypi['username'] == '<username>'
    assert pypi['password'] == '<password>'
    assert ['repository', 'username', 'password'] == list(other)


def test_api_import():
    assert ALL == ['IniConfig', 'ParseError']


@pytest.mark.parametrize("line", [
    "#qwe",
    "  #qwe",
    ";qwe",
    " ;qwe",
])
def test_iscommentline_true(line):
    assert iscommentline(line)
