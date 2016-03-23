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
import json
from collections import OrderedDict
from time import time

from mach.registrar import Registrar
from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, call, check_call
from wptrunner import wptcommandline
from update import updatecommandline
import tidy

SCRIPT_PATH = os.path.split(__file__)[0]
PROJECT_TOPLEVEL_PATH = os.path.abspath(os.path.join(SCRIPT_PATH, "..", ".."))
WEB_PLATFORM_TESTS_PATH = os.path.join("tests", "wpt", "web-platform-tests")
SERVO_TESTS_PATH = os.path.join("tests", "wpt", "mozilla", "tests")


def create_parser_wpt():
    parser = wptcommandline.create_parser()
    parser.add_argument('--release', default=False, action="store_true",
                        help="Run with a release build of servo")
    parser.add_argument('--chaos', default=False, action="store_true",
                        help="Run under chaos mode in rr until a failure is captured")
    return parser


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
            return call([t] + args, env=self.build_env())

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
    @CommandArgument('--faster', default=False, action="store_true",
                     help="Only check changed files and skip the WPT lint in tidy")
    @CommandArgument('--no-progress', default=False, action="store_true",
                     help="Don't show progress for tidy")
    def test(self, params, render_mode=DEFAULT_RENDER_MODE, release=False, faster=False, no_progress=False):
        suites = OrderedDict([
            ("tidy", {"kwargs": {"faster": faster, "no_progress": no_progress},
                      "include_arg": "include"}),
            ("wpt", {"kwargs": {"release": release},
                     "paths": [path.abspath(path.join("tests", "wpt", "web-platform-tests")),
                               path.abspath(path.join("tests", "wpt", "mozilla"))],
                     "include_arg": "include"}),
            ("css", {"kwargs": {"release": release},
                     "paths": [path.abspath(path.join("tests", "wpt", "css-tests"))],
                     "include_arg": "include"}),
            ("unit", {"kwargs": {},
                      "paths": [path.abspath(path.join("tests", "unit"))],
                      "include_arg": "test_name"}),
            ("compiletest", {"kwargs": {"release": release},
                             "paths": [path.abspath(path.join("tests", "compiletest"))],
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
        properties = json.loads(subprocess.check_output([
            sys.executable,
            path.join(self.context.topdir, "components", "style", "list_properties.py")
        ]))
        assert len(properties) >= 100
        assert "margin-top" in properties
        assert "margin" in properties

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

        args = ["cargo", "test"]
        for crate in packages:
            args += ["-p", "%s_tests" % crate]
        args += test_patterns
        result = call(args, env=self.build_env(), cwd=self.servo_crate())
        if result != 0:
            return result

    @Command('test-compiletest',
             description='Run compiletests',
             category='testing')
    @CommandArgument('--package', '-p', default=None, help="Specific package to test")
    @CommandArgument('test_name', nargs=argparse.REMAINDER,
                     help="Only run tests that match this pattern or file path")
    @CommandArgument('--release', default=False, action="store_true",
                     help="Run with a release build of servo")
    def test_compiletest(self, test_name=None, package=None, release=False):
        if test_name is None:
            test_name = []

        self.ensure_bootstrapped()

        if package:
            packages = {package}
        else:
            packages = set()

        test_patterns = []
        for test in test_name:
            # add package if 'tests/compiletest/<package>'
            match = re.search("tests/compiletest/(\\w+)/?$", test)
            if match:
                packages.add(match.group(1))
            # add package & test if '<package>/<test>', 'tests/compiletest/<package>/<test>.rs', or similar
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
            packages = set(os.listdir(path.join(self.context.topdir, "tests", "compiletest")))

        packages.remove("helper")

        args = ["cargo", "test"]
        for crate in packages:
            args += ["-p", "%s_compiletest" % crate]
        args += test_patterns

        env = self.build_env()
        if release:
            env["BUILD_MODE"] = "release"
            args += ["--release"]
        else:
            env["BUILD_MODE"] = "debug"

        result = call(args, env=env, cwd=self.servo_crate())
        if result != 0:
            return result

    @Command('test-ref',
             description='Run the reference tests',
             category='testing')
    @CommandArgument('params', default=None, nargs=argparse.REMAINDER)
    def test_ref(self, params=None):
        print("Ref tests have been replaced by web-platform-tests under "
              "tests/wpt/mozilla/.")
        return 0

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
    @CommandArgument('--faster', default=False, action="store_true",
                     help="Only check changed files and skip the WPT lint in tidy")
    @CommandArgument('--no-progress', default=False, action="store_true",
                     help="Don't show progress for tidy")
    def test_tidy(self, faster, no_progress):
        return tidy.scan(faster, not no_progress)

    @Command('test-webidl',
             description='Run the WebIDL parser tests',
             category='testing')
    @CommandArgument('--quiet', '-q', default=False, action="store_true",
                     help="Don't print passing tests.")
    @CommandArgument('tests', default=None, nargs="...",
                     help="Specific tests to run, relative to the tests directory")
    def test_webidl(self, quiet, tests):
        self.ensure_bootstrapped()

        test_file_dir = path.abspath(path.join(PROJECT_TOPLEVEL_PATH, "components", "script",
                                               "dom", "bindings", "codegen", "parser"))
        # For the `import WebIDL` in runtests.py
        sys.path.insert(0, test_file_dir)

        run_file = path.abspath(path.join(test_file_dir, "runtests.py"))
        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)

        verbose = not quiet
        return run_globals["run_tests"](tests, verbose)

    @Command('test-wpt-failure',
             description='Run the web platform tests',
             category='testing')
    def test_wpt_failure(self):
        self.ensure_bootstrapped()
        return not call([
            "bash",
            path.join("tests", "wpt", "run.sh"),
            "--no-pause-after-test",
            "--include",
            "infrastructure/failing-test.html"
        ], env=self.build_env())

    @Command('test-wpt',
             description='Run the web platform tests',
             category='testing',
             parser=create_parser_wpt)
    def test_wpt(self, **kwargs):
        self.ensure_bootstrapped()
        hosts_file_path = path.join(self.context.topdir, 'tests', 'wpt', 'hosts')
        os.environ["hosts_file_path"] = hosts_file_path
        run_file = path.abspath(path.join(self.context.topdir, "tests", "wpt", "run_wpt.py"))
        return self.wptrunner(run_file, **kwargs)

    # Helper for test_css and test_wpt:
    def wptrunner(self, run_file, **kwargs):
        os.environ["RUST_BACKTRACE"] = "1"
        kwargs["debug"] = not kwargs["release"]
        if kwargs.pop("chaos"):
            kwargs["debugger"] = "rr"
            kwargs["debugger_args"] = "record --chaos"
            kwargs["repeat_until_unexpected"] = True
            # TODO: Delete rr traces from green test runs?

        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["run_tests"](**kwargs)

    @Command('update-wpt',
             description='Update the web platform tests',
             category='testing',
             parser=updatecommandline.create_parser())
    @CommandArgument('--patch', action='store_true', default=False,
                     help='Create an mq patch or git commit containing the changes')
    def update_wpt(self, patch, **kwargs):
        self.ensure_bootstrapped()
        run_file = path.abspath(path.join("tests", "wpt", "update.py"))
        kwargs["no_patch"] = not patch
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
             parser=create_parser_wpt)
    def test_css(self, **kwargs):
        self.ensure_bootstrapped()
        run_file = path.abspath(path.join("tests", "wpt", "run_css.py"))
        return self.wptrunner(run_file, **kwargs)

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
            check_call(
                ["git", "clone", "-b", "servo", "--depth", "1", "https://github.com/servo/jquery", jquery_dir])

        # Run pull in case the jQuery repo was updated since last test run
        check_call(
            ["git", "-C", jquery_dir, "pull"])

        # Check that a release servo build exists
        bin_path = path.abspath(self.get_binary_path(release, dev))

        return call([run_file, cmd, bin_path, base_dir])

    def dromaeo_test_runner(self, tests, release, dev):
        self.ensure_bootstrapped()
        base_dir = path.abspath(path.join("tests", "dromaeo"))
        dromaeo_dir = path.join(base_dir, "dromaeo")
        run_file = path.join(base_dir, "run_dromaeo.py")

        # Clone the Dromaeo repository if it doesn't exist
        if not os.path.isdir(dromaeo_dir):
            check_call(
                ["git", "clone", "-b", "servo", "--depth", "1", "https://github.com/notriddle/dromaeo", dromaeo_dir])

        # Run pull in case the Dromaeo repo was updated since last test run
        check_call(
            ["git", "-C", dromaeo_dir, "pull"])

        # Compile test suite
        check_call(
            ["make", "-C", dromaeo_dir, "web"])

        # Check that a release servo build exists
        bin_path = path.abspath(self.get_binary_path(release, dev))

        return check_call(
            [run_file, "|".join(tests), bin_path, base_dir])


def create_parser_create():
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("--no-editor", action="store_true",
                   help="Don't try to open the test in an editor")
    p.add_argument("-e", "--editor", action="store", help="Editor to use")
    p.add_argument("--no-run", action="store_true",
                   help="Don't try to update the wpt manifest or open the test in a browser")
    p.add_argument('--release', action="store_true",
                   help="Run with a release build of servo")
    p.add_argument("--long-timeout", action="store_true",
                   help="Test should be given a long timeout (typically 60s rather than 10s,"
                   "but varies depending on environment)")
    p.add_argument("--overwrite", action="store_true",
                   help="Allow overwriting an existing test file")
    p.add_argument("-r", "--reftest", action="store_true",
                   help="Create a reftest rather than a testharness (js) test"),
    p.add_argument("-ref", "--reference", dest="ref", help="Path to the reference file")
    p.add_argument("--mismatch", action="store_true",
                   help="Create a mismatch reftest")
    p.add_argument("--wait", action="store_true",
                   help="Create a reftest that waits until takeScreenshot() is called")
    p.add_argument("path", action="store", help="Path to the test file")
    return p


@CommandProvider
class WebPlatformTestsCreator(CommandBase):
    template_prefix = """<!doctype html>
%(documentElement)s<meta charset="utf-8">
"""
    template_long_timeout = "<meta name=timeout content=long>\n"

    template_body_th = """<title></title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script>

</script>
"""

    template_body_reftest = """<title></title>
<link rel="%(match)s" href="%(ref)s">
"""

    template_body_reftest_wait = """<script src="/common/reftest-wait.js"></script>
"""

    def make_test_file_url(self, absolute_file_path):
        # Make the path relative to the project top-level directory so that
        # we can more easily find the right test directory.
        file_path = os.path.relpath(absolute_file_path, PROJECT_TOPLEVEL_PATH)

        if file_path.startswith(WEB_PLATFORM_TESTS_PATH):
            url = file_path[len(WEB_PLATFORM_TESTS_PATH):]
        elif file_path.startswith(SERVO_TESTS_PATH):
            url = "/mozilla" + file_path[len(SERVO_TESTS_PATH):]
        else:  # This test file isn't in any known test directory.
            return None

        return url.replace(os.path.sep, "/")

    def make_test_and_reference_urls(self, test_path, reference_path):
        test_path = os.path.normpath(os.path.abspath(test_path))
        test_url = self.make_test_file_url(test_path)
        if test_url is None:
            return (None, None)

        if reference_path is None:
            return (test_url, '')
        reference_path = os.path.normpath(os.path.abspath(reference_path))

        # If the reference is in the same directory, the URL can just be the
        # name of the refernce file itself.
        reference_path_parts = os.path.split(reference_path)
        if reference_path_parts[0] == os.path.split(test_path)[0]:
            return (test_url, reference_path_parts[1])
        return (test_url, self.make_test_file_url(reference_path))

    @Command("create-wpt",
             category="testing",
             parser=create_parser_create)
    def run_create(self, **kwargs):
        import subprocess

        test_path = kwargs["path"]
        reference_path = kwargs["ref"]

        if reference_path:
            kwargs["reftest"] = True

        (test_url, reference_url) = self.make_test_and_reference_urls(
            test_path, reference_path)

        if test_url is None:
            print("""Test path %s is not in wpt directories:
tests/wpt/web-platform-tests for tests that may be shared
tests/wpt/mozilla/tests for Servo-only tests""" % test_path)
            return 1

        if reference_url is None:
            print("""Reference path %s is not in wpt directories:
testing/web-platform/tests for tests that may be shared
testing/web-platform/mozilla/tests for Servo-only tests""" % reference_path)
            return 1

        if os.path.exists(test_path) and not kwargs["overwrite"]:
            print("Test path already exists, pass --overwrite to replace")
            return 1

        if kwargs["mismatch"] and not kwargs["reftest"]:
            print("--mismatch only makes sense for a reftest")
            return 1

        if kwargs["wait"] and not kwargs["reftest"]:
            print("--wait only makes sense for a reftest")
            return 1

        args = {"documentElement": "<html class=\"reftest-wait\">\n" if kwargs["wait"] else ""}
        template = self.template_prefix % args
        if kwargs["long_timeout"]:
            template += self.template_long_timeout

        if kwargs["reftest"]:
            args = {"match": "match" if not kwargs["mismatch"] else "mismatch",
                    "ref": reference_url}
            template += self.template_body_reftest % args
            if kwargs["wait"]:
                template += self.template_body_reftest_wait
        else:
            template += self.template_body_th
        with open(test_path, "w") as f:
            f.write(template)

        if kwargs["no_editor"]:
            editor = None
        elif kwargs["editor"]:
            editor = kwargs["editor"]
        elif "VISUAL" in os.environ:
            editor = os.environ["VISUAL"]
        elif "EDITOR" in os.environ:
            editor = os.environ["EDITOR"]
        else:
            editor = None

        if editor:
            proc = subprocess.Popen("%s %s" % (editor, test_path), shell=True)

        if not kwargs["no_run"]:
            p = create_parser_wpt()
            args = ["--manifest-update"]
            if kwargs["release"]:
                args.append("--release")
            args.append(test_path)
            wpt_kwargs = vars(p.parse_args(args))
            self.context.commands.dispatch("test-wpt", self.context, **wpt_kwargs)

        if editor:
            proc.wait()
