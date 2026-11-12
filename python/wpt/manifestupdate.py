# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import argparse
import hashlib
import os
import subprocess
import sys

from argparse import ArgumentParser
from typing import Mapping, Any
from wptrunner.wptcommandline import TestRoot

from . import SERVO_ROOT, WPT_PATH
from mozlog import commandline

# This must happen after importing from "." since it adds WPT
# tools to the Python system path.
import manifest as wptmanifest

from wptrunner.wptcommandline import get_test_paths, set_from_config
from wptrunner import wptlogging

CONFIG_PATH = os.path.join(WPT_PATH, "config.ini")
MANIFEST_HASH_PATH = os.path.join(SERVO_ROOT, ".wpt", "manifest.sha256")


def create_parser() -> ArgumentParser:
    p = argparse.ArgumentParser()
    p.add_argument("--rebuild", action="store_true", help="Rebuild the manifest from scratch")
    commandline.add_logging_group(p)

    return p


def get_manifest_update_config() -> Any:
    kwargs = {
        "config": CONFIG_PATH,
        "product": "servo",
        "manifest_path": os.path.join(WPT_PATH, "meta"),
        "tests_root": None,
        "metadata_root": None,
    }
    set_from_config(kwargs)
    return kwargs["config"]


def update(
    rebuild: bool = False,
    logger: Any = None,
    **kwargs: Any,
) -> int:
    if not logger:
        logger = wptlogging.setup(kwargs, {"mach": sys.stdout})
    test_paths = get_test_paths(get_manifest_update_config())

    return _update(logger, test_paths, rebuild)


def _update(logger: Any, test_paths: Mapping[str, TestRoot], rebuild: bool) -> int:
    for url_base, paths in test_paths.items():
        manifest_path = os.path.join(paths.metadata_path, "MANIFEST.json")
        cache_subdir = os.path.relpath(os.path.dirname(manifest_path), os.path.dirname(__file__))
        wptmanifest.manifest.load_and_update(
            paths.tests_path,
            manifest_path,
            url_base,
            working_copy=True,
            rebuild=rebuild,
            cache_root=os.path.join(SERVO_ROOT, ".wpt", cache_subdir),
        )
    return 0


def run_git(args: list[str]) -> bytes:
    return subprocess.check_output(["git", "-C", SERVO_ROOT] + args)


def file_hash(path: str) -> bytes:
    try:
        with open(path, "rb") as file:
            return hashlib.file_digest(file, "sha1").digest()
    except FileNotFoundError:
        return b"MISSING"


def get_manifest_hash(test_roots: Mapping[str, TestRoot]) -> str:
    """Calculate the hash for the manifest files from the git rev-parse output of
    the test directories and the hash of all staged and unstaged changes in them."""
    test_paths = [
        os.path.relpath(test_root.tests_path, SERVO_ROOT).replace(os.sep, "/") for test_root in test_roots.values()
    ]

    # This runs `git rev-parse` on all test directories to see if the hash matches
    # the one that we have stored in the digest.
    test_directory_hashes = run_git(["rev-parse"] + [f"HEAD:{test_path}" for test_path in test_paths])
    hasher = hashlib.sha256()
    hasher.update(test_directory_hashes)

    modified_files = run_git(["status", "-z", "--no-renames", "--untracked-files=all", "--"] + test_paths)
    # We are dropping the last entry here because git porcelain always ends with a separator.
    for file in modified_files.split(b"\0")[:-1]:
        path = file[3:].decode()
        hasher.update(path.encode() + b"\0")
        hasher.update(file_hash(os.path.join(SERVO_ROOT, path)) + b"\0")

    return hasher.hexdigest()


def ensure_manifests_up_to_date(logger: Any) -> None:
    """Checks to see if the auto-generated MANIFEST.json files are up to date. If
    they are not, this function will update them."""

    test_roots = get_test_paths(get_manifest_update_config())
    manifest_hash = get_manifest_hash(test_roots)

    all_exist = all(os.path.exists(test_root.manifest_path) for test_root in test_roots.values())
    if all_exist and manifest_hash_matches(manifest_hash):
        return

    print("Updating WPT test manifests...")
    _update(logger, test_roots, rebuild=False)

    os.makedirs(os.path.dirname(MANIFEST_HASH_PATH), exist_ok=True)
    with open(MANIFEST_HASH_PATH, "w") as file:
        file.write(manifest_hash)


def manifest_hash_matches(manifest_hash: str) -> bool:
    """Return true if the manifest hash written to disk matches the given 'manifest_hash'
    and false otherwise"""

    try:
        with open(MANIFEST_HASH_PATH, "r") as file:
            return file.read() == manifest_hash
    except FileNotFoundError:
        return False
