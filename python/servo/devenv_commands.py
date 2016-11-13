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

import subprocess
import sys

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd, call

CARGO_PATHS = [
    path.join('components', 'servo'),
    path.join('ports', 'cef'),
    path.join('ports', 'geckolib'),
]


@CommandProvider
class MachCommands(CommandBase):
    @Command('cargo',
             description='Run Cargo',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to Cargo")
    def cargo(self, params):
        if not params:
            params = []

        if self.context.topdir == getcwd():
            with cd(path.join('components', 'servo')):
                return call(["cargo"] + params, env=self.build_env())
        return call(['cargo'] + params, env=self.build_env())

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
    def cargo_update(self, params=None, package=None, all_packages=None):
        self.update_cargo(params, package, all_packages)

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
    def update_cargo(self, params=None, package=None, all_packages=None):
        if not params:
            params = []

        if package:
            params += ["-p", package]
        elif all_packages:
            params = []
        else:
            print("Please choose package to update with the --package (-p) ")
            print("flag or update all packages with --all-packages (-a) flag")
            sys.exit(1)

        self.ensure_bootstrapped()

        for cargo_path in CARGO_PATHS:
            with cd(cargo_path):
                print(cargo_path)
                call(["cargo", "update"] + params,
                     env=self.build_env())

    @Command('clippy',
             description='Run Clippy',
             category='devenv')
    @CommandArgument(
        '--package', '-p', default=None,
        help='Updates the selected package')
    @CommandArgument(
        '--json', '-j', action="store_true",
        help='Outputs')
    def clippy(self, package=None, json=False):
        params = ["--features=clippy"]
        if package:
            params += ["-p", package]
        if json:
            params += ["--", "-Zunstable-options", "--error-format", "json"]

        with cd(path.join(self.context.topdir, "components", "servo")):
            return subprocess.call(["cargo", "rustc", "-v"] + params,
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
        return call(["rustc"] + params, env=self.build_env())

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

    @Command('fetch',
             description='Fetch Rust, Cargo and Cargo dependencies',
             category='devenv')
    def fetch(self):
        # Fetch Rust and Cargo
        self.ensure_bootstrapped()

        # Fetch Cargo dependencies
        for cargo_path in CARGO_PATHS:
            with cd(cargo_path):
                print(cargo_path)
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
