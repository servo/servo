from __future__ import print_function, unicode_literals

import os
import os.path as path
import subprocess
from time import time

from mach.registrar import Registrar
from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase
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
        Registrar.dispatch('build-tests', context=self.context)
        self.context.built_tests = True

    def find_test(self, prefix):
        target_contents = os.listdir(path.join(self.context.topdir, "target"))
        for filename in target_contents:
            if filename.startswith(prefix + "-"):
                filepath = path.join(self.context.topdir, "target", filename)
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
            (path.join("tests", "content"), "test-content"),
            (path.join("tests", "wpt"), "test-wpt"),
        ]

        if path.join("tests", "ref") in maybe_path:
            # test-ref is the outcast here in that it does not accept
            # individual files as arguments.
            args = [mach_command, "test-ref"] + params[1:]
        else:
            for test_dir, test_name in test_dirs:
                if test_dir in maybe_path:
                    args = [mach_command, test_name, maybe_path] + params[1:]
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
        for t in ["tidy", "unit", "ref", "content", "wpt"]:
            Registrar.dispatch("test-%s" % t, context=self.context)
        elapsed = time() - test_start

        print("Tests completed in %0.2fs" % elapsed)

    @Command('test-unit',
             description='Run unit tests',
             category='testing')
    @CommandArgument('test_name', default=None, nargs="...",
                     help="Only run tests that match this pattern")
    def test_unit(self, test_name=None):
        if test_name is None:
            test_name = []
        self.ensure_bootstrapped()
        self.ensure_built_tests()

        ret = self.run_test("servo", test_name) != 0

        def cargo_test(component):
            return 0 != subprocess.call(
                ["cargo", "test", "-p", component], env=self.build_env())

        for component in os.listdir("components"):
            ret = ret or cargo_test(component)

        return ret

    @Command('test-ref',
             description='Run the reference tests',
             category='testing')
    @CommandArgument('--kind', '-k', default=None,
                     help="'cpu' or 'gpu' (default both)")
    @CommandArgument('test_name', default=None, nargs="?",
                     help="Only run tests that match this pattern")
    @CommandArgument(
        'servo_params', default=None, nargs="...",
        help="Command-line arguments to be passed through to Servo")
    def test_ref(self, kind=None, test_name=None, servo_params=None):
        self.ensure_bootstrapped()
        self.ensure_built_tests()

        kinds = ["cpu", "gpu"] if kind is None else [kind]
        test_path = path.join(self.context.topdir, "tests", "ref")
        error = False

        test_start = time()
        for k in kinds:
            print("Running %s reftests..." % k)
            test_args = [k, test_path]
            if test_name is not None:
                test_args.append(test_name)
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
    @CommandArgument('test_name', default=None, nargs="?",
                     help="Only run tests that match this pattern")
    def test_content(self, test_name=None):
        self.ensure_bootstrapped()
        self.ensure_built_tests()

        test_path = path.join(self.context.topdir, "tests", "content")
        test_args = ["--source-dir=%s" % test_path]

        if test_name is not None:
            test_args.append(test_name)

        test_start = time()
        ret = self.run_test("contenttest", test_args)
        elapsed = time() - test_start

        print("Content tests completed in %0.2fs" % elapsed)
        return ret

    @Command('test-tidy',
             description='Run the source code tidiness check',
             category='testing')
    def test_tidy(self):
        return tidy.scan()

    @Command('test-wpt',
             description='Run the web platform tests',
             category='testing')
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to wpt/run.sh")
    def test_wpt(self, params=None):
        if params is None:
            params = []
        else:
            # Allow the first argument to be a relative path from Servo's root
            # directory, converting it to `--include some/wpt/test.html`
            maybe_path = path.normpath(params[0])
            wpt_path = path.join('tests', 'wpt', 'web-platform-tests')

            if path.exists(maybe_path) and wpt_path in maybe_path:
                params = ["--include",
                          path.relpath(maybe_path, wpt_path)] + params[1:]

        return subprocess.call(
            ["bash", path.join("tests", "wpt", "run.sh")] + params,
            env=self.build_env())
