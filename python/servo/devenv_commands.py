from __future__ import print_function, unicode_literals

import subprocess

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase


@CommandProvider
class MachCommands(CommandBase):
    @Command('cargo',
             description='Run Cargo',
             category='devenv',
             allow_all_args=True)
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to Cargo")
    def cargo(self, params):
        return subprocess.call(["cargo"] + params,
                               env=self.build_env())

    @Command('rustc',
             description='Run the Rust compiler',
             category='devenv',
             allow_all_args=True)
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
