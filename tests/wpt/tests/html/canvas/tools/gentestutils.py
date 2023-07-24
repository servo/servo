# Current code status:
#
# This was originally written by Philip Taylor for use at
# http://philip.html5.org/tests/canvas/suite/tests/
#
# It has been adapted for use with the Web Platform Test Suite suite at
# https://github.com/web-platform-tests/wpt/
#
# The original version had a number of now-removed features (multiple versions
# of each test case of varying verbosity, Mozilla mochitests, semi-automated
# test harness). It also had a different directory structure.

# To update or add test cases:
#
# * Modify the tests*.yaml files.
#  - 'name' is an arbitrary hierarchical name to help categorise tests.
#  - 'desc' is a rough description of what behaviour the test aims to test.
#  - 'code' is JavaScript code to execute, with some special commands starting
#    with '@'
#  - 'expected' is what the final canvas output should be: a string 'green' or
#    'clear' (100x50 images in both cases), or a string 'size 100 50' (or any
#    other size) followed by Python code using Pycairo to generate the image.
#
# * Run "./build.sh".
# This requires a few Python modules which might not be ubiquitous.
# It will usually emit some warnings, which ideally should be fixed but can
# generally be safely ignored.
#
# * Test the tests, add new ones to Git, remove deleted ones from Git, etc.

from typing import List, Mapping

import re
import importlib
import os
import pathlib
import sys
import textwrap

try:
    import cairocffi as cairo  # type: ignore
except ImportError:
    import cairo

try:
    # Compatible and lots faster.
    import syck as yaml  # type: ignore
except ImportError:
    import yaml


class Error(Exception):
    """Base class for all exceptions raised by this module"""


class InvalidTestDefinitionError(Error):
    """Raised on invalid test definition."""


def _simpleEscapeJS(string: str) -> str:
    return string.replace('\\', '\\\\').replace('"', '\\"')


def _escapeJS(string: str) -> str:
    string = _simpleEscapeJS(string)
    # Kind of an ugly hack, for nicer failure-message output.
    string = re.sub(r'\[(\w+)\]', r'[\\""+(\1)+"\\"]', string)
    return string


def _expand_nonfinite(method: str, argstr: str, tail: str) -> str:
    """
    >>> print _expand_nonfinite('f', '<0 a>, <0 b>', ';')
    f(a, 0);
    f(0, b);
    f(a, b);
    >>> print _expand_nonfinite('f', '<0 a>, <0 b c>, <0 d>', ';')
    f(a, 0, 0);
    f(0, b, 0);
    f(0, c, 0);
    f(0, 0, d);
    f(a, b, 0);
    f(a, b, d);
    f(a, 0, d);
    f(0, b, d);
    """
    # argstr is "<valid-1 invalid1-1 invalid2-1 ...>, ..." (where usually
    # 'invalid' is Infinity/-Infinity/NaN).
    args = []
    for arg in argstr.split(', '):
        match = re.match('<(.*)>', arg)
        if match is None:
            raise InvalidTestDefinitionError(
                f"Expected arg to match format '<(.*)>', but was: {arg}")
        a = match.group(1)
        args.append(a.split(' '))
    calls = []
    # Start with the valid argument list.
    call = [args[j][0] for j in range(len(args))]
    # For each argument alone, try setting it to all its invalid values:
    for i in range(len(args)):
        for a in args[i][1:]:
            c2 = call[:]
            c2[i] = a
            calls.append(c2)
    # For all combinations of >= 2 arguments, try setting them to their
    # first invalid values. (Don't do all invalid values, because the
    # number of combinations explodes.)
    def f(c: List[str], start: int, depth: int) -> None:
        for i in range(start, len(args)):
            if len(args[i]) > 1:
                a = args[i][1]
                c2 = c[:]
                c2[i] = a
                if depth > 0:
                    calls.append(c2)
                f(c2, i + 1, depth + 1)

    f(call, 0, 0)

    return '\n'.join('%s(%s)%s' % (method, ', '.join(c), tail) for c in calls)


def _get_test_sub_dir(name: str, name_to_sub_dir: Mapping[str, str]) -> str:
    for prefix in sorted(name_to_sub_dir.keys(), key=len, reverse=True):
        if name.startswith(prefix):
            return name_to_sub_dir[prefix]
    raise InvalidTestDefinitionError(
        'Test "%s" has no defined target directory mapping' % name)


