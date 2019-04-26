#!/usr/bin/env python
import argparse
import os

from . import manifest
from . import vcs
from .log import get_logger
from .download import download_from_github

here = os.path.dirname(__file__)

wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

logger = get_logger()


def update(tests_root,
           manifest,
           manifest_path=None,
           working_copy=True,
           cache_root=None,
           rebuild=False):
    logger.warning("Deprecated; use manifest.load_and_update instead")
    logger.info("Updating manifest")

    tree = vcs.get_tree(tests_root, manifest, manifest_path, cache_root,
                        working_copy, rebuild)
    return manifest.update(tree)


def update_from_cli(**kwargs):
    tests_root = kwargs["tests_root"]
    path = kwargs["path"]
    assert tests_root is not None

    if not kwargs["rebuild"] and kwargs["download"]:
        download_from_github(path, tests_root)

    manifest.load_and_update(tests_root,
                             path,
                             kwargs["url_base"],
                             update=True,
                             rebuild=kwargs["rebuild"],
                             cache_root=kwargs["cache_root"])


def abs_path(path):
    return os.path.abspath(os.path.expanduser(path))


def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-p", "--path", type=abs_path, help="Path to manifest file.")
    parser.add_argument(
        "--tests-root", type=abs_path, default=wpt_root, help="Path to root of tests.")
    parser.add_argument(
        "-r", "--rebuild", action="store_true", default=False,
        help="Force a full rebuild of the manifest.")
    parser.add_argument(
        "--url-base", action="store", default="/",
        help="Base url to use as the mount point for tests in this manifest.")
    parser.add_argument(
        "--no-download", dest="download", action="store_false", default=True,
        help="Never attempt to download the manifest.")
    parser.add_argument(
        "--cache-root", action="store", default=os.path.join(wpt_root, ".wptcache"),
        help="Path in which to store any caches (default <tests_root>/.wptcache/")
    return parser


def find_top_repo():
    path = here
    rv = None
    while path != "/":
        if vcs.is_git_repo(path):
            rv = path
        path = os.path.abspath(os.path.join(path, os.pardir))

    return rv


def run(*args, **kwargs):
    if kwargs["path"] is None:
        kwargs["path"] = os.path.join(kwargs["tests_root"], "MANIFEST.json")
    update_from_cli(**kwargs)


def main():
    opts = create_parser().parse_args()

    run(**vars(opts))
