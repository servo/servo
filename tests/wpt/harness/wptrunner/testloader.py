import json
import os
import sys
import urlparse
from abc import ABCMeta, abstractmethod
from Queue import Empty
from collections import defaultdict, OrderedDict, deque
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
        self.logger = structured.get_default_logger()

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
    def _group_by_directory(self, manifest_items):
        """Split the list of manifest items into a ordered dict that groups tests in
        so that anything in the same subdirectory beyond a depth of 3 is in the same
        group. So all tests in a/b/c, a/b/c/d and a/b/c/e will be grouped together
        and separate to tests in a/b/f

        Returns: tuple (ordered dict of {test_dir: PathData}, total estimated runtime)
        """

        class PathData(object):
            def __init__(self, path):
                self.path = path
                self.time = 0
                self.tests = []

        by_dir = OrderedDict()
        total_time = 0

        for i, (test_path, tests) in enumerate(manifest_items):
            test_dir = tuple(os.path.split(test_path)[0].split(os.path.sep)[:3])

            if not test_dir in by_dir:
                by_dir[test_dir] = PathData(test_dir)

            data = by_dir[test_dir]
            time = sum(wpttest.DEFAULT_TIMEOUT if test.timeout !=
                       "long" else wpttest.LONG_TIMEOUT for test in tests)
            data.time += time
            total_time += time
            data.tests.append((test_path, tests))

        return by_dir, total_time

    def _maybe_remove(self, chunks, i, direction):
        """Trial removing a chunk from one chunk to an adjacent one.

        :param chunks: - the list of all chunks
        :param i: - the chunk index in the list of chunks to try removing from
        :param direction: either "next" if we are going to move from the end to
                          the subsequent chunk, or "prev" if we are going to move
                          from the start into the previous chunk.

        :returns bool: Did a chunk get moved?"""
        source_chunk = chunks[i]
        if direction == "next":
            target_chunk = chunks[i+1]
            path_index = -1
            move_func = lambda: target_chunk.appendleft(source_chunk.pop())
        elif direction == "prev":
            target_chunk = chunks[i-1]
            path_index = 0
            move_func = lambda: target_chunk.append(source_chunk.popleft())
        else:
            raise ValueError("Unexpected move direction %s" % direction)

        return self._maybe_move(source_chunk, target_chunk, path_index, move_func)

    def _maybe_add(self, chunks, i, direction):
        """Trial adding a chunk from one chunk to an adjacent one.

        :param chunks: - the list of all chunks
        :param i: - the chunk index in the list of chunks to try adding to
        :param direction: either "next" if we are going to remove from the
                          the subsequent chunk, or "prev" if we are going to remove
                          from the the previous chunk.

        :returns bool: Did a chunk get moved?"""
        target_chunk = chunks[i]
        if direction == "next":
            source_chunk = chunks[i+1]
            path_index = 0
            move_func = lambda: target_chunk.append(source_chunk.popleft())
        elif direction == "prev":
            source_chunk = chunks[i-1]
            path_index = -1
            move_func = lambda: target_chunk.appendleft(source_chunk.pop())
        else:
            raise ValueError("Unexpected move direction %s" % direction)

        return self._maybe_move(source_chunk, target_chunk, path_index, move_func)

    def _maybe_move(self, source_chunk, target_chunk, path_index, move_func):
        """Move from one chunk to another, assess the change in badness,
        and keep the move iff it decreases the badness score.

        :param source_chunk: chunk to move from
        :param target_chunk: chunk to move to
        :param path_index: 0 if we are moving from the start or -1 if we are moving from the
                           end
        :param move_func: Function that actually moves between chunks"""
        if len(source_chunk.paths) <= 1:
            return False

        move_time = source_chunk.paths[path_index].time

        new_source_badness = self._badness(source_chunk.time - move_time)
        new_target_badness = self._badness(target_chunk.time + move_time)

        delta_badness = ((new_source_badness + new_target_badness) -
                         (source_chunk.badness + target_chunk.badness))
        if delta_badness < 0:
            move_func()
            return True

        return False

    def _badness(self, time):
        """Metric of badness for a specific chunk

        :param time: the time for a specific chunk"""
        return (time - self.expected_time)**2

    def _get_chunk(self, manifest_items):
        by_dir, total_time = self._group_by_directory(manifest_items)

        if len(by_dir) < self.total_chunks:
            raise ValueError("Tried to split into %i chunks, but only %i subdirectories included" % (
                self.total_chunks, len(by_dir)))

        self.expected_time = float(total_time) / self.total_chunks

        chunks = self._create_initial_chunks(by_dir)

        while True:
            # Move a test from one chunk to the next until doing so no longer
            # reduces the badness
            got_improvement = self._update_chunks(chunks)
            if not got_improvement:
                break

        self.logger.debug(self.expected_time)
        for i, chunk in chunks.iteritems():
            self.logger.debug("%i: %i, %i" % (i + 1, chunk.time, chunk.badness))

        assert self._all_tests(by_dir) == self._chunked_tests(chunks)

        return self._get_tests(chunks)

    @staticmethod
    def _all_tests(by_dir):
        """Return a set of all tests in the manifest from a grouping by directory"""
        return set(x[0] for item in by_dir.itervalues()
                   for x in item.tests)

    @staticmethod
    def _chunked_tests(chunks):
        """Return a set of all tests in the manifest from the chunk list"""
        return set(x[0] for chunk in chunks.itervalues()
                   for path in chunk.paths
                   for x in path.tests)


    def _create_initial_chunks(self, by_dir):
        """Create an initial unbalanced list of chunks.

        :param by_dir: All tests in the manifest grouped by subdirectory
        :returns list: A list of Chunk objects"""

        class Chunk(object):
            def __init__(self, paths, index):
                """List of PathData objects that together form a single chunk of
                tests"""
                self.paths = deque(paths)
                self.time = sum(item.time for item in paths)
                self.index = index

            def appendleft(self, path):
                """Add a PathData object to the start of the chunk"""
                self.paths.appendleft(path)
                self.time += path.time

            def append(self, path):
                """Add a PathData object to the end of the chunk"""
                self.paths.append(path)
                self.time += path.time

            def pop(self):
                """Remove PathData object from the end of the chunk"""
                assert len(self.paths) > 1
                self.time -= self.paths[-1].time
                return self.paths.pop()

            def popleft(self):
                """Remove PathData object from the start of the chunk"""
                assert len(self.paths) > 1
                self.time -= self.paths[0].time
                return self.paths.popleft()

            @property
            def badness(self_):
                """Badness metric for this chunk"""
                return self._badness(self_.time)

        initial_size = len(by_dir) / self.total_chunks
        chunk_boundaries = [initial_size * i
                            for i in xrange(self.total_chunks)] + [len(by_dir)]

        chunks = OrderedDict()
        for i, lower in enumerate(chunk_boundaries[:-1]):
            upper = chunk_boundaries[i + 1]
            paths = by_dir.values()[lower:upper]
            chunks[i] = Chunk(paths, i)

        assert self._all_tests(by_dir) == self._chunked_tests(chunks)

        return chunks

    def _update_chunks(self, chunks):
        """Run a single iteration of the chunk update algorithm.

        :param chunks: - List of chunks
        """
        #TODO: consider replacing this with a heap
        sorted_chunks = sorted(chunks.values(), key=lambda x:-x.badness)
        got_improvement = False
        for chunk in sorted_chunks:
            if chunk.time < self.expected_time:
                f = self._maybe_add
            else:
                f = self._maybe_remove

            if chunk.index == 0:
                order = ["next"]
            elif chunk.index == self.total_chunks - 1:
                order = ["prev"]
            else:
                if chunk.time < self.expected_time:
                    # First try to add a test from the neighboring chunk with the
                    # greatest total time
                    if chunks[chunk.index + 1].time > chunks[chunk.index - 1].time:
                        order = ["next", "prev"]
                    else:
                        order = ["prev", "next"]
                else:
                    # First try to remove a test and add to the neighboring chunk with the
                    # lowest total time
                    if chunks[chunk.index + 1].time > chunks[chunk.index - 1].time:
                        order = ["prev", "next"]
                    else:
                        order = ["next", "prev"]

            for direction in order:
                if f(chunks, chunk.index, direction):
                    got_improvement = True
                    break

            if got_improvement:
                break

        return got_improvement

    def _get_tests(self, chunks):
        """Return the list of tests corresponding to the chunk number we are running.

        :param chunks: List of chunks"""
        tests = []
        for path in chunks[self.chunk_number - 1].paths:
            tests.extend(path.tests)

        return tests

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

