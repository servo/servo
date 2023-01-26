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

from typing import List, Optional

import re
import codecs
import importlib
import os
import sys

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


def genTestUtils(TESTOUTPUTDIR: str, IMAGEOUTPUTDIR: str, TEMPLATEFILE: str,
                 NAME2DIRFILE: str, ISOFFSCREENCANVAS: bool) -> None:

    MISCOUTPUTDIR = './output'

    def simpleEscapeJS(string: str) -> str:
        return string.replace('\\', '\\\\').replace('"', '\\"')

    def escapeJS(string: str) -> str:
        string = simpleEscapeJS(string)
        # Kind of an ugly hack, for nicer failure-message output.
        string = re.sub(r'\[(\w+)\]', r'[\\""+(\1)+"\\"]', string)
        return string

    def expand_nonfinite(method: str, argstr: str, tail: str) -> str:
        """
        >>> print expand_nonfinite('f', '<0 a>, <0 b>', ';')
        f(a, 0);
        f(0, b);
        f(a, b);
        >>> print expand_nonfinite('f', '<0 a>, <0 b c>, <0 d>', ';')
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

        return '\n'.join('%s(%s)%s' % (method, ', '.join(c), tail)
                         for c in calls)

    # Run with --test argument to run unit tests
    if len(sys.argv) > 1 and sys.argv[1] == '--test':
        doctest = importlib.import_module('doctest')
        doctest.testmod()
        sys.exit()

    templates = yaml.safe_load(open(TEMPLATEFILE, 'r').read())
    name_mapping = yaml.safe_load(open(NAME2DIRFILE, 'r').read())

    tests = []
    test_yaml_directory = 'yaml/element'
    if ISOFFSCREENCANVAS:
        test_yaml_directory = 'yaml/offscreen'
    TESTSFILES = [
        os.path.join(test_yaml_directory, f)
        for f in os.listdir(test_yaml_directory) if f.endswith('.yaml')
    ]
    for t in sum([yaml.safe_load(open(f, 'r').read()) for f in TESTSFILES],
                 []):
        if 'DISABLED' in t:
            continue
        if 'meta' in t:
            eval(compile(t['meta'], '<meta test>', 'exec'), {},
                 {'tests': tests})
        else:
            tests.append(t)

    category_names = []
    category_contents_direct = {}
    category_contents_all = {}

    def backref_html(name: str) -> str:
        backrefs = []
        c = ''
        for p in name.split('.')[:-1]:
            c += '.' + p
            backrefs.append('<a href="index%s.html">%s</a>.' % (c, p))
        backrefs.append(name.split('.')[-1])
        return ''.join(backrefs)

    # Ensure the test output directories exist.
    testdirs = [TESTOUTPUTDIR, IMAGEOUTPUTDIR, MISCOUTPUTDIR]
    for map_dir in set(name_mapping.values()):
        testdirs.append('%s/%s' % (TESTOUTPUTDIR, map_dir))
    for d in testdirs:
        try:
            os.mkdir(d)
        except FileExistsError:
            pass  # Ignore if it already exists.

    used_images = {}

    def map_name(name: str) -> Optional[str]:
        mapped_name = None
        for mn in sorted(name_mapping.keys(), key=len, reverse=True):
            if name.startswith(mn):
                mapped_name = '%s/%s' % (name_mapping[mn], name)
                break
        if not mapped_name:
            print('LIKELY ERROR: %s has no defined target directory mapping' %
                  name)
            return None
        if 'manual' in test:
            mapped_name += '-manual'
        return mapped_name

    def expand_test_code(code: str) -> str:
        code = re.sub(r'@nonfinite ([^(]+)\(([^)]+)\)(.*)', lambda m:
                      expand_nonfinite(m.group(1), m.group(2), m.group(3)),
                      code)  # Must come before '@assert throws'.

        code = re.sub(r'@assert pixel (\d+,\d+) == (\d+,\d+,\d+,\d+);',
                      r'_assertPixel(canvas, \1, \2);', code)

        code = re.sub(r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+);',
                      r'_assertPixelApprox(canvas, \1, \2, 2);', code)

        code = re.sub(
            r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+) \+/- (\d+);',
            r'_assertPixelApprox(canvas, \1, \2, \3);', code)

        code = re.sub(r'@assert throws (\S+_ERR) (.*);',
                      r'assert_throws_dom("\1", function() { \2; });', code)

        code = re.sub(r'@assert throws (\S+Error) (.*);',
                      r'assert_throws_js(\1, function() { \2; });', code)

        code = re.sub(
            r'@assert (.*) === (.*);', lambda m:
            '_assertSame(%s, %s, "%s", "%s");' % (m.group(1), m.group(
                2), escapeJS(m.group(1)), escapeJS(m.group(2))), code)

        code = re.sub(
            r'@assert (.*) !== (.*);', lambda m:
            '_assertDifferent(%s, %s, "%s", "%s");' % (m.group(1), m.group(
                2), escapeJS(m.group(1)), escapeJS(m.group(2))), code)

        code = re.sub(
            r'@assert (.*) =~ (.*);', lambda m: 'assert_regexp_match(%s, %s);'
            % (m.group(1), m.group(2)), code)

        code = re.sub(
            r'@assert (.*);', lambda m: '_assert(%s, "%s");' % (m.group(
                1), escapeJS(m.group(1))), code)

        code = re.sub(r' @moz-todo', '', code)

        code = re.sub(r'@moz-UniversalBrowserRead;', '', code)

        assert ('@' not in code)

        return code

    used_tests = {}
    for i in range(len(tests)):
        test = tests[i]

        name = test['name']
        print('\r(%s)' % name, ' ' * 32, '\t')

        if name in used_tests:
            print('Test %s is defined twice' % name)
        used_tests[name] = 1

        mapped_name = map_name(name)
        if not mapped_name:
            if ISOFFSCREENCANVAS:
                continue
            mapped_name = name

        cat_total = ''
        for cat_part in [''] + name.split('.')[:-1]:
            cat_total += cat_part + '.'
            if not cat_total in category_names:
                category_names.append(cat_total)
            category_contents_all.setdefault(cat_total, []).append(name)
        category_contents_direct.setdefault(cat_total, []).append(name)

        if test.get('expected', '') == 'green' and re.search(
                r'@assert pixel .* 0,0,0,0;', test['code']):
            print('Probable incorrect pixel test in %s' % name)

        code = expand_test_code(test['code'])

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

                if mapped_name.endswith('-manual'):
                    png_name = mapped_name[:-len('-manual')]
                else:
                    png_name = mapped_name
                expected += "\nsurface.write_to_png('%s/%s.png')\n" % (
                    IMAGEOUTPUTDIR, png_name)
                eval(compile(expected, '<test %s>' % test['name'], 'exec'), {},
                     {'cairo': cairo})
                expected_img = '%s.png' % name

            if expected_img:
                expectation_html = (
                    '<p class="output expectedtext">Expected output:<p>'
                    '<img src="%s" class="output expected" id="expected" '
                    'alt="">' % expected_img)

        canvas = test.get('canvas', 'width="100" height="50"')

        prev_test = tests[i - 1]['name'] if i != 0 else 'index'
        next_test = tests[i + 1]['name'] if i != len(tests) - 1 else 'index'

        name_wrapped = name.replace('.', '.&#8203;')

        notes = '<p class="notes">%s' % test['notes'] if 'notes' in test else ''

        timeout = ('\n<meta name="timeout" content="%s">' %
                   test['timeout'] if 'timeout' in test else '')

        scripts = ''
        for s in test.get('scripts', []):
            scripts += '<script src="%s"></script>\n' % (s)

        variants = test.get('script-variants', {})
        script_variants = [(v, '<script src="%s"></script>\n' % (s))
                           for (v, s) in variants.items()]
        if not script_variants:
            script_variants = [('', '')]

        images = ''
        for src in test.get('images', []):
            img_id = src.split('/')[-1]
            if '/' not in src:
                used_images[src] = 1
                src = '../images/%s' % src
            images += '<img src="%s" id="%s" class="resource">\n' % (src,
                                                                     img_id)
        for src in test.get('svgimages', []):
            img_id = src.split('/')[-1]
            if '/' not in src:
                used_images[src] = 1
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
        escaped_desc = simpleEscapeJS(desc)

        attributes = test.get('attributes', '')
        if attributes:
            context_args = "'2d', %s" % attributes.strip()
            attributes = ', ' + attributes.strip()
        else:
            context_args = "'2d'"

        for (variant, extra_script) in script_variants:
            name_variant = '' if not variant else '.' + variant

            template_params = {
                'name': name + name_variant,
                'name_wrapped': name_wrapped,
                'backrefs': backref_html(name),
                'mapped_name': mapped_name,
                'desc': desc,
                'escaped_desc': escaped_desc,
                'prev': prev_test,
                'next': next_test,
                'notes': notes,
                'images': images,
                'fonts': fonts,
                'fonthack': fonthack,
                'timeout': timeout,
                'canvas': canvas,
                'expected': expectation_html,
                'code': code,
                'scripts': scripts + extra_script,
                'fallback': fallback,
                'attributes': attributes,
                'context_args': context_args
            }
            if ISOFFSCREENCANVAS:
                f = codecs.open(
                    '%s/%s%s.html' %
                    (TESTOUTPUTDIR, mapped_name, name_variant), 'w', 'utf-8')
                f.write(templates['w3coffscreencanvas'] % template_params)
                timeout = ('// META: timeout=%s\n' %
                           test['timeout'] if 'timeout' in test else '')
                template_params['timeout'] = timeout
                f = codecs.open(
                    '%s/%s%s.worker.js' %
                    (TESTOUTPUTDIR, mapped_name, name_variant), 'w', 'utf-8')
                f.write(templates['w3cworker'] % template_params)
            else:
                f = codecs.open(
                    '%s/%s%s.html' %
                    (TESTOUTPUTDIR, mapped_name, name_variant), 'w', 'utf-8')
                f.write(templates['w3ccanvas'] % template_params)

    print()
