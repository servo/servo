from __future__ import absolute_import, division, unicode_literals

import os
import sys
import unittest
import warnings
from difflib import unified_diff

try:
    unittest.TestCase.assertEqual
except AttributeError:
    unittest.TestCase.assertEqual = unittest.TestCase.assertEquals

from .support import get_data_files, TestData, convertExpected

from html5lib import html5parser, treewalkers, treebuilders, constants


def PullDOMAdapter(node):
    from xml.dom import Node
    from xml.dom.pulldom import START_ELEMENT, END_ELEMENT, COMMENT, CHARACTERS

    if node.nodeType in (Node.DOCUMENT_NODE, Node.DOCUMENT_FRAGMENT_NODE):
        for childNode in node.childNodes:
            for event in PullDOMAdapter(childNode):
                yield event

    elif node.nodeType == Node.DOCUMENT_TYPE_NODE:
        raise NotImplementedError("DOCTYPE nodes are not supported by PullDOM")

    elif node.nodeType == Node.COMMENT_NODE:
        yield COMMENT, node

    elif node.nodeType in (Node.TEXT_NODE, Node.CDATA_SECTION_NODE):
        yield CHARACTERS, node

    elif node.nodeType == Node.ELEMENT_NODE:
        yield START_ELEMENT, node
        for childNode in node.childNodes:
            for event in PullDOMAdapter(childNode):
                yield event
        yield END_ELEMENT, node

    else:
        raise NotImplementedError("Node type not supported: " + str(node.nodeType))

treeTypes = {
    "DOM": {"builder": treebuilders.getTreeBuilder("dom"),
            "walker": treewalkers.getTreeWalker("dom")},
    "PullDOM": {"builder": treebuilders.getTreeBuilder("dom"),
                "adapter": PullDOMAdapter,
                "walker": treewalkers.getTreeWalker("pulldom")},
}

# Try whatever etree implementations are available from a list that are
#"supposed" to work
try:
    import xml.etree.ElementTree as ElementTree
except ImportError:
    pass
else:
    treeTypes['ElementTree'] = \
        {"builder": treebuilders.getTreeBuilder("etree", ElementTree),
         "walker": treewalkers.getTreeWalker("etree", ElementTree)}

try:
    import xml.etree.cElementTree as ElementTree
except ImportError:
    pass
else:
    treeTypes['cElementTree'] = \
        {"builder": treebuilders.getTreeBuilder("etree", ElementTree),
         "walker": treewalkers.getTreeWalker("etree", ElementTree)}


try:
    import lxml.etree as ElementTree  # flake8: noqa
except ImportError:
    pass
else:
    treeTypes['lxml_native'] = \
        {"builder": treebuilders.getTreeBuilder("lxml"),
         "walker": treewalkers.getTreeWalker("lxml")}


try:
    from genshi.core import QName, Attrs
    from genshi.core import START, END, TEXT, COMMENT, DOCTYPE
except ImportError:
    pass
else:
    def GenshiAdapter(tree):
        text = None
        for token in treewalkers.getTreeWalker("dom")(tree):
            type = token["type"]
            if type in ("Characters", "SpaceCharacters"):
                if text is None:
                    text = token["data"]
                else:
                    text += token["data"]
            elif text is not None:
                yield TEXT, text, (None, -1, -1)
                text = None

            if type in ("StartTag", "EmptyTag"):
                if token["namespace"]:
                    name = "{%s}%s" % (token["namespace"], token["name"])
                else:
                    name = token["name"]
                attrs = Attrs([(QName("{%s}%s" % attr if attr[0] is not None else attr[1]), value)
                               for attr, value in token["data"].items()])
                yield (START, (QName(name), attrs), (None, -1, -1))
                if type == "EmptyTag":
                    type = "EndTag"

            if type == "EndTag":
                if token["namespace"]:
                    name = "{%s}%s" % (token["namespace"], token["name"])
                else:
                    name = token["name"]

                yield END, QName(name), (None, -1, -1)

            elif type == "Comment":
                yield COMMENT, token["data"], (None, -1, -1)

            elif type == "Doctype":
                yield DOCTYPE, (token["name"], token["publicId"],
                                token["systemId"]), (None, -1, -1)

            else:
                pass  # FIXME: What to do?

        if text is not None:
            yield TEXT, text, (None, -1, -1)

    treeTypes["genshi"] = \
        {"builder": treebuilders.getTreeBuilder("dom"),
         "adapter": GenshiAdapter,
         "walker": treewalkers.getTreeWalker("genshi")}


def concatenateCharacterTokens(tokens):
    charactersToken = None
    for token in tokens:
        type = token["type"]
        if type in ("Characters", "SpaceCharacters"):
            if charactersToken is None:
                charactersToken = {"type": "Characters", "data": token["data"]}
            else:
                charactersToken["data"] += token["data"]
        else:
            if charactersToken is not None:
                yield charactersToken
                charactersToken = None
            yield token
    if charactersToken is not None:
        yield charactersToken


