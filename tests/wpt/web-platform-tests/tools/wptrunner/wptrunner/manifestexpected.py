import os
from six.moves.urllib.parse import urljoin
from collections import deque

from .wptmanifest.backends import static
from .wptmanifest.backends.base import ManifestItem

from . import expected

"""Manifest structure used to store expected results of a test.

Each manifest file is represented by an ExpectedManifest that
has one or more TestNode children, one per test in the manifest.
Each TestNode has zero or more SubtestNode children, one for each
known subtest of the test.
"""

def data_cls_getter(output_node, visited_node):
    # visited_node is intentionally unused
    if output_node is None:
        return ExpectedManifest
    if isinstance(output_node, ExpectedManifest):
        return TestNode
    if isinstance(output_node, TestNode):
        return SubtestNode
    raise ValueError


def bool_prop(name, node):
    """Boolean property"""
    try:
        return node.get(name)
    except KeyError:
        return None


def int_prop(name, node):
    """Boolean property"""
    try:
        return int(node.get(name))
    except KeyError:
        return None


def list_prop(name, node):
    """List property"""
    try:
        list_prop = node.get(name)
        if isinstance(list_prop, basestring):
            return [list_prop]
        return list(list_prop)
    except KeyError:
        return []


def tags(node):
    """Set of tags that have been applied to the test"""
    try:
        value = node.get("tags")
        if isinstance(value, (str, unicode)):
            return {value}
        return set(value)
    except KeyError:
        return set()


def prefs(node):
    def value(ini_value):
        if isinstance(ini_value, (str, unicode)):
            return tuple(pref_piece.strip() for pref_piece in ini_value.split(':', 1))
        else:
            # this should be things like @Reset, which are apparently type 'object'
            return (ini_value, None)

    try:
        node_prefs = node.get("prefs")
        if type(node_prefs) in (str, unicode):
            rv = dict(value(node_prefs))
        else:
            rv = dict(value(item) for item in node_prefs)
    except KeyError:
        rv = {}
    return rv


def set_prop(name, node):
    try:
        node_items = node.get(name)
        if isinstance(node_items, (str, unicode)):
            rv = {node_items}
        else:
            rv = set(node_items)
    except KeyError:
        rv = set()
    return rv


def leak_threshold(node):
    rv = {}
    try:
        node_items = node.get("leak-threshold")
        if isinstance(node_items, (str, unicode)):
            node_items = [node_items]
        for item in node_items:
            process, value = item.rsplit(":", 1)
            rv[process.strip()] = int(value.strip())
    except KeyError:
        pass
    return rv


def fuzzy_prop(node):
    """Fuzzy reftest match

    This can either be a list of strings or a single string. When a list is
    supplied, the format of each item matches the description below.

    The general format is
    fuzzy = [key ":"] <prop> ";" <prop>
    key = <test name> [reftype <reference name>]
    reftype = "==" | "!="
    prop = [propName "=" ] range
    propName = "maxDifferences" | "totalPixels"
    range = <digits> ["-" <digits>]

    So for example:
      maxDifferences=10;totalPixels=10-20

      specifies that for any test/ref pair for which no other rule is supplied,
      there must be a maximum pixel difference of exactly 10, and betwen 10 and
      20 total pixels different.

      test.html==ref.htm:10;20

      specifies that for a equality comparison between test.html and ref.htm,
      resolved relative to the test path, there can be a maximum difference
      of 10 in the pixel value for any channel and 20 pixels total difference.

      ref.html:10;20

      is just like the above but applies to any comparison involving ref.html
      on the right hand side.

    The return format is [(key, (maxDifferenceRange, totalPixelsRange))], where
    the key is either None where no specific reference is specified, the reference
    name where there is only one component or a tuple (test, ref, reftype) when the
    exact comparison is specified. maxDifferenceRange and totalPixelsRange are tuples
    of integers indicating the inclusive range of allowed values.
"""
    rv = []
    args = ["maxDifference", "totalPixels"]
    try:
        value = node.get("fuzzy")
    except KeyError:
        return rv
    if not isinstance(value, list):
        value = [value]
    for item in value:
        if not isinstance(item, (str, unicode)):
            rv.append(item)
            continue
        parts = item.rsplit(":", 1)
        if len(parts) == 1:
            key = None
            fuzzy_values = parts[0]
        else:
            key, fuzzy_values = parts
            for reftype in ["==", "!="]:
                if reftype in key:
                    key = key.split(reftype)
                    key.append(reftype)
                    key = tuple(key)
        ranges = fuzzy_values.split(";")
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
                range_value = tuple(int(item.strip()) for item in (range_min, range_max))
            except ValueError:
                raise ValueError("Fuzzy value %s must be a range of integers" % range_str_value)
            if name is None:
                arg_values[None].append(range_value)
            else:
                arg_values[name] = range_value
        range_values = []
        for arg_name in args:
            if arg_values.get(arg_name):
                value = arg_values.pop(arg_name)
            else:
                value = arg_values[None].popleft()
            range_values.append(value)
        rv.append((key, tuple(range_values)))
    return rv


