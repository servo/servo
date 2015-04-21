import json
import os
import urlparse
from abc import ABCMeta, abstractmethod
from Queue import Empty
from collections import defaultdict, OrderedDict
from multiprocessing import Queue

import manifestinclude
import manifestexpected
import wpttest
from mozlog import structured

manifest = None
manifest_update = None

def do_delayed_imports():
    # This relies on an already loaded module having set the sys.path correctly :(
    global manifest, manifest_update
    from manifest import manifest
    from manifest import update as manifest_update

class TestChunker(object):
    def __init__(self, total_chunks, chunk_number):
        self.total_chunks = total_chunks
        self.chunk_number = chunk_number
        assert self.chunk_number <= self.total_chunks

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
    def __call__(self):
        chunk_index = self.chunk_number - 1
        for test_path, tests in manifest:
            if hash(test_path) % self.total_chunks == chunk_index:
                yield test_path, tests


class EqualTimeChunker(TestChunker):
    """Chunker that uses the test timeout as a proxy for the running time of the test"""

    def _get_chunk(self, manifest_items):
        # For each directory containing tests, calculate the maximum execution time after running all
        # the tests in that directory. Then work out the index into the manifest corresponding to the
        # directories at fractions of m/N of the running time where m=1..N-1 and N is the total number
        # of chunks. Return an array of these indicies

        total_time = 0
        by_dir = OrderedDict()

        class PathData(object):
            def __init__(self, path):
                self.path = path
                self.time = 0
                self.tests = []

        class Chunk(object):
            def __init__(self):
                self.paths = []
                self.tests = []
                self.time = 0

            def append(self, path_data):
                self.paths.append(path_data.path)
                self.tests.extend(path_data.tests)
                self.time += path_data.time

        class ChunkList(object):
            def __init__(self, total_time, n_chunks):
                self.total_time = total_time
                self.n_chunks = n_chunks

                self.remaining_chunks = n_chunks

                self.chunks = []

                self.update_time_per_chunk()

            def __iter__(self):
                for item in self.chunks:
                    yield item

            def __getitem__(self, i):
                return self.chunks[i]

            def sort_chunks(self):
                self.chunks = sorted(self.chunks, key=lambda x:x.paths[0])

            def get_tests(self, chunk_number):
                return self[chunk_number - 1].tests

            def append(self, chunk):
                if len(self.chunks) == self.n_chunks:
                    raise ValueError("Tried to create more than %n chunks" % self.n_chunks)
                self.chunks.append(chunk)
                self.remaining_chunks -= 1

            @property
            def current_chunk(self):
                if self.chunks:
                    return self.chunks[-1]

            def update_time_per_chunk(self):
                self.time_per_chunk = (self.total_time - sum(item.time for item in self)) / self.remaining_chunks

            def create(self):
                rv = Chunk()
                self.append(rv)
                return rv

            def add_path(self, path_data):
                sum_time = self.current_chunk.time + path_data.time
                if sum_time > self.time_per_chunk and self.remaining_chunks > 0:
                    overshoot = sum_time - self.time_per_chunk
                    undershoot = self.time_per_chunk - self.current_chunk.time
                    if overshoot < undershoot:
                        self.create()
                        self.current_chunk.append(path_data)
                    else:
                        self.current_chunk.append(path_data)
                        self.create()
                else:
                    self.current_chunk.append(path_data)

        for i, (test_path, tests) in enumerate(manifest_items):
            test_dir = tuple(os.path.split(test_path)[0].split(os.path.sep)[:3])

            if not test_dir in by_dir:
                by_dir[test_dir] = PathData(test_dir)

            data = by_dir[test_dir]
            time = sum(wpttest.DEFAULT_TIMEOUT if test.timeout !=
                       "long" else wpttest.LONG_TIMEOUT for test in tests)
            data.time += time
            data.tests.append((test_path, tests))

            total_time += time

        chunk_list = ChunkList(total_time, self.total_chunks)

        if len(by_dir) < self.total_chunks:
            raise ValueError("Tried to split into %i chunks, but only %i subdirectories included" % (
                self.total_chunks, len(by_dir)))

        # Put any individual dirs with a time greater than the time per chunk into their own
        # chunk
        while True:
            to_remove = []
            for path_data in by_dir.itervalues():
                if path_data.time > chunk_list.time_per_chunk:
                    to_remove.append(path_data)
            if to_remove:
                for path_data in to_remove:
                    chunk = chunk_list.create()
                    chunk.append(path_data)
                    del by_dir[path_data.path]
                chunk_list.update_time_per_chunk()
            else:
                break

        chunk = chunk_list.create()
        for path_data in by_dir.itervalues():
            chunk_list.add_path(path_data)

        assert len(chunk_list.chunks) == self.total_chunks, len(chunk_list.chunks)
        assert sum(item.time for item in chunk_list) == chunk_list.total_time

        chunk_list.sort_chunks()

        return chunk_list.get_tests(self.chunk_number)

    def __call__(self, manifest_iter):
        manifest = list(manifest_iter)
        tests = self._get_chunk(manifest)
        for item in tests:
            yield item


