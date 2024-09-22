# mypy: allow-untyped-calls, allow-untyped-defs
from __future__ import annotations

import abc
import hashlib
import itertools
import json
import os
import queue
from urllib.parse import urlsplit
from abc import ABCMeta, abstractmethod
from collections import defaultdict, deque, namedtuple
from typing import (cast, Any, Callable, Dict, Deque, List, Mapping, MutableMapping, Optional, Set,
                    Tuple, Type)

from . import manifestinclude
from . import manifestexpected
from . import manifestupdate
from . import wpttest
from mozlog import structured

manifest = None
manifest_update = None
download_from_github = None

# Mapping from (subsuite, test_type) to [Test]
TestsByType = Mapping[Tuple[str, str], List[wpttest.Test]]


def do_delayed_imports():
    # This relies on an already loaded module having set the sys.path correctly :(
    global manifest, manifest_update, download_from_github
    from manifest import manifest  # type: ignore
    from manifest import update as manifest_update
    from manifest.download import download_from_github  # type: ignore


class WriteQueue:
    def __init__(self, queue_cls: Callable[[], Any] = queue.SimpleQueue):
        """Queue wrapper that is only used for writing items to a queue.

        Once all items are enqueued, call to_read() to get a reader for the queue.
        This will also prevent further writes using this writer."""
        self._raw_queue = queue_cls()

    def put(self, item: Any) -> None:
        if self._raw_queue is None:
            raise ValueError("Tried to write to closed queue")
        self._raw_queue.put(item)

    def to_read(self) -> ReadQueue:
        reader = ReadQueue(self._raw_queue)
        self._raw_queue = None
        return reader


class ReadQueue:
    def __init__(self, raw_queue: Any):
        self._raw_queue = raw_queue

    def get(self) -> Any:
        """Queue wrapper that is only used for reading items from a queue."""
        return self._raw_queue.get(False)



class TestGroups:
    def __init__(self, logger, path, subsuites):
        try:
            with open(path) as f:
                data = json.load(f)
        except ValueError:
            logger.critical("test groups file %s not valid json" % path)
            raise

        self.tests_by_group = defaultdict(set)
        self.group_by_test = {}
        for group, test_ids in data.items():
            id_parts = group.split(":", 1)
            if len(id_parts) == 1:
                group_name = id_parts[0]
                subsuite = ""
            else:
                subsuite, group_name = id_parts
                if subsuite not in subsuites:
                    raise ValueError(f"Unknown subsuite {subsuite} in group data {group}")
            for test_id in test_ids:
                self.group_by_test[(subsuite, test_id)] = group_name
                self.tests_by_group[group_name].add(test_id)


def load_subsuites(logger: Any,
                   base_run_info: wpttest.RunInfo,
                   path: Optional[str],
                   include_subsuites: Set[str]) -> Dict[str, Subsuite]:
    subsuites: Dict[str, Subsuite] = {}
    run_all_subsuites = not include_subsuites
    include_subsuites.add("")

    def maybe_add_subsuite(name: str, data: Dict[str, Any]) -> None:
        if run_all_subsuites or name in include_subsuites:
            subsuites[name] = Subsuite(name,
                                       data.get("config", {}),
                                       base_run_info,
                                       run_info_extras=data.get("run_info", {}),
                                       include=data.get("include"),
                                       tags=set(data["tags"]) if "tags" in data else None)
            if name in include_subsuites:
                include_subsuites.remove(name)

    maybe_add_subsuite("", {})

    if path is None:
        if include_subsuites:
            raise ValueError("Unrecognised subsuites {','.join(include_subsuites)}, missing --subsuite-file?")
        return subsuites

    try:
        with open(path) as f:
            data = json.load(f)
    except ValueError:
        logger.critical("subsuites file %s not valid json" % path)
        raise

    for key, subsuite in data.items():
        if key == "":
            raise ValueError("Subsuites must have a non-empty name")
        maybe_add_subsuite(key, subsuite)

    if include_subsuites:
        raise ValueError(f"Unrecognised subsuites {','.join(include_subsuites)}")

    return subsuites


