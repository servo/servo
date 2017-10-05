import os
import urlparse

from wptmanifest.backends import static
from wptmanifest.backends.static import ManifestItem

import expected

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
            return tuple(ini_value.split(":", 1))
        else:
            return (ini_value, None)

    try:
        node_prefs = node.get("prefs")
        if type(node_prefs) in (str, unicode):
            prefs = {value(node_prefs)}
        rv = dict(value(item) for item in node_prefs)
    except KeyError:
        rv = {}
    return rv


class ExpectedManifest(ManifestItem):
    def __init__(self, name, test_path, url_base):
        """Object representing all the tests in a particular manifest

        :param name: Name of the AST Node associated with this object.
                     Should always be None since this should always be associated with
                     the root node of the AST.
        :param test_path: Path of the test file associated with this manifest.
        :param url_base: Base url for serving the tests in this manifest
        """
        if name is not None:
            raise ValueError("ExpectedManifest should represent the root node")
        if test_path is None:
            raise ValueError("ExpectedManifest requires a test path")
        if url_base is None:
            raise ValueError("ExpectedManifest requires a base url")
        ManifestItem.__init__(self, name)
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
        return urlparse.urljoin(self.url_base,
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
    def tags(self):
        return tags(self)

    @property
    def prefs(self):
        return prefs(self)


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
    def tags(self):
        return tags(self)

    @property
    def prefs(self):
        return prefs(self)


class TestNode(ManifestItem):
    def __init__(self, name):
        """Tree node associated with a particular test in a manifest

        :param name: name of the test"""
        assert name is not None
        ManifestItem.__init__(self, name)
        self.updated_expected = []
        self.new_expected = []
        self.subtests = {}
        self.default_status = None
        self._from_file = True

    @property
    def is_empty(self):
        required_keys = set(["type"])
        if set(self._data.keys()) != required_keys:
            return False
        return all(child.is_empty for child in self.children)

    @property
    def test_type(self):
        return self.get("type")

    @property
    def id(self):
        return urlparse.urljoin(self.parent.url, self.name)

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
    def tags(self):
        return tags(self)

    @property
    def prefs(self):
        return prefs(self)

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
    def __init__(self, name):
        """Tree node associated with a particular subtest in a manifest

        :param name: name of the subtest"""
        TestNode.__init__(self, name)

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