class TagFilter(object):
    def __init__(self, tags):
        self.tags = set(tags)

    def __call__(self, test_iter):
        for test in test_iter:
            if test.tags & self.tags:
                yield test

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
                 meta_filters=None,
                 chunk_type="none",
                 total_chunks=1,
                 chunk_number=1,
                 include_https=True):

        self.test_types = test_types
        self.run_info = run_info

        self.manifest_filters = manifest_filters if manifest_filters is not None else []
        self.meta_filters = meta_filters if meta_filters is not None else []

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

    def get_test(self, manifest_test, inherit_metadata, test_metadata):
        if test_metadata is not None:
            inherit_metadata.append(test_metadata)
            test_metadata = test_metadata.get_test(manifest_test.id)

        return wpttest.from_manifest(manifest_test, inherit_metadata, test_metadata)

    def load_dir_metadata(self, test_manifest, metadata_path, test_path):
        rv = []
        path_parts = os.path.dirname(test_path).split(os.path.sep)
        for i in xrange(1,len(path_parts) + 1):
            path = os.path.join(os.path.sep.join(path_parts[:i]), "__dir__.ini")
            if path not in self.directory_manifests:
                self.directory_manifests[path] = manifestexpected.get_dir_manifest(
                    metadata_path, path, self.run_info)
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

        for manifest in sorted(self.manifests.keys()):
            manifest_iter = iterfilter(self.manifest_filters,
                                       manifest.itertypes(*self.test_types))
            manifest_items.extend(manifest_iter)

        if self.chunker is not None:
            manifest_items = self.chunker(manifest_items)

        for test_path, tests in manifest_items:
            manifest_file = iter(tests).next().manifest
            metadata_path = self.manifests[manifest_file]["metadata_path"]
            inherit_metadata, test_metadata = self.load_metadata(manifest_file, metadata_path, test_path)

            for test in iterfilter(self.meta_filters,
                                   self.iter_wpttest(inherit_metadata, test_metadata, tests)):
                yield test_path, test.test_type, test

    def iter_wpttest(self, inherit_metadata, test_metadata, tests):
        for manifest_test in tests:
            yield self.get_test(manifest_test, inherit_metadata, test_metadata)

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
