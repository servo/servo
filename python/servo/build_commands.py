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

from servo.command_base import CommandBase, cd

@CommandProvider
class MachCommands(CommandBase):
    @Command('build',
             description='Build Servo',
             category='build')
    @CommandArgument('--target', '-t',
                     default=None,
                     help='Cross compile for given target platform')
    @CommandArgument('--release', '-r',
                     action='store_true',
                     help='Build in release mode')
    @CommandArgument('--jobs', '-j',
                     default=None,
                     help='Number of jobs to run in parallel')
    def build(self, target, release=False, jobs=None):
        self.ensure_bootstrapped()

        opts = []
        if release:
            opts += ["--release"]
        if target:
            opts += ["--target", target]
        if jobs is not None:
            opts += ["-j", jobs]

        build_start = time()
        subprocess.check_call(["cargo", "build"] + opts, env=self.build_env())
        elapsed = time() - build_start

        print("Build completed in %0.2fs" % elapsed)

    @Command('build-cef',
             description='Build the Chromium Embedding Framework library',
             category='build')
    @CommandArgument('--jobs', '-j',
                     default=None,
                     help='Number of jobs to run in parallel')
    def build_cef(self, jobs=None):
        self.ensure_bootstrapped()

        ret = None
        opts = []
        if jobs is not None:
            opts += ["-j", jobs]

        build_start = time()
        with cd(path.join("ports", "cef")):
            ret = subprocess.call(["cargo", "build"], env=self.build_env())
        elapsed = time() - build_start

        print("CEF build completed in %0.2fs" % elapsed)

        return ret

    @Command('build-tests',
             description='Build the Servo test suites',
             category='build')
    @CommandArgument('--jobs', '-j',
                     default=None,
                     help='Number of jobs to run in parallel')
    def build_tests(self, jobs=None):
        self.ensure_bootstrapped()
        opts = []
        if jobs is not None:
            opts += ["-j", jobs]
        subprocess.check_call(["cargo", "test", "--no-run"], env=self.build_env())
