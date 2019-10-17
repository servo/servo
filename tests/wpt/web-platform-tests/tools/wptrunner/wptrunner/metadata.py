from __future__ import print_function
import array
import os
from collections import defaultdict, namedtuple

from mozlog import structuredlog
from six.moves import intern

from . import manifestupdate
from . import testloader
from . import wptmanifest
from . import wpttest
from .expected import expected_path
from .vcs import git
manifest = None  # Module that will be imported relative to test_root
manifestitem = None

logger = structuredlog.StructuredLogger("web-platform-tests")

try:
    import ujson as json
except ImportError:
    import json


class RunInfo(object):
    """A wrapper around RunInfo dicts so that they can be hashed by identity"""

    def __init__(self, dict_value):
        self.data = dict_value
        self.canonical_repr = tuple(tuple(item) for item in sorted(dict_value.items()))

    def __getitem__(self, key):
        return self.data[key]

    def __setitem__(self, key, value):
        raise TypeError

    def __hash__(self):
        return hash(self.canonical_repr)

    def __eq__(self, other):
        return self.canonical_repr == other.canonical_repr

    def iteritems(self):
        for key, value in self.data.iteritems():
            yield key, value

    def items(self):
        return list(self.iteritems())


def update_expected(test_paths, serve_root, log_file_names,
                    update_properties, rev_old=None, rev_new="HEAD",
                    full_update=False, sync_root=None, disable_intermittent=None,
                    update_intermittent=False, remove_intermittent=False):
    """Update the metadata files for web-platform-tests based on
    the results obtained in a previous run or runs

    If `disable_intermittent` is not None, assume log_file_names refers to logs from repeated
    test jobs, disable tests that don't behave as expected on all runs

    If `update_intermittent` is True, intermittent statuses will be recorded as `expected` in
    the metadata.

    If `remove_intermittent` is True and used in conjunction with `update_intermittent`, any
    intermittent statuses which are not present in the current run will be removed from the
    metadata, else they are left in."""

    do_delayed_imports(serve_root)

    id_test_map = load_test_data(test_paths)

    for metadata_path, updated_ini in update_from_logs(id_test_map,
                                                       update_properties,
                                                       disable_intermittent,
                                                       update_intermittent,
                                                       remove_intermittent,
                                                       full_update,
                                                       *log_file_names):

        write_new_expected(metadata_path, updated_ini)
        if disable_intermittent:
            for test in updated_ini.iterchildren():
                for subtest in test.iterchildren():
                    if subtest.new_disabled:
                        print("disabled: %s" % os.path.dirname(subtest.root.test_path) + "/" + subtest.name)
                    if test.new_disabled:
                        print("disabled: %s" % test.root.test_path)


def do_delayed_imports(serve_root=None):
    global manifest, manifestitem
    from manifest import manifest, item as manifestitem


def files_in_repo(repo_root):
    return git("ls-tree", "-r", "--name-only", "HEAD").split("\n")


def rev_range(rev_old, rev_new, symmetric=False):
    joiner = ".." if not symmetric else "..."
    return "".join([rev_old, joiner, rev_new])


def paths_changed(rev_old, rev_new, repo):
    data = git("diff", "--name-status", rev_range(rev_old, rev_new), repo=repo)
    lines = [tuple(item.strip() for item in line.strip().split("\t", 1))
             for line in data.split("\n") if line.strip()]
    output = set(lines)
    return output


def load_change_data(rev_old, rev_new, repo):
    changes = paths_changed(rev_old, rev_new, repo)
    rv = {}
    status_keys = {"M": "modified",
                   "A": "new",
                   "D": "deleted"}
    # TODO: deal with renames
    for item in changes:
        rv[item[1]] = status_keys[item[0]]
    return rv


def unexpected_changes(manifests, change_data, files_changed):
    files_changed = set(files_changed)

    root_manifest = None
    for manifest, paths in manifests.iteritems():
        if paths["url_base"] == "/":
            root_manifest = manifest
            break
    else:
        return []

    return [fn for _, fn, _ in root_manifest if fn in files_changed and change_data.get(fn) != "M"]

