# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# pylint: disable=missing-docstring

import os
import subprocess
import shutil
import sys

from wptrunner.update import setup_logging, WPTUpdate  # noqa: F401
from wptrunner.update.base import exit_unclean  # noqa: F401
from wptrunner import wptcommandline  # noqa: F401

from . import WPT_PATH, update_args_for_legacy_layout
from . import manifestupdate

TEST_ROOT = os.path.join(WPT_PATH, 'tests')
META_ROOTS = [
    os.path.join(WPT_PATH, 'meta'),
    os.path.join(WPT_PATH, 'meta-legacy')
]


def do_sync(**kwargs) -> int:
    last_commit = subprocess.check_output(["git", "log", "-1"])

    # Commits should always be authored by the GitHub Actions bot.
    os.environ["GIT_AUTHOR_NAME"] = "Servo WPT Sync"
    os.environ["GIT_AUTHOR_EMAIL"] = "ghbot+wpt-sync@servo.org"
    os.environ["GIT_COMMITTER_NAME"] = os.environ['GIT_AUTHOR_NAME']
    os.environ["GIT_COMMITTER_EMAIL"] = os.environ['GIT_AUTHOR_EMAIL']

    print("Updating WPT from upstream...")
    run_update(**kwargs)

    if last_commit == subprocess.check_output(["git", "log", "-1"]):
        return 255

    # Update the manifest twice to reach a fixed state
    # (https://github.com/servo/servo/issues/22275).
    print("Updating test manifests...")
    manifestupdate.update(check_clean=False)
    manifestupdate.update(check_clean=False)

    remove_unused_metadata()

    if subprocess.check_call(["git", "commit", "-a", "--amend", "--no-edit", "-q"]) != 0:
        print("Ammending commit failed. Bailing out.")
        return 1

    return 0


def remove_unused_metadata():
    print("Removing unused results...")
    unused_files = []
    unused_dirs = []

    for meta_root in META_ROOTS:
        for base_dir, dir_names, files in os.walk(meta_root):
            # Skip recursing into any directories that were previously found to be missing.
            if base_dir in unused_dirs:
                # Skip processing any subdirectories of known missing directories.
                unused_dirs += [os.path.join(base_dir, x) for x in dir_names]
                continue

            for dir_name in dir_names:
                dir_path = os.path.join(base_dir, dir_name)

                # Skip any known directories that are meta-metadata.
                if dir_name == '.cache':
                    unused_dirs.append(dir_path)
                    continue

                # Turn tests/wpt/meta/foo into tests/wpt/tests/foo.
                test_dir = os.path.join(TEST_ROOT, os.path.relpath(dir_path, meta_root))
                if not os.path.exists(test_dir):
                    unused_dirs.append(dir_path)

            for fname in files:
                # Skip any known files that are meta-metadata.
                if not fname.endswith(".ini") or fname == '__dir__.ini':
                    continue

                # Turn tests/wpt/meta/foo/bar.html.ini into tests/wpt/tests/foo/bar.html.
                test_file = os.path.join(
                    TEST_ROOT, os.path.relpath(base_dir, meta_root), fname[:-4])

                if not os.path.exists(test_file):
                    unused_files.append(os.path.join(base_dir, fname))

    for file in unused_files:
        print(f"    - {file}")
        os.remove(file)
    for directory in unused_dirs:
        print(f"    - {directory}")
        shutil.rmtree(directory)


def update_tests(**kwargs) -> int:
    def set_if_none(args: dict, key: str, value):
        if key not in args or args[key] is None:
            args[key] = value

    set_if_none(kwargs, "config", os.path.join(WPT_PATH, "config.ini"))
    kwargs["product"] = "servo"
    kwargs["store_state"] = False

    wptcommandline.set_from_config(kwargs)
    if hasattr(wptcommandline, 'check_paths'):
        wptcommandline.check_paths(kwargs["test_paths"])

    if kwargs.pop("legacy_layout"):
        update_args_for_legacy_layout(kwargs)

    if kwargs.get('sync', False):
        return do_sync(**kwargs)

    return 0 if run_update(**kwargs) else 1


def run_update(**kwargs) -> bool:
    """Run the update process returning True if the process is successful."""
    logger = setup_logging(kwargs, {"mach": sys.stdout})
    return WPTUpdate(logger, **kwargs).run() != exit_unclean


def create_parser(**_kwargs):
    parser = wptcommandline.create_parser_update()
    parser.add_argument("--legacy-layout", "--layout-2013", "--with-layout-2013",
                        default=False, action="store_true",
                        help="Use expected results for the legacy layout engine")
    return parser
