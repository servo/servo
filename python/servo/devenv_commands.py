# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals
from os import path, listdir
from time import time

import sys
import urllib2
import json

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd, call, BIN_SUFFIX
from servo.build_commands import notify_build_done
from servo.util import STATIC_RUST_LANG_ORG_DIST, URLOPEN_KWARGS


@CommandProvider
class MachCommands(CommandBase):
    def run_cargo(self, params, geckolib=False, check=False):
        if not params:
            params = []

        self.ensure_bootstrapped()
        self.ensure_clobbered()
        env = self.build_env(geckolib=geckolib)

        if check:
            params = ['check'] + params

        if geckolib:
            # for c in $(cargo --list | tail -$(($(cargo --list | wc -l) - 1))); do
            #   (cargo help $c 2>&1 | grep "\\--package" >/dev/null 2>&1) && echo $c
            # done
            if params and params[0] in [
                'bench', 'build', 'check', 'clean', 'doc', 'fmt', 'pkgid',
                'run', 'rustc', 'rustdoc', 'test', 'update',
            ]:
                params[1:1] = ['--package', 'geckoservo']

            self.set_use_geckolib_toolchain()

        build_start = time()
        status = self.call_rustup_run(["cargo"] + params, env=env)
        elapsed = time() - build_start

        notify_build_done(self.config, elapsed, status == 0)

        if check and status == 0:
            print('Finished checking, binary NOT updated. Consider ./mach build before ./mach run')

        return status

    @Command('cargo',
             description='Run Cargo',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to Cargo")
    def cargo(self, params):
        return self.run_cargo(params)

    @Command('cargo-geckolib',
             description='Run Cargo with the same compiler version and root crate as build-geckolib',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to Cargo")
    def cargo_geckolib(self, params):
        return self.run_cargo(params, geckolib=True)

    @Command('check',
             description='Run "cargo check"',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to cargo check")
    def check(self, params):
        return self.run_cargo(params, check=True)

    @Command('check-geckolib',
             description='Run "cargo check" with the same compiler version and root crate as build-geckolib',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to cargo check")
    def check_geckolib(self, params):
        return self.run_cargo(params, check=True, geckolib=True)

    @Command('cargo-update',
             description='Same as update-cargo',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help='Command-line arguments to be passed through to cargo update')
    @CommandArgument(
        '--package', '-p', default=None,
        help='Updates selected package')
    @CommandArgument(
        '--all-packages', '-a', action='store_true',
        help='Updates all packages')
    @CommandArgument(
        '--dry-run', '-d', action='store_true',
        help='Show outdated packages.')
    def cargo_update(self, params=None, package=None, all_packages=None, dry_run=None):
        self.update_cargo(params, package, all_packages, dry_run)

    @Command('update-cargo',
             description='Update Cargo dependencies',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help='Command-line arguments to be passed through to cargo update')
    @CommandArgument(
        '--package', '-p', default=None,
        help='Updates the selected package')
    @CommandArgument(
        '--all-packages', '-a', action='store_true',
        help='Updates all packages. NOTE! This is very likely to break your ' +
             'working copy, making it impossible to build servo. Only do ' +
             'this if you really know what you are doing.')
    @CommandArgument(
        '--dry-run', '-d', action='store_true',
        help='Show outdated packages.')
    def update_cargo(self, params=None, package=None, all_packages=None, dry_run=None):
        if not params:
            params = []

        if dry_run:
            import toml
            import httplib
            import colorama

            cargo_file = open(path.join(self.context.topdir, "Cargo.lock"))
            content = toml.load(cargo_file)

            packages = {}
            outdated_packages = 0
            conn = httplib.HTTPSConnection("crates.io")
            for package in content.get("package", []):
                if "replace" in package:
                    continue
                source = package.get("source", "")
                if source == r"registry+https://github.com/rust-lang/crates.io-index":
                    version = package["version"]
                    name = package["name"]
                    if not packages.get(name, "") or packages[name] > version:
                        packages[name] = package["version"]
                        conn.request('GET', '/api/v1/crates/{}/versions'.format(package["name"]))
                        r = conn.getresponse()
                        json_content = json.load(r)
                        for v in json_content.get("versions"):
                            if not v.get("yanked"):
                                max_version = v.get("num")
                                break

                        if version != max_version:
                            outdated_packages += 1
                            version_major, version_minor = (version.split("."))[:2]
                            max_major, max_minor = (max_version.split("."))[:2]

                            if version_major == max_major and version_minor == max_minor and "alpha" not in version:
                                msg = "minor update"
                                msg_color = "\033[93m"
                            else:
                                msg = "update, which may contain breaking changes"
                                msg_color = "\033[91m"

                            colorama.init()
                            print("{}Outdated package `{}`, available {}\033[0m".format(msg_color, name, msg),
                                  "\n\tCurrent version: {}".format(version),
                                  "\n\t Latest version: {}".format(max_version))
            conn.close()

            print("\nFound {} outdated packages from crates.io".format(outdated_packages))
        elif package:
            params += ["-p", package]
        elif all_packages:
            params = []
        else:
            print("Please choose package to update with the --package (-p) ")
            print("flag or update all packages with --all-packages (-a) flag")
            sys.exit(1)

        if params or all_packages:
            self.ensure_bootstrapped()

            with cd(self.context.topdir):
                self.call_rustup_run(["cargo", "update"] + params, env=self.build_env())

    @Command('rustc',
             description='Run the Rust compiler',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to rustc")
    def rustc(self, params):
        if params is None:
            params = []

        self.ensure_bootstrapped()
        return self.call_rustup_run(["rustc"] + params, env=self.build_env())

    @Command('rustc-geckolib',
             description='Run the Rust compiler with the same compiler version and root crate as build-geckolib',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to rustc")
    def rustc_geckolib(self, params):
        if params is None:
            params = []

        self.set_use_geckolib_toolchain()
        self.ensure_bootstrapped()
        env = self.build_env(geckolib=True)

        return self.call_rustup_run(["rustc"] + params, env=env)

    @Command('grep',
             description='`git grep` for selected directories.',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to `git grep`")
    def grep(self, params):
        if not params:
            params = []
        # get all directories under tests/
        tests_dirs = listdir('tests')
        # Directories to be excluded under tests/
        excluded_tests_dirs = ['wpt', 'jquery']
        tests_dirs = filter(lambda dir: dir not in excluded_tests_dirs, tests_dirs)
        # Set of directories in project root
        root_dirs = ['components', 'ports', 'python', 'etc', 'resources']
        # Generate absolute paths for directories in tests/ and project-root/
        tests_dirs_abs = [path.join(self.context.topdir, 'tests', s) for s in tests_dirs]
        root_dirs_abs = [path.join(self.context.topdir, s) for s in root_dirs]
        # Absolute paths for all directories to be considered
        grep_paths = root_dirs_abs + tests_dirs_abs
        return call(
            ["git"] + ["grep"] + params + ['--'] + grep_paths + [':(exclude)*.min.js', ':(exclude)*.min.css'],
            env=self.build_env())

    @Command('rustup',
             description='Update the Rust version to latest Nightly',
             category='devenv')
    def rustup(self):
        url = STATIC_RUST_LANG_ORG_DIST + "/channel-rust-nightly-date.txt"
        nightly_date = urllib2.urlopen(url, **URLOPEN_KWARGS).read()
        toolchain = "nightly-" + nightly_date
        filename = path.join(self.context.topdir, "rust-toolchain")
        with open(filename, "w") as f:
            f.write(toolchain + "\n")
        return call(["rustup" + BIN_SUFFIX, "install", toolchain])

    @Command('fetch',
             description='Fetch Rust, Cargo and Cargo dependencies',
             category='devenv')
    def fetch(self):
        self.ensure_bootstrapped()

        with cd(self.context.topdir):
            return self.call_rustup_run(["cargo", "fetch"], env=self.build_env())
