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
import pathlib

def vendor():
    parser = argparse.ArgumentParser(prog="vendor_servo")
    parser.add_argument("--force", action="store_true")

    args = parser.parse_args()
    # This script is `etc/vendor_servo.py`, so the grandparent of the filename is the servo root directory.
    servo_root = pathlib.Path(os.path.realpath(__file__)).parent.parent

    git_status = subprocess.run(["git", "status", "--porcelain"], capture_output=True, check=True)
    if len(git_status.stdout) > 0 and not args.force:
        print("git working directory is not clean. Check `git status` or run with --force")
        return

    # copying servo into temporary directory
    with tempfile.TemporaryDirectory() as tmpdirname:
        print(f"Copying servo repo to temporary directory {tmpdirname}")
        shutil.copytree(
            servo_root,
            tmpdirname + "/",
            ignore=shutil.ignore_patterns("*.git", "target", "etc", ".venv"),
            dirs_exist_ok=True,
        )
        os.chdir(tmpdirname)
        # vendoring crates
        print("Vendoring Crates")
        vendor_process = subprocess.run(
            ["cargo", "vendor", "vendor/"], capture_output=True, encoding="utf-8", check=True
        )
        out = vendor_process.stdout
        print("Modifying cargo")
        with open("Cargo.toml", "a") as toml:
            toml.write("\n")
            toml.writelines(out)

        file = tempfile.gettempdir() + "/servo"
        print(f"Making archive in {file}.tar.gz")
        shutil.make_archive(file, format="gztar", base_dir="./")
    return


if __name__ == "__main__":
    vendor()
