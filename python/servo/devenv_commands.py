from __future__ import print_function, unicode_literals
from os import path

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
        return subprocess.call(["cargo"] + params,
                               env=self.build_env())

    @Command('update-cargo',
             description='Update Cargo dependencies',
             category='devenv')
    @CommandArgument(
        'params', default=None, nargs='...',
        help='Command-line arguments to be passed through to cargo update')
    def update_cargo(self, params):
        cargo_paths = [path.join('.'),
                       path.join('ports', 'cef'),
                       path.join('ports', 'android', 'glut_app')]

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
        return subprocess.call(["rustc"] + params, env=self.build_env())

    @Command('rust-root',
             description='Print the path to the root of the Rust compiler',
             category='devenv')
    def rust_root(self):
        print(self.config["tools"]["rust-root"])