class Subsuite:
    def __init__(self,
                 name: str,
                 config: Dict[str, Any],
                 base_run_info: Optional[wpttest.RunInfo] = None,
                 run_info_extras: Optional[Dict[str, Any]] = None,
                 include: Optional[List[str]] = None,
                 tags: Optional[Set[str]] = None):
        self.name = name
        self.config = config
        self.run_info_extras = run_info_extras or {}
        self.run_info_extras["subsuite"] = name
        self.include = include
        self.tags = tags

        run_info = base_run_info.copy() if base_run_info is not None else {}
        run_info.update(self.run_info_extras)
        self.run_info = run_info

    def manifest_filters(self, manifests):
        if self.name:
            manifest_filters = [TestFilter(manifests,
                                           include=self.include,
                                           explicit=True)]
            return manifest_filters

        # use base manifest_filters for default subsuite
        return []

    def __repr__(self):
        return "Subsuite('%s', config:%s, run_info:%s)" % (self.name or 'default',
                                                           str(self.config),
                                                           str(self.run_info))


def read_include_from_file(file):
    new_include = []
    with open(file) as f:
        for line in f:
            line = line.strip()
            # Allow whole-line comments;
            # fragments mean we can't have partial line #-based comments
            if len(line) > 0 and not line.startswith("#"):
                new_include.append(line)
    return new_include


def update_include_for_groups(test_groups, include):
    new_include = []
    if include is None:
        # We're just running everything
        for tests in test_groups.tests_by_group.values():
            new_include.extend(tests)
    else:
        for item in include:
            if item in test_groups.tests_by_group:
                new_include.extend(test_groups.tests_by_group[item])
            else:
                new_include.append(item)
    return new_include


class TestChunker(abc.ABC):
    def __init__(self, total_chunks: int, chunk_number: int, **kwargs: Any):
        self.total_chunks = total_chunks
        self.chunk_number = chunk_number
        assert self.chunk_number <= self.total_chunks
        self.logger = structured.get_default_logger()
        assert self.logger
        self.kwargs = kwargs

    @abstractmethod
    def __call__(self, manifest):
        ...


class Unchunked(TestChunker):
    def __init__(self, *args, **kwargs):
        TestChunker.__init__(self, *args, **kwargs)
        assert self.total_chunks == 1

    def __call__(self, manifest, **kwargs):
        yield from manifest


class HashChunker(TestChunker):
    def __call__(self, manifest):
        for test_type, test_path, tests in manifest:
            tests_for_chunk = {
                test for test in tests
                if self._key_in_chunk(self.chunk_key(test_type, test_path, test))
            }
            if tests_for_chunk:
                yield test_type, test_path, tests_for_chunk

    def _key_in_chunk(self, key: str) -> bool:
        chunk_index = self.chunk_number - 1
        digest = hashlib.md5(key.encode()).hexdigest()
        return int(digest, 16) % self.total_chunks == chunk_index

    @abstractmethod
    def chunk_key(self, test_type: str, test_path: str,
                  test: wpttest.Test) -> str:
        ...


class PathHashChunker(HashChunker):
    def chunk_key(self, test_type: str, test_path: str,
                  test: wpttest.Test) -> str:
        return test_path


class IDHashChunker(HashChunker):
    def chunk_key(self, test_type: str, test_path: str,
                  test: wpttest.Test) -> str:
        return cast(str, test.id)


class DirectoryHashChunker(HashChunker):
    """Like HashChunker except the directory is hashed.

    This ensures that all tests in the same directory end up in the same
    chunk.
    """
    def chunk_key(self, test_type: str, test_path: str,
                  test: wpttest.Test) -> str:
        depth = self.kwargs.get("depth")
        if depth:
            return os.path.sep.join(os.path.dirname(test_path).split(os.path.sep, depth)[:depth])
        else:
            return os.path.dirname(test_path)