def convertTokens(tokens):
    output = []
    indent = 0
    for token in concatenateCharacterTokens(tokens):
        type = token["type"]
        if type in ("StartTag", "EmptyTag"):
            if (token["namespace"] and
                    token["namespace"] != constants.namespaces["html"]):
                if token["namespace"] in constants.prefixes:
                    name = constants.prefixes[token["namespace"]]
                else:
                    name = token["namespace"]
                name += " " + token["name"]
            else:
                name = token["name"]
            output.append("%s<%s>" % (" " * indent, name))
            indent += 2
            attrs = token["data"]
            if attrs:
                # TODO: Remove this if statement, attrs should always exist
                for (namespace, name), value in sorted(attrs.items()):
                    if namespace:
                        if namespace in constants.prefixes:
                            outputname = constants.prefixes[namespace]
                        else:
                            outputname = namespace
                        outputname += " " + name
                    else:
                        outputname = name
                    output.append("%s%s=\"%s\"" % (" " * indent, outputname, value))
            if type == "EmptyTag":
                indent -= 2
        elif type == "EndTag":
            indent -= 2
        elif type == "Comment":
            output.append("%s<!-- %s -->" % (" " * indent, token["data"]))
        elif type == "Doctype":
            if token["name"]:
                if token["publicId"]:
                    output.append("""%s<!DOCTYPE %s "%s" "%s">""" %
                                  (" " * indent, token["name"],
                                   token["publicId"],
                                   token["systemId"] and token["systemId"] or ""))
                elif token["systemId"]:
                    output.append("""%s<!DOCTYPE %s "" "%s">""" %
                                  (" " * indent, token["name"],
                                   token["systemId"]))
                else:
                    output.append("%s<!DOCTYPE %s>" % (" " * indent,
                                                       token["name"]))
            else:
                output.append("%s<!DOCTYPE >" % (" " * indent,))
        elif type in ("Characters", "SpaceCharacters"):
            output.append("%s\"%s\"" % (" " * indent, token["data"]))
        else:
            pass  # TODO: what to do with errors?
    return "\n".join(output)

import re
attrlist = re.compile(r"^(\s+)\w+=.*(\n\1\w+=.*)+", re.M)


def sortattrs(x):
    lines = x.group(0).split("\n")
    lines.sort()
    return "\n".join(lines)


class TokenTestCase(unittest.TestCase):
    def test_all_tokens(self):
        expected = [
            {'data': {}, 'type': 'StartTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'html'},
            {'data': {}, 'type': 'StartTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'head'},
            {'data': {}, 'type': 'EndTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'head'},
            {'data': {}, 'type': 'StartTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'body'},
            {'data': 'a', 'type': 'Characters'},
            {'data': {}, 'type': 'StartTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'div'},
            {'data': 'b', 'type': 'Characters'},
            {'data': {}, 'type': 'EndTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'div'},
            {'data': 'c', 'type': 'Characters'},
            {'data': {}, 'type': 'EndTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'body'},
            {'data': {}, 'type': 'EndTag', 'namespace': 'http://www.w3.org/1999/xhtml', 'name': 'html'}
        ]
        for treeName, treeCls in treeTypes.items():
            p = html5parser.HTMLParser(tree=treeCls["builder"])
            document = p.parse("<html><head></head><body>a<div>b</div>c</body></html>")
            document = treeCls.get("adapter", lambda x: x)(document)
            output = treeCls["walker"](document)
            for expectedToken, outputToken in zip(expected, output):
                self.assertEqual(expectedToken, outputToken)


def runTreewalkerTest(innerHTML, input, expected, errors, treeClass):
    warnings.resetwarnings()
    warnings.simplefilter("error")
    try:
        p = html5parser.HTMLParser(tree=treeClass["builder"])
        if innerHTML:
            document = p.parseFragment(input, innerHTML)
        else:
            document = p.parse(input)
    except constants.DataLossWarning:
        # Ignore testcases we know we don't pass
        return

    document = treeClass.get("adapter", lambda x: x)(document)
    try:
        output = convertTokens(treeClass["walker"](document))
        output = attrlist.sub(sortattrs, output)
        expected = attrlist.sub(sortattrs, convertExpected(expected))
        diff = "".join(unified_diff([line + "\n" for line in expected.splitlines()],
                                    [line + "\n" for line in output.splitlines()],
                                    "Expected", "Received"))
        assert expected == output, "\n".join([
            "", "Input:", input,
                "", "Expected:", expected,
                "", "Received:", output,
                "", "Diff:", diff,
        ])
    except NotImplementedError:
        pass  # Amnesty for those that confess...


def test_treewalker():
    sys.stdout.write('Testing tree walkers ' + " ".join(list(treeTypes.keys())) + "\n")

    for treeName, treeCls in treeTypes.items():
        files = get_data_files('tree-construction')
        for filename in files:
            testName = os.path.basename(filename).replace(".dat", "")
            if testName in ("template",):
                continue

            tests = TestData(filename, "data")

            for index, test in enumerate(tests):
                (input, errors,
                 innerHTML, expected) = [test[key] for key in ("data", "errors",
                                                               "document-fragment",
                                                               "document")]
                errors = errors.split("\n")
                yield runTreewalkerTest, innerHTML, input, expected, errors, treeCls


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


def runTreewalkerEditTest(intext, expected, attrs_to_add, tree):
    """tests what happens when we add attributes to the intext"""
    treeName, treeClass = tree
    parser = html5parser.HTMLParser(tree=treeClass["builder"])
    document = parser.parseFragment(intext)
    for nom, val in attrs_to_add:
        set_attribute_on_first_child(document, nom, val, treeName)

    document = treeClass.get("adapter", lambda x: x)(document)
    output = convertTokens(treeClass["walker"](document))
    output = attrlist.sub(sortattrs, output)
    if not output in expected:
        raise AssertionError("TreewalkerEditTest: %s\nExpected:\n%s\nReceived:\n%s" % (treeName, expected, output))


def test_treewalker_six_mix():
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

    for tree in treeTypes.items():
        for intext, attrs, expected in sm_tests:
            yield runTreewalkerEditTest, intext, expected, attrs, tree
