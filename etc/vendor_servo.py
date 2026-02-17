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
    parser.add_argument(
        "--filename",
        type=pathlib.Path,
        help="name of the output archive without the extension, e.g. `servo` for `servo.tar.gz`",
    )

    args = parser.parse_args()
    # This script is `etc/vendor_servo.py`, so the grandparent of the filename is the servo root directory.
    servo_root = pathlib.Path(os.path.realpath(__file__)).parent.parent

    git_status = subprocess.run(["git", "status", "--porcelain"], capture_output=True, check=True)
    if len(git_status.stdout) > 0 and not args.force:
        print("git working directory is not clean. Check `git status` or run with --force")
        return

    git_revision = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, check=True)

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
        # Save the git hash into a file so that the archive can be mapped to a git revision.
        with open("GIT_REVISION", "w") as revision_file:
            revision_file.write(git_revision.stdout)
        # vendoring crates
        print("Vendoring Crates")
        vendor_process = subprocess.run(
            ["cargo", "vendor", "--locked", "vendor/", "--versioned-dirs"],
            capture_output=True,
            encoding="utf-8",
            check=True,
        )
        out = vendor_process.stdout
        print("Modifying .cargo/config.toml")
        with open(".cargo/config.toml", "a") as toml:
            toml.write("\n")
            toml.writelines(out)

        file = tempfile.gettempdir() + "/servo"
        print(f"Making archive in {file}.tar.gz")
        shutil.make_archive(file, format="gztar", base_dir="./")
        tmp_file_path = file + ".tar.gz"
        if args.filename is not None:
            name = pathlib.Path(args.filename)
            if name.is_absolute():
                out_filepath = name
            else:
                name = name.with_name(name.name + ".tar.gz")
                out_filepath = servo_root.joinpath(name)
        else:
            out_filepath = servo_root.joinpath("servo.tar.gz")
        print(f"Moving archive to {out_filepath}")
        shutil.move(tmp_file_path, servo_root)
    return


if __name__ == "__main__":
    vendor()
