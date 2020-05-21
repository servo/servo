import io
import itertools
import json
import os
from copy import deepcopy
from multiprocessing import Pool, cpu_count
from six import (
    PY3,
    binary_type,
    ensure_text,
    iteritems,
    itervalues,
    string_types,
    text_type,
)

from . import vcs
from .item import (ConformanceCheckerTest, ManifestItem, ManualTest, RefTest, SupportFile,
                   TestharnessTest, VisualTest, WebDriverSpecTest, CrashTest)
from .log import get_logger
from .sourcefile import SourceFile
from .typedata import TypeData

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from logging import Logger
    from typing import Any
    from typing import Container
    from typing import Dict
    from typing import IO
    from typing import Iterator
    from typing import Iterable
    from typing import Optional
    from typing import Set
    from typing import Text
    from typing import Tuple
    from typing import Type
    from typing import Union

try:
    import ujson
    fast_json = ujson
except ImportError:
    fast_json = json  # type: ignore

CURRENT_VERSION = 8  # type: int


class ManifestError(Exception):
    pass


class ManifestVersionMismatch(ManifestError):
    pass


item_classes = {"testharness": TestharnessTest,
                "reftest": RefTest,
                "crashtest": CrashTest,
                "manual": ManualTest,
                "wdspec": WebDriverSpecTest,
                "conformancechecker": ConformanceCheckerTest,
                "visual": VisualTest,
                "support": SupportFile}  # type: Dict[str, Type[ManifestItem]]


def compute_manifest_items(source_file):
    # type: (SourceFile) -> Tuple[Tuple[Text, ...], Text, Set[ManifestItem], Text]
    rel_path_parts = source_file.rel_path_parts
    new_type, manifest_items = source_file.manifest_items()
    file_hash = source_file.hash
    return rel_path_parts, new_type, set(manifest_items), file_hash

if MYPY:
    ManifestDataType = Dict[Any, TypeData]
else:
    ManifestDataType = dict

class ManifestData(ManifestDataType):
    def __init__(self, manifest):
        # type: (Manifest) -> None
        """Dictionary subclass containing a TypeData instance for each test type,
        keyed by type name"""
        self.initialized = False  # type: bool
        for key, value in iteritems(item_classes):
            self[key] = TypeData(manifest, value)
        self.initialized = True
        self.json_obj = None  # type: None

    def __setitem__(self, key, value):
        # type: (str, TypeData) -> None
        if self.initialized:
            raise AttributeError
        dict.__setitem__(self, key, value)

    def paths(self):
        # type: () -> Set[Text]
        """Get a list of all paths containing test items
        without actually constructing all the items"""
        rv = set()  # type: Set[Text]
        for item_data in itervalues(self):
            for item in item_data:
                rv.add(os.path.sep.join(item))
        return rv

    def type_by_path(self):
        # type: () -> Dict[Tuple[Text, ...], str]
        rv = {}
        for item_type, item_data in iteritems(self):
            for item in item_data:
                rv[item] = item_type
        return rv



