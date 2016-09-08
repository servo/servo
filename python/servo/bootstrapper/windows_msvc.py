# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import sys
import shutil
from distutils import spawn

from base import BaseBootstrapper
from packages import WINDOWS_MSVC as deps


class WindowsMsvcBootstrapper(BaseBootstrapper):
    '''Bootstrapper for MSVC building on Windows.'''

    def __init__(self, **kwargs):
        BaseBootstrapper.__init__(self, **kwargs)

    def ensure_system_packages(self):
        self.install_system_packages()

    def install_system_packages(self, packages=deps):
        from servo.bootstrap_commands import extract, download_file

        deps_dir = os.path.join(".servo", "msvc-dependencies")
        deps_url = "https://servo-rust.s3.amazonaws.com/msvc-deps/"
        first_run = True

        if self.force:
            if os.path.isdir(deps_dir):
                shutil.rmtree(deps_dir)

        if not os.path.isdir(deps_dir):
            os.makedirs(deps_dir)

        # Read file with installed dependencies, if exist
        installed_deps_file = os.path.join(deps_dir, "installed-dependencies.txt")
        if os.path.exists(installed_deps_file):
            installed_deps = [l.strip() for l in open(installed_deps_file)]
        else:
            installed_deps = []

        # list of dependencies that need to be updated
        update_deps = list(set(packages) - set(installed_deps))

        for dep in packages:
            dep_name = dep.split("-")[0]

            # Don't download CMake if already exists in PATH
            if dep_name == "cmake":
                if spawn.find_executable(dep_name):
                    continue

            dep_dir = os.path.join(deps_dir, dep_name)
            # if not installed or need to be updated
            if not os.path.exists(dep_dir) or dep in update_deps:
                if first_run:
                    print "Installing missing MSVC dependencies..."
                    first_run = False

                dep_version_dir = os.path.join(deps_dir, dep)

                if os.path.exists(dep_version_dir):
                    shutil.rmtree(dep_version_dir)

                dep_zip = dep_version_dir + ".zip"
                if not os.path.isfile(dep_zip):
                    download_file(dep, "%s%s.zip" % (deps_url, dep), dep_zip)

                print "Extracting %s..." % dep,
                extract(dep_zip, deps_dir)
                print "done"

                # Delete directory if exist
                if os.path.exists(dep_dir):
                    shutil.rmtree(dep_dir)
                os.rename(dep_version_dir, dep_dir)

        # Write in installed-dependencies.txt file
        with open(installed_deps_file, 'w') as installed_file:
            for line in packages:
                installed_file.write(line + "\n")

    def install_mobile_android_packages(self):
        sys.exit('We do not support building Android on Windows. Sorry!')
