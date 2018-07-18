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
import platform
import copy
from collections import OrderedDict
import time
import json
import urllib2
import urllib
import base64
import shutil
import subprocess

from mach.registrar import Registrar
from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import (
    BuildNotFound, CommandBase,
    call, check_call, set_osmesa_env,
)
from servo.util import host_triple

from wptrunner import wptcommandline
from update import updatecommandline
from servo_tidy import tidy
from servo_tidy_tests import test_tidy

SCRIPT_PATH = os.path.split(__file__)[0]
PROJECT_TOPLEVEL_PATH = os.path.abspath(os.path.join(SCRIPT_PATH, "..", ".."))
WEB_PLATFORM_TESTS_PATH = os.path.join("tests", "wpt", "web-platform-tests")
SERVO_TESTS_PATH = os.path.join("tests", "wpt", "mozilla", "tests")

TEST_SUITES = OrderedDict([
    ("tidy", {"kwargs": {"all_files": False, "no_progress": False, "self_test": False,
                         "stylo": False},
              "include_arg": "include"}),
    ("wpt", {"kwargs": {"release": False},
             "paths": [path.abspath(WEB_PLATFORM_TESTS_PATH),
                       path.abspath(SERVO_TESTS_PATH)],
             "include_arg": "include"}),
    ("unit", {"kwargs": {},
              "paths": [path.abspath(path.join("tests", "unit"))],
              "include_arg": "test_name"}),
])

TEST_SUITES_BY_PREFIX = {path: k for k, v in TEST_SUITES.iteritems() if "paths" in v for path in v["paths"]}


def create_parser_wpt():
    parser = wptcommandline.create_parser()
    parser.add_argument('--release', default=False, action="store_true",
                        help="Run with a release build of servo")
    parser.add_argument('--rr-chaos', default=False, action="store_true",
                        help="Run under chaos mode in rr until a failure is captured")
    parser.add_argument('--pref', default=[], action="append", dest="prefs",
                        help="Pass preferences to servo")
    parser.add_argument('--always-succeed', default=False, action="store_true",
                        help="Always yield exit code of zero")
    return parser


def create_parser_manifest_update():
    import manifestupdate
    return manifestupdate.create_parser()


def run_update(topdir, check_clean=False, rebuild=False, **kwargs):
    import manifestupdate
    from wptrunner import wptlogging
    logger = wptlogging.setup(kwargs, {"mach": sys.stdout})
    wpt_dir = os.path.abspath(os.path.join(topdir, 'tests', 'wpt'))
    return manifestupdate.update(logger, wpt_dir, check_clean, rebuild)


