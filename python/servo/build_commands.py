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
    @Command('build',
             description='Build Servo',
             category='build')
    @CommandArgument('--target', '-t', default=None)
    def build(self, target):
        self.ensure_bootstrapped()
        
        build_start = time()
        subprocess.check_call(["cargo", "build"], env=self.build_env())
        elapsed = time() - build_start

        print("Build completed in %0.2fs" % elapsed)

    @Command('build-tests',
             description='Build the Servo test suites',
             category='build')
    def build_tests(self):
        self.ensure_bootstrapped()
        subprocess.check_call(["cargo", "test", "--no-run"], env=self.build_env())
