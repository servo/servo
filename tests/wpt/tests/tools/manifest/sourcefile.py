import hashlib
import re
import os
from collections import deque
from fnmatch import fnmatch
from io import BytesIO
from typing import (Any, BinaryIO, Callable, Deque, Dict, Iterable, List, Optional, Pattern,
                    Set, Text, Tuple, Union, cast)
from urllib.parse import urljoin

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
                   SpecItem,
                   SupportFile,
                   TestharnessTest,
                   VisualTest,
                   WebDriverSpecTest)
from .utils import cached_property

wd_pattern = "*.py"
js_meta_re = re.compile(br"//\s*META:\s*(\w*)=(.*)$")
python_meta_re = re.compile(br"#\s*META:\s*(\w*)=(.*)$")

reference_file_re = re.compile(r'(^|[\-_])(not)?ref[0-9]*([\-_]|$)')

space_chars: Text = "".join(html5lib.constants.spaceCharacters)


def replace_end(s: Text, old: Text, new: Text) -> Text:
    """
    Given a string `s` that ends with `old`, replace that occurrence of `old`
    with `new`.
    """
    assert s.endswith(old)
    return s[:-len(old)] + new


def read_script_metadata(f: BinaryIO, regexp: Pattern[bytes]) -> Iterable[Tuple[Text, Text]]:
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


_any_variants: Dict[Text, Dict[Text, Any]] = {
    "window": {"suffix": ".any.html"},
    "serviceworker": {"force_https": True},
    "serviceworker-module": {"force_https": True},
    "sharedworker": {},
    "sharedworker-module": {},
    "dedicatedworker": {"suffix": ".any.worker.html"},
    "dedicatedworker-module": {"suffix": ".any.worker-module.html"},
    "worker": {"longhand": {"dedicatedworker", "sharedworker", "serviceworker"}},
    "worker-module": {},
    "shadowrealm": {},
    "jsshell": {"suffix": ".any.js"},
}


def get_any_variants(item: Text) -> Set[Text]:
    """
    Returns a set of variants (strings) defined by the given keyword.
    """
    assert isinstance(item, str), item

    variant = _any_variants.get(item, None)
    if variant is None:
        return set()

    return variant.get("longhand", {item})


def get_default_any_variants() -> Set[Text]:
    """
    Returns a set of variants (strings) that will be used by default.
    """
    return set({"window", "dedicatedworker"})


def parse_variants(value: Text) -> Set[Text]:
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


def global_suffixes(value: Text) -> Set[Tuple[Text, bool]]:
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


def global_variant_url(url: Text, suffix: Text) -> Text:
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


def _parse_html(f: BinaryIO) -> ElementTree.Element:
    doc = html5lib.parse(f, treebuilder="etree", useChardet=False)
    return cast(ElementTree.Element, doc)

def _parse_xml(f: BinaryIO) -> ElementTree.Element:
    try:
        # raises ValueError with an unsupported encoding,
        # ParseError when there's an undefined entity
        return ElementTree.parse(f).getroot()
    except (ValueError, ElementTree.ParseError):
        f.seek(0)
        return ElementTree.parse(f, XMLParser.XMLParser()).getroot()  # type: ignore


