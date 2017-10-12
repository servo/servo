# Copyright (c) 2010 Philip Taylor
# Released under the BSD license and W3C Test Suite License: see LICENSE.txt

# Current code status:
#
# This was originally written for use at
# http://philip.html5.org/tests/canvas/suite/tests/
#
# It has been adapted for use with the Web Platform Test Suite suite at
# https://github.com/w3c/web-platform-tests/
#
# The original version had a number of now-removed features (multiple versions of
# each test case of varying verbosity, Mozilla mochitests, semi-automated test
# harness). It also had a different directory structure.

# To update or add test cases:
#
# * Modify the tests*.yaml files.
# 'name' is an arbitrary hierarchical name to help categorise tests.
# 'desc' is a rough description of what behaviour the test aims to test.
# 'testing' is a list of references to spec.yaml, to show which spec sentences
# this test case is primarily testing.
# 'code' is JavaScript code to execute, with some special commands starting with '@'
# 'expected' is what the final canvas output should be: a string 'green' or 'clear'
# (100x50 images in both cases), or a string 'size 100 50' (or any other size)
# followed by Python code using Pycairo to generate the image.
#
# * Run "./build.sh".
# This requires a few Python modules which might not be ubiquitous.
# It will usually emit some warnings, which ideally should be fixed but can
# generally be safely ignored.
#
# * Test the tests, add new ones to Git, remove deleted ones from Git, etc.

import re
import codecs
import time
import os
import shutil
import sys
import xml.dom.minidom
from xml.dom.minidom import Node

try:
    import cairocffi as cairo
except ImportError:
    import cairo

try:
    import syck as yaml # compatible and lots faster
except ImportError:
    import yaml