# For each testrun
# Load all files and scan for the suite_start entry
# Build a hash of filename: properties
# For each different set of properties, gather all chunks
# For each chunk in the set of chunks, go through all tests
# for each test, make a map of {conditionals: [(platform, new_value)]}
# Repeat for each platform
# For each test in the list of tests:
#   for each conditional:
#      If all the new values match (or there aren't any) retain that conditional
#      If any new values mismatch:
#           If disable_intermittent and any repeated values don't match, disable the test
#           else mark the test as needing human attention
#   Check if all the RHS values are the same; if so collapse the conditionals


class InternedData(object):
    """Class for interning data of any (hashable) type.

    This class is intended for building a mapping of int <=> value, such
    that the integer may be stored as a proxy for the real value, and then
    the real value obtained later from the proxy value.

    In order to support the use case of packing the integer value as binary,
    it is possible to specify a maximum bitsize of the data; adding more items
    than this allowed will result in a ValueError exception.

    The zero value is reserved to use as a sentinal."""

    type_conv = None
    rev_type_conv = None

    def __init__(self, max_bits=8):
        self.max_idx = 2**max_bits - 2
        # Reserve 0 as a sentinal
        self._data = [None], {}

    def clear(self):
        self.__init__()

    def store(self, obj):
        if self.type_conv is not None:
            obj = self.type_conv(obj)

        objs, obj_to_idx = self._data
        if obj not in obj_to_idx:
            value = len(objs)
            objs.append(obj)
            obj_to_idx[obj] = value
            if value > self.max_idx:
                raise ValueError
        else:
            value = obj_to_idx[obj]
        return value

    def get(self, idx):
        obj = self._data[0][idx]
        if self.rev_type_conv is not None:
            obj = self.rev_type_conv(obj)
        return obj

    def __iter__(self):
        for i in xrange(1, len(self._data[0])):
            yield self.get(i)


class RunInfoInterned(InternedData):
    def type_conv(self, value):
        return tuple(value.items())

    def rev_type_conv(self, value):
        return dict(value)


prop_intern = InternedData(4)
run_info_intern = InternedData(8)
status_intern = InternedData(4)


def pack_result(data):
    # As `status_intern` normally handles one status, if `known_intermittent` is present in
    # the test logs, intern and store this with the `status` in an array until needed.
    if not data.get("known_intermittent"):
        return status_intern.store(data.get("status"))
    result = array.array("B")
    expected = data.get("expected")
    if expected is None:
        expected = data["status"]
    result_parts = [data["status"], expected] + data["known_intermittent"]
    for i, part in enumerate(result_parts):
        value = status_intern.store(part)
        if i % 2 == 0:
            assert value < 16
            result.append(value << 4)
        else:
            result[-1] += value
    return result


def unpack_result(data):
    if isinstance(data, int):
        return (status_intern.get(data), None)
    if isinstance(data, unicode):
        return (data, None)
    # Unpack multiple statuses into a tuple to be used in the Results named tuple below,
    # separating `status` and `known_intermittent`.
    results = []
    for packed_value in data:
        first = status_intern.get(packed_value >> 4)
        second = status_intern.get(packed_value & 0x0F)
        results.append(first)
        if second:
            results.append(second)
    return ((results[0],), tuple(results[1:]))


def load_test_data(test_paths):
    manifest_loader = testloader.ManifestLoader(test_paths, False)
    manifests = manifest_loader.load()

    id_test_map = {}
    for test_manifest, paths in manifests.iteritems():
        id_test_map.update(create_test_tree(paths["metadata_path"],
                                            test_manifest))
    return id_test_map


def update_from_logs(id_test_map, update_properties, disable_intermittent, update_intermittent,
                     remove_intermittent, full_update, *log_filenames):

    updater = ExpectedUpdater(id_test_map)

    for i, log_filename in enumerate(log_filenames):
        print("Processing log %d/%d" % (i + 1, len(log_filenames)))
        with open(log_filename) as f:
            updater.update_from_log(f)

    for item in update_results(id_test_map, update_properties, full_update,
                               disable_intermittent, update_intermittent=update_intermittent,
                               remove_intermittent=remove_intermittent):
        yield item


