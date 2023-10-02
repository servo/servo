import os
from atomicwrites import atomic_write
from copy import deepcopy
from logging import Logger
from multiprocessing import Pool
from typing import (Any, Callable, Container, Dict, IO, Iterator, Iterable, Optional, Set, Text, Tuple, Type,
                    Union)

from . import jsonlib
from . import vcs
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
from .log import get_logger
from .mputil import max_parallelism
from .sourcefile import SourceFile
from .typedata import TypeData


CURRENT_VERSION: int = 8


class ManifestError(Exception):
    pass


class ManifestVersionMismatch(ManifestError):
    pass


class InvalidCacheError(Exception):
    pass


item_classes: Dict[Text, Type[ManifestItem]] = {"testharness": TestharnessTest,
                                                "reftest": RefTest,
                                                "print-reftest": PrintRefTest,
                                                "crashtest": CrashTest,
                                                "manual": ManualTest,
                                                "wdspec": WebDriverSpecTest,
                                                "conformancechecker": ConformanceCheckerTest,
                                                "visual": VisualTest,
                                                "spec": SpecItem,
                                                "support": SupportFile}


def compute_manifest_items(source_file: SourceFile) -> Optional[Tuple[Tuple[Text, ...], Text, Set[ManifestItem], Text]]:
    rel_path_parts = source_file.rel_path_parts
    new_type, manifest_items = source_file.manifest_items()
    file_hash = source_file.hash
    return rel_path_parts, new_type, set(manifest_items), file_hash


def compute_manifest_spec_items(source_file: SourceFile) -> Optional[Tuple[Tuple[Text, ...], Text, Set[ManifestItem], Text]]:
    spec_tuple = source_file.manifest_spec_items()
    if not spec_tuple:
        return None

    new_type, manifest_items = spec_tuple
    rel_path_parts = source_file.rel_path_parts
    file_hash = source_file.hash
    return rel_path_parts, new_type, set(manifest_items), file_hash


ManifestDataType = Dict[Any, TypeData]


class ManifestData(ManifestDataType):
    def __init__(self, manifest: "Manifest") -> None:
        """Dictionary subclass containing a TypeData instance for each test type,
        keyed by type name"""
        self.initialized: bool = False
        for key, value in item_classes.items():
            self[key] = TypeData(manifest, value)
        self.initialized = True
        self.json_obj: None = None

    def __setitem__(self, key: Text, value: TypeData) -> None:
        if self.initialized:
            raise AttributeError
        dict.__setitem__(self, key, value)

    def paths(self) -> Set[Text]:
        """Get a list of all paths containing test items
        without actually constructing all the items"""
        rv: Set[Text] = set()
        for item_data in self.values():
            for item in item_data:
                rv.add(os.path.sep.join(item))
        return rv

    def type_by_path(self) -> Dict[Tuple[Text, ...], Text]:
        rv = {}
        for item_type, item_data in self.items():
            for item in item_data:
                rv[item] = item_type
        return rv


