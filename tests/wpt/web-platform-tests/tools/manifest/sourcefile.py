import hashlib
import re
import os
from collections import deque
from io import BytesIO
from urllib.parse import urljoin
from fnmatch import fnmatch

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any
    from typing import BinaryIO
    from typing import Callable
    from typing import Deque
    from typing import Dict
    from typing import Iterable
    from typing import List
    from typing import Optional
    from typing import Pattern
    from typing import Set
    from typing import Text
    from typing import Tuple
    from typing import Union
    from typing import cast

try:
    from xml.etree import cElementTree as ElementTree
except ImportError:
    from xml.etree import ElementTree as ElementTree  # type: ignore

import html5lib

from . import XMLParser
from .item import (ConformanceCheckerTest,
                   CrashTest,
                   ManifestItem,
                   ManualTest,
                   PrintRefTest,
                   RefTest,
                   SupportFile,
                   TestharnessTest,
                   VisualTest,
                   WebDriverSpecTest)
from .utils import cached_property

wd_pattern = "*.py"
js_meta_re = re.compile(br"//\s*META:\s*(\w*)=(.*)$")
python_meta_re = re.compile(br"#\s*META:\s*(\w*)=(.*)$")

reference_file_re = re.compile(r'(^|[\-_])(not)?ref[0-9]*([\-_]|$)')

space_chars = u"".join(html5lib.constants.spaceCharacters)  # type: Text


def replace_end(s, old, new):
    # type: (Text, Text, Text) -> Text
    """
    Given a string `s` that ends with `old`, replace that occurrence of `old`
    with `new`.
    """
    assert s.endswith(old)
    return s[:-len(old)] + new


def read_script_metadata(f, regexp):
    # type: (BinaryIO, Pattern[bytes]) -> Iterable[Tuple[Text, Text]]
    """
    Yields any metadata (pairs of strings) from the file-like object `f`,
    as specified according to a supplied regexp.

    `regexp` - Regexp containing two groups containing the metadata name and
               value.
    """
    for line in f:
        assert isinstance(line, bytes), line
        m = regexp.match(line)
        if not m:
            break

        yield (m.groups()[0].decode("utf8"), m.groups()[1].decode("utf8"))


_any_variants = {
    "window": {"suffix": ".any.html"},
    "serviceworker": {"force_https": True},
    "serviceworker-module": {"force_https": True},
    "sharedworker": {},
    "sharedworker-module": {},
    "dedicatedworker": {"suffix": ".any.worker.html"},
    "dedicatedworker-module": {"suffix": ".any.worker-module.html"},
    "worker": {"longhand": {"dedicatedworker", "sharedworker", "serviceworker"}},
    "worker-module": {},
    "jsshell": {"suffix": ".any.js"},
}  # type: Dict[Text, Dict[Text, Any]]


def get_any_variants(item):
    # type: (Text) -> Set[Text]
    """
    Returns a set of variants (strings) defined by the given keyword.
    """
    assert isinstance(item, str), item

    variant = _any_variants.get(item, None)
    if variant is None:
        return set()

    return variant.get("longhand", {item})


def get_default_any_variants():
    # type: () -> Set[Text]
    """
    Returns a set of variants (strings) that will be used by default.
    """
    return set({"window", "dedicatedworker"})


def parse_variants(value):
    # type: (Text) -> Set[Text]
    """
    Returns a set of variants (strings) defined by a comma-separated value.
    """
    assert isinstance(value, str), value

    if value == "":
        return get_default_any_variants()

    globals = set()
    for item in value.split(","):
        item = item.strip()
        globals |= get_any_variants(item)
    return globals


def global_suffixes(value):
    # type: (Text) -> Set[Tuple[Text, bool]]
    """
    Yields tuples of the relevant filename suffix (a string) and whether the
    variant is intended to run in a JS shell, for the variants defined by the
    given comma-separated value.
    """
    assert isinstance(value, str), value

    rv = set()

    global_types = parse_variants(value)
    for global_type in global_types:
        variant = _any_variants[global_type]
        suffix = variant.get("suffix", ".any.%s.html" % global_type)
        rv.add((suffix, global_type == "jsshell"))

    return rv


