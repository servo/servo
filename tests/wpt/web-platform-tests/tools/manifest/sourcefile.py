import hashlib
import re
import os
from six import binary_type
from six.moves.urllib.parse import urljoin
from fnmatch import fnmatch
try:
    from xml.etree import cElementTree as ElementTree
except ImportError:
    from xml.etree import ElementTree

import html5lib

from . import XMLParser
from .item import Stub, ManualTest, WebdriverSpecTest, RefTestNode, RefTest, TestharnessTest, SupportFile, ConformanceCheckerTest, VisualTest
from .utils import rel_path_to_url, ContextManagerBytesIO, cached_property

wd_pattern = "*.py"
meta_re = re.compile(b"//\s*META:\s*(\w*)=(.*)$")

reference_file_re = re.compile(r'(^|[\-_])(not)?ref[0-9]*([\-_]|$)')

def replace_end(s, old, new):
    """
    Given a string `s` that ends with `old`, replace that occurrence of `old`
    with `new`.
    """
    assert s.endswith(old)
    return s[:-len(old)] + new


def read_script_metadata(f):
    """
    Yields any metadata (pairs of bytestrings) from the file-like object `f`,
    as specified according to the `meta_re` regex.
    """
    for line in f:
        assert isinstance(line, binary_type), line
        m = meta_re.match(line)
        if not m:
            break

        yield (m.groups()[0], m.groups()[1])


