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
import re
import sys
import os
import os.path as path
import subprocess
from collections import OrderedDict
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
    DEFAULT_RENDER_MODE = "cpu"
    HELP_RENDER_MODE = "Value can be 'cpu', 'gpu' or 'both' (default " + DEFAULT_RENDER_MODE + ")"

    def __init__(self, context):
        CommandBase.__init__(self, context)
        if not hasattr(self.context, "built_tests"):
            self.context.built_tests = False

    def ensure_built_tests(self, release=False):
        if self.context.built_tests:
            return
        returncode = Registrar.dispatch(
            'build-tests', context=self.context, release=release)
        if returncode:
            sys.exit(returncode)
        self.context.built_tests = True

    def find_test(self, prefix, release=False):
        build_mode = "release" if release else "debug"
        target_contents = os.listdir(path.join(
            self.get_target_dir(), build_mode))
        for filename in target_contents:
            if filename.startswith(prefix + "-"):
                filepath = path.join(
                    self.get_target_dir(), build_mode, filename)

                if path.isfile(filepath) and os.access(filepath, os.X_OK):
                    return filepath

    def run_test(self, prefix, args=[], release=False):
        t = self.find_test(prefix, release=release)
        if t:
            return subprocess.call([t] + args, env=self.build_env())

    @Command('test',
             description='Run all Servo tests',
             category='testing')
    @CommandArgument('params', default=None, nargs="...",
                     help="Optionally select test based on "
                          "test file directory")
    @CommandArgument('--render-mode', '-rm', default=DEFAULT_RENDER_MODE,
                     help="The render mode to be used on all tests. " +
                          HELP_RENDER_MODE)
    @CommandArgument('--release', default=False, action="store_true",
                     help="Run with a release build of servo")
    def test(self, params, render_mode=DEFAULT_RENDER_MODE, release=False):
        suites = OrderedDict([
            ("tidy", {}),
            ("ref", {"kwargs": {"kind": render_mode},
                     "paths": [path.abspath(path.join("tests", "ref"))],
                     "include_arg": "name"}),
            ("wpt", {"kwargs": {"release": release},
                     "paths": [path.abspath(path.join("tests", "wpt", "web-platform-tests")),
                               path.abspath(path.join("tests", "wpt", "mozilla"))],
                     "include_arg": "include"}),
            ("css", {"kwargs": {"release": release},
                     "paths": [path.abspath(path.join("tests", "wpt", "css-tests"))],
                     "include_arg": "include"}),
            ("unit", {"kwargs": {},
                      "paths": [path.abspath(path.join("tests", "unit"))],
                      "include_arg": "test_name"})
        ])

        suites_by_prefix = {path: k for k, v in suites.iteritems() if "paths" in v for path in v["paths"]}

        selected_suites = OrderedDict()

        if params is None:
            params = suites.keys()

        for arg in params:
            found = False
            if arg in suites and arg not in selected_suites:
                selected_suites[arg] = []
                found = True

            elif os.path.exists(path.abspath(arg)):
                abs_path = path.abspath(arg)
                for prefix, suite in suites_by_prefix.iteritems():
                    if abs_path.startswith(prefix):
                        if suite not in selected_suites:
                            selected_suites[suite] = []
                        selected_suites[suite].append(arg)
                        found = True
                        break

            if not found:
                print("%s is not a valid test path or suite name" % arg)
                return 1

        test_start = time()
        for suite, tests in selected_suites.iteritems():
            props = suites[suite]
            kwargs = props.get("kwargs", {})
            if tests:
                kwargs[props["include_arg"]] = tests

            Registrar.dispatch("test-%s" % suite, context=self.context, **kwargs)

        elapsed = time() - test_start

        print("Tests completed in %0.2fs" % elapsed)

    @Command('test-unit',
             description='Run unit tests',
             category='testing')
    @CommandArgument('--package', '-p', default=None, help="Specific package to test")
    @CommandArgument('test_name', nargs=argparse.REMAINDER,
                     help="Only run tests that match this pattern or file path")
    def test_unit(self, test_name=None, package=None):
        if test_name is None:
            test_name = []

        self.ensure_bootstrapped()

        if package:
            packages = {package}
        else:
            packages = set()

        test_patterns = []
        for test in test_name:
            # add package if 'tests/unit/<package>'
            match = re.search("tests/unit/(\\w+)/?$", test)
            if match:
                packages.add(match.group(1))
            # add package & test if '<package>/<test>', 'tests/unit/<package>/<test>.rs', or similar
            elif re.search("\\w/\\w", test):
                tokens = test.split("/")
                packages.add(tokens[-2])
                test_prefix = tokens[-1]
                if test_prefix.endswith(".rs"):
                    test_prefix = test_prefix[:-3]
                test_prefix += "::"
                test_patterns.append(test_prefix)
            # add test as-is otherwise
            else:
                test_patterns.append(test)

        if not packages:
            packages = set(os.listdir(path.join(self.context.topdir, "tests", "unit")))

        for crate in packages:
            result = subprocess.call(
                ["cargo", "test", "-p", "%s_tests" % crate] + test_patterns,
                env=self.build_env(), cwd=self.servo_crate())
            if result != 0:
                return result

    @Command('test-ref',
             description='Run the reference tests',
             category='testing')
    @CommandArgument('--kind', '-k', default=DEFAULT_RENDER_MODE,
                     help=HELP_RENDER_MODE)
    @CommandArgument('--release', '-r', action='store_true',
                     help='Run with a release build of Servo')
    @CommandArgument('--name', default=None,
                     help="Only run tests that match this pattern. If the "
                          "path to the ref test directory is included, it "
                          "will automatically be trimmed out.")
    @CommandArgument(
        'servo_params', default=None, nargs=argparse.REMAINDER,
        help="Command-line arguments to be passed through to Servo")
    def test_ref(self, kind=DEFAULT_RENDER_MODE, name=None, servo_params=None,
                 release=False):
        self.ensure_bootstrapped()
        self.ensure_built_tests(release=release)
        assert kind is not None, 'kind cannot be None, see help'

        kinds = ["cpu", "gpu"] if kind == 'both' else [kind]
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
            ret = self.run_test("reftest", test_args, release=release)
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
             parser=wptcommandline.create_parser)
    @CommandArgument('--release', default=False, action="store_true",
                     help="Run with a release build of servo")
    def test_wpt(self, **kwargs):
        self.ensure_bootstrapped()
        hosts_file_path = path.join(self.context.topdir, 'tests', 'wpt', 'hosts')

        os.environ["hosts_file_path"] = hosts_file_path

        kwargs["debug"] = not kwargs["release"]

        run_file = path.abspath(path.join(self.context.topdir, "tests", "wpt", "run_wpt.py"))
        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["run_tests"](**kwargs)

    @Command('update-wpt',
             description='Update the web platform tests',
             category='testing',
             parser=updatecommandline.create_parser())
    def update_wpt(self, **kwargs):
        self.ensure_bootstrapped()
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

    @Command('test-dromaeo',
             description='Run the Dromaeo test suite',
             category='testing')
    @CommandArgument('tests', default=["recommended"], nargs="...",
                     help="Specific tests to run")
    @CommandArgument('--release', '-r', action='store_true',
                     help='Run the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Run the dev build')
    def test_dromaeo(self, tests, release, dev):
        return self.dromaeo_test_runner(tests, release, dev)

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
        run_file = path.abspath(path.join("tests", "wpt", "update_css.py"))
        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["update_tests"](**kwargs)

    @Command('compare_dromaeo',
             description='compare outputs of two runs of ./mach test-dromaeo command',
             category='testing')
    @CommandArgument('params', default=None, nargs="...",
                     help=" filepaths of output files of two runs of dromaeo test ")
    def compare_dromaeo(self, params):
        prev_op_filename = params[0]
        cur_op_filename = params[1]
        result = {'Test': [], 'Prev_Time': [], 'Cur_Time': [], 'Difference(%)': []}
        with open(prev_op_filename, 'r') as prev_op, open(cur_op_filename, 'r') as cur_op:
            l1 = prev_op.readline()
            l2 = cur_op.readline()

            while ((l1.find('[dromaeo] Saving...') and l2.find('[dromaeo] Saving...'))):
                l1 = prev_op.readline()
                l2 = cur_op.readline()

            reach = 3
            while (reach > 0):
                l1 = prev_op.readline()
                l2 = cur_op.readline()
                reach -= 1

            while True:
                l1 = prev_op.readline()
                l2 = cur_op.readline()
                if not l1:
                    break
                result['Test'].append(str(l1).split('|')[0].strip())
                result['Prev_Time'].append(float(str(l1).split('|')[1].strip()))
                result['Cur_Time'].append(float(str(l2).split('|')[1].strip()))
                a = float(str(l1).split('|')[1].strip())
                b = float(str(l2).split('|')[1].strip())
                result['Difference(%)'].append(((b - a) / a) * 100)

            width_col1 = max([len(x) for x in result['Test']])
            width_col2 = max([len(str(x)) for x in result['Prev_Time']])
            width_col3 = max([len(str(x)) for x in result['Cur_Time']])
            width_col4 = max([len(str(x)) for x in result['Difference(%)']])

            for p, q, r, s in zip(['Test'], ['First Run'], ['Second Run'], ['Difference(%)']):
                print ("\033[1m" + "{}|{}|{}|{}".format(p.ljust(width_col1), q.ljust(width_col2), r.ljust(width_col3),
                       s.ljust(width_col4)) + "\033[0m" + "\n" + "--------------------------------------------------"
                       + "-------------------------------------------------------------------------")

            for a1, b1, c1, d1 in zip(result['Test'], result['Prev_Time'], result['Cur_Time'], result['Difference(%)']):
                if d1 > 0:
                    print ("\033[91m" + "{}|{}|{}|{}".format(a1.ljust(width_col1),
                           str(b1).ljust(width_col2), str(c1).ljust(width_col3), str(d1).ljust(width_col4)) + "\033[0m")
                elif d1 < 0:
                    print ("\033[92m" + "{}|{}|{}|{}".format(a1.ljust(width_col1),
                           str(b1).ljust(width_col2), str(c1).ljust(width_col3), str(d1).ljust(width_col4)) + "\033[0m")
                else:
                    print ("{}|{}|{}|{}".format(a1.ljust(width_col1), str(b1).ljust(width_col2),
                           str(c1).ljust(width_col3), str(d1).ljust(width_col4)))

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

    def dromaeo_test_runner(self, tests, release, dev):
        self.ensure_bootstrapped()
        base_dir = path.abspath(path.join("tests", "dromaeo"))
        dromaeo_dir = path.join(base_dir, "dromaeo")
        run_file = path.join(base_dir, "run_dromaeo.py")

        # Clone the Dromaeo repository if it doesn't exist
        if not os.path.isdir(dromaeo_dir):
            subprocess.check_call(
                ["git", "clone", "-b", "servo", "--depth", "1", "https://github.com/notriddle/dromaeo", dromaeo_dir])

        # Run pull in case the Dromaeo repo was updated since last test run
        subprocess.check_call(
            ["git", "-C", dromaeo_dir, "pull"])

        # Compile test suite
        subprocess.check_call(
            ["make", "-C", dromaeo_dir, "web"])

        # Check that a release servo build exists
        bin_path = path.abspath(self.get_binary_path(release, dev))

        return subprocess.check_call(
            [run_file, "|".join(tests), bin_path, base_dir])
