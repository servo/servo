from __future__ import absolute_import, division, unicode_literals

import itertools
import sys

from six import unichr, text_type
import pytest

try:
    import lxml.etree
except ImportError:
    pass

from .support import treeTypes

from html5lib import html5parser, treewalkers
from html5lib.filters.lint import Filter as Lint

import re
attrlist = re.compile(r"^(\s+)\w+=.*(\n\1\w+=.*)+", re.M)


def sortattrs(x):
    lines = x.group(0).split("\n")
    lines.sort()
    return "\n".join(lines)


def test_all_tokens():
    expected = [
        {'data': {}, 'type': 'StartTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'html'},
        {'data': {}, 'type': 'StartTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'head'},
        {'type': 'EndTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'head'},
        {'data': {}, 'type': 'StartTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'body'},
        {'data': 'a', 'type': 'Characters'},
        {'data': {}, 'type': 'StartTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'div'},
        {'data': 'b', 'type': 'Characters'},
        {'type': 'EndTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'div'},
        {'data': 'c', 'type': 'Characters'},
        {'type': 'EndTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'body'},
        {'type': 'EndTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'html'}
    ]
    for _, treeCls in sorted(treeTypes.items()):
        if treeCls is None:
            continue
        p = html5parser.HTMLParser(tree=treeCls["builder"])
        document = p.parse("<html><head></head><body>a<div>b</div>c</body></html>")
        document = treeCls.get("adapter", lambda x: x)(document)
        output = Lint(treeCls["walker"](document))
        for expectedToken, outputToken in zip(expected, output):
            assert expectedToken == outputToken


def set_attribute_on_first_child(docfrag, name, value, treeName):
    """naively sets an attribute on the first child of the document
    fragment passed in"""
    setter = {'ElementTree': lambda d: d[0].set,
              'DOM': lambda d: d.firstChild.setAttribute}
    setter['cElementTree'] = setter['ElementTree']
    try:
        setter.get(treeName, setter['DOM'])(docfrag)(name, value)
    except AttributeError:
        setter['ElementTree'](docfrag)(name, value)


def param_treewalker_six_mix():
    """Str/Unicode mix. If str attrs added to tree"""

    # On Python 2.x string literals are of type str. Unless, like this
    # file, the programmer imports unicode_literals from __future__.
    # In that case, string literals become objects of type unicode.

    # This test simulates a Py2 user, modifying attributes on a document
    # fragment but not using the u'' syntax nor importing unicode_literals
    sm_tests = [
        ('<a href="http://example.com">Example</a>',
         [(str('class'), str('test123'))],
         '<a>\n  class="test123"\n  href="http://example.com"\n  "Example"'),

        ('<link href="http://example.com/cow">',
         [(str('rel'), str('alternate'))],
         '<link>\n  href="http://example.com/cow"\n  rel="alternate"\n  "Example"')
    ]

    for tree in sorted(treeTypes.items()):
        for intext, attrs, expected in sm_tests:
            yield intext, expected, attrs, tree


@pytest.mark.parametrize("intext, expected, attrs_to_add, tree", param_treewalker_six_mix())
def test_treewalker_six_mix(intext, expected, attrs_to_add, tree):
    """tests what happens when we add attributes to the intext"""
    treeName, treeClass = tree
    if treeClass is None:
        pytest.skip("Treebuilder not loaded")
    parser = html5parser.HTMLParser(tree=treeClass["builder"])
    document = parser.parseFragment(intext)
    for nom, val in attrs_to_add:
        set_attribute_on_first_child(document, nom, val, treeName)

    document = treeClass.get("adapter", lambda x: x)(document)
    output = treewalkers.pprint(treeClass["walker"](document))
    output = attrlist.sub(sortattrs, output)
    if output not in expected:
        raise AssertionError("TreewalkerEditTest: %s\nExpected:\n%s\nReceived:\n%s" % (treeName, expected, output))


@pytest.mark.parametrize("tree,char", itertools.product(sorted(treeTypes.items()), ["x", "\u1234"]))
def test_fragment_single_char(tree, char):
    expected = [
        {'data': char, 'type': 'Characters'}
    ]

    treeName, treeClass = tree
    if treeClass is None:
        pytest.skip("Treebuilder not loaded")

    parser = html5parser.HTMLParser(tree=treeClass["builder"])
    document = parser.parseFragment(char)
    document = treeClass.get("adapter", lambda x: x)(document)
    output = Lint(treeClass["walker"](document))

    assert list(output) == expected


@pytest.mark.skipif(treeTypes["lxml"] is None, reason="lxml not importable")
def test_lxml_xml():
    expected = [
        {'data': {}, 'name': 'div', 'namespace': None, 'type': 'StartTag'},
        {'data': {}, 'name': 'div', 'namespace': None, 'type': 'StartTag'},
        {'name': 'div', 'namespace': None, 'type': 'EndTag'},
        {'name': 'div', 'namespace': None, 'type': 'EndTag'}
    ]

    lxmltree = lxml.etree.fromstring('<div><div></div></div>')
    walker = treewalkers.getTreeWalker('lxml')
    output = Lint(walker(lxmltree))

    assert list(output) == expected


@pytest.mark.parametrize("treeName",
                         [pytest.param(treeName, marks=[getattr(pytest.mark, treeName),
                                                        pytest.mark.skipif(
                                                            treeName != "lxml" or
                                                            sys.version_info < (3, 7), reason="dict order undef")])
                          for treeName in sorted(treeTypes.keys())])
def test_maintain_attribute_order(treeName):
    treeAPIs = treeTypes[treeName]
    if treeAPIs is None:
        pytest.skip("Treebuilder not loaded")

    # generate loads to maximize the chance a hash-based mutation will occur
    attrs = [(unichr(x), text_type(i)) for i, x in enumerate(range(ord('a'), ord('z')))]
    data = "<span " + " ".join("%s='%s'" % (x, i) for x, i in attrs) + ">"

    parser = html5parser.HTMLParser(tree=treeAPIs["builder"])
    document = parser.parseFragment(data)

    document = treeAPIs.get("adapter", lambda x: x)(document)
    output = list(Lint(treeAPIs["walker"](document)))

    assert len(output) == 2
    assert output[0]['type'] == 'StartTag'
    assert output[1]['type'] == "EndTag"

    attrs_out = output[0]['data']
    assert len(attrs) == len(attrs_out)

    for (in_name, in_value), (out_name, out_value) in zip(attrs, attrs_out.items()):
        assert (None, in_name) == out_name
        assert in_value == out_value


@pytest.mark.parametrize("treeName",
                         [pytest.param(treeName, marks=[getattr(pytest.mark, treeName),
                                                        pytest.mark.skipif(
                                                            treeName != "lxml" or
                                                            sys.version_info < (3, 7), reason="dict order undef")])
                          for treeName in sorted(treeTypes.keys())])
def test_maintain_attribute_order_adjusted(treeName):
    treeAPIs = treeTypes[treeName]
    if treeAPIs is None:
        pytest.skip("Treebuilder not loaded")

    # generate loads to maximize the chance a hash-based mutation will occur
    data = "<svg a=1 refx=2 b=3 xml:lang=4 c=5>"

    parser = html5parser.HTMLParser(tree=treeAPIs["builder"])
    document = parser.parseFragment(data)

    document = treeAPIs.get("adapter", lambda x: x)(document)
    output = list(Lint(treeAPIs["walker"](document)))

    assert len(output) == 2
    assert output[0]['type'] == 'StartTag'
    assert output[1]['type'] == "EndTag"

    attrs_out = output[0]['data']

    assert list(attrs_out.items()) == [((None, 'a'), '1'),
                                       ((None, 'refX'), '2'),
                                       ((None, 'b'), '3'),
                                       (('http://www.w3.org/XML/1998/namespace', 'lang'), '4'),
                                       ((None, 'c'), '5')]