class ExpectedManifest(ManifestItem):
    def __init__(self, node, test_path, url_base):
        """Object representing all the tests in a particular manifest

        :param name: Name of the AST Node associated with this object.
                     Should always be None since this should always be associated with
                     the root node of the AST.
        :param test_path: Path of the test file associated with this manifest.
        :param url_base: Base url for serving the tests in this manifest
        """
        name = node.data
        if name is not None:
            raise ValueError("ExpectedManifest should represent the root node")
        if test_path is None:
            raise ValueError("ExpectedManifest requires a test path")
        if url_base is None:
            raise ValueError("ExpectedManifest requires a base url")
        ManifestItem.__init__(self, node)
        self.child_map = {}
        self.test_path = test_path
        self.url_base = url_base

    def append(self, child):
        """Add a test to the manifest"""
        ManifestItem.append(self, child)
        self.child_map[child.id] = child

    def _remove_child(self, child):
        del self.child_map[child.id]
        ManifestItem.remove_child(self, child)
        assert len(self.child_map) == len(self.children)

    def get_test(self, test_id):
        """Get a test from the manifest by ID

        :param test_id: ID of the test to return."""
        return self.child_map.get(test_id)

    @property
    def url(self):
        return urljoin(self.url_base,
                       "/".join(self.test_path.split(os.path.sep)))

    @property
    def disabled(self):
        return bool_prop("disabled", self)

    @property
    def restart_after(self):
        return bool_prop("restart-after", self)

    @property
    def leaks(self):
        return bool_prop("leaks", self)

    @property
    def min_assertion_count(self):
        return int_prop("min-asserts", self)

    @property
    def max_assertion_count(self):
        return int_prop("max-asserts", self)

    @property
    def tags(self):
        return tags(self)

    @property
    def prefs(self):
        return prefs(self)

    @property
    def lsan_allowed(self):
        return set_prop("lsan-allowed", self)

    @property
    def leak_allowed(self):
        return set_prop("leak-allowed", self)

    @property
    def leak_threshold(self):
        return leak_threshold(self)

    @property
    def lsan_max_stack_depth(self):
        return int_prop("lsan-max-stack-depth", self)

    @property
    def fuzzy(self):
        return fuzzy_prop(self)

    @property
    def expected(self):
        return list_prop("expected", self)[0]

    @property
    def known_intermittent(self):
        return list_prop("expected", self)[1:]


