import hashlib
import re
import os
from collections import deque
from six import binary_type
from six.moves.urllib.parse import urljoin
from fnmatch import fnmatch
try:
    from xml.etree import cElementTree as ElementTree
except ImportError:
    from xml.etree import ElementTree

import html5lib

from . import XMLParser
from .item import Stub, ManualTest, WebDriverSpecTest, RefTestNode, TestharnessTest, SupportFile, ConformanceCheckerTest, VisualTest
from .utils import ContextManagerBytesIO, cached_property

wd_pattern = "*.py"
js_meta_re = re.compile(b"//\s*META:\s*(\w*)=(.*)$")
python_meta_re = re.compile(b"#\s*META:\s*(\w*)=(.*)$")

reference_file_re = re.compile(r'(^|[\-_])(not)?ref[0-9]*([\-_]|$)')

space_chars = u"".join(html5lib.constants.spaceCharacters)

def replace_end(s, old, new):
    """
    Given a string `s` that ends with `old`, replace that occurrence of `old`
    with `new`.
    """
    assert s.endswith(old)
    return s[:-len(old)] + new


def read_script_metadata(f, regexp):
    """
    Yields any metadata (pairs of bytestrings) from the file-like object `f`,
    as specified according to a supplied regexp.

    `regexp` - Regexp containing two groups containing the metadata name and
               value.
    """
    for line in f:
        assert isinstance(line, binary_type), line
        m = regexp.match(line)
        if not m:
            break

        yield (m.groups()[0], m.groups()[1])


_any_variants = {
    b"default": {"longhand": {b"window", b"dedicatedworker"}},
    b"window": {"suffix": ".any.html"},
    b"serviceworker": {"force_https": True},
    b"sharedworker": {},
    b"dedicatedworker": {"suffix": ".any.worker.html"},
    b"worker": {"longhand": {b"dedicatedworker", b"sharedworker", b"serviceworker"}},
    b"jsshell": {"suffix": ".any.js"},
}


def get_any_variants(item):
    """
    Returns a set of variants (bytestrings) defined by the given keyword.
    """
    assert isinstance(item, binary_type), item
    assert not item.startswith(b"!"), item

    variant = _any_variants.get(item, None)
    if variant is None:
        return set()

    return variant.get("longhand", {item})


def get_default_any_variants():
    """
    Returns a set of variants (bytestrings) that will be used by default.
    """
    return set(_any_variants[b"default"]["longhand"])


def parse_variants(value):
    """
    Returns a set of variants (bytestrings) defined by a comma-separated value.
    """
    assert isinstance(value, binary_type), value

    globals = get_default_any_variants()

    for item in value.split(b","):
        item = item.strip()
        if item.startswith(b"!"):
            globals -= get_any_variants(item[1:])
        else:
            globals |= get_any_variants(item)

    return globals


def global_suffixes(value):
    """
    Yields tuples of the relevant filename suffix (a string) and whether the
    variant is intended to run in a JS shell, for the variants defined by the
    given comma-separated value.
    """
    assert isinstance(value, binary_type), value

    rv = set()

    global_types = parse_variants(value)
    for global_type in global_types:
        variant = _any_variants[global_type]
        suffix = variant.get("suffix", ".any.%s.html" % global_type.decode("utf-8"))
        rv.add((suffix, global_type == b"jsshell"))

    return rv


def global_variant_url(url, suffix):
    """
    Returns a url created from the given url and suffix (all strings).
    """
    url = url.replace(".any.", ".")
    # If the url must be loaded over https, ensure that it will have
    # the form .https.any.js
    if ".https." in url and suffix.startswith(".https."):
        url = url.replace(".https.", ".")
    return replace_end(url, ".js", suffix)


def _parse_xml(f):
    try:
        # raises ValueError with an unsupported encoding,
        # ParseError when there's an undefined entity
        return ElementTree.parse(f)
    except (ValueError, ElementTree.ParseError):
        f.seek(0)
        return ElementTree.parse(f, XMLParser.XMLParser())


