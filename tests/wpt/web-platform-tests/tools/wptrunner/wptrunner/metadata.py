import os
import shutil
import tempfile
import uuid

from mozlog import structuredlog

import manifestupdate
import testloader
import wptmanifest
import wpttest
from expected import expected_path
from vcs import git
manifest = None  # Module that will be imported relative to test_root
manifestitem = None

logger = structuredlog.StructuredLogger("web-platform-tests")


try:
    import ujson as json
except ImportError:
    import json


def load_test_manifests(serve_root, test_paths):
    do_delayed_imports(serve_root)
    manifest_loader = testloader.ManifestLoader(test_paths, False)
    return manifest_loader.load()


def update_expected(test_paths, serve_root, log_file_names,
                    rev_old=None, rev_new="HEAD", ignore_existing=False,
                    sync_root=None, property_order=None, boolean_properties=None,
                    stability=None):
    """Update the metadata files for web-platform-tests based on
    the results obtained in a previous run or runs

    If stability is not None, assume log_file_names refers to logs from repeated
    test jobs, disable tests that don't behave as expected on all runs"""

    manifests = load_test_manifests(serve_root, test_paths)

    for metadata_path, updated_ini in update_from_logs(manifests,
                                                       *log_file_names,
                                                       ignore_existing=ignore_existing,
                                                       property_order=property_order,
                                                       boolean_properties=boolean_properties,
                                                       stability=stability):

        write_new_expected(metadata_path, updated_ini)
        if stability:
            for test in updated_ini.iterchildren():
                for subtest in test.iterchildren():
                    if subtest.new_disabled:
                        print "disabled: %s" % os.path.dirname(subtest.root.test_path) + "/" + subtest.name
                    if test.new_disabled:
                        print "disabled: %s" % test.root.test_path


def do_delayed_imports(serve_root):
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
#           If stability and any repeated values don't match, disable the test
#           else mark the test as needing human attention
#   Check if all the RHS values are the same; if so collapse the conditionals


def update_from_logs(manifests, *log_filenames, **kwargs):
    ignore_existing = kwargs.get("ignore_existing", False)
    property_order = kwargs.get("property_order")
    boolean_properties = kwargs.get("boolean_properties")
    stability = kwargs.get("stability")

    id_test_map = {}

    for test_manifest, paths in manifests.iteritems():
        id_test_map.update(create_test_tree(
            paths["metadata_path"],
            test_manifest))

    updater = ExpectedUpdater(manifests,
                              id_test_map,
                              ignore_existing=ignore_existing)
    for log_filename in log_filenames:
        with open(log_filename) as f:
            updater.update_from_log(f)
    for item in update_results(id_test_map, property_order, boolean_properties, stability):
        yield item


def update_results(id_test_map, property_order, boolean_properties, stability):
    test_file_items = set(id_test_map.itervalues())
    for test_file in test_file_items:
        updated_expected = test_file.update(property_order, boolean_properties, stability)
        if updated_expected is not None and updated_expected.modified:
            yield test_file.metadata_path, updated_expected


def directory_manifests(metadata_path):
    rv = []
    for dirpath, dirname, filenames in os.walk(metadata_path):
        if "__dir__.ini" in filenames:
            rel_path = os.path.relpath(dirpath, metadata_path)
            rv.append(os.path.join(rel_path, "__dir__.ini"))
    return rv


def write_changes(metadata_path, expected):
    # First write the new manifest files to a temporary directory
    temp_path = tempfile.mkdtemp(dir=os.path.split(metadata_path)[0])
    write_new_expected(temp_path, expected)

    # Copy all files in the root to the temporary location since
    # these cannot be ini files
    keep_files = [item for item in os.listdir(metadata_path) if
                  not os.path.isdir(os.path.join(metadata_path, item))]

    for item in keep_files:
        dest_dir = os.path.dirname(os.path.join(temp_path, item))
        if not os.path.exists(dest_dir):
            os.makedirs(dest_dir)
        shutil.copyfile(os.path.join(metadata_path, item),
                        os.path.join(temp_path, item))

    # Then move the old manifest files to a new location
    temp_path_2 = metadata_path + str(uuid.uuid4())
    os.rename(metadata_path, temp_path_2)
    # Move the new files to the destination location and remove the old files
    os.rename(temp_path, metadata_path)
    shutil.rmtree(temp_path_2)


