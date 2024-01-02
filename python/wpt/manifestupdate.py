# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import argparse
import os
import sys
import tempfile
from collections import defaultdict
from six import iterkeys, iteritems

from . import SERVO_ROOT, WPT_PATH
from mozlog.structured import commandline

# This must happen after importing from "." since it adds WPT
# tools to the Python system path.
import manifest as wptmanifest

from wptrunner.wptcommandline import get_test_paths, set_from_config
from wptrunner import wptlogging


def create_parser():
    p = argparse.ArgumentParser()
    p.add_argument("--check-clean", action="store_true",
                   help="Check that updating the manifest doesn't lead to any changes")
    p.add_argument("--rebuild", action="store_true",
                   help="Rebuild the manifest from scratch")
    commandline.add_logging_group(p)

    return p


def update(check_clean=True, rebuild=False, logger=None, **kwargs):
    if not logger:
        logger = wptlogging.setup(kwargs, {"mach": sys.stdout})
    kwargs = {"config": os.path.join(WPT_PATH, "config.ini"),
              "product": "servo",
              "manifest_path": os.path.join(WPT_PATH, "meta"),
              "tests_root": None,
              "metadata_root": None}

    set_from_config(kwargs)
    config = kwargs["config"]
    test_paths = get_test_paths(config)

    if check_clean:
        return _check_clean(logger, test_paths)

    return _update(logger, test_paths, rebuild)


def _update(logger, test_paths, rebuild):
    for url_base, paths in iteritems(test_paths):
        manifest_path = os.path.join(paths.metadata_path, "MANIFEST.json")
        cache_subdir = os.path.relpath(os.path.dirname(manifest_path),
                                       os.path.dirname(__file__))
        wptmanifest.manifest.load_and_update(paths.tests_path,
                                             manifest_path,
                                             url_base,
                                             working_copy=True,
                                             rebuild=rebuild,
                                             cache_root=os.path.join(SERVO_ROOT, ".wpt",
                                                                     cache_subdir))
    return 0


def _check_clean(logger, test_paths):
    manifests_by_path = {}
    rv = 0
    for url_base, paths in iteritems(test_paths):
        tests_path = paths.tests_path
        manifest_path = os.path.join(paths.metadata_path, "MANIFEST.json")

        old_manifest = wptmanifest.manifest.load_and_update(tests_path,
                                                            manifest_path,
                                                            url_base,
                                                            working_copy=False,
                                                            update=False,
                                                            write_manifest=False)

        # Even if no cache is specified, one will be used automatically by the
        # VCS integration. Create a brand new cache every time to ensure that
        # the VCS integration always thinks that any file modifications in the
        # working directory are new and interesting.
        cache_root = tempfile.mkdtemp()
        new_manifest = wptmanifest.manifest.load_and_update(tests_path,
                                                            manifest_path,
                                                            url_base,
                                                            working_copy=True,
                                                            update=True,
                                                            cache_root=cache_root,
                                                            write_manifest=False,
                                                            allow_cached=False)

        manifests_by_path[manifest_path] = (old_manifest, new_manifest)

    for manifest_path, (old_manifest, new_manifest) in iteritems(manifests_by_path):
        if not diff_manifests(logger, manifest_path, old_manifest, new_manifest):
            logger.error("Manifest %s is outdated, use |./mach update-manifest| to fix." % manifest_path)
            rv = 1

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
                if hasattr(test, "script_metadata"):
                    if test.script_metadata is not None:
                        test_id.extend(tuple(item) for item in test.script_metadata)
                if hasattr(test, "references"):
                    test_id.extend(tuple(item) for item in test.references)
                test_id = tuple(test_id)
                items[path].add((test_type, test_id))

    old_paths = set(iterkeys(old_items))
    new_paths = set(iterkeys(new_items))

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
        old_paths = old_manifest.to_json()['items']
        new_paths = new_manifest.to_json()['items']
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