def genTestUtils(TESTOUTPUTDIR, IMAGEOUTPUTDIR, TEMPLATEFILE, NAME2DIRFILE, ISOFFSCREENCANVAS):

    MISCOUTPUTDIR = './output'
    SPECOUTPUTDIR = '../'

    SPECOUTPUTPATH = './' # relative to TESTOUTPUTDIR

    def simpleEscapeJS(str):
        return str.replace('\\', '\\\\').replace('"', '\\"')

    def escapeJS(str):
        str = simpleEscapeJS(str)
        str = re.sub(r'\[(\w+)\]', r'[\\""+(\1)+"\\"]', str) # kind of an ugly hack, for nicer failure-message output
        return str

    def escapeHTML(str):
        return str.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;').replace('"', '&quot;')

    def expand_nonfinite(method, argstr, tail):
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
        # 'invalid' is Infinity/-Infinity/NaN)
        args = []
        for arg in argstr.split(', '):
            a = re.match('<(.*)>', arg).group(1)
            args.append(a.split(' '))
        calls = []
        # Start with the valid argument list
        call = [ args[j][0] for j in range(len(args)) ]
        # For each argument alone, try setting it to all its invalid values:
        for i in range(len(args)):
            for a in args[i][1:]:
                c2 = call[:]
                c2[i] = a
                calls.append(c2)
        # For all combinations of >= 2 arguments, try setting them to their
        # first invalid values. (Don't do all invalid values, because the
        # number of combinations explodes.)
        def f(c, start, depth):
            for i in range(start, len(args)):
                if len(args[i]) > 1:
                    a = args[i][1]
                    c2 = c[:]
                    c2[i] = a
                    if depth > 0: calls.append(c2)
                    f(c2, i+1, depth+1)
        f(call, 0, 0)

        return '\n'.join('%s(%s)%s' % (method, ', '.join(c), tail) for c in calls)

    # Run with --test argument to run unit tests
    if len(sys.argv) > 1 and sys.argv[1] == '--test':
        import doctest
        doctest.testmod()
        sys.exit()

    templates = yaml.load(open(TEMPLATEFILE, "r").read())
    name_mapping = yaml.load(open(NAME2DIRFILE, "r").read())

    SPECFILE = 'spec.yaml'
    if ISOFFSCREENCANVAS:
        SPECFILE = '../../2dcontext/tools/spec.yaml'
    spec_assertions = []
    for s in yaml.load(open(SPECFILE, "r").read())['assertions']:
        if 'meta' in s:
            eval(compile(s['meta'], '<meta spec assertion>', 'exec'), {}, {'assertions':spec_assertions})
        else:
            spec_assertions.append(s)

    tests = []
    TESTSFILES = ['tests.yaml', 'tests2d.yaml', 'tests2dtext.yaml']
    if ISOFFSCREENCANVAS:
        TESTSFILES = ['tests2d.yaml']
    for t in sum([ yaml.load(open(f, "r").read()) for f in TESTSFILES], []):
        if 'DISABLED' in t:
            continue
        if 'meta' in t:
            eval(compile(t['meta'], '<meta test>', 'exec'), {}, {'tests':tests})
        else:
            tests.append(t)

    category_names = []
    category_contents_direct = {}
    category_contents_all = {}

    spec_ids = {}
    for t in spec_assertions: spec_ids[t['id']] = True
    spec_refs = {}

    def backref_html(name):
        backrefs = []
        c = ''
        for p in name.split('.')[:-1]:
            c += '.'+p
            backrefs.append('<a href="index%s.html">%s</a>.' % (c, p))
        backrefs.append(name.split('.')[-1])
        return ''.join(backrefs)

    def make_flat_image(filename, w, h, r,g,b,a):
        if os.path.exists('%s/%s' % (IMAGEOUTPUTDIR, filename)):
            return filename
        surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, w, h)
        cr = cairo.Context(surface)
        cr.set_source_rgba(r, g, b, a)
        cr.rectangle(0, 0, w, h)
        cr.fill()
        surface.write_to_png('%s/%s' % (IMAGEOUTPUTDIR, filename))
        return filename

    # Ensure the test output directories exist
    testdirs = [TESTOUTPUTDIR, IMAGEOUTPUTDIR, MISCOUTPUTDIR]
    for map_dir in set(name_mapping.values()):
        testdirs.append("%s/%s" % (TESTOUTPUTDIR, map_dir))
    for d in testdirs:
        try: os.mkdir(d)
        except: pass # ignore if it already exists

    used_images = {}

    def map_name(name):
        mapped_name = None
        for mn in sorted(name_mapping.keys(), key=len, reverse=True):
            if name.startswith(mn):
                mapped_name = "%s/%s" % (name_mapping[mn], name)
                break
        if not mapped_name:
            print "LIKELY ERROR: %s has no defined target directory mapping" % name
        if 'manual' in test:
            mapped_name += "-manual"
        return mapped_name

    def expand_test_code(code):
        code = re.sub(r'@nonfinite ([^(]+)\(([^)]+)\)(.*)', lambda m: expand_nonfinite(m.group(1), m.group(2), m.group(3)), code) # must come before '@assert throws'

        if ISOFFSCREENCANVAS:
            code = re.sub(r'@assert pixel (\d+,\d+) == (\d+,\d+,\d+,\d+);',
                    r'_assertPixel(offscreenCanvas, \1, \2, "\1", "\2");',
                    code)
        else:
            code = re.sub(r'@assert pixel (\d+,\d+) == (\d+,\d+,\d+,\d+);',
                    r'_assertPixel(canvas, \1, \2, "\1", "\2");',
                    code)

        if ISOFFSCREENCANVAS:
            code = re.sub(r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+);',
                    r'_assertPixelApprox(offscreenCanvas, \1, \2, "\1", "\2", 2);',
                    code)
        else:
            code = re.sub(r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+);',
                    r'_assertPixelApprox(canvas, \1, \2, "\1", "\2", 2);',
                    code)

        if ISOFFSCREENCANVAS:
            code = re.sub(r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+) \+/- (\d+);',
                    r'_assertPixelApprox(offscreenCanvas, \1, \2, "\1", "\2", \3);',
                    code)
        else:
            code = re.sub(r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+) \+/- (\d+);',
                    r'_assertPixelApprox(canvas, \1, \2, "\1", "\2", \3);',
                    code)

        code = re.sub(r'@assert throws (\S+_ERR) (.*);',
                r'assert_throws("\1", function() { \2; });',
                code)

        code = re.sub(r'@assert throws (\S+Error) (.*);',
                r'assert_throws(new \1(), function() { \2; });',
                code)

        code = re.sub(r'@assert (.*) === (.*);',
                lambda m: '_assertSame(%s, %s, "%s", "%s");'
                    % (m.group(1), m.group(2), escapeJS(m.group(1)), escapeJS(m.group(2)))
                , code)

        code = re.sub(r'@assert (.*) !== (.*);',
                lambda m: '_assertDifferent(%s, %s, "%s", "%s");'
                    % (m.group(1), m.group(2), escapeJS(m.group(1)), escapeJS(m.group(2)))
                , code)

        code = re.sub(r'@assert (.*) =~ (.*);',
                lambda m: 'assert_regexp_match(%s, %s);'
                    % (m.group(1), m.group(2))
                , code)

        code = re.sub(r'@assert (.*);',
                lambda m: '_assert(%s, "%s");'
                    % (m.group(1), escapeJS(m.group(1)))
                , code)

        code = re.sub(r' @moz-todo', '', code)

        code = re.sub(r'@moz-UniversalBrowserRead;',
                ""
                , code)

        assert('@' not in code)

        return code

    used_tests = {}
    for i in range(len(tests)):
        test = tests[i]

        name = test['name']
        print "\r(%s)" % name, " "*32, "\t",

        if name in used_tests:
            print "Test %s is defined twice" % name
        used_tests[name] = 1

        mapped_name = map_name(name)
        if not mapped_name:
            if ISOFFSCREENCANVAS:
                continue
            else:
                mapped_name = name


        cat_total = ''
        for cat_part in [''] + name.split('.')[:-1]:
            cat_total += cat_part+'.'
            if not cat_total in category_names: category_names.append(cat_total)
            category_contents_all.setdefault(cat_total, []).append(name)
        category_contents_direct.setdefault(cat_total, []).append(name)

        for ref in test.get('testing', []):
            if ref not in spec_ids:
                print "Test %s uses nonexistent spec point %s" % (name, ref)
            spec_refs.setdefault(ref, []).append(name)

        if not test.get('testing', []):
            print "Test %s doesn't refer to any spec points" % name

        if test.get('expected', '') == 'green' and re.search(r'@assert pixel .* 0,0,0,0;', test['code']):
            print "Probable incorrect pixel test in %s" % name

        code = expand_test_code(test['code'])

        expectation_html = ''
        if 'expected' in test and test['expected'] is not None:
            expected = test['expected']
            expected_img = None
            if expected == 'green':
                expected_img = "/images/" + make_flat_image('green-100x50.png', 100, 50, 0,1,0,1)
            elif expected == 'clear':
                expected_img = "/images/" + make_flat_image('clear-100x50.png', 100, 50, 0,0,0,0)
            else:
                if ';' in expected: print "Found semicolon in %s" % name
                expected = re.sub(r'^size (\d+) (\d+)',
                    r'surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, \1, \2)\ncr = cairo.Context(surface)',
                                  expected)

                if mapped_name.endswith("-manual"):
                    png_name = mapped_name[:-len("-manual")]
                else:
                    png_name = mapped_name
                expected += "\nsurface.write_to_png('%s/%s.png')\n" % (IMAGEOUTPUTDIR, png_name)
                eval(compile(expected, '<test %s>' % test['name'], 'exec'), {}, {'cairo':cairo})
                expected_img = "%s.png" % name

            if expected_img:
                expectation_html = ('<p class="output expectedtext">Expected output:' +
                    '<p><img src="%s" class="output expected" id="expected" alt="">' % (expected_img))

        canvas = test.get('canvas', 'width="100" height="50"')

        prev = tests[i-1]['name'] if i != 0 else 'index'
        next = tests[i+1]['name'] if i != len(tests)-1 else 'index'

        name_wrapped = name.replace('.', '.&#8203;')

        refs = ''.join('<li><a href="%s/annotated-spec.html#testrefs.%s">%s</a>\n' % (SPECOUTPUTPATH, n,n) for n in test.get('testing', []))

        notes = '<p class="notes">%s' % test['notes'] if 'notes' in test else ''

        scripts = ''
        for s in test.get('scripts', []):
            scripts += '<script src="%s"></script>\n' % (s)

        variants = test.get('script-variants', {})
        script_variants = [(v, '<script src="%s"></script>\n' % (s)) for (v, s) in variants.iteritems()]
        if not script_variants:
            script_variants = [('', '')]

        images = ''
        for i in test.get('images', []):
            id = i.split('/')[-1]
            if '/' not in i:
                used_images[i] = 1
                i = '../images/%s' % i
            images += '<img src="%s" id="%s" class="resource">\n' % (i,id)
        images = images.replace("../images/", "/images/")

        fonts = ''
        fonthack = ''
        for i in test.get('fonts', []):
            fonts += '@font-face {\n  font-family: %s;\n  src: url("/fonts/%s.ttf");\n}\n' % (i, i)
            # Browsers require the font to actually be used in the page
            if test.get('fonthack', 1):
                fonthack += '<span style="font-family: %s; position: absolute; visibility: hidden">A</span>\n' % i
        if fonts:
            fonts = '<style>\n%s</style>\n' % fonts

        fallback = test.get('fallback', '<p class="fallback">FAIL (fallback content)</p>')

        desc = test.get('desc', '')
        escaped_desc = simpleEscapeJS(desc)

        for (variant, extra_script) in script_variants:
            name_variant = '' if not variant else '.' + variant

            template_params = {
                'name':name + name_variant,
                'name_wrapped':name_wrapped, 'backrefs':backref_html(name),
                'mapped_name':mapped_name,
                'desc':desc, 'escaped_desc':escaped_desc,
                'prev':prev, 'next':next, 'refs':refs, 'notes':notes, 'images':images,
                'fonts':fonts, 'fonthack':fonthack,
                'canvas':canvas, 'expected':expectation_html, 'code':code,
                'scripts':scripts + extra_script,
                'fallback':fallback
            }

            f = codecs.open('%s/%s%s.html' % (TESTOUTPUTDIR, mapped_name, name_variant), 'w', 'utf-8')
            f.write(templates['w3c'] % template_params)
            if ISOFFSCREENCANVAS:
                f = codecs.open('%s/%s%s.worker.js' % (TESTOUTPUTDIR, mapped_name, name_variant), 'w', 'utf-8')
                f.write(templates['w3cworker'] % template_params)

    print


    def getNodeText(node):
        t, offsets = '', []

        # Skip over any previous annotations we added
        if node.nodeType == node.ELEMENT_NODE and 'testrefs' in node.getAttribute('class').split(' '):
            return t, offsets

        if node.nodeType == node.TEXT_NODE:
            val = node.nodeValue
            val = val.replace(unichr(0xa0), ' ') # replace &nbsp;s
            t += val
            offsets += [ (node, len(node.nodeValue)) ]
        for n in node.childNodes:
            child_t, child_offsets = getNodeText(n)
            t += child_t
            offsets += child_offsets
        return t, offsets

    def htmlSerializer(element):
        element.normalize()
        rv = []
        specialtext = ['style', 'script', 'xmp', 'iframe', 'noembed', 'noframes', 'noscript']
        empty = ['area', 'base', 'basefont', 'bgsound', 'br', 'col', 'embed', 'frame',
                 'hr', 'img', 'input', 'link', 'meta', 'param', 'spacer', 'wbr']

        def serializeElement(element):
            if element.nodeType == Node.DOCUMENT_TYPE_NODE:
                rv.append("<!DOCTYPE %s>" % element.name)
            elif element.nodeType == Node.DOCUMENT_NODE:
                for child in element.childNodes:
                    serializeElement(child)
            elif element.nodeType == Node.COMMENT_NODE:
                rv.append("<!--%s-->" % element.nodeValue)
            elif element.nodeType == Node.TEXT_NODE:
                unescaped = False
                n = element.parentNode
                while n is not None:
                    if n.nodeName in specialtext:
                        unescaped = True
                        break
                    n = n.parentNode
                if unescaped:
                    rv.append(element.nodeValue)
                else:
                    rv.append(escapeHTML(element.nodeValue))
            else:
                rv.append("<%s" % element.nodeName)
                if element.hasAttributes():
                    for name, value in element.attributes.items():
                        rv.append(' %s="%s"' % (name, escapeHTML(value)))
                rv.append(">")
                if element.nodeName not in empty:
                    for child in element.childNodes:
                        serializeElement(child)
                    rv.append("</%s>" % element.nodeName)
        serializeElement(element)
        return '<!DOCTYPE html>\n' + ''.join(rv)

    def write_annotated_spec():
        # Load the stripped-down XHTMLised copy of the spec
        doc = xml.dom.minidom.parse(open('current-work-canvas.xhtml', 'r'))

        # Insert our new stylesheet
        n = doc.getElementsByTagName('head')[0].appendChild(doc.createElement('link'))
        n.setAttribute('rel', 'stylesheet')
        n.setAttribute('href', '../common/canvas-spec.css')
        n.setAttribute('type', 'text/css')

        spec_assertion_patterns = []
        for a in spec_assertions:
            # Warn about problems
            if a['id'] not in spec_refs:
                print "Unused spec statement %s" % a['id']

            pattern_text = a['text']

            if 'keyword' in a:
                # Explicit keyword override
                keyword = a['keyword']
            else:
                # Extract the marked keywords, and remove the markers
                keyword = 'none'
                for kw in ['must', 'should', 'required']:
                    if ('*%s*' % kw) in pattern_text:
                        keyword = kw
                        pattern_text = pattern_text.replace('*%s*' % kw, kw)
                        break
            # Make sure there wasn't >1 keyword
            for kw in ['must', 'should', 'required']:
                assert('*%s*' % kw not in pattern_text)

            # Convert the special pattern format into regexp syntax
            pattern_text = (pattern_text.
                # Escape relevant characters
                replace('*', r'\*').
                replace('+', r'\+').
                replace('.', r'\.').
                replace('(', r'\(').
                replace(')', r'\)').
                replace('[', r'\[').
                replace(']', r'\]').
                # Convert special sequences back into unescaped regexp code
                replace(' ', r'\s+').
                replace(r'<\.\.\.>', r'.+').
                replace('<^>', r'()').
                replace('<eol>', r'\s*?\n')
            )
            pattern = re.compile(pattern_text, re.S)
            spec_assertion_patterns.append( (a['id'], pattern, keyword, a.get('previously', None)) )
        matched_assertions = {}

        def process_element(e):
            t, offsets = getNodeText(e)
            for id, pattern, keyword, previously in spec_assertion_patterns:
                m = pattern.search(t)
                if m:
                    # When the pattern-match isn't enough to uniquely identify a sentence,
                    # allow explicit back-references to earlier paragraphs
                    if previously:
                        if len(previously) >= 3:
                            n, text, exp = previously
                        else:
                            n, text = previously
                            exp = True
                        node = e
                        while n and node.previousSibling:
                            node = node.previousSibling
                            n -= 1
                        if (text not in getNodeText(node)[0]) == exp:
                            continue # discard this match

                    if id in matched_assertions:
                        print "Spec statement %s matches multiple places" % id
                    matched_assertions[id] = True

                    if m.lastindex != 1:
                        print "Spec statement %s has incorrect number of match groups" % id

                    end = m.end(1)
                    end_node = None
                    for end_node, o in offsets:
                        if end < o:
                            break
                        end -= o
                    assert(end_node)

                    n1 = doc.createElement('span')
                    n1.setAttribute('class', 'testrefs kw-%s' % keyword)
                    n1.setAttribute('id', 'testrefs.%s' % id)
                    n1.appendChild(doc.createTextNode(' '))

                    n = n1.appendChild(doc.createElement('a'))
                    n.setAttribute('href', '#testrefs.%s' % id)
                    n.setAttribute('title', id)
                    n.appendChild(doc.createTextNode('#'))

                    n1.appendChild(doc.createTextNode(' '))
                    for test_id in spec_refs.get(id, []):
                        n = n1.appendChild(doc.createElement('a'))
                        n.setAttribute('href', '%s.html' % map_name(test_id))
                        n.appendChild(doc.createTextNode(test_id))
                        n1.appendChild(doc.createTextNode(' '))
                    n0 = doc.createTextNode(end_node.nodeValue[:end])
                    n2 = doc.createTextNode(end_node.nodeValue[end:])

                    p = end_node.parentNode
                    p.replaceChild(n2, end_node)
                    p.insertBefore(n1, n2)
                    p.insertBefore(n0, n1)

                    t, offsets = getNodeText(e)

        for e in doc.getElementsByTagName('body')[0].childNodes:
            process_element(e)

        for s in spec_assertions:
            if s['id'] not in matched_assertions:
                print "Annotation incomplete: Unmatched spec statement %s" % s['id']

        # Convert from XHTML back to HTML
        doc.documentElement.removeAttribute('xmlns')

        head = doc.documentElement.getElementsByTagName('head')[0]
        head.insertBefore(doc.createElement('meta'), head.firstChild).setAttribute('charset', 'UTF-8')

        f = codecs.open('%s/annotated-spec.html' % SPECOUTPUTDIR, 'w', 'utf-8')
        f.write(htmlSerializer(doc))


    if not ISOFFSCREENCANVAS:
        write_annotated_spec()