def global_variant_url(url, suffix):
    # type: (Text, Text) -> Text
    """
    Returns a url created from the given url and suffix (all strings).
    """
    url = url.replace(".any.", ".")
    # If the url must be loaded over https, ensure that it will have
    # the form .https.any.js
    if ".https." in url and suffix.startswith(".https."):
        url = url.replace(".https.", ".")
    elif ".h2." in url and suffix.startswith(".h2."):
        url = url.replace(".h2.", ".")
    return replace_end(url, ".js", suffix)


def _parse_html(f):
    # type: (BinaryIO) -> ElementTree.Element
    doc = html5lib.parse(f, treebuilder="etree", useChardet=False)
    if MYPY:
        return cast(ElementTree.Element, doc)
    return doc

def _parse_xml(f):
    # type: (BinaryIO) -> ElementTree.Element
    try:
        # raises ValueError with an unsupported encoding,
        # ParseError when there's an undefined entity
        return ElementTree.parse(f).getroot()
    except (ValueError, ElementTree.ParseError):
        f.seek(0)
        return ElementTree.parse(f, XMLParser.XMLParser()).getroot()  # type: ignore


class SourceFile(object):
    parsers = {u"html":_parse_html,
               u"xhtml":_parse_xml,
               u"svg":_parse_xml}  # type: Dict[Text, Callable[[BinaryIO], ElementTree.Element]]

    root_dir_non_test = {u"common"}

    dir_non_test = {u"resources",
                    u"support",
                    u"tools"}

    dir_path_non_test = {(u"css21", u"archive"),
                         (u"css", u"CSS2", u"archive"),
                         (u"css", u"common")}  # type: Set[Tuple[Text, ...]]

    def __init__(self, tests_root, rel_path, url_base, hash=None, contents=None):
        # type: (Text, Text, Text, Optional[Text], Optional[bytes]) -> None
        """Object representing a file in a source tree.

        :param tests_root: Path to the root of the source tree
        :param rel_path_str: File path relative to tests_root
        :param url_base: Base URL used when converting file paths to urls
        :param contents: Byte array of the contents of the file or ``None``.
        """

        assert not os.path.isabs(rel_path), rel_path
        if os.name == "nt":
            # do slash normalization on Windows
            rel_path = rel_path.replace(u"/", u"\\")

        dir_path, filename = os.path.split(rel_path)
        name, ext = os.path.splitext(filename)

        type_flag = None
        if "-" in name:
            type_flag = name.rsplit("-", 1)[1].split(".")[0]

        meta_flags = name.split(".")[1:]

        self.tests_root = tests_root  # type: Text
        self.rel_path = rel_path  # type: Text
        self.dir_path = dir_path  # type: Text
        self.filename = filename  # type: Text
        self.name = name  # type: Text
        self.ext = ext  # type: Text
        self.type_flag = type_flag  # type: Optional[Text]
        self.meta_flags = meta_flags  # type: Union[List[bytes], List[Text]]
        self.url_base = url_base
        self.contents = contents
        self.items_cache = None  # type: Optional[Tuple[Text, List[ManifestItem]]]
        self._hash = hash

    def __getstate__(self):
        # type: () -> Dict[str, Any]
        # Remove computed properties if we pickle this class
        rv = self.__dict__.copy()

        if "__cached_properties__" in rv:
            cached_properties = rv["__cached_properties__"]
            rv = {key:value for key, value in rv.items() if key not in cached_properties}
            del rv["__cached_properties__"]
        return rv

    def name_prefix(self, prefix):
        # type: (Text) -> bool
        """Check if the filename starts with a given prefix

        :param prefix: The prefix to check"""
        return self.name.startswith(prefix)

    def is_dir(self):
        # type: () -> bool
        """Return whether this file represents a directory."""
        if self.contents is not None:
            return False

        return os.path.isdir(self.rel_path)

    def open(self):
        # type: () -> BinaryIO
        """
        Return either
        * the contents specified in the constructor, if any;
        * a File object opened for reading the file contents.
        """
        if self.contents is not None:
            file_obj = BytesIO(self.contents)  # type: BinaryIO
        else:
            file_obj = open(self.path, 'rb')
        return file_obj

    @cached_property
    def rel_path_parts(self):
        # type: () -> Tuple[Text, ...]
        return tuple(self.rel_path.split(os.path.sep))

    @cached_property
    def path(self):
        # type: () -> Text
        return os.path.join(self.tests_root, self.rel_path)

    @cached_property
    def rel_url(self):
        # type: () -> Text
        assert not os.path.isabs(self.rel_path), self.rel_path
        return self.rel_path.replace(os.sep, "/")

    @cached_property
    def url(self):
        # type: () -> Text
        return urljoin(self.url_base, self.rel_url)

    @cached_property
    def hash(self):
        # type: () -> Text
        if not self._hash:
            with self.open() as f:
                content = f.read()

            data = b"".join((b"blob ", b"%d" % len(content), b"\0", content))
            self._hash = str(hashlib.sha1(data).hexdigest())

        return self._hash

    def in_non_test_dir(self):
        # type: () -> bool
        if self.dir_path == "":
            return True

        parts = self.rel_path_parts

        if (parts[0] in self.root_dir_non_test or
            any(item in self.dir_non_test for item in parts) or
            any(parts[:len(path)] == path for path in self.dir_path_non_test)):
            return True
        return False

    def in_conformance_checker_dir(self):
        # type: () -> bool
        return self.rel_path_parts[0] == "conformance-checkers"

    @property
    def name_is_non_test(self):
        # type: () -> bool
        """Check if the file name matches the conditions for the file to
        be a non-test file"""
        return (self.is_dir() or
                self.name_prefix(u"MANIFEST") or
                self.filename == u"META.yml" or
                self.filename.startswith(u".") or
                self.filename.endswith(u".headers") or
                self.filename.endswith(u".ini") or
                self.in_non_test_dir())

    @property
    def name_is_conformance(self):
        # type: () -> bool
        return (self.in_conformance_checker_dir() and
                self.type_flag in ("is-valid", "no-valid"))

    @property
    def name_is_conformance_support(self):
        # type: () -> bool
        return self.in_conformance_checker_dir()

    @property
    def name_is_manual(self):
        # type: () -> bool
        """Check if the file name matches the conditions for the file to
        be a manual test file"""
        return self.type_flag == "manual"

    @property
    def name_is_visual(self):
        # type: () -> bool
        """Check if the file name matches the conditions for the file to
        be a visual test file"""
        return self.type_flag == "visual"

    @property
    def name_is_multi_global(self):
        # type: () -> bool
        """Check if the file name matches the conditions for the file to
        be a multi-global js test file"""
        return "any" in self.meta_flags and self.ext == ".js"

    @property
    def name_is_worker(self):
        # type: () -> bool
        """Check if the file name matches the conditions for the file to
        be a worker js test file"""
        return "worker" in self.meta_flags and self.ext == ".js"

    @property
    def name_is_window(self):
        # type: () -> bool
        """Check if the file name matches the conditions for the file to
        be a window js test file"""
        return "window" in self.meta_flags and self.ext == ".js"

    @property
    def name_is_webdriver(self):
        # type: () -> bool
        """Check if the file name matches the conditions for the file to
        be a webdriver spec test file"""
        # wdspec tests are in subdirectories of /webdriver excluding __init__.py
        # files.
        rel_path_parts = self.rel_path_parts
        return (((rel_path_parts[0] == "webdriver" and len(rel_path_parts) > 1) or
                 (rel_path_parts[:2] == ("infrastructure", "webdriver") and
                  len(rel_path_parts) > 2)) and
                self.filename not in ("__init__.py", "conftest.py") and
                fnmatch(self.filename, wd_pattern))

    @property
    def name_is_reference(self):
        # type: () -> bool
        """Check if the file name matches the conditions for the file to
        be a reference file (not a reftest)"""
        return "/reference/" in self.url or bool(reference_file_re.search(self.name))

    @property
    def name_is_crashtest(self):
        # type: () -> bool
        return (self.markup_type is not None and
                (self.type_flag == "crash" or "crashtests" in self.dir_path.split(os.path.sep)))

    @property
    def name_is_tentative(self):
        # type: () -> bool
        """Check if the file name matches the conditions for the file to be a
        tentative file.

        See https://web-platform-tests.org/writing-tests/file-names.html#test-features"""
        return "tentative" in self.meta_flags or "tentative" in self.dir_path.split(os.path.sep)

    @property
    def name_is_print_reftest(self):
        # type: () -> bool
        return (self.markup_type is not None and
                (self.type_flag == "print" or "print" in self.dir_path.split(os.path.sep)))

    @property
    def markup_type(self):
        # type: () -> Optional[Text]
        """Return the type of markup contained in a file, based on its extension,
        or None if it doesn't contain markup"""
        ext = self.ext

        if not ext:
            return None
        if ext[0] == u".":
            ext = ext[1:]
        if ext in [u"html", u"htm"]:
            return u"html"
        if ext in [u"xhtml", u"xht", u"xml"]:
            return u"xhtml"
        if ext == u"svg":
            return u"svg"
        return None

    @cached_property
    def root(self):
        # type: () -> Optional[ElementTree.Element]
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

        return tree

    @cached_property
    def timeout_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes in a test that
        specify timeouts"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='timeout']")

    @cached_property
    def script_metadata(self):
        # type: () -> Optional[List[Tuple[Text, Text]]]
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
        # type: () -> Optional[Text]
        """The timeout of a test or reference file. "long" if the file has an extended timeout
        or None otherwise"""
        if self.script_metadata:
            if any(m == ("timeout", "long") for m in self.script_metadata):
                return "long"

        if self.root is None:
            return None

        if self.timeout_nodes:
            timeout_str = self.timeout_nodes[0].attrib.get("content", None)  # type: Optional[Text]
            if timeout_str and timeout_str.lower() == "long":
                return "long"

        return None

    @cached_property
    def viewport_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes in a test that
        specify viewport sizes"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='viewport-size']")

    @cached_property
    def viewport_size(self):
        # type: () -> Optional[Text]
        """The viewport size of a test or reference file"""
        if self.root is None:
            return None

        if not self.viewport_nodes:
            return None

        return self.viewport_nodes[0].attrib.get("content", None)

    @cached_property
    def dpi_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes in a test that
        specify device pixel ratios"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='device-pixel-ratio']")

    @cached_property
    def dpi(self):
        # type: () -> Optional[Text]
        """The device pixel ratio of a test or reference file"""
        if self.root is None:
            return None

        if not self.dpi_nodes:
            return None

        return self.dpi_nodes[0].attrib.get("content", None)

    def parse_ref_keyed_meta(self, node):
        # type: (ElementTree.Element) -> Tuple[Optional[Tuple[Text, Text, Text]], Text]
        item = node.attrib.get(u"content", u"")  # type: Text

        parts = item.rsplit(u":", 1)
        if len(parts) == 1:
            key = None  # type: Optional[Tuple[Text, Text, Text]]
            value = parts[0]
        else:
            key_part = urljoin(self.url, parts[0])
            reftype = None
            for ref in self.references:  # type: Tuple[Text, Text]
                if ref[0] == key_part:
                    reftype = ref[1]
                    break
            if reftype not in (u"==", u"!="):
                raise ValueError("Key %s doesn't correspond to a reference" % key_part)
            key = (self.url, key_part, reftype)
            value = parts[1]

        return key, value


    @cached_property
    def fuzzy_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes in a test that
        specify reftest fuzziness"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='fuzzy']")


    @cached_property
    def fuzzy(self):
        # type: () -> Dict[Optional[Tuple[Text, Text, Text]], List[List[int]]]
        rv = {}  # type: Dict[Optional[Tuple[Text, Text, Text]], List[List[int]]]
        if self.root is None:
            return rv

        if not self.fuzzy_nodes:
            return rv

        args = [u"maxDifference", u"totalPixels"]

        for node in self.fuzzy_nodes:
            key, value = self.parse_ref_keyed_meta(node)
            ranges = value.split(u";")
            if len(ranges) != 2:
                raise ValueError("Malformed fuzzy value %s" % value)
            arg_values = {}  # type: Dict[Text, List[int]]
            positional_args = deque()  # type: Deque[List[int]]
            for range_str_value in ranges:  # type: Text
                name = None  # type: Optional[Text]
                if u"=" in range_str_value:
                    name, range_str_value = [part.strip()
                                             for part in range_str_value.split(u"=", 1)]
                    if name not in args:
                        raise ValueError("%s is not a valid fuzzy property" % name)
                    if arg_values.get(name):
                        raise ValueError("Got multiple values for argument %s" % name)
                if u"-" in range_str_value:
                    range_min, range_max = range_str_value.split(u"-")
                else:
                    range_min = range_str_value
                    range_max = range_str_value
                try:
                    range_value = [int(x.strip()) for x in (range_min, range_max)]
                except ValueError:
                    raise ValueError("Fuzzy value %s must be a range of integers" %
                                     range_str_value)
                if name is None:
                    positional_args.append(range_value)
                else:
                    arg_values[name] = range_value
            rv[key] = []
            for arg_name in args:
                if arg_values.get(arg_name):
                    arg_value = arg_values.pop(arg_name)
                else:
                    arg_value = positional_args.popleft()
                rv[key].append(arg_value)
            assert len(arg_values) == 0 and len(positional_args) == 0
        return rv

    @cached_property
    def page_ranges_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes in a test that
        specify print-reftest """
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='reftest-pages']")

    @cached_property
    def page_ranges(self):
        # type: () -> Dict[Text, List[List[Optional[int]]]]
        """List of ElementTree Elements corresponding to nodes in a test that
        specify print-reftest page ranges"""
        rv = {}  # type: Dict[Text, List[List[Optional[int]]]]
        for node in self.page_ranges_nodes:
            key_data, value = self.parse_ref_keyed_meta(node)
            # Just key by url
            if key_data is None:
                key = self.url
            else:
                key = key_data[1]
            if key in rv:
                raise ValueError("Duplicate page-ranges value")
            rv[key] = []
            for range_str in value.split(","):
                range_str = range_str.strip()
                if "-" in range_str:
                    range_parts_str = [item.strip() for item in range_str.split("-")]
                    try:
                        range_parts = [int(item) if item else None for item in range_parts_str]
                    except ValueError:
                        raise ValueError("Malformed page-range value %s" % range_str)
                    if any(item == 0 for item in range_parts):
                        raise ValueError("Malformed page-range value %s" % range_str)
                else:
                    try:
                        range_parts = [int(range_str)]
                    except ValueError:
                        raise ValueError("Malformed page-range value %s" % range_str)
                rv[key].append(range_parts)
        return rv

    @cached_property
    def testharness_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes representing a
        testharness.js script"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testharness.js']")

    @cached_property
    def content_is_testharness(self):
        # type: () -> Optional[bool]
        """Boolean indicating whether the file content represents a
        testharness.js test"""
        if self.root is None:
            return None
        return bool(self.testharness_nodes)

    @cached_property
    def variant_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes representing a
        test variant"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='variant']")

    @cached_property
    def test_variants(self):
        # type: () -> List[Text]
        rv = []  # type: List[Text]
        if self.ext == ".js":
            script_metadata = self.script_metadata
            assert script_metadata is not None
            for (key, value) in script_metadata:
                if key == "variant":
                    rv.append(value)
        else:
            for element in self.variant_nodes:
                if "content" in element.attrib:
                    variant = element.attrib["content"]  # type: Text
                    rv.append(variant)

        for variant in rv:
            assert variant == "" or variant[0] in ["#", "?"], variant

        if not rv:
            rv = [""]

        return rv

    @cached_property
    def testdriver_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes representing a
        testdriver.js script"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testdriver.js']")

    @cached_property
    def has_testdriver(self):
        # type: () -> Optional[bool]
        """Boolean indicating whether the file content represents a
        testharness.js test"""
        if self.root is None:
            return None
        return bool(self.testdriver_nodes)

    @cached_property
    def quic_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes in a test that
        specify whether it needs QUIC server."""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='quic']")

    @cached_property
    def quic(self):
        # type: () -> Optional[bool]
        """Boolean indicating whether a test requires QUIC server

        Determined by <meta> elements (`quic_nodes()`) and "// META" comments
        (`script_metadata()`).
        """
        if self.script_metadata:
            if any(m == ("quic", "true") for m in self.script_metadata):
                return True

        if self.root is None:
            return None

        if self.quic_nodes:
            quic_str = self.quic_nodes[0].attrib.get("content", "false")  # type: Text
            if quic_str.lower() == "true":
                return True

        return None

    @cached_property
    def reftest_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes representing a
        to a reftest <link>"""
        if self.root is None:
            return []

        match_links = self.root.findall(".//{http://www.w3.org/1999/xhtml}link[@rel='match']")
        mismatch_links = self.root.findall(".//{http://www.w3.org/1999/xhtml}link[@rel='mismatch']")
        return match_links + mismatch_links

    @cached_property
    def references(self):
        # type: () -> List[Tuple[Text, Text]]
        """List of (ref_url, relation) tuples for any reftest references specified in
        the file"""
        rv = []  # type: List[Tuple[Text, Text]]
        rel_map = {"match": "==", "mismatch": "!="}
        for item in self.reftest_nodes:
            if "href" in item.attrib:
                ref_url = urljoin(self.url, item.attrib["href"].strip(space_chars))
                ref_type = rel_map[item.attrib["rel"]]
                rv.append((ref_url, ref_type))
        return rv

    @cached_property
    def content_is_ref_node(self):
        # type: () -> bool
        """Boolean indicating whether the file is a non-leaf node in a reftest
        graph (i.e. if it contains any <link rel=[mis]match>"""
        return bool(self.references)

    @cached_property
    def css_flag_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes representing a
        flag <meta>"""
        if self.root is None:
            return []
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='flags']")

    @cached_property
    def css_flags(self):
        # type: () -> Set[Text]
        """Set of flags specified in the file"""
        rv = set()  # type: Set[Text]
        for item in self.css_flag_nodes:
            if "content" in item.attrib:
                for flag in item.attrib["content"].split():
                    rv.add(flag)
        return rv

    @cached_property
    def content_is_css_manual(self):
        # type: () -> Optional[bool]
        """Boolean indicating whether the file content represents a
        CSS WG-style manual test"""
        if self.root is None:
            return None
        # return True if the intersection between the two sets is non-empty
        return bool(self.css_flags & {"animated", "font", "history", "interact", "paged", "speech", "userstyle"})

    @cached_property
    def spec_link_nodes(self):
        # type: () -> List[ElementTree.Element]
        """List of ElementTree Elements corresponding to nodes representing a
        <link rel=help>, used to point to specs"""
        if self.root is None:
            return []
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}link[@rel='help']")

    @cached_property
    def spec_links(self):
        # type: () -> Set[Text]
        """Set of spec links specified in the file"""
        rv = set()  # type: Set[Text]
        for item in self.spec_link_nodes:
            if "href" in item.attrib:
                rv.add(item.attrib["href"].strip(space_chars))
        return rv

    @cached_property
    def content_is_css_visual(self):
        # type: () -> Optional[bool]
        """Boolean indicating whether the file content represents a
        CSS WG-style visual test"""
        if self.root is None:
            return None
        return bool(self.ext in {'.xht', '.html', '.xhtml', '.htm', '.xml', '.svg'} and
                    self.spec_links)

    @property
    def type(self):
        # type: () -> Text
        possible_types = self.possible_types
        if len(possible_types) == 1:
            return possible_types.pop()

        rv, _ = self.manifest_items()
        return rv

    @property
    def possible_types(self):
        # type: () -> Set[Text]
        """Determines the set of possible types without reading the file"""

        if self.items_cache:
            return {self.items_cache[0]}

        if self.name_is_non_test:
            return {SupportFile.item_type}

        if self.name_is_manual:
            return {ManualTest.item_type}

        if self.name_is_conformance:
            return {ConformanceCheckerTest.item_type}

        if self.name_is_conformance_support:
            return {SupportFile.item_type}

        if self.name_is_webdriver:
            return {WebDriverSpecTest.item_type}

        if self.name_is_visual:
            return {VisualTest.item_type}

        if self.name_is_crashtest:
            return {CrashTest.item_type}

        if self.name_is_print_reftest:
            return {PrintRefTest.item_type}

        if self.name_is_multi_global:
            return {TestharnessTest.item_type}

        if self.name_is_worker:
            return {TestharnessTest.item_type}

        if self.name_is_window:
            return {TestharnessTest.item_type}

        if self.markup_type is None:
            return {SupportFile.item_type}

        if not self.name_is_reference:
            return {ManualTest.item_type,
                    TestharnessTest.item_type,
                    RefTest.item_type,
                    VisualTest.item_type,
                    SupportFile.item_type}

        return {TestharnessTest.item_type,
                RefTest.item_type,
                SupportFile.item_type}

    def manifest_items(self):
        # type: () -> Tuple[Text, List[ManifestItem]]
        """List of manifest items corresponding to the file. There is typically one
        per test, but in the case of reftests a node may have corresponding manifest
        items without being a test itself."""

        if self.items_cache:
            return self.items_cache

        drop_cached = "root" not in self.__dict__

        if self.name_is_non_test:
            rv = "support", [
                SupportFile(
                    self.tests_root,
                    self.rel_path
                )]  # type: Tuple[Text, List[ManifestItem]]

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

        elif self.name_is_webdriver:
            rv = WebDriverSpecTest.item_type, [
                WebDriverSpecTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url,
                    timeout=self.timeout
                )]

        elif self.name_is_visual:
            rv = VisualTest.item_type, [
                VisualTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url
                )]

        elif self.name_is_crashtest:
            rv = CrashTest.item_type, [
                CrashTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url
                )]

        elif self.name_is_print_reftest:
            references = self.references
            if not references:
                raise ValueError("%s detected as print reftest but doesn't have any refs" %
                                 self.path)
            rv = PrintRefTest.item_type, [
                PrintRefTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url,
                    references=references,
                    timeout=self.timeout,
                    viewport_size=self.viewport_size,
                    fuzzy=self.fuzzy,
                    page_ranges=self.page_ranges,
                )]

        elif self.name_is_multi_global:
            globals = u""
            script_metadata = self.script_metadata
            assert script_metadata is not None
            for (key, value) in script_metadata:
                if key == "global":
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
                    quic=self.quic,
                    script_metadata=self.script_metadata
                )
                for (suffix, jsshell) in sorted(global_suffixes(globals))
                for variant in self.test_variants
            ]   # type: List[ManifestItem]
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
                    quic=self.quic,
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
                    quic=self.quic,
                    script_metadata=self.script_metadata
                )
                for variant in self.test_variants
            ]
            rv = TestharnessTest.item_type, tests

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
                    quic=self.quic,
                    testdriver=testdriver,
                    script_metadata=self.script_metadata
                ))

        elif self.content_is_ref_node:
            rv = RefTest.item_type, [
                RefTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    self.rel_url,
                    references=self.references,
                    timeout=self.timeout,
                    quic=self.quic,
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

        assert rv[0] in self.possible_types
        assert len(rv[1]) == len(set(rv[1]))

        self.items_cache = rv

        if drop_cached and "__cached_properties__" in self.__dict__:
            cached_properties = self.__dict__["__cached_properties__"]
            for prop in cached_properties:
                if prop in self.__dict__:
                    del self.__dict__[prop]
            del self.__dict__["__cached_properties__"]

        return rv