class TestFilter:
    """Callable that restricts the set of tests in a given manifest according
    to initial criteria"""
    def __init__(self, test_manifests, include=None, exclude=None, manifest_path=None, explicit=False):
        if manifest_path is None or include or explicit:
            self.manifest = manifestinclude.IncludeManifest.create()
            self.manifest.set_defaults()
        else:
            self.manifest = manifestinclude.get_manifest(manifest_path)

        if include or explicit:
            self.manifest.set("skip", "true")

        if include:
            for item in include:
                self.manifest.add_include(test_manifests, item)

        if exclude:
            for item in exclude:
                self.manifest.add_exclude(test_manifests, item)

    def __call__(self, manifest_iter):
        for test_type, test_path, tests in manifest_iter:
            include_tests = set()
            for test in tests:
                if self.manifest.include(test):
                    include_tests.add(test)

            if include_tests:
                yield test_type, test_path, include_tests


class TagFilter:
    def __init__(self, include_tags, exclude_tags):
        self.include_tags = set(include_tags) if include_tags else None
        self.exclude_tags = set(exclude_tags) if exclude_tags else None

    def __call__(self, test):
        does_match = True
        if self.include_tags:
            does_match &= bool(test.tags & self.include_tags)
        if self.exclude_tags:
            does_match &= not (test.tags & self.exclude_tags)
        return does_match


class ManifestLoader:
    def __init__(self, test_paths, force_manifest_update=False, manifest_download=False,
                 types=None):
        do_delayed_imports()
        self.test_paths = test_paths
        self.force_manifest_update = force_manifest_update
        self.manifest_download = manifest_download
        self.types = types
        self.logger = structured.get_default_logger()
        if self.logger is None:
            self.logger = structured.structuredlog.StructuredLogger("ManifestLoader")

    def load(self):
        rv = {}
        for url_base, test_root in self.test_paths.items():
            manifest_file = self.load_manifest(url_base, test_root)
            path_data = {"url_base": url_base,
                         "tests_path": test_root.tests_path,
                         "metadata_path": test_root.metadata_path,
                         "manifest_path": test_root.manifest_path}
            rv[manifest_file] = path_data
        return rv

    def load_manifest(self, url_base, test_root):
        cache_root = os.path.join(test_root.metadata_path, ".cache")
        if self.manifest_download:
            download_from_github(test_root.manifest_path, test_root.tests_path)
        return manifest.load_and_update(test_root.tests_path, test_root.manifest_path, url_base,
                                        cache_root=cache_root, update=self.force_manifest_update,
                                        types=self.types)


def iterfilter(filters, iter):
    for f in filters:
        iter = f(iter)
    yield from iter