def _expand_test_code(code: str) -> str:
    code = re.sub(r'@nonfinite ([^(]+)\(([^)]+)\)(.*)', lambda m:
                  _expand_nonfinite(m.group(1), m.group(2), m.group(3)),
                  code)  # Must come before '@assert throws'.

    code = re.sub(r'@assert pixel (\d+,\d+) == (\d+,\d+,\d+,\d+);',
                  r'_assertPixel(canvas, \1, \2);', code)

    code = re.sub(r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+);',
                  r'_assertPixelApprox(canvas, \1, \2, 2);', code)

    code = re.sub(r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+) \+/- (\d+);',
                  r'_assertPixelApprox(canvas, \1, \2, \3);', code)

    code = re.sub(r'@assert throws (\S+_ERR) (.*);',
                  r'assert_throws_dom("\1", function() { \2; });', code)

    code = re.sub(r'@assert throws (\S+Error) (.*);',
                  r'assert_throws_js(\1, function() { \2; });', code)

    code = re.sub(
        r'@assert (.*) === (.*);', lambda m: '_assertSame(%s, %s, "%s", "%s");'
        % (m.group(1), m.group(2), _escapeJS(m.group(1)), _escapeJS(m.group(2))
           ), code)

    code = re.sub(
        r'@assert (.*) !== (.*);', lambda m:
        '_assertDifferent(%s, %s, "%s", "%s");' % (m.group(1), m.group(
            2), _escapeJS(m.group(1)), _escapeJS(m.group(2))), code)

    code = re.sub(
        r'@assert (.*) =~ (.*);', lambda m: 'assert_regexp_match(%s, %s);' % (
            m.group(1), m.group(2)), code)

    code = re.sub(
        r'@assert (.*);', lambda m: '_assert(%s, "%s");' % (m.group(
            1), _escapeJS(m.group(1))), code)

    code = re.sub(r' @moz-todo', '', code)

    code = re.sub(r'@moz-UniversalBrowserRead;', '', code)

    assert ('@' not in code)

    return code


_CANVAS_SIZE_REGEX = re.compile(r'(?P<width>.*), (?P<height>.*)',
                                re.MULTILINE | re.DOTALL)


def _get_canvas_size(test: Mapping[str, str]):
    size = test.get('size', '100, 50')
    match = _CANVAS_SIZE_REGEX.match(size)
    if not match:
        raise InvalidTestDefinitionError(
            'Invalid canvas size "%s" in test %s. Expected a string matching '
            'this pattern: "%%s, %%s" %% (width, height)' %
            (size, test['name']))
    return match.group('width'), match.group('height')