class SourceFile:
    parsers: Dict[Text, Callable[[BinaryIO], ElementTree.Element]] = {"html":_parse_html,
               "xhtml":_parse_xml,
               "svg":_parse_xml}

    root_dir_non_test = {"common"}

    dir_non_test = {"resources",
                    "support",
                    "tools"}

    dir_path_non_test: Set[Tuple[Text, ...]] = {("css21", "archive"),
                                                ("css", "CSS2", "archive"),
                                                ("css", "common")}

    def __init__(self, tests_root: Text,
                 rel_path: Text,
                 url_base: Text,
                 hash: Optional[Text] = None,
                 contents: Optional[bytes] = None) -> None:
        """Object representing a file in a source tree.

        :param tests_root: Path to the root of the source tree
        :param rel_path_str: File path relative to tests_root
        :param url_base: Base URL used when converting file paths to urls
        :param contents: Byte array of the contents of the file or ``None``.
        """

        assert not os.path.isabs(rel_path), rel_path
        if os.name == "nt":
            # do slash normalization on Windows
            rel_path = rel_path.replace("/", "\\")

        dir_path, filename = os.path.split(rel_path)
        name, ext = os.path.splitext(filename)

        type_flag = None
        if "-" in name:
            type_flag = name.rsplit("-", 1)[1].split(".")[0]

        meta_flags = name.split(".")[1:]

        self.tests_root: Text = tests_root
        self.rel_path: Text = rel_path
        self.dir_path: Text = dir_path
        self.filename: Text = filename
        self.name: Text = name
        self.ext: Text = ext
        self.type_flag: Optional[Text] = type_flag
        self.meta_flags: Union[List[bytes], List[Text]] = meta_flags
        self.url_base = url_base
        self.contents = contents
        self.items_cache: Optional[Tuple[Text, List[ManifestItem]]] = None
        self._hash = hash

    def __getstate__(self) -> Dict[str, Any]:
        # Remove computed properties if we pickle this class
        rv = self.__dict__.copy()

        if "__cached_properties__" in rv:
            cached_properties = rv["__cached_properties__"]
            rv = {key:value for key, value in rv.items() if key not in cached_properties}
            del rv["__cached_properties__"]
        return rv

    def name_prefix(self, prefix: Text) -> bool:
        """Check if the filename starts with a given prefix

        :param prefix: The prefix to check"""
        return self.name.startswith(prefix)

    def is_dir(self) -> bool:
        """Return whether this file represents a directory."""
        if self.contents is not None:
            return False

        return os.path.isdir(self.rel_path)

    def open(self) -> BinaryIO:
        """
        Return either
        * the contents specified in the constructor, if any;
        * a File object opened for reading the file contents.
        """
        if self.contents is not None:
            file_obj: BinaryIO = BytesIO(self.contents)
        else:
            file_obj = open(self.path, 'rb')
        return file_obj

    @cached_property
    def rel_path_parts(self) -> Tuple[Text, ...]:
        return tuple(self.rel_path.split(os.path.sep))

    @cached_property
    def path(self) -> Text:
        return os.path.join(self.tests_root, self.rel_path)

    @cached_property
    def rel_url(self) -> Text:
        assert not os.path.isabs(self.rel_path), self.rel_path
        return self.rel_path.replace(os.sep, "/")

    @cached_property
    def url(self) -> Text:
        return urljoin(self.url_base, self.rel_url)

    @cached_property
    def hash(self) -> Text:
        if not self._hash:
            with self.open() as f:
                content = f.read()

            data = b"".join((b"blob ", b"%d" % len(content), b"\0", content))
            self._hash = str(hashlib.sha1(data).hexdigest())

        return self._hash

    def in_non_test_dir(self) -> bool:
        if self.dir_path == "":
            return True

        parts = self.rel_path_parts

        if (parts[0] in self.root_dir_non_test or
            any(item in self.dir_non_test for item in parts) or
            any(parts[:len(path)] == path for path in self.dir_path_non_test)):
            return True
        return False

    def in_conformance_checker_dir(self) -> bool:
        return self.rel_path_parts[0] == "conformance-checkers"

    @property
    def name_is_non_test(self) -> bool:
        """Check if the file name matches the conditions for the file to
        be a non-test file"""
        return (self.is_dir() or
                self.name_prefix("MANIFEST") or
                self.filename == "META.yml" or
                self.filename.startswith(".") or
                self.filename.endswith(".headers") or
                self.filename.endswith(".ini") or
                self.in_non_test_dir())

    @property
    def name_is_conformance(self) -> bool:
        return (self.in_conformance_checker_dir() and
                self.type_flag in ("is-valid", "no-valid"))

    @property
    def name_is_conformance_support(self) -> bool:
        return self.in_conformance_checker_dir()

    @property
    def name_is_manual(self) -> bool:
        """Check if the file name matches the conditions for the file to
        be a manual test file"""
        return self.type_flag == "manual"

    @property
    def name_is_visual(self) -> bool:
        """Check if the file name matches the conditions for the file to
        be a visual test file"""
        return self.type_flag == "visual"

    @property
    def name_is_multi_global(self) -> bool:
        """Check if the file name matches the conditions for the file to
        be a multi-global js test file"""
        return "any" in self.meta_flags and self.ext == ".js"

    @property
    def name_is_worker(self) -> bool:
        """Check if the file name matches the conditions for the file to
        be a worker js test file"""
        return "worker" in self.meta_flags and self.ext == ".js"

    @property
    def name_is_window(self) -> bool:
        """Check if the file name matches the conditions for the file to
        be a window js test file"""
        return "window" in self.meta_flags and self.ext == ".js"

    @property
    def name_is_webdriver(self) -> bool:
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
    def name_is_reference(self) -> bool:
        """Check if the file name matches the conditions for the file to
        be a reference file (not a reftest)"""
        return "/reference/" in self.url or bool(reference_file_re.search(self.name))

    @property
    def name_is_crashtest(self) -> bool:
        return (self.markup_type is not None and
                (self.type_flag == "crash" or "crashtests" in self.dir_path.split(os.path.sep)))

    @property
    def name_is_tentative(self) -> bool:
        """Check if the file name matches the conditions for the file to be a
        tentative file.

        See https://web-platform-tests.org/writing-tests/file-names.html#test-features"""
        return "tentative" in self.meta_flags or "tentative" in self.dir_path.split(os.path.sep)

    @property
    def name_is_print_reftest(self) -> bool:
        return (self.markup_type is not None and
                (self.type_flag == "print" or "print" in self.dir_path.split(os.path.sep)))

    @property
    def markup_type(self) -> Optional[Text]:
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
    def root(self) -> Optional[ElementTree.Element]:
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
    def timeout_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes in a test that
        specify timeouts"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='timeout']")

    @cached_property
    def pac_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes in a test that
        specify PAC (proxy auto-config)"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='pac']")

    @cached_property
    def script_metadata(self) -> Optional[List[Tuple[Text, Text]]]:
        if self.name_is_worker or self.name_is_multi_global or self.name_is_window:
            regexp = js_meta_re
        elif self.name_is_webdriver:
            regexp = python_meta_re
        else:
            return None

        with self.open() as f:
            return list(read_script_metadata(f, regexp))

    @cached_property
    def timeout(self) -> Optional[Text]:
        """The timeout of a test or reference file. "long" if the file has an extended timeout
        or None otherwise"""
        if self.script_metadata:
            if any(m == ("timeout", "long") for m in self.script_metadata):
                return "long"

        if self.root is None:
            return None

        if self.timeout_nodes:
            timeout_str: Optional[Text] = self.timeout_nodes[0].attrib.get("content", None)
            if timeout_str and timeout_str.lower() == "long":
                return "long"

        return None

    @cached_property
    def pac(self) -> Optional[Text]:
        """The PAC (proxy config) of a test or reference file. A URL or null"""
        if self.script_metadata:
            for (meta, content) in self.script_metadata:
                if meta == 'pac':
                    return content

        if self.root is None:
            return None

        if self.pac_nodes:
            return self.pac_nodes[0].attrib.get("content", None)

        return None

    @cached_property
    def viewport_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes in a test that
        specify viewport sizes"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='viewport-size']")

    @cached_property
    def viewport_size(self) -> Optional[Text]:
        """The viewport size of a test or reference file"""
        if self.root is None:
            return None

        if not self.viewport_nodes:
            return None

        return self.viewport_nodes[0].attrib.get("content", None)

    @cached_property
    def dpi_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes in a test that
        specify device pixel ratios"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='device-pixel-ratio']")

    @cached_property
    def dpi(self) -> Optional[Text]:
        """The device pixel ratio of a test or reference file"""
        if self.root is None:
            return None

        if not self.dpi_nodes:
            return None

        return self.dpi_nodes[0].attrib.get("content", None)

    def parse_ref_keyed_meta(self, node: ElementTree.Element) -> Tuple[Optional[Tuple[Text, Text, Text]], Text]:
        item: Text = node.attrib.get("content", "")

        parts = item.rsplit(":", 1)
        if len(parts) == 1:
            key: Optional[Tuple[Text, Text, Text]] = None
            value = parts[0]
        else:
            key_part = urljoin(self.url, parts[0])
            reftype = None
            for ref in self.references:  # type: Tuple[Text, Text]
                if ref[0] == key_part:
                    reftype = ref[1]
                    break
            if reftype not in ("==", "!="):
                raise ValueError("Key %s doesn't correspond to a reference" % key_part)
            key = (self.url, key_part, reftype)
            value = parts[1]

        return key, value


    @cached_property
    def fuzzy_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes in a test that
        specify reftest fuzziness"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='fuzzy']")


    @cached_property
    def fuzzy(self) -> Dict[Optional[Tuple[Text, Text, Text]], List[List[int]]]:
        rv: Dict[Optional[Tuple[Text, Text, Text]], List[List[int]]] = {}
        if self.root is None:
            return rv

        if not self.fuzzy_nodes:
            return rv

        args = ["maxDifference", "totalPixels"]

        for node in self.fuzzy_nodes:
            key, value = self.parse_ref_keyed_meta(node)
            ranges = value.split(";")
            if len(ranges) != 2:
                raise ValueError("Malformed fuzzy value %s" % value)
            arg_values: Dict[Text, List[int]] = {}
            positional_args: Deque[List[int]] = deque()
            for range_str_value in ranges:  # type: Text
                name: Optional[Text] = None
                if "=" in range_str_value:
                    name, range_str_value = (part.strip()
                                             for part in range_str_value.split("=", 1))
                    if name not in args:
                        raise ValueError("%s is not a valid fuzzy property" % name)
                    if arg_values.get(name):
                        raise ValueError("Got multiple values for argument %s" % name)
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
    def page_ranges_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes in a test that
        specify print-reftest """
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='reftest-pages']")

    @cached_property
    def page_ranges(self) -> Dict[Text, List[List[Optional[int]]]]:
        """List of ElementTree Elements corresponding to nodes in a test that
        specify print-reftest page ranges"""
        rv: Dict[Text, List[List[Optional[int]]]] = {}
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
    def testharness_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes representing a
        testharness.js script"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testharness.js']")

    @cached_property
    def content_is_testharness(self) -> Optional[bool]:
        """Boolean indicating whether the file content represents a
        testharness.js test"""
        if self.root is None:
            return None
        return bool(self.testharness_nodes)

    @cached_property
    def variant_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes representing a
        test variant"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='variant']")

    @cached_property
    def test_variants(self) -> List[Text]:
        rv: List[Text] = []
        if self.ext == ".js":
            script_metadata = self.script_metadata
            assert script_metadata is not None
            for (key, value) in script_metadata:
                if key == "variant":
                    rv.append(value)
        else:
            for element in self.variant_nodes:
                if "content" in element.attrib:
                    variant: Text = element.attrib["content"]
                    rv.append(variant)

        for variant in rv:
            if variant != "":
                if variant[0] not in ("#", "?"):
                    raise ValueError("Non-empty variant must start with either a ? or a #")
                if len(variant) == 1 or (variant[0] == "?" and variant[1] == "#"):
                    raise ValueError("Variants must not have empty fragment or query " +
                                     "(omit the empty part instead)")

        if not rv:
            rv = [""]

        return rv

    @cached_property
    def testdriver_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes representing a
        testdriver.js script"""
        assert self.root is not None
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testdriver.js']")

    @cached_property
    def has_testdriver(self) -> Optional[bool]:
        """Boolean indicating whether the file content represents a
        testharness.js test"""
        if self.root is None:
            return None
        return bool(self.testdriver_nodes)

    @cached_property
    def reftest_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes representing a
        to a reftest <link>"""
        if self.root is None:
            return []

        match_links = self.root.findall(".//{http://www.w3.org/1999/xhtml}link[@rel='match']")
        mismatch_links = self.root.findall(".//{http://www.w3.org/1999/xhtml}link[@rel='mismatch']")
        return match_links + mismatch_links

    @cached_property
    def references(self) -> List[Tuple[Text, Text]]:
        """List of (ref_url, relation) tuples for any reftest references specified in
        the file"""
        rv: List[Tuple[Text, Text]] = []
        rel_map = {"match": "==", "mismatch": "!="}
        for item in self.reftest_nodes:
            if "href" in item.attrib:
                ref_url = urljoin(self.url, item.attrib["href"].strip(space_chars))
                ref_type = rel_map[item.attrib["rel"]]
                rv.append((ref_url, ref_type))
        return rv

    @cached_property
    def content_is_ref_node(self) -> bool:
        """Boolean indicating whether the file is a non-leaf node in a reftest
        graph (i.e. if it contains any <link rel=[mis]match>"""
        return bool(self.references)

    @cached_property
    def css_flag_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes representing a
        flag <meta>"""
        if self.root is None:
            return []
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}meta[@name='flags']")

    @cached_property
    def css_flags(self) -> Set[Text]:
        """Set of flags specified in the file"""
        rv: Set[Text] = set()
        for item in self.css_flag_nodes:
            if "content" in item.attrib:
                for flag in item.attrib["content"].split():
                    rv.add(flag)
        return rv

    @cached_property
    def content_is_css_manual(self) -> Optional[bool]:
        """Boolean indicating whether the file content represents a
        CSS WG-style manual test"""
        if self.root is None:
            return None
        # return True if the intersection between the two sets is non-empty
        return bool(self.css_flags & {"animated", "font", "history", "interact", "paged", "speech", "userstyle"})

    @cached_property
    def spec_link_nodes(self) -> List[ElementTree.Element]:
        """List of ElementTree Elements corresponding to nodes representing a
        <link rel=help>, used to point to specs"""
        if self.root is None:
            return []
        return self.root.findall(".//{http://www.w3.org/1999/xhtml}link[@rel='help']")

    @cached_property
    def spec_links(self) -> Set[Text]:
        """Set of spec links specified in the file"""
        rv: Set[Text] = set()
        for item in self.spec_link_nodes:
            if "href" in item.attrib:
                rv.add(item.attrib["href"].strip(space_chars))
        return rv

    @cached_property
    def content_is_css_visual(self) -> Optional[bool]:
        """Boolean indicating whether the file content represents a
        CSS WG-style visual test"""
        if self.root is None:
            return None
        return bool(self.ext in {'.xht', '.html', '.xhtml', '.htm', '.xml', '.svg'} and
                    self.spec_links)

    @property
    def type(self) -> Text:
        possible_types = self.possible_types
        if len(possible_types) == 1:
            return possible_types.pop()

        rv, _ = self.manifest_items()
        return rv

    @property
    def possible_types(self) -> Set[Text]:
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

    def manifest_items(self) -> Tuple[Text, List[ManifestItem]]:
        """List of manifest items corresponding to the file. There is typically one
        per test, but in the case of reftests a node may have corresponding manifest
        items without being a test itself."""

        if self.items_cache:
            return self.items_cache

        drop_cached = "root" not in self.__dict__

        if self.name_is_non_test:
            rv: Tuple[Text, List[ManifestItem]] = ("support", [
                SupportFile(
                    self.tests_root,
                    self.rel_path
                )])

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
            globals = ""
            script_metadata = self.script_metadata
            assert script_metadata is not None
            for (key, value) in script_metadata:
                if key == "global":
                    globals = value
                    break

            tests: List[ManifestItem] = [
                TestharnessTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    global_variant_url(self.rel_url, suffix) + variant,
                    timeout=self.timeout,
                    pac=self.pac,
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
                    pac=self.pac,
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
                    pac=self.pac,
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
                    pac=self.pac,
                    testdriver=testdriver,
                    script_metadata=self.script_metadata
                ))

        elif self.content_is_ref_node:
            rv = RefTest.item_type, []
            for variant in self.test_variants:
                url = self.rel_url + variant
                rv[1].append(RefTest(
                    self.tests_root,
                    self.rel_path,
                    self.url_base,
                    url,
                    references=[
                        (ref[0] + variant, ref[1])
                        for ref in self.references
                    ],
                    timeout=self.timeout,
                    viewport_size=self.viewport_size,
                    dpi=self.dpi,
                    fuzzy=self.fuzzy
                ))

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

    def manifest_spec_items(self) -> Optional[Tuple[Text, List[ManifestItem]]]:
        specs = list(self.spec_links)
        if not specs:
            return None
        rv: Tuple[Text, List[ManifestItem]] = (SpecItem.item_type, [
            SpecItem(
                self.tests_root,
                self.rel_path,
                specs
            )])
        return rv
