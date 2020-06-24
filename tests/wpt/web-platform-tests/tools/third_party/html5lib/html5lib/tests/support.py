from __future__ import absolute_import, division, unicode_literals

# pylint:disable=wrong-import-position

import os
import sys
import codecs
import glob
import xml.sax.handler

base_path = os.path.split(__file__)[0]

test_dir = os.path.join(base_path, 'testdata')
sys.path.insert(0, os.path.abspath(os.path.join(base_path,
                                                os.path.pardir,
                                                os.path.pardir)))

from html5lib import treebuilders, treewalkers, treeadapters  # noqa
del base_path

# Build a dict of available trees
treeTypes = {}

# DOM impls
treeTypes["DOM"] = {
    "builder": treebuilders.getTreeBuilder("dom"),
    "walker": treewalkers.getTreeWalker("dom")
}

# ElementTree impls
import xml.etree.ElementTree as ElementTree  # noqa
treeTypes['ElementTree'] = {
    "builder": treebuilders.getTreeBuilder("etree", ElementTree, fullTree=True),
    "walker": treewalkers.getTreeWalker("etree", ElementTree)
}

try:
    import xml.etree.cElementTree as cElementTree  # noqa
except ImportError:
    treeTypes['cElementTree'] = None
else:
    # On Python 3.3 and above cElementTree is an alias, don't run them twice.
    if cElementTree.Element is ElementTree.Element:
        treeTypes['cElementTree'] = None
    else:
        treeTypes['cElementTree'] = {
            "builder": treebuilders.getTreeBuilder("etree", cElementTree, fullTree=True),
            "walker": treewalkers.getTreeWalker("etree", cElementTree)
        }

try:
    import lxml.etree as lxml  # noqa
except ImportError:
    treeTypes['lxml'] = None
else:
    treeTypes['lxml'] = {
        "builder": treebuilders.getTreeBuilder("lxml"),
        "walker": treewalkers.getTreeWalker("lxml")
    }

# Genshi impls
try:
    import genshi  # noqa
except ImportError:
    treeTypes["genshi"] = None
else:
    treeTypes["genshi"] = {
        "builder": treebuilders.getTreeBuilder("dom"),
        "adapter": lambda tree: treeadapters.genshi.to_genshi(treewalkers.getTreeWalker("dom")(tree)),
        "walker": treewalkers.getTreeWalker("genshi")
    }

# pylint:enable=wrong-import-position


def get_data_files(subdirectory, files='*.dat', search_dir=test_dir):
    return sorted(glob.glob(os.path.join(search_dir, subdirectory, files)))


class DefaultDict(dict):
    def __init__(self, default, *args, **kwargs):
        self.default = default
        dict.__init__(self, *args, **kwargs)

    def __getitem__(self, key):
        return dict.get(self, key, self.default)


class TestData(object):
    def __init__(self, filename, newTestHeading="data", encoding="utf8"):
        if encoding is None:
            self.f = open(filename, mode="rb")
        else:
            self.f = codecs.open(filename, encoding=encoding)
        self.encoding = encoding
        self.newTestHeading = newTestHeading

    def __iter__(self):
        data = DefaultDict(None)
        key = None
        for line in self.f:
            heading = self.isSectionHeading(line)
            if heading:
                if data and heading == self.newTestHeading:
                    # Remove trailing newline
                    data[key] = data[key][:-1]
                    yield self.normaliseOutput(data)
                    data = DefaultDict(None)
                key = heading
                data[key] = "" if self.encoding else b""
            elif key is not None:
                data[key] += line
        if data:
            yield self.normaliseOutput(data)

    def isSectionHeading(self, line):
        """If the current heading is a test section heading return the heading,
        otherwise return False"""
        # print(line)
        if line.startswith("#" if self.encoding else b"#"):
            return line[1:].strip()
        else:
            return False

    def normaliseOutput(self, data):
        # Remove trailing newlines
        for key, value in data.items():
            if value.endswith("\n" if self.encoding else b"\n"):
                data[key] = value[:-1]
        return data


def convert(stripChars):
    def convertData(data):
        """convert the output of str(document) to the format used in the testcases"""
        data = data.split("\n")
        rv = []
        for line in data:
            if line.startswith("|"):
                rv.append(line[stripChars:])
            else:
                rv.append(line)
        return "\n".join(rv)
    return convertData


convertExpected = convert(2)


def errorMessage(input, expected, actual):
    msg = ("Input:\n%s\nExpected:\n%s\nReceived\n%s\n" %
           (repr(input), repr(expected), repr(actual)))
    if sys.version_info[0] == 2:
        msg = msg.encode("ascii", "backslashreplace")
    return msg


class TracingSaxHandler(xml.sax.handler.ContentHandler):
    def __init__(self):
        xml.sax.handler.ContentHandler.__init__(self)
        self.visited = []

    def startDocument(self):
        self.visited.append('startDocument')

    def endDocument(self):
        self.visited.append('endDocument')

    def startPrefixMapping(self, prefix, uri):
        # These are ignored as their order is not guaranteed
        pass

    def endPrefixMapping(self, prefix):
        # These are ignored as their order is not guaranteed
        pass

    def startElement(self, name, attrs):
        self.visited.append(('startElement', name, attrs))

    def endElement(self, name):
        self.visited.append(('endElement', name))

    def startElementNS(self, name, qname, attrs):
        self.visited.append(('startElementNS', name, qname, dict(attrs)))

    def endElementNS(self, name, qname):
        self.visited.append(('endElementNS', name, qname))

    def characters(self, content):
        self.visited.append(('characters', content))

    def ignorableWhitespace(self, whitespace):
        self.visited.append(('ignorableWhitespace', whitespace))

    def processingInstruction(self, target, data):
        self.visited.append(('processingInstruction', target, data))

    def skippedEntity(self, name):
        self.visited.append(('skippedEntity', name))