class TestFilter(object):
    def __init__(self, test_manifests, include=None, exclude=None, manifest_path=None):
        if manifest_path is not None and include is None:
            self.manifest = manifestinclude.get_manifest(manifest_path)
        else:
            self.manifest = manifestinclude.IncludeManifest.create()

        if include:
            self.manifest.set("skip", "true")
            for item in include:
                self.manifest.add_include(test_manifests, item)

        if exclude:
            for item in exclude:
                self.manifest.add_exclude(test_manifests, item)

    def __call__(self, manifest_iter):
        for test_path, tests in manifest_iter:
            include_tests = set()
            for test in tests:
                if self.manifest.include(test):
                    include_tests.add(test)

            if include_tests:
                yield test_path, include_tests


class ManifestLoader(object):
    def __init__(self, test_paths, force_manifest_update=False):
        do_delayed_imports()
        self.test_paths = test_paths
        self.force_manifest_update = force_manifest_update
        self.logger = structured.get_default_logger()
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

    def create_manifest(self, manifest_path, tests_path, url_base="/"):
        self.update_manifest(manifest_path, tests_path, url_base, recreate=True)

    def update_manifest(self, manifest_path, tests_path, url_base="/",
                        recreate=False):
        self.logger.info("Updating test manifest %s" % manifest_path)

        json_data = None
        if not recreate:
            try:
                with open(manifest_path) as f:
                    json_data = json.load(f)
            except IOError:
                #If the existing file doesn't exist just create one from scratch
                pass

        if not json_data:
            manifest_file = manifest.Manifest(None, url_base)
        else:
            try:
                manifest_file = manifest.Manifest.from_json(tests_path, json_data)
            except manifest.ManifestVersionMismatch:
                manifest_file = manifest.Manifest(None, url_base)

            manifest_update.update(tests_path, url_base, manifest_file)

        manifest.write(manifest_file, manifest_path)

    def load_manifest(self, tests_path, metadata_path, url_base="/"):
        manifest_path = os.path.join(metadata_path, "MANIFEST.json")
        if (not os.path.exists(manifest_path) or
            self.force_manifest_update):
            self.update_manifest(manifest_path, tests_path, url_base)
        manifest_file = manifest.load(tests_path, manifest_path)
        if manifest_file.url_base != url_base:
            self.logger.info("Updating url_base in manifest from %s to %s" % (manifest_file.url_base,
                                                                              url_base))
            manifest_file.url_base = url_base
            manifest.write(manifest_file, manifest_path)

        return manifest_file

