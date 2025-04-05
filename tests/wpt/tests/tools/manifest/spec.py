#!/usr/bin/env python3
import argparse
import os
from typing import Any, Optional, Text

from . import vcs
from .manifest import compute_manifest_spec_items, InvalidCacheError, Manifest, write
from .log import get_logger, enable_debug_logging


here = os.path.dirname(__file__)

wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

logger = get_logger()


def update_spec(tests_root: Text,
                manifest_path: Text,
                url_base: Text,
                cache_root: Optional[Text] = None,
                working_copy: bool = True,
                parallel: bool = True
                ) -> None:

    manifest = Manifest(tests_root, url_base)

    logger.info("Updating SPEC_MANIFEST")
    try:
        tree = vcs.get_tree(tests_root, manifest, manifest_path, cache_root,
                            None, working_copy, True)
        changed = manifest.update(tree, parallel, compute_manifest_spec_items)
    except InvalidCacheError:
        logger.error("Manifest cache in spec.py was invalid.")
        return

    if changed:
        write(manifest, manifest_path)
    tree.dump_caches()


def update_from_cli(**kwargs: Any) -> None:
    tests_root = kwargs["tests_root"]
    path = kwargs["path"]
    assert tests_root is not None

    update_spec(tests_root,
                path,
                kwargs["url_base"],
                cache_root=kwargs["cache_root"],
                parallel=kwargs["parallel"])


def abs_path(path: str) -> str:
    return os.path.abspath(os.path.expanduser(path))


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-v", "--verbose", action="store_true",
        help="Turn on verbose logging")
    parser.add_argument(
        "-p", "--path", type=abs_path, help="Path to manifest file.")
    parser.add_argument(
        "--tests-root", type=abs_path, default=wpt_root, help="Path to root of tests.")
    parser.add_argument(
        "--url-base", default="/",
        help="Base url to use as the mount point for tests in this manifest.")
    parser.add_argument(
        "--cache-root", default=os.path.join(wpt_root, ".wptcache"),
        help="Path in which to store any caches (default <tests_root>/.wptcache/)")
    parser.add_argument(
        "--no-parallel", dest="parallel", action="store_false",
        help="Do not parallelize building the manifest")
    return parser


def run(*args: Any, **kwargs: Any) -> None:
    if kwargs["path"] is None:
        kwargs["path"] = os.path.join(kwargs["tests_root"], "SPEC_MANIFEST.json")
    if kwargs["verbose"]:
        enable_debug_logging()
    update_from_cli(**kwargs)