class Manifest:
    def __init__(self, tests_root: Text, url_base: Text = "/") -> None:
        assert url_base is not None
        self._data: ManifestData = ManifestData(self)
        self.tests_root: Text = tests_root
        self.url_base: Text = url_base

    def __iter__(self) -> Iterator[Tuple[Text, Text, Set[ManifestItem]]]:
        return self.itertypes()

    def itertypes(self, *types: Text) -> Iterator[Tuple[Text, Text, Set[ManifestItem]]]:
        for item_type in (types or sorted(self._data.keys())):
            for path in self._data[item_type]:
                rel_path = os.sep.join(path)
                tests = self._data[item_type][path]
                yield item_type, rel_path, tests

    def iterpath(self, path: Text) -> Iterable[ManifestItem]:
        tpath = tuple(path.split(os.path.sep))

        for type_tests in self._data.values():
            i = type_tests.get(tpath, set())
            assert i is not None
            yield from i

    def iterdir(self, dir_name: Text) -> Iterable[ManifestItem]:
        tpath = tuple(dir_name.split(os.path.sep))
        tpath_len = len(tpath)

        for type_tests in self._data.values():
            for path, tests in type_tests.items():
                if path[:tpath_len] == tpath:
                    yield from tests

    def update(self, tree: Iterable[Tuple[Text, Optional[Text], bool]], parallel: bool = True,
               update_func: Callable[..., Any] = compute_manifest_items) -> bool:
        """Update the manifest given an iterable of items that make up the updated manifest.

        The iterable must either generate tuples of the form (SourceFile, True) for paths
        that are to be updated, or (path, False) for items that are not to be updated. This
        unusual API is designed as an optimistaion meaning that SourceFile items need not be
        constructed in the case we are not updating a path, but the absence of an item from
        the iterator may be used to remove defunct entries from the manifest."""

        logger = get_logger()

        changed = False

        # Create local variable references to these dicts so we avoid the
        # attribute access in the hot loop below
        data = self._data

        types = data.type_by_path()
        remaining_manifest_paths = set(types)

        to_update = []

        for path, file_hash, updated in tree:
            path_parts = tuple(path.split(os.path.sep))
            is_new = path_parts not in remaining_manifest_paths

            if not updated and is_new:
                # This is kind of a bandaid; if we ended up here the cache
                # was invalid but we've been using it anyway. That's obviously
                # bad; we should fix the underlying issue that we sometimes
                # use an invalid cache. But at least this fixes the immediate
                # problem
                raise InvalidCacheError

            if not updated:
                remaining_manifest_paths.remove(path_parts)
            else:
                assert self.tests_root is not None
                source_file = SourceFile(self.tests_root,
                                         path,
                                         self.url_base,
                                         file_hash)

                hash_changed: bool = False

                if not is_new:
                    if file_hash is None:
                        file_hash = source_file.hash
                    remaining_manifest_paths.remove(path_parts)
                    old_type = types[path_parts]
                    old_hash = data[old_type].hashes[path_parts]
                    if old_hash != file_hash:
                        hash_changed = True
                        del data[old_type][path_parts]

                if is_new or hash_changed:
                    to_update.append(source_file)

        if to_update:
            logger.debug("Computing manifest update for %s items" % len(to_update))
            changed = True

        # 25 items was derived experimentally (2020-01) to be approximately the
        # point at which it is quicker to create a Pool and parallelize update.
        pool = None
        processes = max_parallelism()
        if parallel and len(to_update) > 25 and processes > 1:
            pool = Pool(processes)

            # chunksize set > 1 when more than 10000 tests, because
            # chunking is a net-gain once we get to very large numbers
            # of items (again, experimentally, 2020-01)
            chunksize = max(1, len(to_update) // 10000)
            logger.debug("Doing a multiprocessed update. "
                "Processes: %s, chunksize: %s" % (processes, chunksize))
            results: Iterator[Optional[Tuple[Tuple[Text, ...],
                                    Text,
                                    Set[ManifestItem], Text]]] = pool.imap_unordered(
                                        update_func,
                                        to_update,
                                        chunksize=chunksize)
        else:
            results = map(update_func, to_update)

        for result in results:
            if not result:
                continue
            rel_path_parts, new_type, manifest_items, file_hash = result
            data[new_type][rel_path_parts] = manifest_items
            data[new_type].hashes[rel_path_parts] = file_hash

        # Make sure to terminate the Pool, to avoid hangs on Python 3.
        # https://docs.python.org/3/library/multiprocessing.html#multiprocessing.pool.Pool
        if pool is not None:
            pool.terminate()

        if remaining_manifest_paths:
            changed = True
            for rel_path_parts in remaining_manifest_paths:
                for test_data in data.values():
                    if rel_path_parts in test_data:
                        del test_data[rel_path_parts]

        return changed

    def to_json(self, caller_owns_obj: bool = True) -> Dict[Text, Any]:
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
            for test_type, type_paths in self._data.items() if type_paths
        }

        if caller_owns_obj:
            out_items = deepcopy(out_items)

        rv: Dict[Text, Any] = {"url_base": self.url_base,
              "items": out_items,
              "version": CURRENT_VERSION}
        return rv

    @classmethod
    def from_json(cls,
                  tests_root: Text,
                  obj: Dict[Text, Any],
                  types: Optional[Container[Text]] = None,
                  callee_owns_obj: bool = False) -> "Manifest":
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

        for test_type, type_paths in obj["items"].items():
            if test_type not in item_classes:
                raise ManifestError

            if types and test_type not in types:
                continue

            if not callee_owns_obj:
                type_paths = deepcopy(type_paths)

            self._data[test_type].set_json(type_paths)

        return self