class Manifest(object):
    def __init__(self, tests_root=None, url_base="/"):
        # type: (Optional[str], Text) -> None
        assert url_base is not None
        self._data = ManifestData(self)  # type: ManifestData
        self.tests_root = tests_root  # type: Optional[str]
        self.url_base = url_base  # type: Text

    def __iter__(self):
        # type: () -> Iterator[Tuple[str, Text, Set[ManifestItem]]]
        return self.itertypes()

    def itertypes(self, *types):
        # type: (*str) -> Iterator[Tuple[str, Text, Set[ManifestItem]]]
        for item_type in (types or sorted(self._data.keys())):
            for path in self._data[item_type]:
                str_path = os.sep.join(path)
                tests = self._data[item_type][path]
                yield item_type, str_path, tests

    def iterpath(self, path):
        # type: (Text) -> Iterable[ManifestItem]
        tpath = tuple(path.split(os.path.sep))

        for type_tests in self._data.values():
            i = type_tests.get(tpath, set())
            assert i is not None
            for test in i:
                yield test

    def iterdir(self, dir_name):
        # type: (Text) -> Iterable[ManifestItem]
        tpath = tuple(dir_name.split(os.path.sep))
        tpath_len = len(tpath)

        for type_tests in self._data.values():
            for path, tests in iteritems(type_tests):
                if path[:tpath_len] == tpath:
                    for test in tests:
                        yield test

    def update(self, tree, parallel=True):
        # type: (Iterable[Tuple[Union[SourceFile, bytes], bool]], bool) -> bool
        """Update the manifest given an iterable of items that make up the updated manifest.

        The iterable must either generate tuples of the form (SourceFile, True) for paths
        that are to be updated, or (path, False) for items that are not to be updated. This
        unusual API is designed as an optimistaion meaning that SourceFile items need not be
        constructed in the case we are not updating a path, but the absence of an item from
        the iterator may be used to remove defunct entries from the manifest."""

        changed = False

        # Create local variable references to these dicts so we avoid the
        # attribute access in the hot loop below
        data = self._data

        types = data.type_by_path()
        deleted = set(types)

        to_update = []

        for source_file_or_path, update in tree:
            if not update:
                assert isinstance(source_file_or_path, (binary_type, text_type))
                path = ensure_text(source_file_or_path)
                deleted.remove(tuple(path.split(os.path.sep)))
            else:
                assert not isinstance(source_file_or_path, (binary_type, text_type))
                source_file = source_file_or_path
                rel_path_parts = source_file.rel_path_parts
                assert isinstance(rel_path_parts, tuple)

                is_new = rel_path_parts not in deleted  # type: bool
                hash_changed = False  # type: bool

                if not is_new:
                    deleted.remove(rel_path_parts)
                    old_type = types[rel_path_parts]
                    old_hash = data[old_type].hashes[rel_path_parts]
                    file_hash = source_file.hash  # type: Text
                    if old_hash != file_hash:
                        hash_changed = True
                        del data[old_type][rel_path_parts]

                if is_new or hash_changed:
                    to_update.append(source_file)

        if to_update:
            changed = True

        if parallel and len(to_update) > 25 and cpu_count() > 1:
            # 25 derived experimentally (2020-01) to be approximately
            # the point at which it is quicker to create Pool and
            # parallelize this
            pool = Pool()

            # chunksize set > 1 when more than 10000 tests, because
            # chunking is a net-gain once we get to very large numbers
            # of items (again, experimentally, 2020-01)
            results = pool.imap_unordered(compute_manifest_items,
                                          to_update,
                                          chunksize=max(1, len(to_update) // 10000)
                                          )  # type: Iterator[Tuple[Tuple[Text, ...], Text, Set[ManifestItem], Text]]
        elif PY3:
            results = map(compute_manifest_items, to_update)
        else:
            results = itertools.imap(compute_manifest_items, to_update)

        for result in results:
            rel_path_parts, new_type, manifest_items, file_hash = result
            data[new_type][rel_path_parts] = manifest_items
            data[new_type].hashes[rel_path_parts] = file_hash

        if deleted:
            changed = True
            for rel_path_parts in deleted:
                for test_data in itervalues(data):
                    if rel_path_parts in test_data:
                        del test_data[rel_path_parts]

        return changed

    def to_json(self, caller_owns_obj=True):
        # type: (bool) -> Dict[Text, Any]
        """Dump a manifest into a object which can be serialized as JSON

        If caller_owns_obj is False, then the return value remains
        owned by the manifest; it is _vitally important_ that _no_
        (even read) operation is done on the manifest, as otherwise
        objects within the object graph rooted at the return value can
        be mutated. This essentially makes this mode very dangerous
        and only to be used under extreme care.

        """
        out_items = {
            test_type: type_paths.to_json()
            for test_type, type_paths in iteritems(self._data) if type_paths
        }

        if caller_owns_obj:
            out_items = deepcopy(out_items)

        rv = {"url_base": self.url_base,
              "items": out_items,
              "version": CURRENT_VERSION}  # type: Dict[Text, Any]
        return rv

    @classmethod
    def from_json(cls, tests_root, obj, types=None, callee_owns_obj=False):
        # type: (str, Dict[Text, Any], Optional[Container[Text]], bool) -> Manifest
        """Load a manifest from a JSON object

        This loads a manifest for a given local test_root path from an
        object obj, potentially partially loading it to only load the
        types given by types.

        If callee_owns_obj is True, then ownership of obj transfers
        to this function when called, and the caller must never mutate
        the obj or anything referred to in the object graph rooted at
        obj.

        """
        version = obj.get("version")
        if version != CURRENT_VERSION:
            raise ManifestVersionMismatch

        self = cls(tests_root, url_base=obj.get("url_base", "/"))
        if not hasattr(obj, "items"):
            raise ManifestError

        for test_type, type_paths in iteritems(obj["items"]):
            if test_type not in item_classes:
                raise ManifestError

            if types and test_type not in types:
                continue

            if not callee_owns_obj:
                type_paths = deepcopy(type_paths)

            self._data[test_type].set_json(type_paths)

        return self


def load(tests_root, manifest, types=None):
    # type: (str, Union[IO[bytes], str], Optional[Container[Text]]) -> Optional[Manifest]
    logger = get_logger()

    logger.warning("Prefer load_and_update instead")
    return _load(logger, tests_root, manifest, types)


__load_cache = {}  # type: Dict[str, Manifest]


def _load(logger,  # type: Logger
          tests_root,  # type: str
          manifest,  # type: Union[IO[bytes], str]
          types=None,  # type: Optional[Container[Text]]
          allow_cached=True  # type: bool
          ):
    # type: (...) -> Optional[Manifest]
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
            with io.open(manifest, "r", encoding="utf-8") as f:
                rv = Manifest.from_json(tests_root,
                                        fast_json.load(f),
                                        types=types,
                                        callee_owns_obj=True)
        except IOError:
            return None
        except ValueError:
            logger.warning("%r may be corrupted", manifest)
            return None
    else:
        rv = Manifest.from_json(tests_root,
                                fast_json.load(manifest),
                                types=types,
                                callee_owns_obj=True)

    if allow_cached:
        __load_cache[manifest_path] = rv
    return rv


def load_and_update(tests_root,  # type: bytes
                    manifest_path,  # type: bytes
                    url_base,  # type: Text
                    update=True,  # type: bool
                    rebuild=False,  # type: bool
                    metadata_path=None,  # type: Optional[bytes]
                    cache_root=None,  # type: Optional[bytes]
                    working_copy=True,  # type: bool
                    types=None,  # type: Optional[Container[Text]]
                    write_manifest=True,  # type: bool
                    allow_cached=True,  # type: bool
                    parallel=True  # type: bool
                    ):
    # type: (...) -> Manifest
    logger = get_logger()

    manifest = None
    if not rebuild:
        try:
            manifest = _load(logger,
                             tests_root,
                             manifest_path,
                             types=types,
                             allow_cached=allow_cached)
        except ManifestVersionMismatch:
            logger.info("Manifest version changed, rebuilding")

        if manifest is not None and manifest.url_base != url_base:
            logger.info("Manifest url base did not match, rebuilding")
            manifest = None

    if manifest is None:
        manifest = Manifest(tests_root, url_base)
        rebuild = True
        update = True

    if rebuild or update:
        tree = vcs.get_tree(tests_root, manifest, manifest_path, cache_root,
                            working_copy, rebuild)
        changed = manifest.update(tree, parallel)
        if write_manifest and changed:
            write(manifest, manifest_path)
        tree.dump_caches()

    return manifest


def write(manifest, manifest_path):
    # type: (Manifest, bytes) -> None
    dir_name = os.path.dirname(manifest_path)
    if not os.path.exists(dir_name):
        os.makedirs(dir_name)
    with open(manifest_path, "w") as f:
        # Use ',' instead of the default ', ' separator to prevent trailing
        # spaces: https://docs.python.org/2/library/json.html#json.dump
        json.dump(manifest.to_json(caller_owns_obj=True), f,
                  sort_keys=True, indent=1, separators=(',', ': '))
        f.write("\n")