@CommandProvider
class MachCommands(CommandBase):
    DEFAULT_RENDER_MODE = "cpu"
    HELP_RENDER_MODE = "Value can be 'cpu', 'gpu' or 'both' (default " + DEFAULT_RENDER_MODE + ")"

    def __init__(self, context):
        CommandBase.__init__(self, context)
        if not hasattr(self.context, "built_tests"):
            self.context.built_tests = False

    @Command('test',
             description='Run specified Servo tests',
             category='testing')
    @CommandArgument('params', default=None, nargs="...",
                     help="Optionally select test based on "
                          "test file directory")
    @CommandArgument('--render-mode', '-rm', default=DEFAULT_RENDER_MODE,
                     help="The render mode to be used on all tests. " +
                          HELP_RENDER_MODE)
    @CommandArgument('--release', default=False, action="store_true",
                     help="Run with a release build of servo")
    @CommandArgument('--tidy-all', default=False, action="store_true",
                     help="Check all files, and run the WPT lint in tidy, "
                          "even if unchanged")
    @CommandArgument('--no-progress', default=False, action="store_true",
                     help="Don't show progress for tidy")
    @CommandArgument('--self-test', default=False, action="store_true",
                     help="Run unit tests for tidy")
    @CommandArgument('--all', default=False, action="store_true", dest="all_suites",
                     help="Run all test suites")
    def test(self, params, render_mode=DEFAULT_RENDER_MODE, release=False, tidy_all=False,
             no_progress=False, self_test=False, all_suites=False):
        suites = copy.deepcopy(TEST_SUITES)
        suites["tidy"]["kwargs"] = {"all_files": tidy_all, "no_progress": no_progress, "self_test": self_test,
                                    "stylo": False}
        suites["wpt"]["kwargs"] = {"release": release}
        suites["unit"]["kwargs"] = {}

        selected_suites = OrderedDict()

        if params is None:
            if all_suites:
                params = suites.keys()
            else:
                print("Specify a test path or suite name, or pass --all to run all test suites.\n\nAvailable suites:")
                for s in suites:
                    print("    %s" % s)
                return 1

        for arg in params:
            found = False
            if arg in suites and arg not in selected_suites:
                selected_suites[arg] = []
                found = True
            else:
                suite = self.suite_for_path(arg)
                if suite is not None:
                    if suite not in selected_suites:
                        selected_suites[suite] = []
                    selected_suites[suite].append(arg)
                    found = True
                    break

            if not found:
                print("%s is not a valid test path or suite name" % arg)
                return 1

        test_start = time.time()
        for suite, tests in selected_suites.iteritems():
            props = suites[suite]
            kwargs = props.get("kwargs", {})
            if tests:
                kwargs[props["include_arg"]] = tests

            Registrar.dispatch("test-%s" % suite, context=self.context, **kwargs)

        elapsed = time.time() - test_start

        print("Tests completed in %0.2fs" % elapsed)

    # Helper to determine which test suite owns the path
    def suite_for_path(self, path_arg):
        if os.path.exists(path.abspath(path_arg)):
            abs_path = path.abspath(path_arg)
            for prefix, suite in TEST_SUITES_BY_PREFIX.iteritems():
                if abs_path.startswith(prefix):
                    return suite
        return None

    @Command('test-perf',
             description='Run the page load performance test',
             category='testing')
    @CommandArgument('--base', default=None,
                     help="the base URL for testcases")
    @CommandArgument('--date', default=None,
                     help="the datestamp for the data")
    @CommandArgument('--submit', '-a', default=False, action="store_true",
                     help="submit the data to perfherder")
    def test_perf(self, base=None, date=None, submit=False):
        self.set_software_rendering_env(True)

        self.ensure_bootstrapped()
        env = self.build_env()
        cmd = ["bash", "test_perf.sh"]
        if base:
            cmd += ["--base", base]
        if date:
            cmd += ["--date", date]
        if submit:
            cmd += ["--submit"]
        return call(cmd,
                    env=env,
                    cwd=path.join("etc", "ci", "performance"))

    @Command('test-unit',
             description='Run unit tests',
             category='testing')
    @CommandArgument('test_name', nargs=argparse.REMAINDER,
                     help="Only run tests that match this pattern or file path")
    @CommandArgument('--package', '-p', default=None, help="Specific package to test")
    @CommandArgument('--bench', default=False, action="store_true",
                     help="Run in bench mode")
    @CommandArgument('--nocapture', default=False, action="store_true",
                     help="Run tests with nocapture ( show test stdout )")
    def test_unit(self, test_name=None, package=None, bench=False, nocapture=False):
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

        self_contained_tests = [
            "gfx",
            "layout",
            "msg",
            "net",
            "net_traits",
            "selectors",
            "servo_config",
            "servo_remutex",
        ]
        if not packages:
            packages = set(os.listdir(path.join(self.context.topdir, "tests", "unit"))) - set(['.DS_Store'])
            packages |= set(self_contained_tests)

        in_crate_packages = []
        for crate in self_contained_tests:
            try:
                packages.remove(crate)
                in_crate_packages += [crate]
            except KeyError:
                pass

        packages.discard('stylo')

        env = self.build_env(test_unit=True)
        env["RUST_BACKTRACE"] = "1"

        if "msvc" in host_triple():
            # on MSVC, we need some DLLs in the path. They were copied
            # in to the servo.exe build dir, so just point PATH to that.
            env["PATH"] = "%s%s%s" % (path.dirname(self.get_binary_path(False, False)), os.pathsep, env["PATH"])

        features = self.servo_features()
        if len(packages) > 0 or len(in_crate_packages) > 0:
            args = ["cargo", "bench" if bench else "test", "--manifest-path", self.ports_servo_manifest()]
            for crate in packages:
                args += ["-p", "%s_tests" % crate]
            for crate in in_crate_packages:
                args += ["-p", crate]
            args += test_patterns

            if features:
                args += ["--features", "%s" % ' '.join(features)]

            if nocapture:
                args += ["--", "--nocapture"]

            err = self.call_rustup_run(args, env=env)
            if err is not 0:
                return err

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
    @CommandArgument('--all', default=False, action="store_true", dest="all_files",
                     help="Check all files, and run the WPT lint in tidy, "
                          "even if unchanged")
    @CommandArgument('--no-progress', default=False, action="store_true",
                     help="Don't show progress for tidy")
    @CommandArgument('--self-test', default=False, action="store_true",
                     help="Run unit tests for tidy")
    @CommandArgument('--stylo', default=False, action="store_true",
                     help="Only handle files in the stylo tree")
    def test_tidy(self, all_files, no_progress, self_test, stylo):
        if self_test:
            return test_tidy.do_tests()
        else:
            manifest_dirty = run_update(self.context.topdir, check_clean=True)
            tidy_failed = tidy.scan(not all_files, not no_progress, stylo=stylo)
            return tidy_failed or manifest_dirty

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
             description='Run the tests harness that verifies that the test failures are reported correctly',
             category='testing',
             parser=create_parser_wpt)
    def test_wpt_failure(self, **kwargs):
        self.ensure_bootstrapped()
        kwargs["pause_after_test"] = False
        kwargs["include"] = ["infrastructure/failing-test.html"]
        return not self._test_wpt(**kwargs)

    @Command('test-wpt',
             description='Run the regular web platform test suite',
             category='testing',
             parser=create_parser_wpt)
    def test_wpt(self, **kwargs):
        self.ensure_bootstrapped()
        ret = self.run_test_list_or_dispatch(kwargs["test_list"], "wpt", self._test_wpt, **kwargs)
        if kwargs["always_succeed"]:
            return 0
        else:
            return ret

    def _test_wpt(self, **kwargs):
        hosts_file_path = path.join(self.context.topdir, 'tests', 'wpt', 'hosts')
        os.environ["hosts_file_path"] = hosts_file_path
        run_file = path.abspath(path.join(self.context.topdir, "tests", "wpt", "run_wpt.py"))
        return self.wptrunner(run_file, **kwargs)

    # Helper to ensure all specified paths are handled, otherwise dispatch to appropriate test suite.
    def run_test_list_or_dispatch(self, requested_paths, correct_suite, correct_function, **kwargs):
        if not requested_paths:
            return correct_function(**kwargs)
        # Paths specified on command line. Ensure they can be handled, re-dispatch otherwise.
        all_handled = True
        for test_path in requested_paths:
            suite = self.suite_for_path(test_path)
            if suite is not None and correct_suite != suite:
                all_handled = False
                print("Warning: %s is not a %s test. Delegating to test-%s." % (test_path, correct_suite, suite))
        if all_handled:
            return correct_function(**kwargs)
        # Dispatch each test to the correct suite via test()
        Registrar.dispatch("test", context=self.context, params=requested_paths)

    # Helper for test_css and test_wpt:
    def wptrunner(self, run_file, **kwargs):
        self.set_software_rendering_env(kwargs['release'])

        # By default, Rayon selects the number of worker threads
        # based on the available CPU count. This doesn't work very
        # well when running tests on CI, since we run so many
        # Servo processes in parallel. The result is a lot of
        # extra timeouts. Instead, force Rayon to assume we are
        # running on a 2 CPU environment.
        os.environ['RAYON_RS_NUM_CPUS'] = "2"

        os.environ["RUST_BACKTRACE"] = "1"
        kwargs["debug"] = not kwargs["release"]
        if kwargs.pop("rr_chaos"):
            kwargs["debugger"] = "rr"
            kwargs["debugger_args"] = "record --chaos"
            kwargs["repeat_until_unexpected"] = True
            # TODO: Delete rr traces from green test runs?
        prefs = kwargs.pop("prefs")
        if prefs:
            binary_args = []
            for pref in prefs:
                binary_args.append("--pref=" + pref)
            kwargs["binary_args"] = binary_args

        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["run_tests"](**kwargs)

    @Command('update-manifest',
             description='Run test-wpt --manifest-update SKIP_TESTS to regenerate MANIFEST.json',
             category='testing',
             parser=create_parser_manifest_update)
    def update_manifest(self, **kwargs):
        return run_update(self.context.topdir, **kwargs)

    @Command('update-wpt',
             description='Update the web platform tests',
             category='testing',
             parser=updatecommandline.create_parser())
    def update_wpt(self, **kwargs):
        self.ensure_bootstrapped()
        run_file = path.abspath(path.join("tests", "wpt", "update.py"))
        patch = kwargs.get("patch", False)

        if not patch and kwargs["sync"]:
            print("Are you sure you don't want a patch?")
            return 1

        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["update_tests"](**kwargs)

    @Command('filter-intermittents',
             description='Given a WPT error summary file, filter out intermittents and other cruft.',
             category='testing')
    @CommandArgument('summary',
                     help="Error summary log to take un")
    @CommandArgument('--log-filteredsummary', default=None,
                     help='Print filtered log to file')
    @CommandArgument('--log-intermittents', default=None,
                     help='Print intermittents to file')
    @CommandArgument('--auth', default=None,
                     help='File containing basic authorization credentials for Github API (format `username:password`)')
    @CommandArgument('--tracker-api', default=None, action='store',
                     help='The API endpoint for tracking known intermittent failures.')
    @CommandArgument('--reporter-api', default=None, action='store',
                     help='The API endpoint for reporting tracked intermittent failures.')
    def filter_intermittents(self, summary, log_filteredsummary, log_intermittents, auth, tracker_api, reporter_api):
        encoded_auth = None
        if auth:
            with open(auth, "r") as file:
                encoded_auth = base64.encodestring(file.read().strip()).replace('\n', '')
        failures = []
        with open(summary, "r") as file:
            for line in file:
                line_json = json.loads(line)
                if 'status' in line_json:
                    failures += [line_json]
        actual_failures = []
        intermittents = []
        for failure in failures:
            if tracker_api:
                if tracker_api == 'default':
                    tracker_api = "http://build.servo.org/intermittent-tracker"
                elif tracker_api.endswith('/'):
                    tracker_api = tracker_api[0:-1]

                query = urllib2.quote(failure['test'], safe='')
                request = urllib2.Request("%s/query.py?name=%s" % (tracker_api, query))
                search = urllib2.urlopen(request)
                data = json.load(search)
                if len(data) == 0:
                    actual_failures += [failure]
                else:
                    intermittents += [failure]
            else:
                qstr = "repo:servo/servo+label:I-intermittent+type:issue+state:open+%s" % failure['test']
                # we want `/` to get quoted, but not `+` (github's API doesn't like that), so we set `safe` to `+`
                query = urllib2.quote(qstr, safe='+')
                request = urllib2.Request("https://api.github.com/search/issues?q=%s" % query)
                if encoded_auth:
                    request.add_header("Authorization", "Basic %s" % encoded_auth)
                search = urllib2.urlopen(request)
                data = json.load(search)
                if data['total_count'] == 0:
                    actual_failures += [failure]
                else:
                    intermittents += [failure]

        if reporter_api:
            if reporter_api == 'default':
                reporter_api = "http://build.servo.org/intermittent-failure-tracker"
            if reporter_api.endswith('/'):
                reporter_api = reporter_api[0:-1]
            reported = set()

            proc = subprocess.Popen(
                ["git", "log", "--merges", "--oneline", "-1"],
                stdout=subprocess.PIPE)
            (last_merge, _) = proc.communicate()

            # Extract the issue reference from "abcdef Auto merge of #NNN"
            pull_request = int(last_merge.split(' ')[4][1:])

            for intermittent in intermittents:
                if intermittent['test'] in reported:
                    continue
                reported.add(intermittent['test'])

                data = {
                    'test_file': intermittent['test'],
                    'platform': platform.system(),
                    'builder': os.environ.get('BUILDER_NAME', 'BUILDER NAME MISSING'),
                    'number': pull_request,
                }
                request = urllib2.Request("%s/record.py" % reporter_api, urllib.urlencode(data))
                request.add_header('Accept', 'application/json')
                response = urllib2.urlopen(request)
                data = json.load(response)
                if data['status'] != "success":
                    print('Error reporting test failure: ' + data['error'])

        if log_intermittents:
            with open(log_intermittents, "w") as intermittents_file:
                for intermittent in intermittents:
                    json.dump(intermittent, intermittents_file)
                    print("\n", end='', file=intermittents_file)

        if len(actual_failures) == 0:
            return 0

        output = open(log_filteredsummary, "w") if log_filteredsummary else sys.stdout
        for failure in actual_failures:
            json.dump(failure, output)
            print("\n", end='', file=output)

        if output is not sys.stdout:
            output.close()
        return 1

    @Command('test-android-startup',
             description='Extremely minimal testing of Servo for Android',
             category='testing')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Run the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Run the dev build')
    def test_android_startup(self, release, dev):
        if (release and dev) or not (release or dev):
            print("Please specify one of --dev or --release.")
            return 1
        target = "i686-linux-android"
        print("Assuming --target " + target)
        env = self.build_env(target=target)
        assert self.handle_android_target(target)

        emulator_port = "5580"
        adb = [self.android_adb_path(env), "-s", "emulator-" + emulator_port]
        emulator_process = subprocess.Popen([
            self.android_emulator_path(env),
            "@servo-x86",
            "-no-window",
            "-gpu", "guest",
            "-port", emulator_port,
        ])
        try:
            # This is hopefully enough time for the emulator to exit
            # if it cannot start because of a configuration problem,
            # and probably more time than it needs to boot anyway
            time.sleep(1)
            if emulator_process.poll() is not None:
                # The process has terminated already, wait-for-device would block indefinitely
                return 1

            subprocess.call(adb + ["wait-for-device"])

            # https://stackoverflow.com/a/38896494/1162888
            while 1:
                stdout, stderr = subprocess.Popen(
                    adb + ["shell", "getprop", "sys.boot_completed"],
                    stdout=subprocess.PIPE,
                ).communicate()
                if "1" in stdout:
                    break
                print("Waiting for the emulator to boot")
                time.sleep(1)

            apk_path = self.get_apk_path(release)
            result = subprocess.call(adb + ["install", "-r", apk_path])
            if result != 0:
                return result

            html = """
                <script>
                    console.log("JavaScript is running!")
                </script>
            """
            url = "data:text/html;base64," + html.encode("base64").replace("\n", "")
            result = subprocess.call(adb + ["shell", """
                mkdir -p /sdcard/Android/data/com.mozilla.servo/files/
                echo 'servo' > /sdcard/Android/data/com.mozilla.servo/files/android_params
                echo '%s' >> /sdcard/Android/data/com.mozilla.servo/files/android_params
                am start com.mozilla.servo/com.mozilla.servo.MainActivity
            """ % url])
            if result != 0:
                return result

            logcat = adb + ["logcat", "RustAndroidGlueStdouterr:D", "*:S", "-v", "raw"]
            logcat_process = subprocess.Popen(logcat, stdout=subprocess.PIPE)
            while 1:
                line = logcat_process.stdout.readline()
                if "JavaScript is running!" in line:
                    print(line)
                    break
            logcat_process.kill()
        finally:
            try:
                emulator_process.kill()
            except OSError:
                pass

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

    @Command('compare_dromaeo',
             description='Compare outputs of two runs of ./mach test-dromaeo command',
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

    def set_software_rendering_env(self, use_release):
        # On Linux and mac, find the OSMesa software rendering library and
        # add it to the dynamic linker search path.
        try:
            bin_path = self.get_binary_path(use_release, not use_release)
            if not set_osmesa_env(bin_path, os.environ):
                print("Warning: Cannot set the path to OSMesa library.")
        except BuildNotFound:
            # This can occur when cross compiling (e.g. arm64), in which case
            # we won't run the tests anyway so can safely ignore this step.
            pass


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

    @Command('update-net-cookies',
             description='Update the net unit tests with cookie tests from http-state',
             category='testing')
    def update_net_cookies(self):
        cache_dir = path.join(self.config["tools"]["cache-dir"], "tests")
        run_file = path.abspath(path.join(PROJECT_TOPLEVEL_PATH,
                                          "components", "net", "tests",
                                          "cookie_http_state_utils.py"))
        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["update_test_file"](cache_dir)

    @Command('update-webgl',
             description='Update the WebGL conformance suite tests from Khronos repo',
             category='testing')
    @CommandArgument('--version', default='2.0.0',
                     help='WebGL conformance suite version')
    def update_webgl(self, version=None):
        self.ensure_bootstrapped()

        base_dir = path.abspath(path.join(PROJECT_TOPLEVEL_PATH,
                                "tests", "wpt", "mozilla", "tests", "webgl"))
        run_file = path.join(base_dir, "tools", "import-conformance-tests.py")
        dest_folder = path.join(base_dir, "conformance-%s" % version)
        patches_dir = path.join(base_dir, "tools")
        # Clean dest folder if exists
        if os.path.exists(dest_folder):
            shutil.rmtree(dest_folder)

        run_globals = {"__file__": run_file}
        execfile(run_file, run_globals)
        return run_globals["update_conformance"](version, dest_folder, None, patches_dir)
