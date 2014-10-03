from __future__ import print_function, unicode_literals

import os.path as path
import subprocess
from time import time

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
    @CommandArgument('--android',
                     default=None,
                     action='store_true',
                     help='Build for Android')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    def build(self, target=None, release=False, jobs=None, android=None,
              verbose=False):
        self.ensure_bootstrapped()

        if android is None:
            android = self.config["build"]["android"]

        opts = []
        if release:
            opts += ["--release"]
        if target:
            opts += ["--target", target]
        elif android:
            opts += ["--target", "arm-linux-androideabi"]
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]

        build_start = time()
        status = subprocess.call(
            ["cargo", "build"] + opts,
            env=self.build_env())
        if android:
            status = status or subprocess.call(
                ["make", "-C", "ports/android"],
                env=self.build_env())
        elapsed = time() - build_start

        print("Build completed in %0.2fs" % elapsed)
        return status

    @Command('build-cef',
             description='Build the Chromium Embedding Framework library',
             category='build')
    @CommandArgument('--jobs', '-j',
                     default=None,
                     help='Number of jobs to run in parallel')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    def build_cef(self, jobs=None, verbose=False):
        self.ensure_bootstrapped()

        ret = None
        opts = []
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]

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
        return subprocess.call(
            ["cargo", "test", "--no-run"], env=self.build_env())

    @Command('clean',
             description='Clean the build directory.',
             category='build')
    @CommandArgument('--manifest-path',
                     default=None,
                     help='Path to the manifest to the package to clean')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    def clean(self, manifest_path, verbose=False):
        self.ensure_bootstrapped()

        opts = []
        if manifest_path:
            opts += ["--manifest-path", manifest_path]
        if verbose:
            opts += ["-v"]

        return subprocess.call(["cargo", "clean"] + opts, env=self.build_env())
