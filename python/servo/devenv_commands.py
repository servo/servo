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
    @Command('cargo',
             description='Run Cargo',
             category='devenv',
             allow_all_args=True)
    @CommandArgument('params', default=None, nargs='...',
                     help="Command-line arguments to be passed through to Cargo")
    def run(self, params):
        return subprocess.call(["cargo"] + params,
                               env=self.build_env())
