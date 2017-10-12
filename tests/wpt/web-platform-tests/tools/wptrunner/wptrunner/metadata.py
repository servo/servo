import os
import shutil
import sys
import tempfile
import types
import uuid
from collections import defaultdict

from mozlog import reader
from mozlog import structuredlog

import expected
import manifestupdate
import testloader
import wptmanifest
import wpttest
from vcs import git
manifest = None  # Module that will be imported relative to test_root
manifestitem = None

logger = structuredlog.StructuredLogger("web-platform-tests")


def load_test_manifests(serve_root, test_paths):
    do_delayed_imports(serve_root)
    manifest_loader = testloader.ManifestLoader(test_paths, False)
    return manifest_loader.load()


def update_expected(test_paths, serve_root, log_file_names,
                    rev_old=None, rev_new="HEAD", ignore_existing=False,
                    sync_root=None, property_order=None, boolean_properties=None):
    """Update the metadata files for web-platform-tests based on
    the results obtained in a previous run"""

    manifests = load_test_manifests(serve_root, test_paths)

    change_data = {}

    if sync_root is not None:
        if rev_old is not None:
            rev_old = git("rev-parse", rev_old, repo=sync_root).strip()
        rev_new = git("rev-parse", rev_new, repo=sync_root).strip()

        if rev_old is not None:
            change_data = load_change_data(rev_old, rev_new, repo=sync_root)


    expected_map_by_manifest = update_from_logs(manifests,
                                                *log_file_names,
                                                ignore_existing=ignore_existing,
                                                property_order=property_order,
                                                boolean_properties=boolean_properties)

    for test_manifest, expected_map in expected_map_by_manifest.iteritems():
        url_base = manifests[test_manifest]["url_base"]
        metadata_path = test_paths[url_base]["metadata_path"]
        write_changes(metadata_path, expected_map)

    results_changed = [item.test_path for item in expected_map.itervalues() if item.modified]

    return unexpected_changes(manifests, change_data, results_changed)


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

    rv = []

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
#      If any new values mismatch mark the test as needing human attention
#   Check if all the RHS values are the same; if so collapse the conditionals


def update_from_logs(manifests, *log_filenames, **kwargs):
    ignore_existing = kwargs.get("ignore_existing", False)
    property_order = kwargs.get("property_order")
    boolean_properties = kwargs.get("boolean_properties")

    expected_map = {}
    id_test_map = {}

    for test_manifest, paths in manifests.iteritems():
        expected_map_manifest, id_path_map_manifest = create_test_tree(
            paths["metadata_path"],
            test_manifest,
            property_order=property_order,
            boolean_properties=boolean_properties)
        expected_map[test_manifest] = expected_map_manifest
        id_test_map.update(id_path_map_manifest)

    updater = ExpectedUpdater(manifests, expected_map, id_test_map,
                              ignore_existing=ignore_existing)
    for log_filename in log_filenames:
        with open(log_filename) as f:
            updater.update_from_log(f)

    for manifest_expected in expected_map.itervalues():
        for tree in manifest_expected.itervalues():
            for test in tree.iterchildren():
                for subtest in test.iterchildren():
                    subtest.coalesce_expected()
                test.coalesce_expected()

    return expected_map

def directory_manifests(metadata_path):
    rv = []
    for dirpath, dirname, filenames in os.walk(metadata_path):
        if "__dir__.ini" in filenames:
            rel_path = os.path.relpath(dirpath, metadata_path)
            rv.append(os.path.join(rel_path, "__dir__.ini"))
    return rv

def write_changes(metadata_path, expected_map):
    # First write the new manifest files to a temporary directory
    temp_path = tempfile.mkdtemp(dir=os.path.split(metadata_path)[0])
    write_new_expected(temp_path, expected_map)

    # Keep all __dir__.ini files (these are not in expected_map because they
    # aren't associated with a specific test)
    keep_files = directory_manifests(metadata_path)

    # Copy all files in the root to the temporary location since
    # these cannot be ini files
    keep_files.extend(item for item in os.listdir(metadata_path) if
                      not os.path.isdir(os.path.join(metadata_path, item)))

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