def update_results(id_test_map,
                   update_properties,
                   full_update,
                   disable_intermittent,
                   update_intermittent,
                   remove_intermittent):
    test_file_items = set(id_test_map.itervalues())

    default_expected_by_type = {}
    for test_type, test_cls in wpttest.manifest_test_cls.iteritems():
        if test_cls.result_cls:
            default_expected_by_type[(test_type, False)] = test_cls.result_cls.default_expected
        if test_cls.subtest_result_cls:
            default_expected_by_type[(test_type, True)] = test_cls.subtest_result_cls.default_expected

    for test_file in test_file_items:
        updated_expected = test_file.update(default_expected_by_type, update_properties,
                                            full_update, disable_intermittent, update_intermittent,
                                            remove_intermittent)
        if updated_expected is not None and updated_expected.modified:
            yield test_file.metadata_path, updated_expected


def directory_manifests(metadata_path):
    rv = []
    for dirpath, dirname, filenames in os.walk(metadata_path):
        if "__dir__.ini" in filenames:
            rel_path = os.path.relpath(dirpath, metadata_path)
            rv.append(os.path.join(rel_path, "__dir__.ini"))
    return rv


def write_new_expected(metadata_path, expected):
    # Serialize the data back to a file
    path = expected_path(metadata_path, expected.test_path)
    if not expected.is_empty:
        manifest_str = wptmanifest.serialize(expected.node,
                                             skip_empty_data=True)
        assert manifest_str != ""
        dir = os.path.split(path)[0]
        if not os.path.exists(dir):
            os.makedirs(dir)
        tmp_path = path + ".tmp"
        try:
            with open(tmp_path, "wb") as f:
                f.write(manifest_str)
            os.rename(tmp_path, path)
        except (Exception, KeyboardInterrupt):
            try:
                os.unlink(tmp_path)
            except OSError:
                pass
    else:
        try:
            os.unlink(path)
        except OSError:
            pass


