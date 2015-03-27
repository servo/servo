import json
import os
from collections import defaultdict

from item import item_types, ManualTest, WebdriverSpecTest, Stub, RefTest, TestharnessTest
from log import get_logger
from sourcefile import SourceFile


CURRENT_VERSION = 2


class ManifestError(Exception):
    pass


class ManifestVersionMismatch(ManifestError):
    pass

class Manifest(object):
    def __init__(self, git_rev=None, url_base="/"):
        # Dict of item_type: {path: set(manifest_items)}
        self._data = dict((item_type, defaultdict(set))
                          for item_type in item_types)
        self.rev = git_rev
        self.url_base = url_base
        self.local_changes = LocalChanges(self)
        # reftest nodes arranged as {path: set(manifest_items)}
        self.reftest_nodes = defaultdict(set)
        self.reftest_nodes_by_url = {}

    def _included_items(self, include_types=None):
        if include_types is None:
            include_types = item_types

        for item_type in include_types:
            paths = self._data[item_type].copy()
            for local_types, local_paths in self.local_changes.itertypes(item_type):
                for path, items in local_paths.iteritems():
                    paths[path] = items
                for path in self.local_changes.iterdeleted():
                    if path in paths:
                        del paths[path]

            yield item_type, paths

    def contains_path(self, path):
        return any(path in paths for _, paths in self._included_items())

    def add(self, item):
        if item is None:
            return

        is_reference = False
        if isinstance(item, RefTest):
            self.reftest_nodes[item.path].add(item)
            self.reftest_nodes_by_url[item.url] = item
            is_reference = item.is_reference

        if not is_reference:
            self._add(item)

        item.manifest = self

    def _add(self, item):
        self._data[item.item_type][item.path].add(item)

    def extend(self, items):
        for item in items:
            self.add(item)

    def remove_path(self, path):
        for item_type in item_types:
            if path in self._data[item_type]:
                del self._data[item_type][path]

    def itertypes(self, *types):
        if not types:
            types = None
        for item_type, items in self._included_items(types):
            for item in sorted(items.items()):
                yield item

    def __iter__(self):
        for item in self.itertypes():
            yield item

    def __getitem__(self, path):
        for _, paths in self._included_items():
            if path in paths:
                return paths[path]
        raise KeyError

    def get_reference(self, url):
        if url in self.local_changes.reftest_nodes_by_url:
            return self.local_changes.reftest_nodes_by_url[url]

        if url in self.reftest_nodes_by_url:
            return self.reftest_nodes_by_url[url]

        return None

    def _committed_with_path(self, rel_path):
        rv = set()

        for paths_items in self._data.itervalues():
            rv |= paths_items.get(rel_path, set())

        if rel_path in self.reftest_nodes:
            rv |= self.reftest_nodes[rel_path]

        return rv

    def _committed_paths(self):
        rv = set()
        for paths_items in self._data.itervalues():
            rv |= set(paths_items.keys())
        return rv

    def update(self,
               tests_root,
               url_base,
               new_rev,
               committed_changes=None,
               local_changes=None,
               remove_missing_local=False):

        if local_changes is None:
            local_changes = {}

        if committed_changes is not None:
            for rel_path, status in committed_changes:
                self.remove_path(rel_path)
                if status == "modified":
                    use_committed = rel_path in local_changes
                    source_file = SourceFile(tests_root,
                                             rel_path,
                                             url_base,
                                             use_committed=use_committed)
                    self.extend(source_file.manifest_items())

        self.local_changes = LocalChanges(self)

        local_paths = set()
        for rel_path, status in local_changes.iteritems():
            local_paths.add(rel_path)

            if status == "modified":
                existing_items = self._committed_with_path(rel_path)
                source_file = SourceFile(tests_root,
                                         rel_path,
                                         url_base,
                                         use_committed=False)
                local_items = set(source_file.manifest_items())

                updated_items = local_items - existing_items
                self.local_changes.extend(updated_items)
            else:
                self.local_changes.add_deleted(rel_path)

        if remove_missing_local:
            for path in self._committed_paths() - local_paths:
                self.local_changes.add_deleted(path)

        self.update_reftests()

        if new_rev is not None:
            self.rev = new_rev
        self.url_base = url_base

    def update_reftests(self):
        reftest_nodes = self.reftest_nodes.copy()
        for path, items in self.local_changes.reftest_nodes.iteritems():
            reftest_nodes[path] |= items

        #TODO: remove locally deleted files
        tests = set()
        for items in reftest_nodes.values():
            tests |= set(item for item in items if not item.is_reference)

        has_inbound = set()
        for path, items in reftest_nodes.iteritems():
            for item in items:
                for ref_url, ref_type in item.references:
                    has_inbound.add(ref_url)

        if self.local_changes.reftest_nodes:
            target = self.local_changes
        else:
            target = self

        #TODO: Warn if there exist unreachable reftest nodes

        for path, items in reftest_nodes.iteritems():
            for item in items:
                if item.url in has_inbound:
                    continue
                target._data["reftest"][path].add(item)

    def to_json(self):
        out_items = {
            item_type: sorted(
                test.to_json()
                for _, tests in items.iteritems()
                for test in tests
            )
            for item_type, items in self._data.iteritems()
        }

        reftest_nodes = {key:[v.to_json() for v in value]
                         for key, value in self.reftest_nodes.iteritems()}

        rv = {"url_base": self.url_base,
              "rev": self.rev,
              "local_changes": self.local_changes.to_json(),
              "items": out_items,
              "reftest_nodes": reftest_nodes,
              "version": CURRENT_VERSION}
        return rv

    @classmethod
    def from_json(cls, tests_root, obj):
        version = obj.get("version")
        if version != CURRENT_VERSION:
            raise ManifestVersionMismatch

        self = cls(git_rev=obj["rev"],
                   url_base=obj.get("url_base", "/"))
        if not hasattr(obj, "iteritems"):
            raise ManifestError

        item_classes = {"testharness": TestharnessTest,
                        "reftest": RefTest,
                        "manual": ManualTest,
                        "stub": Stub,
                        "wdspec": WebdriverSpecTest}

        source_files = {}

        for k, values in obj["items"].iteritems():
            if k not in item_types:
                raise ManifestError
            for v in values:
                manifest_item = item_classes[k].from_json(self, tests_root, v,
                                                          source_files=source_files)
                self._add(manifest_item)

        for path, values in obj["reftest_nodes"].iteritems():
            for v in values:
                item = RefTest.from_json(self, tests_root, v,
                                         source_files=source_files)
                self.reftest_nodes[path].add(item)
                self.reftest_nodes_by_url[v["url"]] = item

        self.local_changes = LocalChanges.from_json(self,
                                                    tests_root,
                                                    obj["local_changes"],
                                                    source_files=source_files)
        return self