class SourceFile(object):
    parsers = {"html":lambda x:html5lib.parse(x, treebuilder="etree", useChardet=False),
               "xhtml":_parse_xml,
               "svg":_parse_xml}

    root_dir_non_test = set(["common"])

    dir_non_test = set(["resources",
                        "support",
                        "tools"])

    dir_path_non_test = {("css21", "archive"),
                         ("css", "CSS2", "archive"),
                         ("css", "common")}

    def __init__(self, tests_root, rel_path, url_base, hash=None, contents=None):
        """Object representing a file in a source tree.

        :param tests_root: Path to the root of the source tree
        :param rel_path: File path relative to tests_root
        :param url_base: Base URL used when converting file paths to urls
        :param contents: Byte array of the contents of the file or ``None``.
        """

        assert not os.path.isabs(rel_path), rel_path

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
        self._hash = hash

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
    def rel_url(self):
        assert not os.path.isabs(self.rel_path), self.rel_path
        return self.rel_path.replace(os.sep, "/")

    @cached_property
    def url(self):
        return urljoin(self.url_base, self.rel_url)

    @cached_property
    def hash(self):
        if not self._hash:
            with self.open() as f:
                content = f.read()
            data = "".join(("blob ", str(len(content)), "\0", content))
            self._hash = hashlib.sha1(data).hexdigest()
        return self._hash

    def in_non_test_dir(self):
        if self.dir_path == "":
            return True

        parts = self.dir_path.split(os.path.sep)

        if (parts[0] in self.root_dir_non_test or
            any(item in self.dir_non_test for item in parts) or
            any(parts[:len(path)] == list(path) for path in self.dir_path_non_test)):
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
                self.filename == "META.yml" or
                self.filename.startswith(".") or
                self.filename.endswith(".headers") or
                self.type_flag == "support" or
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
    def name_is_window(self):
        """Check if the file name matches the conditions for the file to
        be a window js test file"""
        return "window" in self.meta_flags and self.ext == ".js"

    @property
    def name_is_webdriver(self):
        """Check if the file name matches the conditions for the file to
        be a webdriver spec test file"""
        # wdspec tests are in subdirectories of /webdriver excluding __init__.py
        # files.
        rel_dir_tree = self.rel_path.split(os.path.sep)
        return (((rel_dir_tree[0] == "webdriver" and len(rel_dir_tree) > 1) or
                 (rel_dir_tree[:2] == ["infrastructure", "webdriver"] and
                  len(rel_dir_tree) > 2)) and
                self.filename not in ("__init__.py", "conftest.py") and
                fnmatch(self.filename, wd_pattern))

    @property
    def name_is_reference(self):
        """Check if the file name matches the conditions for the file to
        be a reference file (not a reftest)"""
        return "/reference/" in self.url or bool(reference_file_re.search(self.name))

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
        if self.name_is_worker or self.name_is_multi_global or self.name_is_window:
            regexp = js_meta_re
        elif self.name_is_webdriver:
            regexp = python_meta_re
        else:
            return None

        with self.open() as f:
            return list(read_script_metadata(f, regexp))

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
    def fuzzy_nodes(self):
        """List of ElementTree Elements corresponding to nodes in a test that
        specify reftest fuzziness"""
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='fuzzy']")

    @cached_property
    def fuzzy(self):
        rv = {}
        if self.root is None:
            return rv

        if not self.fuzzy_nodes:
            return rv

        args = ["maxDifference", "totalPixels"]

        for node in self.fuzzy_nodes:
            item = node.attrib.get("content", "")

            parts = item.rsplit(":", 1)
            if len(parts) == 1:
                key = None
                value = parts[0]
            else:
                key = urljoin(self.url, parts[0])
                reftype = None
                for ref in self.references:
                    if ref[0] == key:
                        reftype = ref[1]
                        break
                if reftype not in ("==", "!="):
                    raise ValueError("Fuzzy key %s doesn't correspond to a references" % key)
                key = (self.url, key, reftype)
                value = parts[1]
            ranges = value.split(";")
            if len(ranges) != 2:
                raise ValueError("Malformed fuzzy value %s" % item)
            arg_values = {None: deque()}
            for range_str_value in ranges:
                if "=" in range_str_value:
                    name, range_str_value = [part.strip()
                                             for part in range_str_value.split("=", 1)]
                    if name not in args:
                        raise ValueError("%s is not a valid fuzzy property" % name)
                    if arg_values.get(name):
                        raise ValueError("Got multiple values for argument %s" % name)
                else:
                    name = None
                if "-" in range_str_value:
                    range_min, range_max = range_str_value.split("-")
                else:
                    range_min = range_str_value
                    range_max = range_str_value
                try:
                    range_value = [int(x.strip()) for x in (range_min, range_max)]
                except ValueError:
                    raise ValueError("Fuzzy value %s must be a range of integers" %
                                     range_str_value)
                if name is None:
                    arg_values[None].append(range_value)
                else:
                    arg_values[name] = range_value
            rv[key] = []
            for arg_name in args:
                if arg_values.get(arg_name):
                    value = arg_values.pop(arg_name)
                else:
                    value = arg_values[None].popleft()
                rv[key].append(value)
            assert list(arg_values.keys()) == [None] and len(arg_values[None]) == 0
        return rv

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
        if self.ext == ".js":
            for (key, value) in self.script_metadata:
                if key == b"variant":
                    rv.append(value.decode("utf-8"))
        else:
            for element in self.variant_nodes:
                if "content" in element.attrib:
                    variant = element.attrib["content"]
                    rv.append(variant)

        for variant in rv:
            assert variant == "" or variant[0] in ["#", "?"], variant

        if not rv:
            rv = [""]

        return rv

    @cached_property
    def testdriver_nodes(self):
        """List of ElementTree Elements corresponding to nodes representing a
        testdriver.js script"""
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testdriver.js']")

    @cached_property
    def has_testdriver(self):
        """Boolean indicating whether the file content represents a
        testharness.js test"""
        if self.root is None:
            return None
        return bool(self.testdriver_nodes)

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
                ref_url = urljoin(self.url, item.attrib["href"].strip(space_chars))
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
                rv.add(item.attrib["href"].strip(space_chars))
        return rv

    @cached_property
    def content_is_css_visual(self):
        """Boolean indicating whether the file content represents a
        CSS WG-style visual test"""
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
            rv = "support", [
                SupportFile(
                    self.tests_root,
                    self.rel_path
                )]

        elif self.name_is_stub:
            rv = Stub.item_type, [
                Stub(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url
                )]

        elif self.name_is_manual:
            rv = ManualTest.item_type, [
                ManualTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url
                )]

        elif self.name_is_conformance:
            rv = ConformanceCheckerTest.item_type, [
                ConformanceCheckerTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url
                )]

        elif self.name_is_conformance_support:
            rv = "support", [
                SupportFile(
                    self.tests_root,
                    self.rel_path
                )]

        elif self.name_is_visual:
            rv = VisualTest.item_type, [
                VisualTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url
                )]

        elif self.name_is_multi_global:
            globals = b""
            for (key, value) in self.script_metadata:
                if key == b"global":
                    globals = value
                    break

            tests = [
                TestharnessTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    global_variant_url(self.rel_url, suffix) + variant,
                    timeout=self.timeout,
                    jsshell=jsshell,
                    script_metadata=self.script_metadata
                )
                for (suffix, jsshell) in sorted(global_suffixes(globals))
                for variant in self.test_variants
            ]
            rv = TestharnessTest.item_type, tests

        elif self.name_is_worker:
            test_url = replace_end(self.rel_url, ".worker.js", ".worker.html")
            tests = [
                TestharnessTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    test_url + variant,
                    timeout=self.timeout,
                    script_metadata=self.script_metadata
                )
                for variant in self.test_variants
            ]
            rv = TestharnessTest.item_type, tests

        elif self.name_is_window:
            test_url = replace_end(self.rel_url, ".window.js", ".window.html")
            tests = [
                TestharnessTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    test_url + variant,
                    timeout=self.timeout,
                    script_metadata=self.script_metadata
                )
                for variant in self.test_variants
            ]
            rv = TestharnessTest.item_type, tests

        elif self.name_is_webdriver:
            rv = WebDriverSpecTest.item_type, [
                WebDriverSpecTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url,
                    timeout=self.timeout
                )]

        elif self.content_is_css_manual and not self.name_is_reference:
            rv = ManualTest.item_type, [
                ManualTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url
                )]

        elif self.content_is_testharness:
            rv = TestharnessTest.item_type, []
            testdriver = self.has_testdriver
            for variant in self.test_variants:
                url = self.rel_url + variant
                rv[1].append(TestharnessTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    url,
                    timeout=self.timeout,
                    testdriver=testdriver,
                    script_metadata=self.script_metadata
                ))

        elif self.content_is_ref_node:
            rv = RefTestNode.item_type, [
                RefTestNode(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url,
                    references=self.references,
                    timeout=self.timeout,
                    viewport_size=self.viewport_size,
                    dpi=self.dpi,
                    fuzzy=self.fuzzy
                )]

        elif self.content_is_css_visual and not self.name_is_reference:
            rv = VisualTest.item_type, [
                VisualTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url
                )]

        else:
            rv = "support", [
                SupportFile(
                    self.tests_root,
                    self.rel_path
                )]

        assert len(rv[1]) == len(set(rv[1]))

        self.items_cache = rv

        return rv
