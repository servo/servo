import itertools
import json
import os
from collections import defaultdict
from six import iteritems, iterkeys, itervalues, string_types

from . import vcs
from .item import (ManualTest, WebDriverSpecTest, Stub, RefTestNode, RefTest,
                   TestharnessTest, SupportFile, ConformanceCheckerTest, VisualTest)
from .log import get_logger
from .utils import from_os_path, to_os_path

try:
    import ujson as fast_json
except ImportError:
    fast_json = json

CURRENT_VERSION = 5


class ManifestError(Exception):
    pass


class ManifestVersionMismatch(ManifestError):
    pass


def iterfilter(filters, iter):
    for f in filters:
        iter = f(iter)
    for item in iter:
        yield item


item_classes = {"testharness": TestharnessTest,
                "reftest": RefTest,
                "reftest_node": RefTestNode,
                "manual": ManualTest,
                "stub": Stub,
                "wdspec": WebDriverSpecTest,
                "conformancechecker": ConformanceCheckerTest,
                "visual": VisualTest,
                "support": SupportFile}


class TypeData(object):
    def __init__(self, manifest, type_cls, meta_filters):
        """Dict-like object containing the TestItems for each test type.

        Loading an actual Item class for each test is unnecessarily
        slow, so this class allows lazy-loading of the test
        items. When the manifest is loaded we store the raw json
        corresponding to the test type, and only create an Item
        subclass when the test is accessed. In order to remain
        API-compatible with consumers that depend on getting an Item
        from iteration, we do egerly load all items when iterating
        over the class."""
        self.manifest = manifest
        self.type_cls = type_cls
        self.json_data = {}
        self.tests_root = None
        self.data = {}
        self.meta_filters = meta_filters or []

    def __getitem__(self, key):
        if key not in self.data:
            self.load(key)
        return self.data[key]

    def __bool__(self):
        return bool(self.data)

    def __len__(self):
        rv = len(self.data)
        if self.json_data is not None:
            rv += len(self.json_data)
        return rv

    def __delitem__(self, key):
        if key in self.data:
            del self.data[key]
        elif self.json_data is not None:
            del self.json_data[from_os_path(key)]
        else:
            raise KeyError

    def __setitem__(self, key, value):
        self.data[key] = value

    def __contains__(self, key):
        self.load_all()
        return key in self.data

    def __iter__(self):
        self.load_all()
        return self.data.__iter__()

    def pop(self, key, default=None):
        try:
            value = self[key]
        except ValueError:
            value = default
        else:
            del self.data[key]
        return value

    def get(self, key, default=None):
        try:
            return self[key]
        except ValueError:
            return default

    def itervalues(self):
        self.load_all()
        return itervalues(self.data)

    def iteritems(self):
        self.load_all()
        return iteritems(self.data)

    def values(self):
        return self.itervalues()

    def items(self):
        return self.iteritems()

    def load(self, key):
        """Load a specific Item given a path"""
        if self.json_data is not None:
            data = set()
            path = from_os_path(key)
            for test in iterfilter(self.meta_filters, self.json_data.get(path, [])):
                manifest_item = self.type_cls.from_json(self.manifest, path, test)
                data.add(manifest_item)
            try:
                del self.json_data[path]
            except KeyError:
                pass
            self.data[key] = data
        else:
            raise ValueError

    def load_all(self):
        """Load all test items in this class"""
        if self.json_data is not None:
            for path, value in iteritems(self.json_data):
                key = to_os_path(path)
                if key in self.data:
                    continue
                data = set()
                for test in iterfilter(self.meta_filters, self.json_data.get(path, [])):
                    manifest_item = self.type_cls.from_json(self.manifest, path, test)
                    data.add(manifest_item)
                self.data[key] = data
            self.json_data = None

    def set_json(self, tests_root, data):
        if not isinstance(data, dict):
            raise ValueError("Got a %s expected a dict" % (type(data)))
        self.tests_root = tests_root
        self.json_data = data

    def to_json(self):
        data = {
            from_os_path(path):
            [t for t in sorted(test.to_json() for test in tests)]
            for path, tests in iteritems(self.data)
        }

        if self.json_data is not None:
            if not data:
                # avoid copying if there's nothing here yet
                return self.json_data
            data.update(self.json_data)

        return data

    def paths(self):
        """Get a list of all paths containing items of this type,
        without actually constructing all the items"""
        rv = set(iterkeys(self.data))
        if self.json_data:
            rv |= set(to_os_path(item) for item in iterkeys(self.json_data))
        return rv