class LocalChanges(object):
    def __init__(self, manifest):
        self.manifest = manifest
        self._data = dict((item_type, defaultdict(set)) for item_type in item_types)
        self._deleted = set()
        self.reftest_nodes = defaultdict(set)
        self.reftest_nodes_by_url = {}

    def add(self, item):
        if item is None:
            return

        is_reference = False
        if isinstance(item, RefTest):
            self.reftest_nodes[item.path].add(item)
            self.reftest_nodes_by_url[item.url] = item
            is_reference = item.is_reference

        if not is_reference:
            self._add(item)

        item.manifest = self.manifest

    def _add(self, item):
        self._data[item.item_type][item.path].add(item)

    def extend(self, items):
        for item in items:
            self.add(item)

    def add_deleted(self, path):
        self._deleted.add(path)

    def is_deleted(self, path):
        return path in self._deleted

    def itertypes(self, *types):
        for item_type in types:
            yield item_type, self._data[item_type]

    def iterdeleted(self):
        for item in self._deleted:
            yield item

    def __getitem__(self, item_type):
        return self._data[item_type]

    def to_json(self):
        reftest_nodes = {key:[v.to_json() for v in value]
                         for key, value in self.reftest_nodes.iteritems()}

        rv = {"items": defaultdict(dict),
              "reftest_nodes": reftest_nodes,
              "deleted": []}

        rv["deleted"].extend(self._deleted)

        for test_type, paths in self._data.iteritems():
            for path, tests in paths.iteritems():
                rv["items"][test_type][path] = [test.to_json() for test in tests]

        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, obj, source_files=None):
        self = cls(manifest)
        if not hasattr(obj, "iteritems"):
            raise ManifestError

        item_classes = {"testharness": TestharnessTest,
                        "reftest": RefTest,
                        "manual": ManualTest,
                        "stub": Stub,
                        "wdspec": WebdriverSpecTest}

        for test_type, paths in obj["items"].iteritems():
            for path, tests in paths.iteritems():
                for test in tests:
                    manifest_item = item_classes[test_type].from_json(manifest,
                                                                      tests_root,
                                                                      test,
                                                                      source_files=source_files)
                    self.add(manifest_item)

        for path, values in obj["reftest_nodes"].iteritems():
            for v in values:
                item = RefTest.from_json(self.manifest, tests_root, v,
                                         source_files=source_files)
                self.reftest_nodes[path].add(item)
                self.reftest_nodes_by_url[item.url] = item

        for item in obj["deleted"]:
            self.add_deleted(item)

        return self

def load(tests_root, manifest_path):
    logger = get_logger()

    if os.path.exists(manifest_path):
        logger.debug("Opening manifest at %s" % manifest_path)
    else:
        logger.debug("Creating new manifest at %s" % manifest_path)
    try:
        with open(manifest_path) as f:
            manifest = Manifest.from_json(tests_root, json.load(f))
    except IOError:
        manifest = Manifest(None)

    return manifest

def write(manifest, manifest_path):
    with open(manifest_path, "w") as f:
        json.dump(manifest.to_json(), f, sort_keys=True, indent=2, separators=(',', ': '))
