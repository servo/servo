# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals

import argparse
import sys
import os
import os.path as path
import subprocess
from distutils.spawn import find_executable
from time import time

from mach.registrar import Registrar
from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase
from wptrunner import wptcommandline
from update import updatecommandline
import tidy


@CommandProvider
class MachCommands(CommandBase):
    def __init__(self, context):
        CommandBase.__init__(self, context)
        if not hasattr(self.context, "built_tests"):
            self.context.built_tests = False

    def ensure_built_tests(self):
        if self.context.built_tests:
            return
        returncode = Registrar.dispatch('build-tests', context=self.context)
        if returncode:
            sys.exit(returncode)
        self.context.built_tests = True

    def find_test(self, prefix):
        target_contents = os.listdir(path.join(
            self.context.topdir, "components", "servo", "target", "debug"))
        for filename in target_contents:
            if filename.startswith(prefix + "-"):
                filepath = path.join(
                    self.context.topdir, "components", "servo",
                    "target", "debug", filename)

                if path.isfile(filepath) and os.access(filepath, os.X_OK):
                    return filepath

    def run_test(self, prefix, args=[]):
        t = self.find_test(prefix)
        if t:
            return subprocess.call([t] + args, env=self.build_env())

    def infer_test_by_dir(self, params):
        maybe_path = path.normpath(params[0])
        mach_command = path.join(self.context.topdir, "mach")
        args = None

        if not path.exists(maybe_path):
            print("%s is not a valid file or directory" % maybe_path)
            return 1

        test_dirs = [
            # path, mach test command, optional flag for path argument
            (path.join("tests", "wpt"), "test-wpt", None),
            (path.join("tests", "ref"), "test-ref", ["--name"]),
        ]

        for test_dir, test_name, path_flag in test_dirs:
            if not path_flag:
                path_flag = []
            if test_dir in maybe_path:
                args = ([mach_command, test_name] + path_flag +
                        [maybe_path] + params[1:])
                break
        else:
            print("%s is not a valid test file or directory" % maybe_path)
            return 1

        return subprocess.call(args, env=self.build_env())

    @Command('test',
             description='Run all Servo tests',
             category='testing')
    @CommandArgument('params', default=None, nargs="...",
                     help="Optionally select test based on "
                          "test file directory")
    def test(self, params):
        if params:
            return self.infer_test_by_dir(params)

        test_start = time()
        for t in ["tidy", "ref", "wpt", "css", "unit"]:
            Registrar.dispatch("test-%s" % t, context=self.context)
        elapsed = time() - test_start

        print("Tests completed in %0.2fs" % elapsed)

    @Command('test-unit',
             description='Run unit tests',
             category='testing')
    @CommandArgument('--package', '-p', default=None, help="Specific package to test")
    @CommandArgument('test_name', nargs=argparse.REMAINDER,
                     help="Only run tests that match this pattern")
    def test_unit(self, test_name=None, package=None):
        if test_name is None:
            test_name = []

        self.ensure_bootstrapped()

        if package:
            packages = [package]
        else:
            packages = os.listdir(path.join(self.context.topdir, "tests", "unit"))

        for crate in packages:
            result = subprocess.call(
                ["cargo", "test", "-p", "%s_tests" % crate] + test_name,
                env=self.build_env(), cwd=self.servo_crate())
            if result != 0:
                return result

    @Command('test-ref',
             description='Run the reference tests',
             category='testing')
    @CommandArgument('--kind', '-k', default=None,
                     help="'cpu' or 'gpu' (default both)")
    @CommandArgument('--name', default=None,
                     help="Only run tests that match this pattern. If the "
                          "path to the ref test directory is included, it "
                          "will automatically be trimmed out.")
    @CommandArgument(
        'servo_params', default=None, nargs=argparse.REMAINDER,
        help="Command-line arguments to be passed through to Servo")
    def test_ref(self, kind=None, name=None, servo_params=None):
        self.ensure_bootstrapped()
        self.ensure_built_tests()

        kinds = ["cpu", "gpu"] if kind is None else [kind]
        test_path = path.join(self.context.topdir, "tests", "ref")
        error = False

        test_start = time()
        for k in kinds:
            print("Running %s reftests..." % k)
            test_args = [k, test_path]
            if name is not None:
                maybe_path = path.normpath(name)
                ref_path = path.join("tests", "ref")

                # Check to see if we were passed something leading with the
                # path to the ref test directory, and trim it so that reftest
                # knows how to filter it.
                if ref_path in maybe_path:
                    test_args.append(path.relpath(maybe_path, ref_path))
                else:
                    test_args.append(name)
            if servo_params is not None:
                test_args += ["--"] + servo_params
            ret = self.run_test("reftest", test_args)
            error = error or ret != 0
        elapsed = time() - test_start

        print("Reference tests completed in %0.2fs" % elapsed)

        if error:
            return 1

    @Command('test-content',
             description='Run the content tests',
             category='testing')
    def test_content(self):
        print("Content tests have been replaced by web-platform-tests under "
              "tests/wpt/mozilla/.")
        return 0

    @Command('test-tidy',
             description='Run the source code tidiness check',
             category='testing')
    def test_tidy(self):
        return tidy.scan()

    @Command('test-wpt-failure',
             description='Run the web platform tests',
             category='testing')
    def test_wpt_failure(self):
        self.ensure_bootstrapped()
        return not subprocess.call([
            "bash",
            path.join("tests", "wpt", "run.sh"),
            "--no-pause-after-test",
            "--include",
            "infrastructure/failing-test.html"
        ], env=self.build_env())

    @Command('test-wpt',
             description='Run the web platform tests',
             category='testing',
             parser=wptcommandline.create_parser())
    @CommandArgument('--release', default=False, action="store_true",
                     help="Run with a release build of servo")
    def test_wpt(self, **kwargs):
        self.ensure_bootstrapped()
        self.ensure_wpt_virtualenv()
        hosts_file_path = path.join('tests', 'wpt', 'hosts')

        os.environ["hosts_file_path"] = hosts_file_path

        run_file = path.abspath(path.join("tests", "wpt", "run_wpt.py"))
        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["run_tests"](**kwargs)

    @Command('update-wpt',
             description='Update the web platform tests',
             category='testing',
             parser=updatecommandline.create_parser())
    def update_wpt(self, **kwargs):
        self.ensure_bootstrapped()
        self.ensure_wpt_virtualenv()
        run_file = path.abspath(path.join("tests", "wpt", "update.py"))
        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["update_tests"](**kwargs)

    @Command('test-jquery',
             description='Run the jQuery test suite',
             category='testing')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Run the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Run the dev build')
    def test_jquery(self, release, dev):
        return self.jquery_test_runner("test", release, dev)

    @Command('update-jquery',
             description='Update the jQuery test suite expected results',
             category='testing')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Run the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Run the dev build')
    def update_jquery(self, release, dev):
        return self.jquery_test_runner("update", release, dev)

    @Command('test-css',
             description='Run the web platform tests',
             category='testing',
             parser=wptcommandline.create_parser())
    @CommandArgument('--release', default=False, action="store_true",
                     help="Run with a release build of servo")
    def test_css(self, **kwargs):
        self.ensure_bootstrapped()
        self.ensure_wpt_virtualenv()

        run_file = path.abspath(path.join("tests", "wpt", "run_css.py"))
        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["run_tests"](**kwargs)

    @Command('update-css',
             description='Update the web platform tests',
             category='testing',
             parser=updatecommandline.create_parser())
    def update_css(self, **kwargs):
        self.ensure_bootstrapped()
        self.ensure_wpt_virtualenv()
        run_file = path.abspath(path.join("tests", "wpt", "update_css.py"))
        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["update_tests"](**kwargs)

    def ensure_wpt_virtualenv(self):
        virtualenv_path = path.join("tests", "wpt", "_virtualenv")
        python = self.get_exec("python2", "python")

        if not os.path.exists(virtualenv_path):
            virtualenv = self.get_exec("virtualenv2", "virtualenv")
            subprocess.check_call([virtualenv, "-p", python, virtualenv_path])

        activate_path = path.join(virtualenv_path, "bin", "activate_this.py")

        execfile(activate_path, dict(__file__=activate_path))

        try:
            import wptrunner  # noqa
            from wptrunner.browsers import servo  # noqa
        except ImportError:
            subprocess.check_call(["pip", "install", "-r",
                                   path.join("tests", "wpt", "harness", "requirements.txt")])
            subprocess.check_call(["pip", "install", "-r",
                                   path.join("tests", "wpt", "harness", "requirements_servo.txt")])
        try:
            import blessings
        except ImportError:
            subprocess.check_call(["pip", "install", "blessings"])

        # This is an unfortunate hack. Because mozlog gets imported by wptcommandline
        # before the virtualenv is initalised it doesn't see the blessings module so we don't
        # get coloured output. Setting the blessings global explicitly fixes that.
        from mozlog.structured.formatters import machformatter
        import blessings  # noqa
        machformatter.blessings = blessings

    def get_exec(self, name, default=None):
        path = find_executable(name)
        if not path:
            return default

        return path

    def jquery_test_runner(self, cmd, release, dev):
        self.ensure_bootstrapped()
        base_dir = path.abspath(path.join("tests", "jquery"))
        jquery_dir = path.join(base_dir, "jquery")
        run_file = path.join(base_dir, "run_jquery.py")

        # Clone the jQuery repository if it doesn't exist
        if not os.path.isdir(jquery_dir):
            subprocess.check_call(
                ["git", "clone", "-b", "servo", "--depth", "1", "https://github.com/servo/jquery", jquery_dir])

        # Run pull in case the jQuery repo was updated since last test run
        subprocess.check_call(
            ["git", "-C", jquery_dir, "pull"])

        # Check that a release servo build exists
        bin_path = path.abspath(self.get_binary_path(release, dev))

        return subprocess.check_call(
            [run_file, cmd, bin_path, base_dir])