def load(tests_root: Text, manifest: Union[IO[bytes], Text], types: Optional[Container[Text]] = None) -> Optional[Manifest]:
    logger = get_logger()

    logger.warning("Prefer load_and_update instead")
    return _load(logger, tests_root, manifest, types)


__load_cache: Dict[Text, Manifest] = {}


def _load(logger: Logger,
          tests_root: Text,
          manifest: Union[IO[bytes], Text],
          types: Optional[Container[Text]] = None,
          allow_cached: bool = True
          ) -> Optional[Manifest]:
    manifest_path = (manifest if isinstance(manifest, str)
                     else manifest.name)
    if allow_cached and manifest_path in __load_cache:
        return __load_cache[manifest_path]

    if isinstance(manifest, str):
        if os.path.exists(manifest):
            logger.debug("Opening manifest at %s" % manifest)
        else:
            logger.debug("Creating new manifest at %s" % manifest)
        try:
            with open(manifest, encoding="utf-8") as f:
                rv = Manifest.from_json(tests_root,
                                        jsonlib.load(f),
                                        types=types,
                                        callee_owns_obj=True)
        except OSError:
            return None
        except ValueError:
            logger.warning("%r may be corrupted", manifest)
            return None
    else:
        rv = Manifest.from_json(tests_root,
                                jsonlib.load(manifest),
                                types=types,
                                callee_owns_obj=True)

    if allow_cached:
        __load_cache[manifest_path] = rv
    return rv


def load_and_update(tests_root: Text,
                    manifest_path: Text,
                    url_base: Text,
                    update: bool = True,
                    rebuild: bool = False,
                    metadata_path: Optional[Text] = None,
                    cache_root: Optional[Text] = None,
                    working_copy: bool = True,
                    types: Optional[Container[Text]] = None,
                    write_manifest: bool = True,
                    allow_cached: bool = True,
                    parallel: bool = True
                    ) -> Manifest:

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
        except ManifestError:
            logger.warning("Failed to load manifest, rebuilding")

        if manifest is not None and manifest.url_base != url_base:
            logger.info("Manifest url base did not match, rebuilding")
            manifest = None

    if manifest is None:
        manifest = Manifest(tests_root, url_base)
        rebuild = True
        update = True

    if rebuild or update:
        logger.info("Updating manifest")
        for retry in range(2):
            try:
                tree = vcs.get_tree(tests_root, manifest, manifest_path, cache_root,
                                    working_copy, rebuild)
                changed = manifest.update(tree, parallel)
                break
            except InvalidCacheError:
                logger.warning("Manifest cache was invalid, doing a complete rebuild")
                rebuild = True
        else:
            # If we didn't break there was an error
            raise
        if write_manifest and changed:
            write(manifest, manifest_path)
        tree.dump_caches()

    return manifest


def write(manifest: Manifest, manifest_path: Text) -> None:
    dir_name = os.path.dirname(manifest_path)
    if not os.path.exists(dir_name):
        os.makedirs(dir_name)
    with atomic_write(manifest_path, overwrite=True) as f:
        # Use ',' instead of the default ', ' separator to prevent trailing
        # spaces: https://docs.python.org/2/library/json.html#json.dump
        jsonlib.dump_dist(manifest.to_json(caller_owns_obj=True), f)
        f.write("\n")
