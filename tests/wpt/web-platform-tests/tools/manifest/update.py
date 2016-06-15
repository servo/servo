#!/usr/bin/env python
import argparse
import imp
import os
import sys

import manifest
from . import vcs
from .log import get_logger
from .tree import GitTree, NoVCSTree

here = os.path.dirname(__file__)
localpaths = imp.load_source("localpaths", os.path.abspath(os.path.join(here, os.pardir, "localpaths.py")))

def update(tests_root, url_base, manifest, ignore_local=False):
    if vcs.is_git_repo(tests_root):
        tests_tree = GitTree(tests_root, url_base)
        remove_missing_local = False
    else:
        tests_tree = NoVCSTree(tests_root, url_base)
        remove_missing_local = not ignore_local

    if not ignore_local:
        local_changes = tests_tree.local_changes()
    else:
        local_changes = None

    manifest.update(tests_root,
                    url_base,
                    tests_tree.current_rev(),
                    tests_tree.committed_changes(manifest.rev),
                    local_changes,
                    remove_missing_local=remove_missing_local)


def update_from_cli(**kwargs):
    tests_root = kwargs["tests_root"]
    path = kwargs["path"]
    assert tests_root is not None

    m = None
    logger = get_logger()

    if not kwargs.get("rebuild", False):
        try:
            m = manifest.load(tests_root, path)
        except manifest.ManifestVersionMismatch:
            logger.info("Manifest version changed, rebuilding")
            m = None
        else:
            logger.info("Updating manifest")

    if m is None:
        m = manifest.Manifest(None)


    update(tests_root,
           kwargs["url_base"],
           m,
           ignore_local=kwargs.get("ignore_local", False))
    manifest.write(m, path)


def abs_path(path):
    return os.path.abspath(os.path.expanduser(path))


def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-p", "--path", type=abs_path, help="Path to manifest file.")
    parser.add_argument(
        "--tests-root", type=abs_path, help="Path to root of tests.")
    parser.add_argument(
        "-r", "--rebuild", action="store_true", default=False,
        help="Force a full rebuild of the manifest.")
    parser.add_argument(
        "--ignore-local", action="store_true", default=False,
        help="Don't include uncommitted local changes in the manifest.")
    parser.add_argument(
        "--url-base", action="store", default="/",
        help="Base url to use as the mount point for tests in this manifest.")
    return parser


def find_top_repo():
    path = here
    rv = None
    while path != "/":
        if vcs.is_git_repo(path):
            rv = path
        path = os.path.abspath(os.path.join(path, os.pardir))

    return rv

def main(default_tests_root=None):
    opts = create_parser().parse_args()

    if opts.tests_root is None:
        tests_root = None
        if default_tests_root is not None:
            tests_root = default_tests_root
        else:
            tests_root = find_top_repo()

        if tests_root is None:
            print >> sys.stderr, """No git repo found; could not determine test root.
Run again with --test-root"""
            sys.exit(1)

        opts.tests_root = tests_root

    if opts.path is None:
        opts.path = os.path.join(opts.tests_root, "MANIFEST.json")

    update_from_cli(**vars(opts))


if __name__ == "__main__":
    main()