class ExpectedUpdater(object):
    def __init__(self, id_test_map):
        self.id_test_map = id_test_map
        self.run_info = None
        self.action_map = {"suite_start": self.suite_start,
                           "test_start": self.test_start,
                           "test_status": self.test_status,
                           "test_end": self.test_end,
                           "assertion_count": self.assertion_count,
                           "lsan_leak": self.lsan_leak,
                           "mozleak_object": self.mozleak_object,
                           "mozleak_total": self.mozleak_total}
        self.tests_visited = {}

    def update_from_log(self, log_file):
        # We support three possible formats:
        # * wptreport format; one json object in the file, possibly pretty-printed
        # * wptreport format; one run per line
        # * raw log format

        # Try reading a single json object in wptreport format
        self.run_info = None
        success = self.get_wptreport_data(log_file.read())

        if success:
            return

        # Try line-separated json objects in wptreport format
        log_file.seek(0)
        for line in log_file:
            success = self.get_wptreport_data(line)
            if not success:
                break
        else:
            return

        # Assume the file is a raw log
        log_file.seek(0)
        self.update_from_raw_log(log_file)

    def get_wptreport_data(self, input_str):
        try:
            data = json.loads(input_str)
        except Exception:
            pass
        else:
            if "action" not in data and "results" in data:
                self.update_from_wptreport_log(data)
                return True
        return False

    def update_from_raw_log(self, log_file):
        action_map = self.action_map
        for line in log_file:
            try:
                data = json.loads(line)
            except ValueError:
                # Just skip lines that aren't json
                continue
            action = data["action"]
            if action in action_map:
                action_map[action](data)

    def update_from_wptreport_log(self, data):
        action_map = self.action_map
        action_map["suite_start"]({"run_info": data["run_info"]})
        for test in data["results"]:
            action_map["test_start"]({"test": test["test"]})
            for subtest in test["subtests"]:
                action_map["test_status"]({"test": test["test"],
                                           "subtest": subtest["name"],
                                           "status": subtest["status"],
                                           "expected": subtest.get("expected"),
                                           "known_intermittent": subtest.get("known_intermittent", [])})
            action_map["test_end"]({"test": test["test"],
                                    "status": test["status"],
                                    "expected": test.get("expected"),
                                    "known_intermittent": test.get("known_intermittent", [])})
            if "asserts" in test:
                asserts = test["asserts"]
                action_map["assertion_count"]({"test": test["test"],
                                               "count": asserts["count"],
                                               "min_expected": asserts["min"],
                                               "max_expected": asserts["max"]})
        for item in data.get("lsan_leaks", []):
            action_map["lsan_leak"](item)

        mozleak_data = data.get("mozleak", {})
        for scope, scope_data in mozleak_data.iteritems():
            for key, action in [("objects", "mozleak_object"),
                                ("total", "mozleak_total")]:
                for item in scope_data.get(key, []):
                    item_data = {"scope": scope}
                    item_data.update(item)
                    action_map[action](item_data)

    def suite_start(self, data):
        self.run_info = run_info_intern.store(RunInfo(data["run_info"]))

    def test_start(self, data):
        test_id = intern(data["test"].encode("utf8"))
        try:
            self.id_test_map[test_id]
        except KeyError:
            print("Test not found %s, skipping" % test_id)
            return

        self.tests_visited[test_id] = set()

    def test_status(self, data):
        test_id = intern(data["test"].encode("utf8"))
        subtest = intern(data["subtest"].encode("utf8"))
        test_data = self.id_test_map.get(test_id)
        if test_data is None:
            return

        self.tests_visited[test_id].add(subtest)

        result = pack_result(data)

        test_data.set(test_id, subtest, "status", self.run_info, result)
        if data.get("expected") and data["expected"] != data["status"]:
            test_data.set_requires_update()

    def test_end(self, data):
        if data["status"] == "SKIP":
            return

        test_id = intern(data["test"].encode("utf8"))
        test_data = self.id_test_map.get(test_id)
        if test_data is None:
            return

        result = pack_result(data)

        test_data.set(test_id, None, "status", self.run_info, result)
        if data.get("expected") and data["expected"] != data["status"]:
            test_data.set_requires_update()
        del self.tests_visited[test_id]

    def assertion_count(self, data):
        test_id = intern(data["test"].encode("utf8"))
        test_data = self.id_test_map.get(test_id)
        if test_data is None:
            return

        test_data.set(test_id, None, "asserts", self.run_info, data["count"])
        if data["count"] < data["min_expected"] or data["count"] > data["max_expected"]:
            test_data.set_requires_update()

    def test_for_scope(self, data):
        dir_path = data.get("scope", "/")
        dir_id = intern(os.path.join(dir_path, "__dir__").replace(os.path.sep, "/").encode("utf8"))
        if dir_id.startswith("/"):
            dir_id = dir_id[1:]
        return dir_id, self.id_test_map[dir_id]

    def lsan_leak(self, data):
        dir_id, test_data = self.test_for_scope(data)
        test_data.set(dir_id, None, "lsan",
                      self.run_info, (data["frames"], data.get("allowed_match")))
        if not data.get("allowed_match"):
            test_data.set_requires_update()

    def mozleak_object(self, data):
        dir_id, test_data = self.test_for_scope(data)
        test_data.set(dir_id, None, "leak-object",
                      self.run_info, ("%s:%s", (data["process"], data["name"]),
                                      data.get("allowed")))
        if not data.get("allowed"):
            test_data.set_requires_update()

    def mozleak_total(self, data):
        if data["bytes"]:
            dir_id, test_data = self.test_for_scope(data)
            test_data.set(dir_id, None, "leak-threshold",
                          self.run_info, (data["process"], data["bytes"], data["threshold"]))
            if data["bytes"] > data["threshold"] or data["bytes"] < 0:
                test_data.set_requires_update()


