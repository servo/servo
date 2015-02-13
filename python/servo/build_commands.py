from __future__ import print_function, unicode_literals

import os
import os.path as path
import subprocess
from time import time

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd

def is_headless_build():
    return int(os.getenv('SERVO_HEADLESS', 0)) == 1

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
    @CommandArgument('--debug-mozjs',
                     default=None,
                     action='store_true',
                     help='Enable debug assertions in mozjs')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    @CommandArgument('params', nargs='...',
                     help="Command-line arguments to be passed through to Cargo")
    def build(self, target=None, release=False, jobs=None, android=None,
              verbose=False, debug_mozjs=False, params=None):
        self.ensure_bootstrapped()

        if android is None:
            android = self.config["build"]["android"]

        opts = params or []
        features = []

        if release:
            opts += ["--release"]
        if target:
            opts += ["--target", target]
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if android:
            # Ensure the APK builder submodule has been built first
            apk_builder_dir = "support/android-rs-glue"
            with cd(path.join(apk_builder_dir, "apk-builder")):
                subprocess.call(["cargo", "build"], env=self.build_env())

            opts += ["--target", "arm-linux-androideabi"]

        if debug_mozjs or self.config["build"]["debug-mozjs"]:
            features += ["script/debugmozjs"]

        if is_headless_build():
            opts += ["--no-default-features"]
            features += ["headless"]

        if android:
            features += ["android_glue"]

        if features:
            opts += ["--features", "%s" % ' '.join(features)]

        build_start = time()
        env = self.build_env()
        if android:
            # Build OpenSSL for android
            make_cmd = ["make"]
            if jobs is not None:
                make_cmd += ["-j" + jobs]
            with cd(self.android_support_dir()):
                status = subprocess.call(
                    make_cmd + ["-f", "openssl.makefile"],
                    env=self.build_env())
            env['OPENSSL_PATH'] = path.join(self.android_support_dir(), "openssl-1.0.1k")

        status = subprocess.call(
            ["cargo", "build"] + opts,
            env=env, cwd=self.servo_crate())
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
    @CommandArgument('--release', '-r',
                     action='store_true',
                     help='Build in release mode')
    def build_cef(self, jobs=None, verbose=False, release=False):
        self.ensure_bootstrapped()

        ret = None
        opts = []
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if release:
            opts += ["--release"]

        build_start = time()
        with cd(path.join("ports", "cef")):
            ret = subprocess.call(["cargo", "build"] + opts,
                                  env=self.build_env())
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
        args = ["cargo", "test", "--no-run"]
        if is_headless_build():
            args += ["--no-default-features", "--features", "headless"]
        return subprocess.call(
            args,
            env=self.build_env(), cwd=self.servo_crate())

    @Command('clean',
             description='Clean the build directory.',
             category='build')
    @CommandArgument('--manifest-path',
                     default=None,
                     help='Path to the manifest to the package to clean')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    
    @CommandArgument('params', nargs='...',
                     help="Command-line arguments to be passed through to Cargo")
    def clean(self, manifest_path, params, verbose=False):
        self.ensure_bootstrapped()

        opts = []
        if manifest_path:
            opts += ["--manifest-path", manifest_path]
        if verbose:
            opts += ["-v"]
	opts += params
        return subprocess.call(["cargo", "clean"] + opts,
                               env=self.build_env(), cwd=self.servo_crate())
