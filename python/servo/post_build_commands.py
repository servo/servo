from __future__ import print_function, unicode_literals

import json
import os
import os.path as path
import shutil
import subprocess
import sys
import tarfile
from time import time
import urllib

from mach.registrar import Registrar
from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase

@CommandProvider
class MachCommands(CommandBase):
    @Command('run',
             description='Run Servo',
             category='post-build',
             allow_all_args=True)
    @CommandArgument('params', default=None, nargs='...',
                     help="Command-line arguments to be passed through to Servo")
    def run(self, params):
        subprocess.check_call([path.join("target", "servo")] + params,
                              env=self.build_env())

    @Command('doc',
             description='Generate documentation',
             category='post-build',
             allow_all_args=True)
    @CommandArgument('params', default=None, nargs='...',
                     help="Command-line arguments to be passed through to cargo doc")
    def doc(self, params):
        self.ensure_bootstrapped()
        return subprocess.call(["cargo", "doc"] + params,
                               env=self.build_env())

