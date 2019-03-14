import hashlib
import os
import urlparse
from abc import ABCMeta, abstractmethod
from Queue import Empty
from collections import defaultdict, deque
from multiprocessing import Queue

import manifestinclude
import manifestexpected
import wpttest
from mozlog import structured

manifest = None
manifest_update = None
download_from_github = None

def do_delayed_imports():
    # This relies on an already loaded module having set the sys.path correctly :(
    global manifest, manifest_update, download_from_github
    from manifest import manifest
    from manifest import update as manifest_update
    from manifest.download import download_from_github


class TestChunker(object):
    def __init__(self, total_chunks, chunk_number):
        self.total_chunks = total_chunks
        self.chunk_number = chunk_number
        assert self.chunk_number <= self.total_chunks
        self.logger = structured.get_default_logger()
        assert self.logger

    def __call__(self, manifest):
        raise NotImplementedError


class Unchunked(TestChunker):
    def __init__(self, *args, **kwargs):
        TestChunker.__init__(self, *args, **kwargs)
        assert self.total_chunks == 1

    def __call__(self, manifest):
        for item in manifest:
            yield item


class HashChunker(TestChunker):
    def __call__(self, manifest):
        chunk_index = self.chunk_number - 1
        for test_type, test_path, tests in manifest:
            h = int(hashlib.md5(test_path).hexdigest(), 16)
            if h % self.total_chunks == chunk_index:
                yield test_type, test_path, tests


class DirectoryHashChunker(TestChunker):
    """Like HashChunker except the directory is hashed.

    This ensures that all tests in the same directory end up in the same
    chunk.
    """
    def __call__(self, manifest):
        chunk_index = self.chunk_number - 1
        for test_type, test_path, tests in manifest:
            h = int(hashlib.md5(os.path.dirname(test_path)).hexdigest(), 16)
            if h % self.total_chunks == chunk_index:
                yield test_type, test_path, tests


class TestFilter(object):
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
                 types=None, meta_filters=None):
        do_delayed_imports()
        self.test_paths = test_paths
        self.force_manifest_update = force_manifest_update
        self.manifest_download = manifest_download
        self.types = types
        self.logger = structured.get_default_logger()
        self.meta_filters = meta_filters
        if self.logger is None:
            self.logger = structured.structuredlog.StructuredLogger("ManifestLoader")

    def load(self):
        rv = {}
        for url_base, paths in self.test_paths.iteritems():
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
                                        meta_filters=self.meta_filters, types=self.types)


def iterfilter(filters, iter):
    for f in filters:
        iter = f(iter)
    for item in iter:
        yield item


class TestLoader(object):
    def __init__(self,
                 test_manifests,
                 test_types,
                 run_info,
                 manifest_filters=None,
                 chunk_type="none",
                 total_chunks=1,
                 chunk_number=1,
                 include_https=True,
                 skip_timeout=False):

        self.test_types = test_types
        self.run_info = run_info

        self.manifest_filters = manifest_filters if manifest_filters is not None else []

        self.manifests = test_manifests
        self.tests = None
        self.disabled_tests = None
        self.include_https = include_https
        self.skip_timeout = skip_timeout

        self.chunk_type = chunk_type
        self.total_chunks = total_chunks
        self.chunk_number = chunk_number

        self.chunker = {"none": Unchunked,
                        "hash": HashChunker,
                        "dir_hash": DirectoryHashChunker}[chunk_type](total_chunks,
                                                                      chunk_number)

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
        for i in xrange(len(path_parts) + 1):
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
            manifest_file = manifests_by_url_base[iter(tests).next().url_base]
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
            if self.skip_timeout and test.expected() == "TIMEOUT":
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


class TestSource(object):
    __metaclass__ = ABCMeta

    def __init__(self, test_queue):
        self.test_queue = test_queue
        self.current_group = None
        self.current_metadata = None

    @abstractmethod
    # noqa: N805
    #@classmethod (doesn't compose with @abstractmethod)
    def make_queue(cls, tests, **kwargs):
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
        test_queue = Queue()
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


class SingleTestSource(TestSource):
    @classmethod
    def make_queue(cls, tests, **kwargs):
        test_queue = Queue()
        processes = kwargs["processes"]
        queues = [deque([]) for _ in xrange(processes)]
        metadatas = [cls.group_metadata(None) for _ in xrange(processes)]
        for test in tests:
            idx = hash(test.id) % processes
            group = queues[idx]
            metadata = metadatas[idx]
            group.append(test)
            test.update_metadata(metadata)

        for item in zip(queues, metadatas):
            test_queue.put(item)

        return test_queue


class PathGroupedSource(GroupedSource):
    @classmethod
    def new_group(cls, state, test, **kwargs):
        depth = kwargs.get("depth")
        if depth is True or depth == 0:
            depth = None
        path = urlparse.urlsplit(test.url).path.split("/")[1:-1][:depth]
        rv = path != state.get("prev_path")
        state["prev_path"] = path
        return rv

    @classmethod
    def group_metadata(cls, state):
        return {"scope": "/%s" % "/".join(state["prev_path"])}