def write_new_expected(metadata_path, expected_map):
    # Serialize the data back to a file
    for tree in expected_map.itervalues():
        if not tree.is_empty:
            manifest_str = wptmanifest.serialize(tree.node, skip_empty_data=True)
            assert manifest_str != ""
            path = expected.expected_path(metadata_path, tree.test_path)
            dir = os.path.split(path)[0]
            if not os.path.exists(dir):
                os.makedirs(dir)
            with open(path, "wb") as f:
                f.write(manifest_str)


class ExpectedUpdater(object):
    def __init__(self, test_manifests, expected_tree, id_path_map, ignore_existing=False):
        self.test_manifests = test_manifests
        self.expected_tree = expected_tree
        self.id_path_map = id_path_map
        self.ignore_existing = ignore_existing
        self.run_info = None
        self.action_map = {"suite_start": self.suite_start,
                           "test_start": self.test_start,
                           "test_status": self.test_status,
                           "test_end": self.test_end}
        self.tests_visited = {}

        self.test_cache = {}

    def update_from_log(self, log_file):
        self.run_info = None
        log_reader = reader.read(log_file)
        reader.each_log(log_reader, self.action_map)

    def suite_start(self, data):
        self.run_info = data["run_info"]

    def test_id(self, id):
        if type(id) in types.StringTypes:
            return id
        else:
            return tuple(id)

    def test_start(self, data):
        test_id = self.test_id(data["test"])
        try:
            test_manifest, test = self.id_path_map[test_id]
            expected_node = self.expected_tree[test_manifest][test].get_test(test_id)
        except KeyError:
            print "Test not found %s, skipping" % test_id
            return
        self.test_cache[test_id] = expected_node

        if test_id not in self.tests_visited:
            if self.ignore_existing:
                expected_node.clear_expected()
            self.tests_visited[test_id] = set()

    def test_status(self, data):
        test_id = self.test_id(data["test"])
        test = self.test_cache.get(test_id)
        if test is None:
            return
        test_cls = wpttest.manifest_test_cls[test.test_type]

        subtest = test.get_subtest(data["subtest"])

        self.tests_visited[test.id].add(data["subtest"])

        result = test_cls.subtest_result_cls(
            data["subtest"],
            data["status"],
            data.get("message"))

        subtest.set_result(self.run_info, result)

    def test_end(self, data):
        test_id = self.test_id(data["test"])
        test = self.test_cache.get(test_id)
        if test is None:
            return
        test_cls = wpttest.manifest_test_cls[test.test_type]

        if data["status"] == "SKIP":
            return

        result = test_cls.result_cls(
            data["status"],
            data.get("message"))

        test.set_result(self.run_info, result)
        del self.test_cache[test_id]


def create_test_tree(metadata_path, test_manifest, property_order=None,
                     boolean_properties=None):
    expected_map = {}
    id_test_map = {}
    exclude_types = frozenset(["stub", "helper", "manual", "support", "conformancechecker"])
    all_types = [item.item_type for item in manifestitem.__dict__.itervalues()
                 if type(item) == type and
                 issubclass(item, manifestitem.ManifestItem) and
                 item.item_type is not None]
    include_types = set(all_types) - exclude_types
    for _, test_path, tests in test_manifest.itertypes(*include_types):
        expected_data = load_expected(test_manifest, metadata_path, test_path, tests,
                                      property_order=property_order,
                                      boolean_properties=boolean_properties)
        if expected_data is None:
            expected_data = create_expected(test_manifest,
                                            test_path,
                                            tests,
                                            property_order=property_order,
                                            boolean_properties=boolean_properties)

        for test in tests:
            id_test_map[test.id] = (test_manifest, test)
            expected_map[test] = expected_data

    return expected_map, id_test_map


def create_expected(test_manifest, test_path, tests, property_order=None,
                    boolean_properties=None):
    expected = manifestupdate.ExpectedManifest(None, test_path, test_manifest.url_base,
                                               property_order=property_order,
                                               boolean_properties=boolean_properties)
    for test in tests:
        expected.append(manifestupdate.TestNode.create(test.item_type, test.id))
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

    tests_by_id = {item.id: item for item in tests}

    # Remove expected data for tests that no longer exist
    for test in expected_manifest.iterchildren():
        if not test.id in tests_by_id:
            test.remove()

    # Add tests that don't have expected data
    for test in tests:
        if not expected_manifest.has_test(test.id):
            expected_manifest.append(manifestupdate.TestNode.create(test.item_type, test.id))

    return expected_manifest