class DirectoryManifest(ManifestItem):
    @property
    def disabled(self):
        return bool_prop("disabled", self)

    @property
    def restart_after(self):
        return bool_prop("restart-after", self)

    @property
    def leaks(self):
        return bool_prop("leaks", self)

    @property
    def min_assertion_count(self):
        return int_prop("min-asserts", self)

    @property
    def max_assertion_count(self):
        return int_prop("max-asserts", self)

    @property
    def tags(self):
        return tags(self)

    @property
    def prefs(self):
        return prefs(self)

    @property
    def lsan_allowed(self):
        return set_prop("lsan-allowed", self)

    @property
    def leak_allowed(self):
        return set_prop("leak-allowed", self)

    @property
    def leak_threshold(self):
        return leak_threshold(self)

    @property
    def lsan_max_stack_depth(self):
        return int_prop("lsan-max-stack-depth", self)

    @property
    def fuzzy(self):
        return fuzzy_prop(self)


class TestNode(ManifestItem):
    def __init__(self, node, **kwargs):
        """Tree node associated with a particular test in a manifest

        :param name: name of the test"""
        assert node.data is not None
        ManifestItem.__init__(self, node, **kwargs)
        self.updated_expected = []
        self.new_expected = []
        self.subtests = {}
        self.default_status = None
        self._from_file = True

    @property
    def is_empty(self):
        required_keys = {"type"}
        if set(self._data.keys()) != required_keys:
            return False
        return all(child.is_empty for child in self.children)

    @property
    def test_type(self):
        return self.get("type")

    @property
    def id(self):
        return urljoin(self.parent.url, self.name)

    @property
    def disabled(self):
        return bool_prop("disabled", self)

    @property
    def restart_after(self):
        return bool_prop("restart-after", self)

    @property
    def leaks(self):
        return bool_prop("leaks", self)

    @property
    def min_assertion_count(self):
        return int_prop("min-asserts", self)

    @property
    def max_assertion_count(self):
        return int_prop("max-asserts", self)

    @property
    def tags(self):
        return tags(self)

    @property
    def prefs(self):
        return prefs(self)

    @property
    def lsan_allowed(self):
        return set_prop("lsan-allowed", self)

    @property
    def leak_allowed(self):
        return set_prop("leak-allowed", self)

    @property
    def leak_threshold(self):
        return leak_threshold(self)

    @property
    def lsan_max_stack_depth(self):
        return int_prop("lsan-max-stack-depth", self)

    @property
    def fuzzy(self):
        return fuzzy_prop(self)

    @property
    def expected(self):
        return list_prop("expected", self)[0]

    @property
    def known_intermittent(self):
        return list_prop("expected", self)[1:]

    def append(self, node):
        """Add a subtest to the current test

        :param node: AST Node associated with the subtest"""
        child = ManifestItem.append(self, node)
        self.subtests[child.name] = child

    def get_subtest(self, name):
        """Get the SubtestNode corresponding to a particular subtest, by name

        :param name: Name of the node to return"""
        if name in self.subtests:
            return self.subtests[name]
        return None


class SubtestNode(TestNode):
    @property
    def is_empty(self):
        if self._data:
            return False
        return True


def get_manifest(metadata_root, test_path, url_base, run_info):
    """Get the ExpectedManifest for a particular test path, or None if there is no
    metadata stored for that test path.

    :param metadata_root: Absolute path to the root of the metadata directory
    :param test_path: Path to the test(s) relative to the test root
    :param url_base: Base url for serving the tests in this manifest
    :param run_info: Dictionary of properties of the test run for which the expectation
                     values should be computed.
    """
    manifest_path = expected.expected_path(metadata_root, test_path)
    try:
        with open(manifest_path) as f:
            return static.compile(f,
                                  run_info,
                                  data_cls_getter=data_cls_getter,
                                  test_path=test_path,
                                  url_base=url_base)
    except IOError:
        return None


def get_dir_manifest(path, run_info):
    """Get the ExpectedManifest for a particular test path, or None if there is no
    metadata stored for that test path.

    :param path: Full path to the ini file
    :param run_info: Dictionary of properties of the test run for which the expectation
                     values should be computed.
    """
    try:
        with open(path) as f:
            return static.compile(f,
                                  run_info,
                                  data_cls_getter=lambda x,y: DirectoryManifest)
    except IOError:
        return None