def create_test_tree(metadata_path, test_manifest):
    """Create a map of test_id to TestFileData for that test.
    """
    do_delayed_imports()
    id_test_map = {}
    exclude_types = frozenset(["manual", "support", "conformancechecker"])
    all_types = set(manifestitem.item_types.keys())
    assert all_types > exclude_types
    include_types = all_types - exclude_types
    for item_type, test_path, tests in test_manifest.itertypes(*include_types):
        test_file_data = TestFileData(intern(test_manifest.url_base.encode("utf8")),
                                      intern(item_type.encode("utf8")),
                                      metadata_path,
                                      test_path,
                                      tests)
        for test in tests:
            id_test_map[intern(test.id.encode("utf8"))] = test_file_data

        dir_path = os.path.split(test_path)[0].replace(os.path.sep, "/")
        while True:
            if dir_path:
                dir_id = dir_path + "/__dir__"
            else:
                dir_id = "__dir__"
            dir_id = intern((test_manifest.url_base + dir_id).lstrip("/").encode("utf8"))
            if dir_id not in id_test_map:
                test_file_data = TestFileData(intern(test_manifest.url_base.encode("utf8")),
                                              None,
                                              metadata_path,
                                              dir_id,
                                              [])
                id_test_map[dir_id] = test_file_data
            if not dir_path or dir_path in id_test_map:
                break
            dir_path = dir_path.rsplit("/", 1)[0] if "/" in dir_path else ""

    return id_test_map


class PackedResultList(object):
    """Class for storing test results.

    Results are stored as an array of 2-byte integers for compactness.
    The first 4 bits represent the property name, the second 4 bits
    represent the test status (if it's a result with a status code), and
    the final 8 bits represent the run_info. If the result doesn't have a
    simple status code but instead a richer type, we place that richer type
    in a dictionary and set the status part of the result type to 0.

    This class depends on the global prop_intern, run_info_intern and
    status_intern InteredData objects to convert between the bit values
    and corresponding Python objects."""

    def __init__(self):
        self.data = array.array("H")

    __slots__ = ("data", "raw_data")

    def append(self, prop, run_info, value):
        out_val = (prop << 12) + run_info
        if prop == prop_intern.store("status") and isinstance(value, int):
            out_val += value << 8
        else:
            if not hasattr(self, "raw_data"):
                self.raw_data = {}
            self.raw_data[len(self.data)] = value
        self.data.append(out_val)

    def unpack(self, idx, packed):
        prop = prop_intern.get((packed & 0xF000) >> 12)

        value_idx = (packed & 0x0F00) >> 8
        if value_idx == 0:
            value = self.raw_data[idx]
        else:
            value = status_intern.get(value_idx)

        run_info = run_info_intern.get(packed & 0x00FF)

        return prop, run_info, value

    def __iter__(self):
        for i, item in enumerate(self.data):
            yield self.unpack(i, item)