class SourceFile(object):
    parsers = {"html":lambda x:html5lib.parse(x, treebuilder="etree"),
               "xhtml":lambda x:ElementTree.parse(x, XMLParser.XMLParser()),
               "svg":lambda x:ElementTree.parse(x, XMLParser.XMLParser())}

    root_dir_non_test = set(["common",
                             "work-in-progress"])

    dir_non_test = set(["resources",
                        "support",
                        "tools"])

    dir_path_non_test = {("css21", "archive")}

    def __init__(self, tests_root, rel_path, url_base, contents=None):
        """Object representing a file in a source tree.

        :param tests_root: Path to the root of the source tree
        :param rel_path: File path relative to tests_root
        :param url_base: Base URL used when converting file paths to urls
        :param contents: Byte array of the contents of the file or ``None``.
        """

        self.tests_root = tests_root
        if os.name == "nt":
            # do slash normalization on Windows
            if isinstance(rel_path, binary_type):
                self.rel_path = rel_path.replace(b"/", b"\\")
            else:
                self.rel_path = rel_path.replace(u"/", u"\\")
        else:
            self.rel_path = rel_path
        self.url_base = url_base
        self.contents = contents

        self.dir_path, self.filename = os.path.split(self.rel_path)
        self.name, self.ext = os.path.splitext(self.filename)

        self.type_flag = None
        if "-" in self.name:
            self.type_flag = self.name.rsplit("-", 1)[1].split(".")[0]

        self.meta_flags = self.name.split(".")[1:]

        self.items_cache = None

    def __getstate__(self):
        # Remove computed properties if we pickle this class
        rv = self.__dict__.copy()

        if "__cached_properties__" in rv:
            cached_properties = rv["__cached_properties__"]
            for key in rv.keys():
                if key in cached_properties:
                    del rv[key]
            del rv["__cached_properties__"]
        return rv

    def name_prefix(self, prefix):
        """Check if the filename starts with a given prefix

        :param prefix: The prefix to check"""
        return self.name.startswith(prefix)

    def is_dir(self):
        """Return whether this file represents a directory."""
        if self.contents is not None:
            return False

        return os.path.isdir(self.rel_path)

    def open(self):
        """
        Return either
        * the contents specified in the constructor, if any;
        * a File object opened for reading the file contents.
        """

        if self.contents is not None:
            file_obj = ContextManagerBytesIO(self.contents)
        else:
            file_obj = open(self.path, 'rb')
        return file_obj

    @cached_property
    def path(self):
        return os.path.join(self.tests_root, self.rel_path)

    @cached_property
    def url(self):
        return rel_path_to_url(self.rel_path, self.url_base)

    @cached_property
    def hash(self):
        with self.open() as f:
            return hashlib.sha1(f.read()).hexdigest()

    def in_non_test_dir(self):
        if self.dir_path == "":
            return True

        parts = self.dir_path.split(os.path.sep)

        if parts[0] in self.root_dir_non_test:
            return True
        elif any(item in self.dir_non_test for item in parts):
            return True
        else:
            for path in self.dir_path_non_test:
                if parts[:len(path)] == list(path):
                    return True
        return False

    def in_conformance_checker_dir(self):
        return (self.dir_path == "conformance-checkers" or
                self.dir_path.startswith("conformance-checkers" + os.path.sep))

    @property
    def name_is_non_test(self):
        """Check if the file name matches the conditions for the file to
        be a non-test file"""
        return (self.is_dir() or
                self.name_prefix("MANIFEST") or
                self.filename.startswith(".") or
                self.in_non_test_dir())

    @property
    def name_is_conformance(self):
        return (self.in_conformance_checker_dir() and
                self.type_flag in ("is-valid", "no-valid"))

    @property
    def name_is_conformance_support(self):
        return self.in_conformance_checker_dir()

    @property
    def name_is_stub(self):
        """Check if the file name matches the conditions for the file to
        be a stub file"""
        return self.name_prefix("stub-")

    @property
    def name_is_manual(self):
        """Check if the file name matches the conditions for the file to
        be a manual test file"""
        return self.type_flag == "manual"

    @property
    def name_is_visual(self):
        """Check if the file name matches the conditions for the file to
        be a visual test file"""
        return self.type_flag == "visual"

    @property
    def name_is_multi_global(self):
        """Check if the file name matches the conditions for the file to
        be a multi-global js test file"""
        return "any" in self.meta_flags and self.ext == ".js"

    @property
    def name_is_worker(self):
        """Check if the file name matches the conditions for the file to
        be a worker js test file"""
        return "worker" in self.meta_flags and self.ext == ".js"

    @property
    def name_is_webdriver(self):
        """Check if the file name matches the conditions for the file to
        be a webdriver spec test file"""
        # wdspec tests are in subdirectories of /webdriver excluding __init__.py
        # files.
        rel_dir_tree = self.rel_path.split(os.path.sep)
        return (rel_dir_tree[0] == "webdriver" and
                len(rel_dir_tree) > 1 and
                self.filename != "__init__.py" and
                fnmatch(self.filename, wd_pattern))

    @property
    def name_is_reference(self):
        """Check if the file name matches the conditions for the file to
        be a reference file (not a reftest)"""
        return "/reference/" in self.url or "/reftest/" in self.url or bool(reference_file_re.search(self.name))

    @property
    def markup_type(self):
        """Return the type of markup contained in a file, based on its extension,
        or None if it doesn't contain markup"""
        ext = self.ext

        if not ext:
            return None
        if ext[0] == ".":
            ext = ext[1:]
        if ext in ["html", "htm"]:
            return "html"
        if ext in ["xhtml", "xht", "xml"]:
            return "xhtml"
        if ext == "svg":
            return "svg"
        return None

    @cached_property
    def root(self):
        """Return an ElementTree Element for the root node of the file if it contains
        markup, or None if it does not"""
        if not self.markup_type:
            return None

        parser = self.parsers[self.markup_type]

        with self.open() as f:
            try:
                tree = parser(f)
            except Exception:
                return None

        if hasattr(tree, "getroot"):
            root = tree.getroot()
        else:
            root = tree

        return root

    @cached_property
    def timeout_nodes(self):
        """List of ElementTree Elements corresponding to nodes in a test that
        specify timeouts"""
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='timeout']")

    @cached_property
    def script_metadata(self):
        if not self.name_is_worker and not self.name_is_multi_global:
            return None

        with self.open() as f:
            return list(read_script_metadata(f))

    @cached_property
    def timeout(self):
        """The timeout of a test or reference file. "long" if the file has an extended timeout
        or None otherwise"""
        if self.script_metadata:
            if any(m == (b"timeout", b"long") for m in self.script_metadata):
                return "long"

        if self.root is None:
            return None

        if self.timeout_nodes:
            timeout_str = self.timeout_nodes[0].attrib.get("content", None)
            if timeout_str and timeout_str.lower() == "long":
                return "long"

        return None

    @cached_property
    def viewport_nodes(self):
        """List of ElementTree Elements corresponding to nodes in a test that
        specify viewport sizes"""
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='viewport-size']")

    @cached_property
    def viewport_size(self):
        """The viewport size of a test or reference file"""
        if self.root is None:
            return None

        if not self.viewport_nodes:
            return None

        return self.viewport_nodes[0].attrib.get("content", None)

    @cached_property
    def dpi_nodes(self):
        """List of ElementTree Elements corresponding to nodes in a test that
        specify device pixel ratios"""
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='device-pixel-ratio']")

    @cached_property
    def dpi(self):
        """The device pixel ratio of a test or reference file"""
        if self.root is None:
            return None

        if not self.dpi_nodes:
            return None

        return self.dpi_nodes[0].attrib.get("content", None)

    @cached_property
    def testharness_nodes(self):
        """List of ElementTree Elements corresponding to nodes representing a
        testharness.js script"""
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testharness.js']")

    @cached_property
    def content_is_testharness(self):
        """Boolean indicating whether the file content represents a
        testharness.js test"""
        if self.root is None:
            return None
        return bool(self.testharness_nodes)

    @cached_property
    def variant_nodes(self):
        """List of ElementTree Elements corresponding to nodes representing a
        test variant"""
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='variant']")

    @cached_property
    def test_variants(self):
        rv = []
        for element in self.variant_nodes:
            if "content" in element.attrib:
                variant = element.attrib["content"]
                assert variant == "" or variant[0] in ["#", "?"]
                rv.append(variant)

        if not rv:
            rv = [""]

        return rv

    @cached_property
    def reftest_nodes(self):
        """List of ElementTree Elements corresponding to nodes representing a
        to a reftest <link>"""
        if self.root is None:
            return []

        match_links = self.root.findall(".//{http://www.w3.org/1999/xhtml}link[@rel='match']")
        mismatch_links = self.root.findall(".//{http://www.w3.org/1999/xhtml}link[@rel='mismatch']")
        return match_links + mismatch_links

    @cached_property
    def references(self):
        """List of (ref_url, relation) tuples for any reftest references specified in
        the file"""
        rv = []
        rel_map = {"match": "==", "mismatch": "!="}
        for item in self.reftest_nodes:
            if "href" in item.attrib:
                ref_url = urljoin(self.url, item.attrib["href"])
                ref_type = rel_map[item.attrib["rel"]]
                rv.append((ref_url, ref_type))
        return rv

    @cached_property
    def content_is_ref_node(self):
        """Boolean indicating whether the file is a non-leaf node in a reftest
        graph (i.e. if it contains any <link rel=[mis]match>"""
        return bool(self.references)

    @cached_property
    def css_flag_nodes(self):
        """List of ElementTree Elements corresponding to nodes representing a
        flag <meta>"""
        if self.root is None:
            return []
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='flags']")

    @cached_property
    def css_flags(self):
        """Set of flags specified in the file"""
        rv = set()
        for item in self.css_flag_nodes:
            if "content" in item.attrib:
                for flag in item.attrib["content"].split():
                    rv.add(flag)
        return rv

    @cached_property
    def content_is_css_manual(self):
        """Boolean indicating whether the file content represents a
        CSS WG-style manual test"""
        if self.root is None:
            return None
        # return True if the intersection between the two sets is non-empty
        return bool(self.css_flags & {"animated", "font", "history", "interact", "paged", "speech", "userstyle"})

    @cached_property
    def spec_link_nodes(self):
        """List of ElementTree Elements corresponding to nodes representing a
        <link rel=help>, used to point to specs"""
        if self.root is None:
            return []
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}link[@rel='help']")

    @cached_property
    def spec_links(self):
        """Set of spec links specified in the file"""
        rv = set()
        for item in self.spec_link_nodes:
            if "href" in item.attrib:
                rv.add(item.attrib["href"])
        return rv

    @cached_property
    def content_is_css_visual(self):
        """Boolean indicating whether the file content represents a
        CSS WG-style manual test"""
        if self.root is None:
            return None
        return bool(self.ext in {'.xht', '.html', '.xhtml', '.htm', '.xml', '.svg'} and
                    self.spec_links)

    @property
    def type(self):
        rv, _ = self.manifest_items()
        return rv

    def manifest_items(self):
        """List of manifest items corresponding to the file. There is typically one
        per test, but in the case of reftests a node may have corresponding manifest
        items without being a test itself."""

        if self.items_cache:
            return self.items_cache

        if self.name_is_non_test:
            rv = "support", [SupportFile(self)]

        elif self.name_is_stub:
            rv = Stub.item_type, [Stub(self, self.url)]

        elif self.name_is_manual:
            rv = ManualTest.item_type, [ManualTest(self, self.url)]

        elif self.name_is_conformance:
            rv = ConformanceCheckerTest.item_type, [ConformanceCheckerTest(self, self.url)]

        elif self.name_is_conformance_support:
            rv = "support", [SupportFile(self)]

        elif self.name_is_visual:
            rv = VisualTest.item_type, [VisualTest(self, self.url)]

        elif self.name_is_multi_global:
            rv = TestharnessTest.item_type, [
                TestharnessTest(self, replace_end(self.url, ".any.js", ".any.html"), timeout=self.timeout),
                TestharnessTest(self, replace_end(self.url, ".any.js", ".any.worker.html"), timeout=self.timeout),
            ]

        elif self.name_is_worker:
            rv = (TestharnessTest.item_type,
                  [TestharnessTest(self, replace_end(self.url, ".worker.js", ".worker.html"),
                                   timeout=self.timeout)])

        elif self.name_is_webdriver:
            rv = WebdriverSpecTest.item_type, [WebdriverSpecTest(self, self.url)]

        elif self.content_is_css_manual and not self.name_is_reference:
            rv = ManualTest.item_type, [ManualTest(self, self.url)]

        elif self.content_is_testharness:
            rv = TestharnessTest.item_type, []
            for variant in self.test_variants:
                url = self.url + variant
                rv[1].append(TestharnessTest(self, url, timeout=self.timeout))

        elif self.content_is_ref_node:
            rv = (RefTestNode.item_type,
                  [RefTestNode(self, self.url, self.references, timeout=self.timeout,
                               viewport_size=self.viewport_size, dpi=self.dpi)])

        elif self.content_is_css_visual and not self.name_is_reference:
            rv = VisualTest.item_type, [VisualTest(self, self.url)]

        else:
            rv = "support", [SupportFile(self)]

        self.items_cache = rv

        return rv