class TestLoader:
    """Loads tests according to a WPT manifest and any associated expectation files"""
    def __init__(self,
                 test_manifests,
                 test_types,
                 base_run_info,
                 subsuites=None,
                 manifest_filters=None,
                 test_filters=None,
                 chunk_type="none",
                 total_chunks=1,
                 chunk_number=1,
                 include_https=True,
                 include_h2=True,
                 include_webtransport_h3=False,
                 skip_timeout=False,
                 skip_crash=False,
                 skip_implementation_status=None,
                 chunker_kwargs=None):

        self.test_types = test_types
        self.base_run_info = base_run_info
        self.subsuites = subsuites or {}

        self.manifest_filters = manifest_filters if manifest_filters is not None else []
        self.test_filters = test_filters if test_filters is not None else []

        self.manifests = test_manifests
        self.tests = None
        self.disabled_tests = None
        self.include_https = include_https
        self.include_h2 = include_h2
        self.include_webtransport_h3 = include_webtransport_h3
        self.skip_timeout = skip_timeout
        self.skip_crash = skip_crash
        self.skip_implementation_status = skip_implementation_status

        self.chunk_type = chunk_type
        self.total_chunks = total_chunks
        self.chunk_number = chunk_number

        if chunker_kwargs is None:
            chunker_kwargs = {}
        self.chunker = {"none": Unchunked,
                        "hash": PathHashChunker,
                        "id_hash": IDHashChunker,
                        "dir_hash": DirectoryHashChunker}[chunk_type](total_chunks,
                                                                      chunk_number,
                                                                      **chunker_kwargs)

        self._test_ids = None

        self.directory_manifests = {}
        self._load_tests()

    @property
    def test_ids(self):
        if self._test_ids is None:
            self._test_ids = []
            for test_dict in [self.disabled_tests, self.tests]:
                for subsuite in self.subsuites:
                    for test_type in self.test_types:
                        self._test_ids += [item.id for item in test_dict[subsuite][test_type]]
        return self._test_ids

    def get_test(self, manifest_file, manifest_test, inherit_metadata, test_metadata):
        if test_metadata is not None:
            inherit_metadata.append(test_metadata)
            test_metadata = test_metadata.get_test(manifestupdate.get_test_name(manifest_test.id))

        return wpttest.from_manifest(manifest_file, manifest_test, inherit_metadata, test_metadata)

    def load_dir_metadata(self, run_info, test_manifest, metadata_path, test_path):
        rv = []
        path_parts = os.path.dirname(test_path).split(os.path.sep)
        for i in range(len(path_parts) + 1):
            path = os.path.join(metadata_path, os.path.sep.join(path_parts[:i]), "__dir__.ini")
            if path not in self.directory_manifests:
                self.directory_manifests[path] = manifestexpected.get_dir_manifest(path,
                                                                                   run_info)
            manifest = self.directory_manifests[path]
            if manifest is not None:
                rv.append(manifest)
        return rv

    def load_metadata(self, run_info, test_manifest, metadata_path, test_path):
        inherit_metadata = self.load_dir_metadata(run_info, test_manifest, metadata_path, test_path)
        test_metadata = manifestexpected.get_manifest(
            metadata_path, test_path, run_info)
        return inherit_metadata, test_metadata

    def iter_tests(self, run_info, manifest_filters):
        manifest_items = []
        manifests_by_url_base = {}

        for manifest in sorted(self.manifests.keys(), key=lambda x:x.url_base):
            manifest_iter = iterfilter(manifest_filters,
                                       manifest.itertypes(*self.test_types))
            manifest_items.extend(manifest_iter)
            manifests_by_url_base[manifest.url_base] = manifest

        if self.chunker is not None:
            manifest_items = self.chunker(manifest_items)

        for test_type, test_path, tests in manifest_items:
            manifest_file = manifests_by_url_base[next(iter(tests)).url_base]
            metadata_path = self.manifests[manifest_file]["metadata_path"]

            inherit_metadata, test_metadata = self.load_metadata(run_info, manifest_file, metadata_path, test_path)
            for test in tests:
                wpt_test = self.get_test(manifest_file, test, inherit_metadata, test_metadata)
                if all(f(wpt_test) for f in self.test_filters):
                    yield test_path, test_type, wpt_test

    def _load_tests(self):
        """Read in the tests from the manifest file"""
        tests_enabled = {}
        tests_disabled = {}

        for subsuite_name, subsuite in self.subsuites.items():
            tests_enabled[subsuite_name] = defaultdict(list)
            tests_disabled[subsuite_name] = defaultdict(list)
            run_info = subsuite.run_info
            if not subsuite_name:
                manifest_filters = self.manifest_filters
            else:
                manifest_filters = subsuite.manifest_filters(self.manifests)
            for test_path, test_type, test in self.iter_tests(run_info, manifest_filters):
                enabled = not test.disabled()
                if not self.include_https and test.environment["protocol"] == "https":
                    enabled = False
                if not self.include_h2 and test.environment["protocol"] == "h2":
                    enabled = False
                if self.skip_timeout and test.expected() == "TIMEOUT":
                    enabled = False
                if self.skip_crash and test.expected() == "CRASH":
                    enabled = False
                if self.skip_implementation_status and test.implementation_status() in self.skip_implementation_status:
                    # for backlog, we want to run timeout/crash:
                    if not (test.implementation_status() == "implementing" and test.expected() in ["TIMEOUT", "CRASH"]):
                        enabled = False
                target = tests_enabled if enabled else tests_disabled
                target[subsuite_name][test_type].append(test)

        self.tests = tests_enabled
        self.disabled_tests = tests_disabled