class TestLoader(object):
    def __init__(self,
                 test_manifests,
                 test_types,
                 test_filter,
                 run_info,
                 chunk_type="none",
                 total_chunks=1,
                 chunk_number=1,
                 include_https=True):

        self.test_types = test_types
        self.test_filter = test_filter
        self.run_info = run_info
        self.manifests = test_manifests
        self.tests = None
        self.disabled_tests = None
        self.include_https = include_https

        self.chunk_type = chunk_type
        self.total_chunks = total_chunks
        self.chunk_number = chunk_number

        self.chunker = {"none": Unchunked,
                        "hash": HashChunker,
                        "equal_time": EqualTimeChunker}[chunk_type](total_chunks,
                                                                    chunk_number)

        self._test_ids = None
        self._load_tests()

    @property
    def test_ids(self):
        if self._test_ids is None:
            self._test_ids = []
            for test_dict in [self.disabled_tests, self.tests]:
                for test_type in self.test_types:
                    self._test_ids += [item.id for item in test_dict[test_type]]
        return self._test_ids

    def get_test(self, manifest_test, expected_file):
        if expected_file is not None:
            expected = expected_file.get_test(manifest_test.id)
        else:
            expected = None

        return wpttest.from_manifest(manifest_test, expected)

    def load_expected_manifest(self, test_manifest, metadata_path, test_path):
        return manifestexpected.get_manifest(metadata_path, test_path, test_manifest.url_base, self.run_info)

    def iter_tests(self):
        manifest_items = []

        for manifest in self.manifests.keys():
            manifest_items.extend(self.test_filter(manifest.itertypes(*self.test_types)))

        if self.chunker is not None:
            manifest_items = self.chunker(manifest_items)

        for test_path, tests in manifest_items:
            manifest_file = iter(tests).next().manifest
            metadata_path = self.manifests[manifest_file]["metadata_path"]
            expected_file = self.load_expected_manifest(manifest_file, metadata_path, test_path)

            for manifest_test in tests:
                test = self.get_test(manifest_test, expected_file)
                test_type = manifest_test.item_type
                yield test_path, test_type, test

    def _load_tests(self):
        """Read in the tests from the manifest file and add them to a queue"""
        tests = {"enabled":defaultdict(list),
                 "disabled":defaultdict(list)}

        for test_path, test_type, test in self.iter_tests():
            enabled = not test.disabled()
            if not self.include_https and test.environment["protocol"] == "https":
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

    @abstractmethod
    def queue_tests(self, test_queue):
        pass

    @abstractmethod
    def requeue_test(self, test):
        pass

    def __enter__(self):
        return self

    def __exit__(self, *args, **kwargs):
        pass


class SingleTestSource(TestSource):
    def __init__(self, test_queue):
        self.test_queue = test_queue

    @classmethod
    def queue_tests(cls, test_queue, test_type, tests):
        for test in tests[test_type]:
            test_queue.put(test)

    def get_queue(self):
        if self.test_queue.empty():
            return None
        return self.test_queue

    def requeue_test(self, test):
        self.test_queue.put(test)

class PathGroupedSource(TestSource):
    def __init__(self, test_queue):
        self.test_queue = test_queue
        self.current_queue = None

    @classmethod
    def queue_tests(cls, test_queue, test_type, tests, depth=None):
        if depth is True:
            depth = None

        prev_path = None
        group = None

        for test in tests[test_type]:
            path = urlparse.urlsplit(test.url).path.split("/")[1:-1][:depth]
            if path != prev_path:
                group = []
                test_queue.put(group)
                prev_path = path

            group.append(test)

    def get_queue(self):
        if not self.current_queue or self.current_queue.empty():
            try:
                data = self.test_queue.get(block=True, timeout=1)
                self.current_queue = Queue()
                for item in data:
                    self.current_queue.put(item)
            except Empty:
                return None

        return self.current_queue

    def requeue_test(self, test):
        self.current_queue.put(test)

    def __exit__(self, *args, **kwargs):
        if self.current_queue:
            self.current_queue.close()
