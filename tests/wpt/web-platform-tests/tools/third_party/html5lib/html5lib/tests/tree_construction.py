from __future__ import absolute_import, division, unicode_literals

import itertools
import re
import warnings
from difflib import unified_diff

import pytest

from .support import TestData, convert, convertExpected, treeTypes
from html5lib import html5parser, constants, treewalkers
from html5lib.filters.lint import Filter as Lint

_attrlist_re = re.compile(r"^(\s+)\w+=.*(\n\1\w+=.*)+", re.M)


def sortattrs(s):
    def replace(m):
        lines = m.group(0).split("\n")
        lines.sort()
        return "\n".join(lines)
    return _attrlist_re.sub(replace, s)


class TreeConstructionFile(pytest.File):
    def collect(self):
        tests = TestData(str(self.fspath), "data")
        for i, test in enumerate(tests):
            yield TreeConstructionTest(str(i), self, testdata=test)


class TreeConstructionTest(pytest.Collector):
    def __init__(self, name, parent=None, config=None, session=None, testdata=None):
        super(TreeConstructionTest, self).__init__(name, parent, config, session)
        self.testdata = testdata

    def collect(self):
        for treeName, treeAPIs in sorted(treeTypes.items()):
            for x in itertools.chain(self._getParserTests(treeName, treeAPIs),
                                     self._getTreeWalkerTests(treeName, treeAPIs)):
                yield x

    def _getParserTests(self, treeName, treeAPIs):
        if treeAPIs is not None and "adapter" in treeAPIs:
            return
        for namespaceHTMLElements in (True, False):
            if namespaceHTMLElements:
                nodeid = "%s::parser::namespaced" % treeName
            else:
                nodeid = "%s::parser::void-namespace" % treeName
            item = ParserTest(nodeid,
                              self,
                              self.testdata,
                              treeAPIs["builder"] if treeAPIs is not None else None,
                              namespaceHTMLElements)
            item.add_marker(getattr(pytest.mark, treeName))
            item.add_marker(pytest.mark.parser)
            if namespaceHTMLElements:
                item.add_marker(pytest.mark.namespaced)
            yield item

    def _getTreeWalkerTests(self, treeName, treeAPIs):
        nodeid = "%s::treewalker" % treeName
        item = TreeWalkerTest(nodeid,
                              self,
                              self.testdata,
                              treeAPIs)
        item.add_marker(getattr(pytest.mark, treeName))
        item.add_marker(pytest.mark.treewalker)
        yield item


def convertTreeDump(data):
    return "\n".join(convert(3)(data).split("\n")[1:])


namespaceExpected = re.compile(r"^(\s*)<(\S+)>", re.M).sub


class ParserTest(pytest.Item):
    def __init__(self, name, parent, test, treeClass, namespaceHTMLElements):
        super(ParserTest, self).__init__(name, parent)
        self.test = test
        self.treeClass = treeClass
        self.namespaceHTMLElements = namespaceHTMLElements

    def runtest(self):
        if self.treeClass is None:
            pytest.skip("Treebuilder not loaded")

        p = html5parser.HTMLParser(tree=self.treeClass,
                                   namespaceHTMLElements=self.namespaceHTMLElements)

        input = self.test['data']
        fragmentContainer = self.test['document-fragment']
        expected = convertExpected(self.test['document'])
        expectedErrors = self.test['errors'].split("\n") if self.test['errors'] else []

        scripting = False
        if 'script-on' in self.test:
            scripting = True

        with warnings.catch_warnings():
            warnings.simplefilter("error")
            try:
                if fragmentContainer:
                    document = p.parseFragment(input, fragmentContainer, scripting=scripting)
                else:
                    document = p.parse(input, scripting=scripting)
            except constants.DataLossWarning:
                pytest.skip("data loss warning")

        output = convertTreeDump(p.tree.testSerializer(document))

        expected = expected
        if self.namespaceHTMLElements:
            expected = namespaceExpected(r"\1<html \2>", expected)

        errorMsg = "\n".join(["\n\nInput:", input, "\nExpected:", expected,
                              "\nReceived:", output])
        assert expected == output, errorMsg

        errStr = []
        for (line, col), errorcode, datavars in p.errors:
            assert isinstance(datavars, dict), "%s, %s" % (errorcode, repr(datavars))
            errStr.append("Line: %i Col: %i %s" % (line, col,
                                                   constants.E[errorcode] % datavars))

        errorMsg2 = "\n".join(["\n\nInput:", input,
                               "\nExpected errors (" + str(len(expectedErrors)) + "):\n" + "\n".join(expectedErrors),
                               "\nActual errors (" + str(len(p.errors)) + "):\n" + "\n".join(errStr)])
        if False:  # we're currently not testing parse errors
            assert len(p.errors) == len(expectedErrors), errorMsg2

    def repr_failure(self, excinfo):
        traceback = excinfo.traceback
        ntraceback = traceback.cut(path=__file__)
        excinfo.traceback = ntraceback.filter()

        return excinfo.getrepr(funcargs=True,
                               showlocals=False,
                               style="short", tbfilter=False)


class TreeWalkerTest(pytest.Item):
    def __init__(self, name, parent, test, treeAPIs):
        super(TreeWalkerTest, self).__init__(name, parent)
        self.test = test
        self.treeAPIs = treeAPIs

    def runtest(self):
        if self.treeAPIs is None:
            pytest.skip("Treebuilder not loaded")

        p = html5parser.HTMLParser(tree=self.treeAPIs["builder"])

        input = self.test['data']
        fragmentContainer = self.test['document-fragment']
        expected = convertExpected(self.test['document'])

        scripting = False
        if 'script-on' in self.test:
            scripting = True

        with warnings.catch_warnings():
            warnings.simplefilter("error")
            try:
                if fragmentContainer:
                    document = p.parseFragment(input, fragmentContainer, scripting=scripting)
                else:
                    document = p.parse(input, scripting=scripting)
            except constants.DataLossWarning:
                pytest.skip("data loss warning")

        poutput = convertTreeDump(p.tree.testSerializer(document))
        namespace_expected = namespaceExpected(r"\1<html \2>", expected)
        if poutput != namespace_expected:
            pytest.skip("parser output incorrect")

        document = self.treeAPIs.get("adapter", lambda x: x)(document)

        try:
            output = treewalkers.pprint(Lint(self.treeAPIs["walker"](document)))
            output = sortattrs(output)
            expected = sortattrs(expected)
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
            pytest.skip("tree walker NotImplementedError")

    def repr_failure(self, excinfo):
        traceback = excinfo.traceback
        ntraceback = traceback.cut(path=__file__)
        excinfo.traceback = ntraceback.filter()

        return excinfo.getrepr(funcargs=True,
                               showlocals=False,
                               style="short", tbfilter=False)