def _generate_test(test: Mapping[str, str], templates: Mapping[str, str],
                   sub_dir: str, test_output_dir: str, image_output_dir: str,
                   is_offscreen_canvas: bool):
    name = test['name']

    if test.get('expected', '') == 'green' and re.search(
            r'@assert pixel .* 0,0,0,0;', test['code']):
        print('Probable incorrect pixel test in %s' % name)

    code = _expand_test_code(test['code'].strip())
    code = textwrap.indent(code, '  ')

    expectation_html = ''
    if 'expected' in test and test['expected'] is not None:
        expected = test['expected']
        expected_img = None
        if expected == 'green':
            expected_img = '/images/green-100x50.png'
        elif expected == 'clear':
            expected_img = '/images/clear-100x50.png'
        else:
            if ';' in expected:
                print('Found semicolon in %s' % name)
            expected = re.sub(
                r'^size (\d+) (\d+)',
                r'surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, \1, \2)'
                r'\ncr = cairo.Context(surface)', expected)

            expected += ("\nsurface.write_to_png('%s.png')\n" %
                         os.path.join(image_output_dir, sub_dir, name))
            eval(compile(expected, '<test %s>' % name, 'exec'), {},
                 {'cairo': cairo})
            expected_img = '%s.png' % name

        if expected_img:
            expectation_html = (
                '<p class="output expectedtext">Expected output:<p>'
                '<img src="%s" class="output expected" id="expected" '
                'alt="">' % expected_img)

    canvas = ' ' + test['canvas'] if 'canvas' in test else ''
    width, height = _get_canvas_size(test)

    notes = '<p class="notes">%s' % test['notes'] if 'notes' in test else ''

    timeout = ('\n<meta name="timeout" content="%s">' %
               test['timeout'] if 'timeout' in test else '')
    timeout_js = ('// META: timeout=%s\n' % test['timeout']
                  if 'timeout' in test else '')

    images = ''
    for src in test.get('images', []):
        img_id = src.split('/')[-1]
        if '/' not in src:
            src = '../images/%s' % src
        images += '<img src="%s" id="%s" class="resource">\n' % (src, img_id)
    for src in test.get('svgimages', []):
        img_id = src.split('/')[-1]
        if '/' not in src:
            src = '../images/%s' % src
        images += ('<svg><image xlink:href="%s" id="%s" class="resource">'
                   '</svg>\n' % (src, img_id))
    images = images.replace('../images/', '/images/')

    fonts = ''
    fonthack = ''
    for font in test.get('fonts', []):
        fonts += ('@font-face {\n  font-family: %s;\n'
                  '  src: url("/fonts/%s.ttf");\n}\n' % (font, font))
        # Browsers require the font to actually be used in the page.
        if test.get('fonthack', 1):
            fonthack += ('<span style="font-family: %s; position: '
                         'absolute; visibility: hidden">A</span>\n' % font)
    if fonts:
        fonts = '<style>\n%s</style>\n' % fonts

    fallback = test.get('fallback',
                        '<p class="fallback">FAIL (fallback content)</p>')

    desc = test.get('desc', '')
    escaped_desc = _simpleEscapeJS(desc)

    attributes = test.get('attributes', '')
    if attributes:
        context_args = "'2d', %s" % attributes.strip()
        attributes = ', ' + attributes.strip()
    else:
        context_args = "'2d'"

    template_params = {
        'name': name,
        'desc': desc,
        'escaped_desc': escaped_desc,
        'notes': notes,
        'images': images,
        'fonts': fonts,
        'fonthack': fonthack,
        'timeout': timeout,
        'timeout_js': timeout_js,
        'canvas': canvas,
        'width': width,
        'height': height,
        'expected': expectation_html,
        'code': code,
        'fallback': fallback,
        'attributes': attributes,
        'context_args': context_args
    }

    test_path = os.path.join(test_output_dir, sub_dir, name)
    if 'manual' in test:
        test_path += '-manual'

    if is_offscreen_canvas:
        pathlib.Path(f'{test_path}.html').write_text(
            templates['offscreen'] % template_params, 'utf-8')
        pathlib.Path(f'{test_path}.worker.js').write_text(
            templates['worker'] % template_params, 'utf-8')
    else:
        pathlib.Path(f'{test_path}.html').write_text(
            templates['element'] % template_params, 'utf-8')


def genTestUtils(TESTOUTPUTDIR: str, IMAGEOUTPUTDIR: str, TEMPLATEFILE: str,
                 NAME2DIRFILE: str, ISOFFSCREENCANVAS: bool) -> None:
    # Run with --test argument to run unit tests
    if len(sys.argv) > 1 and sys.argv[1] == '--test':
        doctest = importlib.import_module('doctest')
        doctest.testmod()
        sys.exit()

    templates = yaml.safe_load(pathlib.Path(TEMPLATEFILE).read_text())
    name_to_sub_dir = yaml.safe_load(pathlib.Path(NAME2DIRFILE).read_text())

    tests = []
    test_yaml_directory = 'yaml/element'
    if ISOFFSCREENCANVAS:
        test_yaml_directory = 'yaml/offscreen'
    TESTSFILES = [
        os.path.join(test_yaml_directory, f)
        for f in os.listdir(test_yaml_directory) if f.endswith('.yaml')
    ]
    for t in sum(
        [yaml.safe_load(pathlib.Path(f).read_text()) for f in TESTSFILES], []):
        if 'DISABLED' in t:
            continue
        if 'meta' in t:
            eval(compile(t['meta'], '<meta test>', 'exec'), {},
                 {'tests': tests})
        else:
            tests.append(t)

    # Ensure the test output directories exist.
    testdirs = [TESTOUTPUTDIR, IMAGEOUTPUTDIR]
    for sub_dir in set(name_to_sub_dir.values()):
        testdirs.append('%s/%s' % (TESTOUTPUTDIR, sub_dir))
    for d in testdirs:
        try:
            os.mkdir(d)
        except FileExistsError:
            pass  # Ignore if it already exists.

    used_tests = {}
    for test in tests:
        name = test['name']
        print('\r(%s)' % name, ' ' * 32, '\t')

        if name in used_tests:
            print('Test %s is defined twice' % name)
        used_tests[name] = 1

        sub_dir = _get_test_sub_dir(name, name_to_sub_dir)
        _generate_test(test, templates, sub_dir, TESTOUTPUTDIR, IMAGEOUTPUTDIR,
                       ISOFFSCREENCANVAS)

    print()
