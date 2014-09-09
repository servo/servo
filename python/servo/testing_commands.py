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
import tidy

@CommandProvider
class MachCommands(CommandBase):
    def __init__(self, context):
        CommandBase.__init__(self, context)
        if not hasattr(self.context, "built_tests"):
            self.context.built_tests = False

    def ensure_built_tests(self):
        if self.context.built_tests: return
        Registrar.dispatch('build-tests', context=self.context)
        self.context.built_tests = True

    def find_test(self, prefix):
        candidates = [f for f in os.listdir(path.join(self.context.topdir, "target"))
                      if f.startswith(prefix + "-")]
        if candidates:
            return path.join(self.context.topdir, "target", candidates[0])
        return None

    def run_test(self, prefix, args=[]):
        t = self.find_test(prefix)
        if t:
            return subprocess.call([t] + args, env=self.build_env())

    @Command('test',
             description='Run all Servo tests',
             category='testing')
    def test(self):
        test_start = time()
        for t in ["tidy", "unit", "ref", "content", "wpt"]:
            Registrar.dispatch("test-%s" % t, context=self.context)
        elapsed = time() - test_start

        print("Tests completed in %0.2fs" % elapsed)

    @Command('test-unit',
             description='Run libservo unit tests',
             category='testing')
    def test_unit(self):
        self.ensure_bootstrapped()
        self.ensure_built_tests()
        return self.run_test("servo")

    @Command('test-ref',
             description='Run the reference tests',
             category='testing')
    @CommandArgument('--kind', '-k', default=None)
    def test_ref(self, kind=None):
        self.ensure_bootstrapped()
        self.ensure_built_tests()

        kinds = ["cpu", "gpu"] if kind is None else [kind]
        test_path = path.join(self.context.topdir, "tests", "ref")
        error = False

        test_start = time()
        for k in kinds:
            print("Running %s reftests..." % k)
            ret = self.run_test("reftest", [k, test_path])
            error = error or ret != 0
        elapsed = time() - test_start

        print("Reference tests completed in %0.2fs" % elapsed)

        if error: return 1

    @Command('test-content',
             description='Run the content tests',
             category='testing')
    def test_content(self):
        self.ensure_bootstrapped()
        self.ensure_built_tests()

        test_path = path.join(self.context.topdir, "tests", "content")
        test_start = time()
        ret = self.run_test("contenttest", ["--source-dir=%s" % test_path])
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
             category='testing',
             allow_all_args=True)
    @CommandArgument('params', default=None, nargs='...',
                     help="Command-line arguments to be passed through to wpt/run.sh")
    def test_wpt(self, params):
        return subprocess.call(["bash", path.join("tests", "wpt", "run.sh")] + params,
                               env=self.build_env())