def write_new_expected(metadata_path, expected):
    # Serialize the data back to a file
    path = expected_path(metadata_path, expected.test_path)
    if not expected.is_empty:
        manifest_str = wptmanifest.serialize(expected.node, skip_empty_data=True)
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
    def __init__(self, test_manifests, id_test_map, ignore_existing=False):
        self.id_test_map = id_test_map
        self.ignore_existing = ignore_existing
        self.run_info = None
        self.action_map = {"suite_start": self.suite_start,
                           "test_start": self.test_start,
                           "test_status": self.test_status,
                           "test_end": self.test_end,
                           "assertion_count": self.assertion_count,
                           "lsan_leak": self.lsan_leak}
        self.tests_visited = {}

        self.types_by_path = {}
        for manifest in test_manifests.iterkeys():
            for test_type, path, _ in manifest:
                if test_type in wpttest.manifest_test_cls:
                    self.types_by_path[path] = wpttest.manifest_test_cls[test_type]
        self.run_infos = []

    def update_from_log(self, log_file):
        self.run_info = None
        try:
            data = json.load(log_file)
        except Exception:
            pass
        else:
            if "action" not in data and "results" in data:
                self.update_from_wptreport_log(data)
                return

        log_file.seek(0)
        self.update_from_raw_log(log_file)

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
                                           "expected": subtest.get("expected")})
            action_map["test_end"]({"test": test["test"],
                                    "status": test["status"],
                                    "expected": test.get("expected")})
            if "asserts" in test:
                asserts = test["asserts"]
                action_map["assertion_count"]({"test": test["test"],
                                               "count": asserts["count"],
                                               "min_expected": asserts["min"],
                                               "max_expected": asserts["max"]})
        for item in data.get("lsan_leaks", []):
            action_map["lsan_leak"](item)

    def suite_start(self, data):
        self.run_info = data["run_info"]

    def test_start(self, data):
        test_id = data["test"]
        try:
            test_data = self.id_test_map[test_id]
        except KeyError:
            print "Test not found %s, skipping" % test_id
            return

        if self.ignore_existing:
            test_data.set_requires_update()
            test_data.clear.append("expected")
        self.tests_visited[test_id] = set()

    def test_status(self, data):
        test_id = data["test"]
        subtest = data["subtest"]
        test_data = self.id_test_map.get(test_id)
        if test_data is None:
            return

        test_cls = self.types_by_path[test_data.test_path]

        self.tests_visited[test_id].add(subtest)

        result = test_cls.subtest_result_cls(
            subtest,
            data["status"],
            None)

        test_data.set(test_id, subtest, "status", self.run_info, result)
        if data.get("expected") and data["expected"] != data["status"]:
            test_data.set_requires_update()

    def test_end(self, data):
        if data["status"] == "SKIP":
            return

        test_id = data["test"]
        test_data = self.id_test_map.get(test_id)
        if test_data is None:
            return
        test_cls = self.types_by_path[test_data.test_path]


        result = test_cls.result_cls(
            data["status"],
            None)
        test_data.set(test_id, None, "status", self.run_info, result)
        if data.get("expected") and data["status"] != data["expected"]:
            test_data.set_requires_update()
        del self.tests_visited[test_id]

    def assertion_count(self, data):
        test_id = data["test"]
        test_data = self.id_test_map.get(test_id)
        if test_data is None:
            return

        test_data.set(test_id, None, "asserts", self.run_info, data["count"])
        if data["count"] < data["min_expected"] or data["count"] > data["max_expected"]:
            test_data.set_requires_update()

    def lsan_leak(self, data):
        dir_path = data.get("scope", "/")
        dir_id = os.path.join(dir_path, "__dir__").replace(os.path.sep, "/")
        if dir_id.startswith("/"):
            dir_id = dir_id[1:]
        test_data = self.id_test_map[dir_id]
        test_data.set(dir_id, None, "lsan", self.run_info, (data["frames"], data.get("allowed_match")))
        if not data.get("allowed_match"):
            test_data.set_requires_update()