class TestFileData(object):
    __slots__ = ("url_base", "item_type", "test_path", "metadata_path", "tests",
                 "_requires_update", "data")

    def __init__(self, url_base, item_type, metadata_path, test_path, tests):
        self.url_base = url_base
        self.item_type = item_type
        self.test_path = test_path
        self.metadata_path = metadata_path
        self.tests = {intern(item.id.encode("utf8")) for item in tests}
        self._requires_update = False
        self.data = defaultdict(lambda: defaultdict(PackedResultList))

    def set_requires_update(self):
        self._requires_update = True

    @property
    def requires_update(self):
        return self._requires_update

    def set(self, test_id, subtest_id, prop, run_info, value):
        self.data[test_id][subtest_id].append(prop_intern.store(prop),
                                              run_info,
                                              value)

    def expected(self, update_properties, update_intermittent, remove_intermittent):
        expected_data = load_expected(self.url_base,
                                      self.metadata_path,
                                      self.test_path,
                                      self.tests,
                                      update_properties,
                                      update_intermittent,
                                      remove_intermittent)
        if expected_data is None:
            expected_data = create_expected(self.url_base,
                                            self.test_path,
                                            update_properties,
                                            update_intermittent,
                                            remove_intermittent)
        return expected_data

    def is_disabled(self, test):
        # This conservatively assumes that anything that was disabled remains disabled
        # we could probably do better by checking if it's in the full set of run infos
        return test.has_key("disabled")

    def orphan_subtests(self, expected):
        # Return subtest nodes present in the expected file, but missing from the data
        rv = []

        for test_id, subtests in self.data.iteritems():
            test = expected.get_test(test_id.decode("utf8"))
            if not test:
                continue
            seen_subtests = set(item.decode("utf8") for item in subtests.iterkeys() if item is not None)
            missing_subtests = set(test.subtests.keys()) - seen_subtests
            for item in missing_subtests:
                expected_subtest = test.get_subtest(item)
                if not self.is_disabled(expected_subtest):
                    rv.append(expected_subtest)
            for name in seen_subtests:
                subtest = test.get_subtest(name)
                # If any of the items have children (ie subsubtests) we want to prune thes
                if subtest.children:
                    rv.extend(subtest.children)

        return rv

    def update(self, default_expected_by_type, update_properties,
               full_update=False, disable_intermittent=None, update_intermittent=False,
               remove_intermittent=False):
        # If we are doing a full update, we may need to prune missing nodes
        # even if the expectations didn't change
        if not self.requires_update and not full_update:
            return

        expected = self.expected(update_properties,
                                 update_intermittent=update_intermittent,
                                 remove_intermittent=remove_intermittent)

        if full_update:
            orphans = self.orphan_subtests(expected)

            if not self.requires_update and not orphans:
                return

            if orphans:
                expected.modified = True
                for item in orphans:
                    item.remove()

        expected_by_test = {}

        for test_id in self.tests:
            if not expected.has_test(test_id):
                expected.append(manifestupdate.TestNode.create(test_id))
            test_expected = expected.get_test(test_id)
            expected_by_test[test_id] = test_expected

        for test_id, test_data in self.data.iteritems():
            test_id = test_id.decode("utf8")
            for subtest_id, results_list in test_data.iteritems():
                for prop, run_info, value in results_list:
                    # Special case directory metadata
                    if subtest_id is None and test_id.endswith("__dir__"):
                        if prop == "lsan":
                            expected.set_lsan(run_info, value)
                        elif prop == "leak-object":
                            expected.set_leak_object(run_info, value)
                        elif prop == "leak-threshold":
                            expected.set_leak_threshold(run_info, value)
                        continue

                    test_expected = expected_by_test[test_id]
                    if subtest_id is None:
                        item_expected = test_expected
                    else:
                        if isinstance(subtest_id, str):
                            subtest_id = subtest_id.decode("utf8")
                        item_expected = test_expected.get_subtest(subtest_id)

                    if prop == "status":
                        status, known_intermittent = unpack_result(value)
                        value = Result(status,
                                       known_intermittent,
                                       default_expected_by_type[self.item_type,
                                                                subtest_id is not None])
                        item_expected.set_result(run_info, value)
                    elif prop == "asserts":
                        item_expected.set_asserts(run_info, value)

        expected.update(full_update=full_update,
                        disable_intermittent=disable_intermittent)
        for test in expected.iterchildren():
            for subtest in test.iterchildren():
                subtest.update(full_update=full_update,
                               disable_intermittent=disable_intermittent)
            test.update(full_update=full_update,
                        disable_intermittent=disable_intermittent)

        return expected


Result = namedtuple("Result", ["status", "known_intermittent", "default_expected"])


def create_expected(url_base, test_path, run_info_properties, update_intermittent, remove_intermittent):
    expected = manifestupdate.ExpectedManifest(None,
                                               test_path,
                                               url_base,
                                               run_info_properties,
                                               update_intermittent,
                                               remove_intermittent)
    return expected


def load_expected(url_base, metadata_path, test_path, tests, run_info_properties, update_intermittent, remove_intermittent):
    expected_manifest = manifestupdate.get_manifest(metadata_path,
                                                    test_path,
                                                    url_base,
                                                    run_info_properties,
                                                    update_intermittent,
                                                    remove_intermittent)
    return expected_manifest
