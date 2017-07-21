# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals
from os import path, getcwd, listdir
from time import time

import sys
import urllib2
import json

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd, call
from servo.build_commands import notify_build_done


@CommandProvider
class MachCommands(CommandBase):
    def run_cargo(self, params, geckolib=False, check=False):
        if geckolib:
            self.set_use_stable_rust()
            crate_dir = path.join('ports', 'geckolib')
        else:
            crate_dir = path.join('components', 'servo')

        self.ensure_bootstrapped()
        self.ensure_clobbered()
        env = self.build_env(geckolib=geckolib)

        if not params:
            params = []

        if check:
            params = ['check'] + params

        build_start = time()
        if self.context.topdir == getcwd():
            with cd(crate_dir):
                status = call(['cargo'] + params, env=env)
        else:
            status = call(['cargo'] + params, env=env)
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
            import json
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
                call(["cargo", "update"] + params,
                     env=self.build_env())

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

        return call(["rustc"] + params, env=self.build_env())

    @Command('rustc-geckolib',
             description='Run the Rust compiler with the same compiler version and root crate as build-geckolib',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to rustc")
    def rustc_geckolib(self, params):
        if params is None:
            params = []

        self.set_use_stable_rust()
        self.ensure_bootstrapped()
        env = self.build_env(geckolib=True)

        return call(["rustc"] + params, env=env)

    @Command('rust-root',
             description='Print the path to the root of the Rust compiler',
             category='devenv')
    def rust_root(self):
        print(self.config["tools"]["rust-root"])

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
    @CommandArgument('--master',
                     action='store_true',
                     help='Use the latest commit of the "master" branch')
    def rustup(self, master=False):
        if master:
            url = "https://api.github.com/repos/rust-lang/rust/git/refs/heads/master"
            commit = json.load(urllib2.urlopen(url))["object"]["sha"]
        else:
            import toml
            import re
            url = "https://static.rust-lang.org/dist/channel-rust-nightly.toml"
            version = toml.load(urllib2.urlopen(url))["pkg"]["rustc"]["version"]
            short_commit = re.search("\(([0-9a-f]+) ", version).group(1)
            url = "https://api.github.com/repos/rust-lang/rust/commits/" + short_commit
            commit = json.load(urllib2.urlopen(url))["sha"]
        filename = path.join(self.context.topdir, "rust-commit-hash")
        with open(filename, "w") as f:
            f.write(commit + "\n")

        # Reset self.config["tools"]["rust-root"]
        self._rust_version = None
        self.set_use_stable_rust(False)

        # Reset self.config["tools"]["cargo-root"]
        self._cargo_build_id = None
        self.set_cargo_root()

        self.fetch()

    @Command('fetch',
             description='Fetch Rust, Cargo and Cargo dependencies',
             category='devenv')
    def fetch(self):
        # Fetch Rust and Cargo
        self.ensure_bootstrapped()

        # Fetch Cargo dependencies
        with cd(self.context.topdir):
            call(["cargo", "fetch"], env=self.build_env())

    @Command('wptrunner-upgrade',
             description='upgrade wptrunner.',
             category='devenv')
    def upgrade_wpt_runner(self):
        env = self.build_env()
        with cd(path.join(self.context.topdir, 'tests', 'wpt', 'harness')):
            code = call(["git", "init"], env=env)
            if code:
                return code
            # No need to report an error if this fails, as it will for the first use
            call(["git", "remote", "rm", "upstream"], env=env)
            code = call(
                ["git", "remote", "add", "upstream", "https://github.com/w3c/wptrunner.git"], env=env)
            if code:
                return code
            code = call(["git", "fetch", "upstream"], env=env)
            if code:
                return code
            code = call(["git", "reset", "--hard", "remotes/upstream/master"], env=env)
            if code:
                return code
            code = call(["rm", "-rf", ".git"], env=env)
            if code:
                return code
            return 0