def get_test_queue_builder(**kwargs: Any) -> Tuple[TestQueueBuilder, Mapping[str, Any]]:
    builder_kwargs = {"processes": kwargs["processes"],
                      "logger": kwargs["logger"]}
    chunker_kwargs = {}
    builder_cls: Type[TestQueueBuilder]
    if kwargs["fully_parallel"]:
        builder_cls = FullyParallelGroupedSource
    elif kwargs["run_by_dir"] is not False:
        # A value of None indicates infinite depth
        builder_cls = PathGroupedSource
        builder_kwargs["depth"] = kwargs["run_by_dir"]
        chunker_kwargs["depth"] = kwargs["run_by_dir"]
    elif kwargs["test_groups"]:
        builder_cls = GroupFileTestSource
        builder_kwargs["test_groups"] = kwargs["test_groups"]
    else:
        builder_cls = SingleTestSource
    return builder_cls(**builder_kwargs), chunker_kwargs


TestGroup = namedtuple("TestGroup", ["group", "subsuite", "test_type", "metadata"])
GroupMetadata = Mapping[str, Any]


class TestQueueBuilder:
    __metaclass__ = ABCMeta

    def __init__(self, **kwargs: Any):
        """Class for building a queue of groups of tests to run.

        Each item in the queue is a TestGroup, which consists of an iterable of
        tests to run, the name of the subsuite, the name of the test type, and
        a dictionary containing group-specific metadata.

        Tests in the same group are run in the same TestRunner in the
        provided order."""
        self.kwargs = kwargs

    def make_queue(self, tests_by_type: TestsByType) -> Tuple[ReadQueue, int]:
        test_queue = WriteQueue()
        groups = self.make_groups(tests_by_type)
        processes = self.process_count(self.kwargs["processes"], len(groups))
        if processes > 1:
            groups.sort(key=lambda group: (
                # Place groups of the same subsuite, test type together to
                # minimize browser restarts.
                group.subsuite,
                group.test_type,
                # Next, run larger groups first to avoid straggler runners. Use
                # timeout to give slow tests greater relative weight.
                sum(test.timeout for test in group.group),
            ), reverse=True)
        for item in groups:
            test_queue.put(item)

        return test_queue.to_read(), processes

    @abstractmethod
    def make_groups(self, tests_by_type: TestsByType) -> List[TestGroup]:
        """Divide a given set of tests into groups that will be run together."""
        pass

    @abstractmethod
    def tests_by_group(self, tests_by_type: TestsByType) -> Mapping[str, List[str]]:
        pass

    def group_metadata(self, state: Mapping[str, Any]) -> GroupMetadata:
        return {"scope": "/"}

    def process_count(self, requested_processes: int, num_test_groups: int) -> int:
        """Get the number of processes to use.

        This must always be at least one, but otherwise not more than the number of test groups"""
        return max(1, min(requested_processes, num_test_groups))


class SingleTestSource(TestQueueBuilder):
    def make_groups(self, tests_by_type: TestsByType) -> List[TestGroup]:
        groups = []
        for (subsuite, test_type), tests in tests_by_type.items():
            processes = self.kwargs["processes"]
            queues: List[Deque[TestGroup]] = [deque([]) for _ in range(processes)]
            metadatas = [self.group_metadata({}) for _ in range(processes)]
            for test in tests:
                idx = hash(test.id) % processes
                group = queues[idx]
                metadata = metadatas[idx]
                group.append(test)
                test.update_metadata(metadata)

            for item in zip(queues,
                            itertools.repeat(subsuite),
                            itertools.repeat(test_type),
                            metadatas):
                if len(item[0]) > 0:
                    groups.append(TestGroup(*item))
        return groups

    def tests_by_group(self, tests_by_type: TestsByType) -> Mapping[str, List[str]]:
        groups: MutableMapping[str, List[str]] = defaultdict(list)
        for (subsuite, test_type), tests in tests_by_type.items():
            group_name = f"{subsuite}:{self.group_metadata({})['scope']}"
            groups[group_name].extend(test.id for test in tests)
        return groups