def create_test_tree(metadata_path, test_manifest):
    """Create a map of expectation manifests for all tests in test_manifest,
    reading existing manifests under manifest_path

    :returns: A map of test_id to (manifest, test, expectation_data)
    """
    id_test_map = {}
    exclude_types = frozenset(["stub", "helper", "manual", "support", "conformancechecker"])
    all_types = manifestitem.item_types.keys()
    include_types = set(all_types) - exclude_types
    for _, test_path, tests in test_manifest.itertypes(*include_types):
        test_file_data = TestFileData(test_manifest,
                                      metadata_path,
                                      test_path,
                                      tests)
        for test in tests:
            id_test_map[test.id] = test_file_data

        dir_path = os.path.split(test_path)[0].replace(os.path.sep, "/")
        while True:
            if dir_path:
                dir_id = dir_path + "/__dir__"
            else:
                dir_id = "__dir__"
            dir_id = (test_manifest.url_base + dir_id).lstrip("/")
            if dir_id not in id_test_map:
                test_file_data = TestFileData(test_manifest,
                                              metadata_path,
                                              dir_id,
                                              [])
                id_test_map[dir_id] = test_file_data
            if not dir_path or dir_path in id_test_map:
                break
            dir_path = dir_path.rsplit("/", 1)[0] if "/" in dir_path else ""

    return id_test_map


class TestFileData(object):
    def __init__(self, test_manifest, metadata_path, test_path, tests):
        self.test_manifest = test_manifest
        self.test_path = test_path
        self.metadata_path = metadata_path
        self.tests = tests
        self._expected = None
        self._requires_update = False
        self.clear = set()
        self.data = []

    def set_requires_update(self):
        self._requires_update = True

    def set(self, test_id, subtest_id, prop, run_info, value):
        self.data.append((test_id, subtest_id, prop, run_info, value))

    def expected(self, property_order, boolean_properties):
        if self._expected is None:
            expected_data = load_expected(self.test_manifest,
                                          self.metadata_path,
                                          self.test_path,
                                          self.tests,
                                          property_order,
                                          boolean_properties)
            if expected_data is None:
                expected_data = create_expected(self.test_manifest,
                                                self.test_path,
                                                property_order,
                                                boolean_properties)
            self._expected = expected_data
        return self._expected

    def update(self, property_order, boolean_properties, stability):
        if not self._requires_update:
            return

        expected = self.expected(property_order, boolean_properties)
        expected_by_test = {}

        for test in self.tests:
            if not expected.has_test(test.id):
                expected.append(manifestupdate.TestNode.create(test.id))
            test_expected = expected.get_test(test.id)
            expected_by_test[test.id] = test_expected
            for prop in self.clear:
                test_expected.clear(prop)

        for (test_id, subtest_id, prop, run_info, value) in self.data:
            # Special case directory metadata
            if subtest_id is None and test_id.endswith("__dir__"):
                if prop == "lsan":
                    expected.set_lsan(run_info, value)
                continue

            test_expected = expected_by_test[test_id]
            if subtest_id is None:
                item_expected = test_expected
            else:
                item_expected = test_expected.get_subtest(subtest_id)
            if prop == "status":
                item_expected.set_result(run_info, value)
            elif prop == "asserts":
                item_expected.set_asserts(run_info, value)

        expected.coalesce_properties(stability=stability)
        for test in expected.iterchildren():
            for subtest in test.iterchildren():
                subtest.coalesce_properties(stability=stability)
            test.coalesce_properties(stability=stability)

        return expected


def create_expected(test_manifest, test_path, property_order=None,
                    boolean_properties=None):
    expected = manifestupdate.ExpectedManifest(None, test_path, test_manifest.url_base,
                                               property_order=property_order,
                                               boolean_properties=boolean_properties)
    return expected


def load_expected(test_manifest, metadata_path, test_path, tests, property_order=None,
                  boolean_properties=None):
    expected_manifest = manifestupdate.get_manifest(metadata_path,
                                                    test_path,
                                                    test_manifest.url_base,
                                                    property_order=property_order,
                                                    boolean_properties=boolean_properties)
    if expected_manifest is None:
        return

    tests_by_id = {item.id for item in tests}

    # Remove expected data for tests that no longer exist
    for test in expected_manifest.iterchildren():
        if test.id not in tests_by_id:
            test.remove()

    return expected_manifest