class ManifestData(dict):
    def __init__(self, manifest, meta_filters=None):
        """Dictionary subclass containing a TypeData instance for each test type,
        keyed by type name"""
        self.initialized = False
        for key, value in iteritems(item_classes):
            self[key] = TypeData(manifest, value, meta_filters=meta_filters)
        self.initialized = True
        self.json_obj = None

    def __setitem__(self, key, value):
        if self.initialized:
            raise AttributeError
        dict.__setitem__(self, key, value)

    def paths(self):
        """Get a list of all paths containing test items
        without actually constructing all the items"""
        rv = set()
        for item_data in itervalues(self):
            rv |= set(item_data.paths())
        return rv


class Manifest(object):
    def __init__(self, tests_root=None, url_base="/", meta_filters=None):
        assert url_base is not None
        self._path_hash = {}
        self._data = ManifestData(self, meta_filters)
        self._reftest_nodes_by_url = None
        self.tests_root = tests_root
        self.url_base = url_base

    def __iter__(self):
        return self.itertypes()

    def itertypes(self, *types):
        if not types:
            types = sorted(self._data.keys())
        for item_type in types:
            for path in sorted(self._data[item_type]):
                tests = self._data[item_type][path]
                yield item_type, path, tests

    def iterpath(self, path):
        for type_tests in self._data.values():
            for test in type_tests.get(path, set()):
                yield test

    def iterdir(self, dir_name):
        if not dir_name.endswith(os.path.sep):
            dir_name = dir_name + os.path.sep
        for type_tests in self._data.values():
            for path, tests in type_tests.iteritems():
                if path.startswith(dir_name):
                    for test in tests:
                        yield test

    @property
    def reftest_nodes_by_url(self):
        if self._reftest_nodes_by_url is None:
            by_url = {}
            for path, nodes in itertools.chain(iteritems(self._data.get("reftest", {})),
                                               iteritems(self._data.get("reftest_node", {}))):
                for node in nodes:
                    by_url[node.url] = node
            self._reftest_nodes_by_url = by_url
        return self._reftest_nodes_by_url

    def get_reference(self, url):
        return self.reftest_nodes_by_url.get(url)

    def update(self, tree):
        """Update the manifest given an iterable of items that make up the updated manifest.

        The iterable must either generate tuples of the form (SourceFile, True) for paths
        that are to be updated, or (path, False) for items that are not to be updated. This
        unusual API is designed as an optimistaion meaning that SourceFile items need not be
        constructed in the case we are not updating a path, but the absence of an item from
        the iterator may be used to remove defunct entries from the manifest."""
        reftest_nodes = []
        seen_files = set()

        changed = False
        reftest_changes = False

        # Create local variable references to these dicts so we avoid the
        # attribute access in the hot loop below
        path_hash = self._path_hash
        data = self._data

        prev_files = data.paths()

        reftest_types = ("reftest", "reftest_node")

        for source_file, update in tree:
            if not update:
                rel_path = source_file
                seen_files.add(rel_path)
                assert rel_path in path_hash
                old_hash, old_type = path_hash[rel_path]
                if old_type in reftest_types:
                    manifest_items = data[old_type][rel_path]
                    reftest_nodes.extend((item, old_hash) for item in manifest_items)
            else:
                rel_path = source_file.rel_path
                seen_files.add(rel_path)

                file_hash = source_file.hash

                is_new = rel_path not in path_hash
                hash_changed = False

                if not is_new:
                    old_hash, old_type = path_hash[rel_path]
                    if old_hash != file_hash:
                        new_type, manifest_items = source_file.manifest_items()
                        hash_changed = True
                        if new_type != old_type:
                            del data[old_type][rel_path]
                            if old_type in reftest_types:
                                reftest_changes = True
                    else:
                        new_type = old_type
                        if old_type in reftest_types:
                            manifest_items = data[old_type][rel_path]
                else:
                    new_type, manifest_items = source_file.manifest_items()

                if new_type in reftest_types:
                    reftest_nodes.extend((item, file_hash) for item in manifest_items)
                    if is_new or hash_changed:
                        reftest_changes = True
                elif is_new or hash_changed:
                    data[new_type][rel_path] = set(manifest_items)

                if is_new or hash_changed:
                    path_hash[rel_path] = (file_hash, new_type)
                    changed = True

        deleted = prev_files - seen_files
        if deleted:
            changed = True
            for rel_path in deleted:
                if rel_path in path_hash:
                    _, old_type = path_hash[rel_path]
                    if old_type in reftest_types:
                        reftest_changes = True
                    del path_hash[rel_path]
                    try:
                        del data[old_type][rel_path]
                    except KeyError:
                        pass
                else:
                    for test_data in itervalues(data):
                        if rel_path in test_data:
                            del test_data[rel_path]

        if reftest_changes:
            reftests, reftest_nodes, changed_hashes = self._compute_reftests(reftest_nodes)
            data["reftest"].data = reftests
            data["reftest_node"].data = reftest_nodes
            path_hash.update(changed_hashes)

        return changed

    def _compute_reftests(self, reftest_nodes):
        self._reftest_nodes_by_url = {}
        has_inbound = set()
        for item, _ in reftest_nodes:
            for ref_url, ref_type in item.references:
                has_inbound.add(ref_url)

        reftests = defaultdict(set)
        references = defaultdict(set)
        changed_hashes = {}

        for item, file_hash in reftest_nodes:
            if item.url in has_inbound:
                # This is a reference
                if isinstance(item, RefTest):
                    item = item.to_RefTestNode()
                    changed_hashes[item.path] = (file_hash,
                                                 item.item_type)
                references[item.path].add(item)
            else:
                if isinstance(item, RefTestNode):
                    item = item.to_RefTest()
                    changed_hashes[item.path] = (file_hash,
                                                 item.item_type)
                reftests[item.path].add(item)
            self._reftest_nodes_by_url[item.url] = item

        return reftests, references, changed_hashes

    def to_json(self):
        out_items = {
            test_type: type_paths.to_json()
            for test_type, type_paths in iteritems(self._data) if type_paths
        }
        rv = {"url_base": self.url_base,
              "paths": {from_os_path(k): v for k, v in iteritems(self._path_hash)},
              "items": out_items,
              "version": CURRENT_VERSION}
        return rv

    @classmethod
    def from_json(cls, tests_root, obj, types=None, meta_filters=None):
        version = obj.get("version")
        if version != CURRENT_VERSION:
            raise ManifestVersionMismatch

        self = cls(tests_root, url_base=obj.get("url_base", "/"), meta_filters=meta_filters)
        if not hasattr(obj, "items") and hasattr(obj, "paths"):
            raise ManifestError

        self._path_hash = {to_os_path(k): v for k, v in iteritems(obj["paths"])}

        for test_type, type_paths in iteritems(obj["items"]):
            if test_type not in item_classes:
                raise ManifestError

            if types and test_type not in types:
                continue

            self._data[test_type].set_json(tests_root, type_paths)

        return self


