# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

"""Manifest structure used to store paths that should be included in a test run.

The manifest is represented by a tree of IncludeManifest objects, the root
representing the file and each subnode representing a subdirectory that should
be included or excluded.
"""
import os

from wptmanifest.node import DataNode
from wptmanifest.backends import conditional
from wptmanifest.backends.conditional import ManifestItem


class IncludeManifest(ManifestItem):
    def __init__(self, node):
        """Node in a tree structure representing the paths
        that should be included or excluded from the test run.

        :param node: AST Node corresponding to this Node.
        """
        ManifestItem.__init__(self, node)
        self.child_map = {}

    @classmethod
    def create(cls):
        """Create an empty IncludeManifest tree"""
        node = DataNode(None)
        return cls(node)

    def append(self, child):
        ManifestItem.append(self, child)
        self.child_map[child.name] = child
        assert len(self.child_map) == len(self.children)

    def include(self, test):
        """Return a boolean indicating whether a particular test should be
        included in a test run, based on the IncludeManifest tree rooted on
        this object.

        :param test: The test object"""
        path_components = self._get_path_components(test)
        return self._include(test, path_components)

    def _include(self, test, path_components):
        if path_components:
            next_path_part = path_components.pop()
            if next_path_part in self.child_map:
                return self.child_map[next_path_part]._include(test, path_components)

        node = self
        while node:
            try:
                skip_value = self.get("skip", {"test_type": test.item_type}).lower()
                assert skip_value in ("true", "false")
                return False if skip_value == "true" else True
            except KeyError:
                if node.parent is not None:
                    node = node.parent
                else:
                    # Include by default
                    return True

    def _get_path_components(self, test):
        test_url = test.url
        assert test_url[0] == "/"
        return [item for item in reversed(test_url.split("/")) if item]

    def _add_rule(self, test_manifests, url, direction):
        maybe_path = os.path.abspath(os.path.join(os.curdir, url))
        if os.path.exists(maybe_path):
            for manifest, data in test_manifests.iteritems():
                rel_path = os.path.relpath(maybe_path, data["tests_path"])
                if ".." not in rel_path.split(os.sep):
                    url = rel_path

        assert direction in ("include", "exclude")
        components = [item for item in reversed(url.split("/")) if item]

        node = self
        while components:
            component = components.pop()
            if component not in node.child_map:
                new_node = IncludeManifest(DataNode(component))
                node.append(new_node)

            node = node.child_map[component]

        skip = False if direction == "include" else True
        node.set("skip", str(skip))

    def add_include(self, test_manifests, url_prefix):
        """Add a rule indicating that tests under a url path
        should be included in test runs

        :param url_prefix: The url prefix to include
        """
        return self._add_rule(test_manifests, url_prefix, "include")

    def add_exclude(self, test_manifests, url_prefix):
        """Add a rule indicating that tests under a url path
        should be excluded from test runs

        :param url_prefix: The url prefix to exclude
        """
        return self._add_rule(test_manifests, url_prefix, "exclude")


def get_manifest(manifest_path):
    with open(manifest_path) as f:
        return conditional.compile(f, data_cls_getter=lambda x, y: IncludeManifest)
