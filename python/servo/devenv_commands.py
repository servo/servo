from __future__ import print_function, unicode_literals
from os import path, getcwd, listdir

import subprocess

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd


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
                return subprocess.call(["cargo"] + params,
                               env=self.build_env())
        return subprocess.call(['cargo'] + params,
                               env=self.build_env())

    @Command('cargo-update',
             description='Same as update-cargo',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help='Command-line arguments to be passed through to cargo update')
    def cargo_update(self, params=None):
        self.update_cargo(params)

    @Command('update-cargo',
             description='Update Cargo dependencies',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help='Command-line arguments to be passed through to cargo update')
    def update_cargo(self, params=None):
        if not params:
            params = []

        cargo_paths = [path.join('components', 'servo'),
                       path.join('ports', 'cef'),
                       path.join('ports', 'gonk')]

        for cargo_path in cargo_paths:
            with cd(cargo_path):
                print(cargo_path)
                subprocess.call(["cargo", "update"] + params,
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
        return subprocess.call(["rustc"] + params, env=self.build_env())

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
        # Remove 'wpt' from obtained dir list
        tests_dirs = filter(lambda dir: dir != 'wpt', tests_dirs)
        # Set of directories in project root
        root_dirs = ['components', 'ports', 'python', 'etc', 'resources']
        # Generate absolute paths for directories in tests/ and project-root/
        tests_dirs_abs = [path.join(self.context.topdir, 'tests', s) for s in tests_dirs]
        root_dirs_abs = [path.join(self.context.topdir, s) for s in root_dirs]
        # Absolute paths for all directories to be considered
        grep_paths = root_dirs_abs + tests_dirs_abs
        return subprocess.call(["git"] + ["grep"] + params + ['--'] + grep_paths, env=self.build_env())
