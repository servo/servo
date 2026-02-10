#!/usr/bin/env python3

# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.


# A simple script to use cargo vendor to vendor the dependencies and then create an archive.
import argparse
import os
import shutil
import subprocess
import tempfile


def vendor():
    parser = argparse.ArgumentParser(prog="vendor_servo")
    parser.add_argument("--force", action="store_true")

    args = parser.parse_args()
    if os.path.basename(os.getcwd()) != "servo":
        print("Please call from top level servo directory")
        return

    git_clean = subprocess.run(["git", "clean", "--dry-run", "-x"], capture_output=True)
    if git_clean is not None and not args.force:
        print("git clean -x would have removed some files. These would end up in the archive. Aborting")
        print("run git clean --dry-run -x")
        print("Or run with --force")
        return

    # coying servo into directory
    with tempfile.TemporaryDirectory() as tmpdirname:
        print(f"Copying servo repo to temporary directory {tmpdirname}")
        shutil.copytree(
            ".",
            tmpdirname + "/",
            ignore=shutil.ignore_patterns("*.git", "target", "etc", ".venv"),
            dirs_exist_ok=True,
        )
        os.chdir(tmpdirname)
        # shutil.rmtree('.git/')
        # vendoring crates
        print("Vendoring Crates")
        vendor_process = subprocess.run(["cargo", "vendor", "vendor/"], capture_output=True, encoding="Utf8")
        out = vendor_process.stdout
        print("Modifying cargo")
        with open("Cargo.toml", "a") as toml:
            toml.write("\n")
            toml.writelines(out)
        print("Making archive in /tmp/servo.tar.gz. This might take a while.")
        shutil.make_archive("/tmp/servo", format="gztar", base_dir="./")
    return 0


if __name__ == "__main__":
    vendor()
