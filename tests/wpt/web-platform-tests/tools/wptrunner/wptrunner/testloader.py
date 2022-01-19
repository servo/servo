import hashlib
import json
import os
from urllib.parse import urlsplit
from abc import ABCMeta, abstractmethod
from queue import Empty
from collections import defaultdict, deque

from . import manifestinclude
from . import manifestexpected
from . import mpcontext
from . import wpttest
from mozlog import structured

manifest = None
manifest_update = None
download_from_github = None


def do_delayed_imports():
    # This relies on an already loaded module having set the sys.path correctly :(
    global manifest, manifest_update, download_from_github
    from manifest import manifest  # type: ignore
    from manifest import update as manifest_update
    from manifest.download import download_from_github  # type: ignore


class TestGroupsFile(object):
    """
    Mapping object representing {group name: [test ids]}
    """

    def __init__(self, logger, path):
        try:
            with open(path) as f:
                self._data = json.load(f)
        except ValueError:
            logger.critical("test groups file %s not valid json" % path)
            raise

        self.group_by_test = {}
        for group, test_ids in self._data.items():
            for test_id in test_ids:
                self.group_by_test[test_id] = group

    def __contains__(self, key):
        return key in self._data

    def __getitem__(self, key):
        return self._data[key]

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
    if include is None:
        # We're just running everything
        return

    new_include = []
    for item in include:
        if item in test_groups:
            new_include.extend(test_groups[item])
        else:
            new_include.append(item)
    return new_include


class TestChunker(object):
    def __init__(self, total_chunks, chunk_number, **kwargs):
        self.total_chunks = total_chunks
        self.chunk_number = chunk_number
        assert self.chunk_number <= self.total_chunks
        self.logger = structured.get_default_logger()
        assert self.logger
        self.kwargs = kwargs

    def __call__(self, manifest):
        raise NotImplementedError


class Unchunked(TestChunker):
    def __init__(self, *args, **kwargs):
        TestChunker.__init__(self, *args, **kwargs)
        assert self.total_chunks == 1

    def __call__(self, manifest, **kwargs):
        for item in manifest:
            yield item


class HashChunker(TestChunker):
    def __call__(self, manifest):
        chunk_index = self.chunk_number - 1
        for test_type, test_path, tests in manifest:
            h = int(hashlib.md5(test_path.encode()).hexdigest(), 16)
            if h % self.total_chunks == chunk_index:
                yield test_type, test_path, tests


class DirectoryHashChunker(TestChunker):
    """Like HashChunker except the directory is hashed.

    This ensures that all tests in the same directory end up in the same
    chunk.
    """
    def __call__(self, manifest):
        chunk_index = self.chunk_number - 1
        depth = self.kwargs.get("depth")
        for test_type, test_path, tests in manifest:
            if depth:
                hash_path = os.path.sep.join(os.path.dirname(test_path).split(os.path.sep, depth)[:depth])
            else:
                hash_path = os.path.dirname(test_path)
            h = int(hashlib.md5(hash_path.encode()).hexdigest(), 16)
            if h % self.total_chunks == chunk_index:
                yield test_type, test_path, tests


class TestFilter(object):
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


class TagFilter(object):
    def __init__(self, tags):
        self.tags = set(tags)

    def __call__(self, test_iter):
        for test in test_iter:
            if test.tags & self.tags:
                yield test


class ManifestLoader(object):
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
        for url_base, paths in self.test_paths.items():
            manifest_file = self.load_manifest(url_base=url_base,
                                               **paths)
            path_data = {"url_base": url_base}
            path_data.update(paths)
            rv[manifest_file] = path_data
        return rv

    def load_manifest(self, tests_path, manifest_path, metadata_path, url_base="/", **kwargs):
        cache_root = os.path.join(metadata_path, ".cache")
        if self.manifest_download:
            download_from_github(manifest_path, tests_path)
        return manifest.load_and_update(tests_path, manifest_path, url_base,
                                        cache_root=cache_root, update=self.force_manifest_update,
                                        types=self.types)


def iterfilter(filters, iter):
    for f in filters:
        iter = f(iter)
    for item in iter:
        yield item