class PathGroupedSource(TestQueueBuilder):
    def new_group(self,
                  state: MutableMapping[str, Any],
                  subsuite: str,
                  test_type: str,
                  test: wpttest.Test) -> bool:
        depth = self.kwargs.get("depth")
        if depth is True or depth == 0:
            depth = None
        path = urlsplit(test.url).path.split("/")[1:-1][:depth]
        rv = (subsuite, test_type, path) != state.get("prev_group_key")
        state["prev_group_key"] = (subsuite, test_type, path)
        return rv

    def in_one_group(self,
                     subsuite: str,
                     tests: List[wpttest.Test]) -> bool:
        small_subsuite_size = self.kwargs.get("small_subsuite_size", 0)
        return len(subsuite) > 0 and len(tests) <= small_subsuite_size

    def make_groups(self, tests_by_type: TestsByType) -> List[TestGroup]:
        groups = []
        state: MutableMapping[str, Any] = {}
        for (subsuite, test_type), tests in tests_by_type.items():
            # For subsuites only have few tests, put them in one group so that
            # it will be picked up by one worker and only one restart will be
            # needed. Tests have a different test type will still be put into
            # different group though.
            in_one_group = self.in_one_group(subsuite, tests)
            if in_one_group:
                state["prev_group_key"] = (subsuite, test_type, [""])
                group_metadata = self.group_metadata(state)
                groups.append(TestGroup(deque(), subsuite, test_type, group_metadata))

            for test in tests:
                if not in_one_group and self.new_group(state, subsuite, test_type, test):
                    group_metadata = self.group_metadata(state)
                    groups.append(TestGroup(deque(), subsuite, test_type, group_metadata))
                group, _, _, metadata = groups[-1]
                group.append(test)
                test.update_metadata(metadata)
        return groups

    def tests_by_group(self, tests_by_type: TestsByType) -> Mapping[str, List[str]]:
        groups = defaultdict(list)
        state: MutableMapping[str, Any] = {}
        for (subsuite, test_type), tests in tests_by_type.items():
            in_one_group = self.in_one_group(subsuite, tests)
            for test in tests:
                if not in_one_group and self.new_group(state, subsuite, test_type, test):
                    group = self.group_metadata(state)['scope']
                if in_one_group:
                    group_name = f"{subsuite}:/"
                elif subsuite:
                    group_name = f"{subsuite}:{group}"
                else:
                    group_name = group
                groups[group_name].append(test.id)
        return groups

    def group_metadata(self, state: Mapping[str, Any]) -> GroupMetadata:
        return {"scope": "/%s" % "/".join(state["prev_group_key"][2])}


class FullyParallelGroupedSource(PathGroupedSource):
    def in_one_group(self,
                     subsuite: str,
                     tests: List[wpttest.Test]) -> bool:
        return False

    # Chuck every test into a different group, so that they can run
    # fully parallel with each other. Useful to run a lot of tests
    # clustered in a few directories.
    def new_group(self,
                  state: MutableMapping[str, Any],
                  subsuite: str,
                  test_type: str,
                  test: wpttest.Test) -> bool:
        path = urlsplit(test.url).path.split("/")[1:-1]
        state["prev_group_key"] = (subsuite, test_type, path)
        return True


class GroupFileTestSource(TestQueueBuilder):
    def make_groups(self, tests_by_type: TestsByType) -> List[TestGroup]:
        groups = []
        for (subsuite, test_type), tests in tests_by_type.items():
            tests_by_group = self.tests_by_group({(subsuite, test_type): tests})
            ids_to_tests = {test.id: test for test in tests}
            for group_name, test_ids in tests_by_group.items():
                group_metadata = {"scope": group_name}
                group: Deque[wpttest.Test] = deque()
                for test_id in test_ids:
                    test = ids_to_tests[test_id]
                    group.append(test)
                    test.update_metadata(group_metadata)
                groups.append(TestGroup(group, subsuite, test_type, group_metadata))
        return groups

    def tests_by_group(self, tests_by_type: TestsByType) -> Mapping[str, List[str]]:
        test_groups = self.kwargs["test_groups"]

        tests_by_group = defaultdict(list)
        for (subsuite, test_type), tests in tests_by_type.items():
            for test in tests:
                try:
                    group = test_groups.group_by_test[(subsuite, test.id)]
                except KeyError:
                    print(f"{test.id} is missing from test groups file")
                    raise
                if subsuite:
                    group_name = f"{subsuite}:{group}"
                else:
                    group_name = group
                tests_by_group[group_name].append(test.id)

        return tests_by_group
