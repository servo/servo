#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
# Adapted from https://github.com/servo/mozjs/blob/main/mozjs-sys/etc/update.py

import os
import shutil
import subprocess
import tempfile

TARGET = "perf_tests"


def extract_tarball(tarball, commit):
    print("Extracting tarball.")

    if not os.path.exists(tarball):
        raise Exception("Tarball not found at %s" % tarball)

    with tempfile.TemporaryDirectory() as directory:
        subprocess.check_call(["tar", "-xf", tarball, "-C", directory])

        dirname = os.path.dirname(__file__)
        filter_file = os.path.abspath(os.path.join(dirname, "filters.txt"))

        subprocess.check_call(
            [
                "rsync",
                "--delete-excluded",
                f"--filter=merge {filter_file}",
                "--prune-empty-dirs",
                "--quiet",
                "--recursive",
                os.path.join(directory, ""),
                os.path.join(dirname, TARGET, ""),
            ]
        )

    if commit:
        subprocess.check_call(
            ["git", "add", "--all", TARGET], stdout=subprocess.DEVNULL
        )
        subprocess.check_call(
            ["git", "commit", "-s", "-m", "tests: Update blink perf tests."],
            stdout=subprocess.DEVNULL,
        )


def apply_patches():
    print("Applying patches.")
    dirname = os.path.dirname(__file__)
    patch_dir = os.path.abspath(os.path.join(dirname, "patches"))
    patches = sorted(
        os.path.join(patch_dir, p)
        for p in os.listdir(patch_dir)
        if p.endswith(".patch")
    )
    for p in patches:
        print("  Applying patch: %s." % p)
        subprocess.check_call(
            ["git", "apply", "--reject", "--directory=" + TARGET, p],
            stdout=subprocess.DEVNULL,
        )


def main(args):
    extract = None
    patch = True
    commit = True
    for arg in args:
        if arg == "--no-patch":
            patch = False
        elif arg == "--no-commit":
            commit = False
        else:
            extract = arg
    if extract:
        extract_tarball(os.path.abspath(extract), commit)
    if patch:
        apply_patches()


if __name__ == "__main__":
    import sys

    main(sys.argv[1:])
