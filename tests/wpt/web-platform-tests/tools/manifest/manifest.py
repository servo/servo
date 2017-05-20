import json
import os
import re
from collections import defaultdict
from six import iteritems, itervalues, viewkeys

from .item import ManualTest, WebdriverSpecTest, Stub, RefTestNode, RefTest, TestharnessTest, SupportFile, ConformanceCheckerTest, VisualTest
from .log import get_logger
from .utils import from_os_path, to_os_path, rel_path_to_url


CURRENT_VERSION = 4


class ManifestError(Exception):
    pass


class ManifestVersionMismatch(ManifestError):
    pass


def sourcefile_items(args):
    tests_root, url_base, rel_path, status = args
    source_file = SourceFile(tests_root,
                             rel_path,
                             url_base)
    return rel_path, source_file.manifest_items()


class Manifest(object):
    def __init__(self, url_base="/"):
        assert url_base is not None
        self._path_hash = {}
        self._data = defaultdict(dict)
        self._reftest_nodes_by_url = None
        self.url_base = url_base

    def __iter__(self):
        return self.itertypes()

    def itertypes(self, *types):
        if not types:
            types = sorted(self._data.keys())
        for item_type in types:
            for path, tests in sorted(iteritems(self._data[item_type])):
                yield item_type, path, tests

    def iterpath(self, path):
        for type_tests in self._data.values():
            for test in type_tests.get(path, set()):
                yield test

    @property
    def reftest_nodes_by_url(self):
        if self._reftest_nodes_by_url is None:
            by_url = {}
            for path, nodes in iteritems(self._data.get("reftests", {})):
                for node in nodes:
                    by_url[node.url] = node
            self._reftest_nodes_by_url = by_url
        return self._reftest_nodes_by_url

    def get_reference(self, url):
        return self.reftest_nodes_by_url.get(url)

    def update(self, tree):
        new_data = defaultdict(dict)
        new_hashes = {}

        reftest_nodes = []
        old_files = defaultdict(set, {k: set(viewkeys(v)) for k, v in iteritems(self._data)})

        changed = False
        reftest_changes = False

        for source_file in tree:
            rel_path = source_file.rel_path
            file_hash = source_file.hash

            is_new = rel_path not in self._path_hash
            hash_changed = False

            if not is_new:
                old_hash, old_type = self._path_hash[rel_path]
                old_files[old_type].remove(rel_path)
                if old_hash != file_hash:
                    new_type, manifest_items = source_file.manifest_items()
                    hash_changed = True
                else:
                    new_type, manifest_items = old_type, self._data[old_type][rel_path]
            else:
                new_type, manifest_items = source_file.manifest_items()

            if new_type in ("reftest", "reftest_node"):
                reftest_nodes.extend(manifest_items)
                if is_new or hash_changed:
                    reftest_changes = True
            elif new_type:
                new_data[new_type][rel_path] = set(manifest_items)

            new_hashes[rel_path] = (file_hash, new_type)

            if is_new or hash_changed:
                changed = True

        if reftest_changes or old_files["reftest"] or old_files["reftest_node"]:
            reftests, reftest_nodes, changed_hashes = self._compute_reftests(reftest_nodes)
            new_data["reftest"] = reftests
            new_data["reftest_node"] = reftest_nodes
            new_hashes.update(changed_hashes)
        else:
            new_data["reftest"] = self._data["reftest"]
            new_data["reftest_node"] = self._data["reftest_node"]

        if any(itervalues(old_files)):
            changed = True

        self._data = new_data
        self._path_hash = new_hashes

        return changed

    def _compute_reftests(self, reftest_nodes):
        self._reftest_nodes_by_url = {}
        has_inbound = set()
        for item in reftest_nodes:
            for ref_url, ref_type in item.references:
                has_inbound.add(ref_url)

        reftests = defaultdict(set)
        references = defaultdict(set)
        changed_hashes = {}

        for item in reftest_nodes:
            if item.url in has_inbound:
                # This is a reference
                if isinstance(item, RefTest):
                    item = item.to_RefTestNode()
                    changed_hashes[item.source_file.rel_path] = (item.source_file.hash,
                                                                 item.item_type)
                references[item.source_file.rel_path].add(item)
                self._reftest_nodes_by_url[item.url] = item
            else:
                if isinstance(item, RefTestNode):
                    item = item.to_RefTest()
                    changed_hashes[item.source_file.rel_path] = (item.source_file.hash,
                                                                 item.item_type)
                reftests[item.source_file.rel_path].add(item)

        return reftests, references, changed_hashes

    def to_json(self):
        out_items = {
            test_type: {
                from_os_path(path):
                [t for t in sorted(test.to_json() for test in tests)]
                for path, tests in iteritems(type_paths)
            }
            for test_type, type_paths in iteritems(self._data)
        }
        rv = {"url_base": self.url_base,
              "paths": {from_os_path(k): v for k, v in iteritems(self._path_hash)},
              "items": out_items,
              "version": CURRENT_VERSION}
        return rv

    @classmethod
    def from_json(cls, tests_root, obj):
        version = obj.get("version")
        if version != CURRENT_VERSION:
            raise ManifestVersionMismatch

        self = cls(url_base=obj.get("url_base", "/"))
        if not hasattr(obj, "items") and hasattr(obj, "paths"):
            raise ManifestError

        self._path_hash = {to_os_path(k): v for k, v in iteritems(obj["paths"])}

        item_classes = {"testharness": TestharnessTest,
                        "reftest": RefTest,
                        "reftest_node": RefTestNode,
                        "manual": ManualTest,
                        "stub": Stub,
                        "wdspec": WebdriverSpecTest,
                        "conformancechecker": ConformanceCheckerTest,
                        "visual": VisualTest,
                        "support": SupportFile}

        source_files = {}

        for test_type, type_paths in iteritems(obj["items"]):
            if test_type not in item_classes:
                raise ManifestError
            test_cls = item_classes[test_type]
            tests = defaultdict(set)
            for path, manifest_tests in iteritems(type_paths):
                path = to_os_path(path)
                for test in manifest_tests:
                    manifest_item = test_cls.from_json(self,
                                                       tests_root,
                                                       path,
                                                       test,
                                                       source_files=source_files)
                    tests[path].add(manifest_item)
            self._data[test_type] = tests

        return self


def load(tests_root, manifest):
    logger = get_logger()

    # "manifest" is a path or file-like object.
    if isinstance(manifest, basestring):
        if os.path.exists(manifest):
            logger.debug("Opening manifest at %s" % manifest)
        else:
            logger.debug("Creating new manifest at %s" % manifest)
        try:
            with open(manifest) as f:
                rv = Manifest.from_json(tests_root, json.load(f))
        except IOError:
            return None
        return rv

    return Manifest.from_json(tests_root, json.load(manifest))


def write(manifest, manifest_path):
    with open(manifest_path, "wb") as f:
        json.dump(manifest.to_json(), f, sort_keys=True, indent=1, separators=(',', ': '))
        f.write("\n")
