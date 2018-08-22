# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import argparse
import imp
import os
import sys
from collections import defaultdict

from mozlog.structured import commandline
from wptrunner.wptcommandline import get_test_paths, set_from_config

manifest = None


def do_delayed_imports(wpt_dir):
    global manifest
    sys.path.insert(0, os.path.join(wpt_dir, "tools", "manifest"))
    import manifest  # noqa


def create_parser():
    p = argparse.ArgumentParser()
    p.add_argument("--check-clean", action="store_true",
                   help="Check that updating the manifest doesn't lead to any changes")
    p.add_argument("--rebuild", action="store_true",
                   help="Rebuild the manifest from scratch")
    commandline.add_logging_group(p)

    return p


def update(logger, wpt_dir, check_clean=True, rebuild=False):
    localpaths = imp.load_source("localpaths",  # noqa
                                 os.path.join(wpt_dir, "web-platform-tests", "tools", "localpaths.py"))
    kwargs = {"config": os.path.join(wpt_dir, "config.ini"),
              "manifest_path": os.path.join(wpt_dir, "metadata"),
              "tests_root": None,
              "metadata_root": None}

    set_from_config(kwargs)
    config = kwargs["config"]
    test_paths = get_test_paths(config)

    do_delayed_imports(wpt_dir)

    if check_clean:
        return _check_clean(logger, test_paths)

    return _update(logger, test_paths, rebuild)


def _update(logger, test_paths, rebuild):
    for url_base, paths in test_paths.iteritems():
        manifest_path = os.path.join(paths["metadata_path"], "MANIFEST.json")
        m = None
        if not rebuild:
            try:
                m = manifest.manifest.load(paths["tests_path"], manifest_path)
            except manifest.manifest.ManifestVersionMismatch:
                logger.info("Manifest format changed, rebuilding")
        if m is None:
            m = manifest.manifest.Manifest(url_base)
        manifest.update.update(paths["tests_path"], m, working_copy=True)
        manifest.manifest.write(m, manifest_path)
    return 0


def _check_clean(logger, test_paths):
    manifests_by_path = {}
    rv = 0
    for url_base, paths in test_paths.iteritems():
        tests_path = paths["tests_path"]
        manifest_path = os.path.join(paths["metadata_path"], "MANIFEST.json")
        old_manifest = manifest.manifest.load(tests_path, manifest_path)
        new_manifest = manifest.manifest.Manifest.from_json(tests_path,
                                                            old_manifest.to_json())
        manifest.update.update(tests_path, new_manifest, working_copy=True)
        manifests_by_path[manifest_path] = (old_manifest, new_manifest)

    for manifest_path, (old_manifest, new_manifest) in manifests_by_path.iteritems():
        if not diff_manifests(logger, manifest_path, old_manifest, new_manifest):
            rv = 1
    if rv:
        logger.error("Manifest %s is outdated, use |./mach update-manifest| to fix." % manifest_path)

    return rv


def diff_manifests(logger, manifest_path, old_manifest, new_manifest):
    """Lint the differences between old and new versions of a
    manifest. Differences are considered significant (and so produce
    lint errors) if they produce a meaningful difference in the actual
    tests run.

    :param logger: mozlog logger to use for output
    :param manifest_path: Path to the manifest being linted
    :param old_manifest: Manifest object representing the initial manifest
    :param new_manifest: Manifest object representing the updated manifest
    """
    logger.info("Diffing old and new manifests %s" % manifest_path)
    old_items, new_items = defaultdict(set), defaultdict(set)
    for manifest, items in [(old_manifest, old_items),
                            (new_manifest, new_items)]:
        for test_type, path, tests in manifest:
            for test in tests:
                test_id = [test.id]
                test_id.extend(tuple(item) if isinstance(item, list) else item
                               for item in test.meta_key())
                if hasattr(test, "references"):
                    test_id.extend(tuple(item) for item in test.references)
                test_id = tuple(test_id)
                items[path].add((test_type, test_id))

    old_paths = set(old_items.iterkeys())
    new_paths = set(new_items.iterkeys())

    added_paths = new_paths - old_paths
    deleted_paths = old_paths - new_paths

    common_paths = new_paths & old_paths

    clean = True

    for path in added_paths:
        clean = False
        log_error(logger, manifest_path, "%s in source but not in manifest." % path)
    for path in deleted_paths:
        clean = False
        log_error(logger, manifest_path, "%s in manifest but removed from source." % path)

    for path in common_paths:
        old_tests = old_items[path]
        new_tests = new_items[path]
        added_tests = new_tests - old_tests
        removed_tests = old_tests - new_tests
        if added_tests or removed_tests:
            clean = False
            log_error(logger, manifest_path, "%s changed test types or metadata" % path)

    if clean:
        # Manifest currently has some list vs tuple inconsistencies that break
        # a simple equality comparison.
        new_paths = {(key, value[0], value[1])
                     for (key, value) in new_manifest.to_json()["paths"].iteritems()}
        old_paths = {(key, value[0], value[1])
                     for (key, value) in old_manifest.to_json()["paths"].iteritems()}
        if old_paths != new_paths:
            logger.warning("Manifest %s contains correct tests but file hashes changed." % manifest_path)  # noqa
            clean = False

    return clean


def log_error(logger, manifest_path, msg):
    logger.lint_error(path=manifest_path,
                      message=msg,
                      lineno=0,
                      source="",
                      linter="wpt-manifest")