def load(tests_root, manifest, types=None, meta_filters=None):
    logger = get_logger()

    logger.warning("Prefer load_and_update instead")
    return _load(logger, tests_root, manifest, types, meta_filters)


__load_cache = {}


def _load(logger, tests_root, manifest, types=None, meta_filters=None, allow_cached=True):
    # "manifest" is a path or file-like object.
    manifest_path = (manifest if isinstance(manifest, string_types)
                     else manifest.name)
    if allow_cached and manifest_path in __load_cache:
        return __load_cache[manifest_path]

    if isinstance(manifest, string_types):
        if os.path.exists(manifest):
            logger.debug("Opening manifest at %s" % manifest)
        else:
            logger.debug("Creating new manifest at %s" % manifest)
        try:
            with open(manifest) as f:
                rv = Manifest.from_json(tests_root,
                                        fast_json.load(f),
                                        types=types,
                                        meta_filters=meta_filters)
        except IOError:
            return None
        except ValueError:
            logger.warning("%r may be corrupted", manifest)
            return None
    else:
        rv = Manifest.from_json(tests_root,
                                fast_json.load(manifest),
                                types=types,
                                meta_filters=meta_filters)

    if allow_cached:
        __load_cache[manifest_path] = rv
    return rv


def load_and_update(tests_root,
                    manifest_path,
                    url_base,
                    update=True,
                    rebuild=False,
                    metadata_path=None,
                    cache_root=None,
                    working_copy=False,
                    types=None,
                    meta_filters=None,
                    write_manifest=True,
                    allow_cached=True):
    logger = get_logger()

    manifest = None
    if not rebuild:
        try:
            manifest = _load(logger,
                             tests_root,
                             manifest_path,
                             types=types,
                             meta_filters=meta_filters,
                             allow_cached=allow_cached)
        except ManifestVersionMismatch:
            logger.info("Manifest version changed, rebuilding")

        if manifest is not None and manifest.url_base != url_base:
            logger.info("Manifest url base did not match, rebuilding")

    if manifest is None:
        manifest = Manifest(tests_root, url_base, meta_filters=meta_filters)
        update = True

    if update:
        tree = vcs.get_tree(tests_root, manifest, manifest_path, cache_root,
                            working_copy, rebuild)
        changed = manifest.update(tree)
        if write_manifest and changed:
            write(manifest, manifest_path)
        tree.dump_caches()

    return manifest


def write(manifest, manifest_path):
    dir_name = os.path.dirname(manifest_path)
    if not os.path.exists(dir_name):
        os.makedirs(dir_name)
    with open(manifest_path, "wb") as f:
        # Use ',' instead of the default ', ' separator to prevent trailing
        # spaces: https://docs.python.org/2/library/json.html#json.dump
        json.dump(manifest.to_json(), f,
                  sort_keys=True, indent=1, separators=(',', ': '))
        f.write("\n")
