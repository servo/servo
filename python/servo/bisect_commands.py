# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals

import os
import os.path as path
import subprocess
import time
import tarfile

from shutil import copytree, rmtree, copy2
from servo.util import extract, download_file

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase


def read_file(filename, if_exists=False):
    if if_exists and not path.exists(filename):
        return None
    with open(filename) as f:
        return f.read()


@CommandProvider
class BisectCommands(CommandBase):
    @Command('bisect',
             description='Run Servo using downloaded nightlies',
             category='bisect')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to Servo")
    def bisect(self, params):
        # Run the command
        SCRIPT_PATH = os.path.split(__file__)[0]
        command_to_run = [os.path.abspath(
            os.path.join(SCRIPT_PATH, "..", "..", "mach"))]
        command_to_run.extend(params)

        print("Wrapping bisect around command {}".format(
            " ".join(command_to_run)))
        bisect_start = time.time()
        self.run_once_with_version(command_to_run, "2017-01-01T01-21-52Z")
        bisect_end = time.time()
        print("Bisect completed in {} seconds.".format(
            round(bisect_end - bisect_start, 2)))

    def run_once_with_version(self, command, version):
        fetch_version(version)
        print("Running command {} with version {} ".format(command, version))
        start = time.time()
        proc = subprocess.Popen(command)
        output = proc.communicate()
        if proc.returncode:
            print(
                "Bisection on nightly {} was bad, trying older version".format(version))
        else:
            print("Bisection on nightly {} was good, trying newer version".format(
                "FOOBAR11234"))
        end = time.time()
        print("Command {} with servo version {} completed in {} seconds.".format(
            command, version, round(end - start, 2)))


def get_all_versions(os):
    url = "https://servo-builds.s3.amazonaws.com/?list-type=2&prefix=nightly/linux"


def fetch_version(version):
    # Download from
    cache_folder = "./nightlies/"
    build_name = "-servo-tech-demo"
    extension = ".tar.gz"
    file_name = version + build_name
    destination = cache_folder + file_name + extension
    version_folder = cache_folder + file_name
    source = "https://servo-builds.s3.amazonaws.com/nightly/linux/" + file_name + extension

    if not os.path.exists(cache_folder):
        os.mkdir(cache_folder)

    if os.path.isfile(destination):
        print("The nightly version {} has already been downloaded.".format(version))
    else:
        download_file(file_name,source, destination)

    if os.path.isdir(version_folder):
        print("The version {} has already been extracted.".format(version))
    else:
        print("Extracting to {}...".format(version_folder), end='')
        with tarfile.open(destination, "r") as tar:
            tar.extractall(version_folder)