class TestLoader(object):
    """Loads tests according to a WPT manifest and any associated expectation files"""
    def __init__(self,
                 test_manifests,
                 test_types,
                 run_info,
                 manifest_filters=None,
                 chunk_type="none",
                 total_chunks=1,
                 chunk_number=1,
                 include_https=True,
                 include_h2=True,
                 include_webtransport_h3=False,
                 skip_timeout=False,
                 skip_implementation_status=None,
                 chunker_kwargs=None):

        self.test_types = test_types
        self.run_info = run_info

        self.manifest_filters = manifest_filters if manifest_filters is not None else []

        self.manifests = test_manifests
        self.tests = None
        self.disabled_tests = None
        self.include_https = include_https
        self.include_h2 = include_h2
        self.include_webtransport_h3 = include_webtransport_h3
        self.skip_timeout = skip_timeout
        self.skip_implementation_status = skip_implementation_status

        self.chunk_type = chunk_type
        self.total_chunks = total_chunks
        self.chunk_number = chunk_number

        if chunker_kwargs is None:
            chunker_kwargs = {}
        self.chunker = {"none": Unchunked,
                        "hash": HashChunker,
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
                for test_type in self.test_types:
                    self._test_ids += [item.id for item in test_dict[test_type]]
        return self._test_ids

    def get_test(self, manifest_file, manifest_test, inherit_metadata, test_metadata):
        if test_metadata is not None:
            inherit_metadata.append(test_metadata)
            test_metadata = test_metadata.get_test(manifest_test.id)

        return wpttest.from_manifest(manifest_file, manifest_test, inherit_metadata, test_metadata)

    def load_dir_metadata(self, test_manifest, metadata_path, test_path):
        rv = []
        path_parts = os.path.dirname(test_path).split(os.path.sep)
        for i in range(len(path_parts) + 1):
            path = os.path.join(metadata_path, os.path.sep.join(path_parts[:i]), "__dir__.ini")
            if path not in self.directory_manifests:
                self.directory_manifests[path] = manifestexpected.get_dir_manifest(path,
                                                                                   self.run_info)
            manifest = self.directory_manifests[path]
            if manifest is not None:
                rv.append(manifest)
        return rv

    def load_metadata(self, test_manifest, metadata_path, test_path):
        inherit_metadata = self.load_dir_metadata(test_manifest, metadata_path, test_path)
        test_metadata = manifestexpected.get_manifest(
            metadata_path, test_path, test_manifest.url_base, self.run_info)
        return inherit_metadata, test_metadata

    def iter_tests(self):
        manifest_items = []
        manifests_by_url_base = {}

        for manifest in sorted(self.manifests.keys(), key=lambda x:x.url_base):
            manifest_iter = iterfilter(self.manifest_filters,
                                       manifest.itertypes(*self.test_types))
            manifest_items.extend(manifest_iter)
            manifests_by_url_base[manifest.url_base] = manifest

        if self.chunker is not None:
            manifest_items = self.chunker(manifest_items)

        for test_type, test_path, tests in manifest_items:
            manifest_file = manifests_by_url_base[next(iter(tests)).url_base]
            metadata_path = self.manifests[manifest_file]["metadata_path"]

            inherit_metadata, test_metadata = self.load_metadata(manifest_file, metadata_path, test_path)
            for test in tests:
                yield test_path, test_type, self.get_test(manifest_file, test, inherit_metadata, test_metadata)

    def _load_tests(self):
        """Read in the tests from the manifest file and add them to a queue"""
        tests = {"enabled":defaultdict(list),
                 "disabled":defaultdict(list)}

        for test_path, test_type, test in self.iter_tests():
            enabled = not test.disabled()
            if not self.include_https and test.environment["protocol"] == "https":
                enabled = False
            if not self.include_h2 and test.environment["protocol"] == "h2":
                enabled = False
            if self.skip_timeout and test.expected() == "TIMEOUT":
                enabled = False
            if self.skip_implementation_status and test.implementation_status() in self.skip_implementation_status:
                enabled = False
            key = "enabled" if enabled else "disabled"
            tests[key][test_type].append(test)

        self.tests = tests["enabled"]
        self.disabled_tests = tests["disabled"]

    def groups(self, test_types, chunk_type="none", total_chunks=1, chunk_number=1):
        groups = set()

        for test_type in test_types:
            for test in self.tests[test_type]:
                group = test.url.split("/")[1]
                groups.add(group)

        return groups


def get_test_src(**kwargs):
    test_source_kwargs = {"processes": kwargs["processes"],
                          "logger": kwargs["logger"]}
    chunker_kwargs = {}
    if kwargs["run_by_dir"] is not False:
        # A value of None indicates infinite depth
        test_source_cls = PathGroupedSource
        test_source_kwargs["depth"] = kwargs["run_by_dir"]
        chunker_kwargs["depth"] = kwargs["run_by_dir"]
    elif kwargs["test_groups"]:
        test_source_cls = GroupFileTestSource
        test_source_kwargs["test_groups"] = kwargs["test_groups"]
    else:
        test_source_cls = SingleTestSource
    return test_source_cls, test_source_kwargs, chunker_kwargs


class TestSource(object):
    __metaclass__ = ABCMeta

    def __init__(self, test_queue):
        self.test_queue = test_queue
        self.current_group = None
        self.current_metadata = None

    @abstractmethod
    #@classmethod (doesn't compose with @abstractmethod in < 3.3)
    def make_queue(cls, tests, **kwargs):  # noqa: N805
        pass

    @abstractmethod
    def tests_by_group(cls, tests, **kwargs):  # noqa: N805
        pass

    @classmethod
    def group_metadata(cls, state):
        return {"scope": "/"}

    def group(self):
        if not self.current_group or len(self.current_group) == 0:
            try:
                self.current_group, self.current_metadata = self.test_queue.get(block=False)
            except Empty:
                return None, None
        return self.current_group, self.current_metadata


class GroupedSource(TestSource):
    @classmethod
    def new_group(cls, state, test, **kwargs):
        raise NotImplementedError

    @classmethod
    def make_queue(cls, tests, **kwargs):
        mp = mpcontext.get_context()
        test_queue = mp.Queue()
        groups = []

        state = {}

        for test in tests:
            if cls.new_group(state, test, **kwargs):
                group_metadata = cls.group_metadata(state)
                groups.append((deque(), group_metadata))

            group, metadata = groups[-1]
            group.append(test)
            test.update_metadata(metadata)

        for item in groups:
            test_queue.put(item)
        return test_queue

    @classmethod
    def tests_by_group(cls, tests, **kwargs):
        groups = defaultdict(list)
        state = {}
        current = None
        for test in tests:
            if cls.new_group(state, test, **kwargs):
                current = cls.group_metadata(state)['scope']
            groups[current].append(test.id)
        return groups


class SingleTestSource(TestSource):
    @classmethod
    def make_queue(cls, tests, **kwargs):
        mp = mpcontext.get_context()
        test_queue = mp.Queue()
        processes = kwargs["processes"]
        queues = [deque([]) for _ in range(processes)]
        metadatas = [cls.group_metadata(None) for _ in range(processes)]
        for test in tests:
            idx = hash(test.id) % processes
            group = queues[idx]
            metadata = metadatas[idx]
            group.append(test)
            test.update_metadata(metadata)

        for item in zip(queues, metadatas):
            test_queue.put(item)

        return test_queue

    @classmethod
    def tests_by_group(cls, tests, **kwargs):
        return {cls.group_metadata(None)['scope']: [t.id for t in tests]}


class PathGroupedSource(GroupedSource):
    @classmethod
    def new_group(cls, state, test, **kwargs):
        depth = kwargs.get("depth")
        if depth is True or depth == 0:
            depth = None
        path = urlsplit(test.url).path.split("/")[1:-1][:depth]
        rv = path != state.get("prev_path")
        state["prev_path"] = path
        return rv

    @classmethod
    def group_metadata(cls, state):
        return {"scope": "/%s" % "/".join(state["prev_path"])}


class GroupFileTestSource(TestSource):
    @classmethod
    def make_queue(cls, tests, **kwargs):
        tests_by_group = cls.tests_by_group(tests, **kwargs)

        ids_to_tests = {test.id: test for test in tests}

        mp = mpcontext.get_context()
        test_queue = mp.Queue()

        for group_name, test_ids in tests_by_group.items():
            group_metadata = {"scope": group_name}
            group = deque()

            for test_id in test_ids:
                test = ids_to_tests[test_id]
                group.append(test)
                test.update_metadata(group_metadata)

            test_queue.put((group, group_metadata))

        return test_queue

    @classmethod
    def tests_by_group(cls, tests, **kwargs):
        logger = kwargs["logger"]
        test_groups = kwargs["test_groups"]

        tests_by_group = defaultdict(list)
        for test in tests:
            try:
                group = test_groups.group_by_test[test.id]
            except KeyError:
                logger.error("%s is missing from test groups file" % test.id)
                raise
            tests_by_group[group].append(test.id)

        return tests_by_group
