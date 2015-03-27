import os
import urlparse
from fnmatch import fnmatch
try:
    from xml.etree import cElementTree as ElementTree
except ImportError:
    from xml.etree import ElementTree

import html5lib

import vcs
from item import Stub, ManualTest, WebdriverSpecTest, RefTest, TestharnessTest
from utils import rel_path_to_url, is_blacklisted, ContextManagerStringIO, cached_property

wd_pattern = "*.py"

class SourceFile(object):
    parsers = {"html":lambda x:html5lib.parse(x, treebuilder="etree"),
               "xhtml":ElementTree.parse,
               "svg":ElementTree.parse}

    def __init__(self, tests_root, rel_path, url_base, use_committed=False):
        """Object representing a file in a source tree.

        :param tests_root: Path to the root of the source tree
        :param rel_path: File path relative to tests_root
        :param url_base: Base URL used when converting file paths to urls
        :param use_committed: Work with the last committed version of the file
                              rather than the on-disk version.
        """

        self.tests_root = tests_root
        self.rel_path = rel_path
        self.url_base = url_base
        self.use_committed = use_committed

        self.url = rel_path_to_url(rel_path, url_base)
        self.path = os.path.join(tests_root, rel_path)

        self.dir_path, self.filename = os.path.split(self.path)
        self.name, self.ext = os.path.splitext(self.filename)

        self.type_flag = None
        if "-" in self.name:
            self.type_flag = self.name.rsplit("-", 1)[1]

        self.meta_flags = self.name.split(".")[1:]

    def name_prefix(self, prefix):
        """Check if the filename starts with a given prefix

        :param prefix: The prefix to check"""
        return self.name.startswith(prefix)

    def open(self):
        """Return a File object opened for reading the file contents,
        or the contents of the file when last committed, if
        use_comitted is true."""

        if self.use_committed:
            git = vcs.get_git_func(os.path.dirname(__file__))
            blob = git("show", "HEAD:%s" % self.rel_path)
            file_obj = ContextManagerStringIO(blob)
        else:
            file_obj = open(self.path)
        return file_obj

    @property
    def name_is_non_test(self):
        """Check if the file name matches the conditions for the file to
        be a non-test file"""
        return (os.path.isdir(self.rel_path) or
                self.name_prefix("MANIFEST") or
                self.filename.startswith(".") or
                is_blacklisted(self.url))

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
                len(rel_dir_tree) > 2 and
                self.filename != "__init__.py" and
                fnmatch(self.filename, wd_pattern))

    @property
    def name_is_reference(self):
        """Check if the file name matches the conditions for the file to
        be a reference file (not a reftest)"""
        return self.type_flag in ("ref", "notref")

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
        if ext in ["xhtml", "xht"]:
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
    def timeout(self):
        """The timeout of a test or reference file. "long" if the file has an extended timeout
        or None otherwise"""
        if not self.root:
            return

        if self.timeout_nodes:
            timeout_str = self.timeout_nodes[0].attrib.get("content", None)
            if timeout_str and timeout_str.lower() == "long":
                return timeout_str

    @cached_property
    def testharness_nodes(self):
        """List of ElementTree Elements corresponding to nodes representing a
        testharness.js script"""
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testharness.js']")

    @cached_property
    def content_is_testharness(self):
        """Boolean indicating whether the file content represents a
        testharness.js test"""
        if not self.root:
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
        if not self.root:
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
                ref_url = urlparse.urljoin(self.url, item.attrib["href"])
                ref_type = rel_map[item.attrib["rel"]]
                rv.append((ref_url, ref_type))
        return rv

    @cached_property
    def content_is_ref_node(self):
        """Boolean indicating whether the file is a non-leaf node in a reftest
        graph (i.e. if it contains any <link rel=[mis]match>"""
        return bool(self.references)

    def manifest_items(self):
        """List of manifest items corresponding to the file. There is typically one
        per test, but in the case of reftests a node may have corresponding manifest
        items without being a test itself."""

        if self.name_is_non_test:
            rv = []

        elif self.name_is_stub:
            rv = [Stub(self, self.url)]

        elif self.name_is_manual:
            rv = [ManualTest(self, self.url)]

        elif self.name_is_worker:
            rv = [TestharnessTest(self, self.url[:-3])]

        elif self.name_is_webdriver:
            rv = [WebdriverSpecTest(self)]

        elif self.content_is_testharness:
            rv = []
            for variant in self.test_variants:
                url = self.url + variant
                rv.append(TestharnessTest(self, url, timeout=self.timeout))

        elif self.content_is_ref_node:
            rv = [RefTest(self, self.url, self.references, timeout=self.timeout)]

        else:
            # If nothing else it's a helper file, which we don't have a specific type for
            rv = []

        return rv
