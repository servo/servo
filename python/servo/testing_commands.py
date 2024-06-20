# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import argparse
import logging
import re
import sys
import os
import os.path as path
import shutil
import subprocess
import textwrap

import wpt
import wpt.manifestupdate
import wpt.run
import wpt.update

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

import servo.try_parser
import tidy

from servo.command_base import BuildType, CommandBase, call, check_call
from servo.util import delete

SCRIPT_PATH = os.path.split(__file__)[0]
PROJECT_TOPLEVEL_PATH = os.path.abspath(os.path.join(SCRIPT_PATH, "..", ".."))
WEB_PLATFORM_TESTS_PATH = os.path.join("tests", "wpt", "tests")
SERVO_TESTS_PATH = os.path.join("tests", "wpt", "mozilla", "tests")

# Servo depends on several `rustfmt` options that are unstable. These are still
# supported by stable `rustfmt` if they are passed as these command-line arguments.
UNSTABLE_RUSTFMT_ARGUMENTS = [
    "--config", "unstable_features=true",
    "--config", "binop_separator=Back",
    "--config", "imports_granularity=Module",
    "--config", "group_imports=StdExternalCrate",
]

# Listing these globs manually is a work-around for very slow `taplo` invocation
# on MacOS machines. If `taplo` runs fast without the globs on MacOS, this
# can be removed.
TOML_GLOBS = [
    "*.toml",
    ".cargo/*.toml",
    "components/*/*.toml",
    "components/shared/*.toml",
    "ports/*/*.toml",
    "support/*/*.toml",
]


def format_toml_files_with_taplo(check_only: bool = True) -> int:
    taplo = shutil.which("taplo")
    if taplo is None:
        print("Could not find `taplo`. Run `./mach bootstrap` or `cargo install taplo-cli --locked`")
        return 1

    if check_only:
        return call([taplo, "fmt", "--check", *TOML_GLOBS], env={'RUST_LOG': 'error'})
    else:
        return call([taplo, "fmt", *TOML_GLOBS], env={'RUST_LOG': 'error'})


@CommandProvider
class MachCommands(CommandBase):
    DEFAULT_RENDER_MODE = "cpu"
    HELP_RENDER_MODE = "Value can be 'cpu', 'gpu' or 'both' (default " + DEFAULT_RENDER_MODE + ")"

    def __init__(self, context):
        CommandBase.__init__(self, context)
        if not hasattr(self.context, "built_tests"):
            self.context.built_tests = False

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
    @CommandBase.common_command_arguments(build_configuration=True, build_type=True)
    def test_unit(self, build_type: BuildType, test_name=None, package=None, bench=False, nocapture=False, **kwargs):
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
            "background_hang_monitor",
            "base",
            "compositing",
            "constellation",
            "crown",
            "fonts",
            "hyper_serde",
            "layout_2013",
            "layout_2020",
            "net",
            "net_traits",
            "pixels",
            "script_traits",
            "selectors",
            "servo_config",
            "servoshell",
            "style_config",
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

        # Return if there is nothing to do.
        if len(packages) == 0 and len(in_crate_packages) == 0:
            return 0

        # Gather Cargo build timings (https://doc.rust-lang.org/cargo/reference/timings.html).
        args = ["--timings"]

        if build_type.is_release():
            args += ["--release"]
        elif build_type.is_dev():
            pass  # there is no argument for debug
        else:
            args += ["--profile", build_type.profile]

        for crate in packages:
            args += ["-p", "%s_tests" % crate]
        for crate in in_crate_packages:
            args += ["-p", crate]
        args += test_patterns

        if nocapture:
            args += ["--", "--nocapture"]

        env = self.build_env()
        return self.run_cargo_build_like_command(
            "bench" if bench else "test",
            args,
            env=env,
            **kwargs)

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
    def test_tidy(self, all_files, no_progress):
        tidy_failed = tidy.scan(not all_files, not no_progress)

        print("\r ➤  Checking formatting of Rust files...")
        rustfmt_failed = call(["cargo", "fmt", "--", *UNSTABLE_RUSTFMT_ARGUMENTS, "--check"])
        if rustfmt_failed:
            print("Run `./mach fmt` to fix the formatting")

        print("\r ➤  Checking formatting of toml files...")
        taplo_failed = format_toml_files_with_taplo()

        tidy_failed = tidy_failed or rustfmt_failed or taplo_failed
        print()
        if tidy_failed:
            print("\r ❌ test-tidy reported errors.")
        else:
            print("\r ✅ test-tidy reported no errors.")

        return tidy_failed

    @Command('test-scripts',
             description='Run tests for all build and support scripts.',
             category='testing')
    @CommandArgument('--verbose', '-v', default=False, action="store_true",
                     help="Enable verbose output")
    @CommandArgument('--very-verbose', '-vv', default=False, action="store_true",
                     help="Enable very verbose output")
    @CommandArgument('--all', '-a', default=False, action="store_true",
                     help="Run all script tests, even the slow ones.")
    @CommandArgument('tests', default=None, nargs="...",
                     help="Specific WebIDL tests to run, relative to the tests directory")
    def test_scripts(self, verbose, very_verbose, all, tests):
        if very_verbose:
            logging.getLogger().level = logging.DEBUG
        elif verbose:
            logging.getLogger().level = logging.INFO
        else:
            logging.getLogger().level = logging.WARN

        passed = True

        print("Running tidy tests...")
        passed = tidy.run_tests() and passed

        import python.servo.try_parser as try_parser
        print("Running try_parser tests...")
        passed = try_parser.run_tests() and passed

        print("Running WPT tests...")
        passed = wpt.run_tests() and passed

        if all or tests:
            print("Running WebIDL tests...")

            test_file_dir = path.abspath(path.join(PROJECT_TOPLEVEL_PATH, "third_party", "WebIDL"))
            # For the `import WebIDL` in runtests.py
            sys.path.insert(0, test_file_dir)
            run_file = path.abspath(path.join(test_file_dir, "runtests.py"))
            run_globals = {"__file__": run_file}
            exec(compile(open(run_file).read(), run_file, 'exec'), run_globals)
            passed = run_globals["run_tests"](tests, verbose or very_verbose) and passed

        return 0 if passed else 1

    @Command('test-wpt-failure',
             description='Run the tests harness that verifies that the test failures are reported correctly',
             category='testing',
             parser=wpt.create_parser)
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def test_wpt_failure(self, build_type: BuildType, **kwargs):
        kwargs["pause_after_test"] = False
        kwargs["include"] = ["infrastructure/failing-test.html"]
        return not self._test_wpt(build_type=build_type, **kwargs)

    @Command('test-wpt',
             description='Run the regular web platform test suite',
             category='testing',
             parser=wpt.create_parser)
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def test_wpt(self, build_type: BuildType, with_asan=False, **kwargs):
        return self._test_wpt(build_type=build_type, with_asan=with_asan, **kwargs)

    @Command('test-wpt-android',
             description='Run the web platform test suite in an Android emulator',
             category='testing',
             parser=wpt.create_parser)
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def test_wpt_android(self, build_type: BuildType, binary_args=None, **kwargs):
        kwargs.update(
            product="servodriver",
            processes=1,
            binary_args=self.in_android_emulator(build_type) + (binary_args or []),
            binary=sys.executable,
        )
        return self._test_wpt(build_type=build_type, android=True, **kwargs)

    def _test_wpt(self, build_type: BuildType, with_asan=False, android=False, **kwargs):
        if not android:
            os.environ.update(self.build_env())

        # TODO(mrobinson): Why do we pass the wrong binary path in when running WPT on Android?
        binary_path = self.get_binary_path(build_type=build_type, asan=with_asan)
        return_value = wpt.run.run_tests(binary_path, **kwargs)
        return return_value if not kwargs["always_succeed"] else 0

    @Command('update-manifest',
             description='Run test-wpt --manifest-update SKIP_TESTS to regenerate MANIFEST.json',
             category='testing',
             parser=wpt.manifestupdate.create_parser)
    def update_manifest(self, **kwargs):
        return wpt.manifestupdate.update(check_clean=False)

    @Command('fmt',
             description='Format Rust and TOML files',
             category='testing')
    def format_code(self):
        result = format_toml_files_with_taplo(check_only=False)
        if result != 0:
            return result

        return call(["cargo", "fmt", "--", *UNSTABLE_RUSTFMT_ARGUMENTS])

    @Command('update-wpt',
             description='Update the web platform tests',
             category='testing',
             parser=wpt.update.create_parser)
    def update_wpt(self, **kwargs):
        patch = kwargs.get("patch", False)
        if not patch and kwargs["sync"]:
            print("Are you sure you don't want a patch?")
            return 1
        return wpt.update.update_tests(**kwargs)

    @Command('test-android-startup',
             description='Extremely minimal testing of Servo for Android',
             category='testing')
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def test_android_startup(self, build_type: BuildType):
        html = """
            <script>
                window.alert("JavaScript is running!")
            </script>
        """
        url = "data:text/html;base64," + html.encode("base64").replace("\n", "")
        args = self.in_android_emulator(build_type)
        args = [sys.executable] + args + [url]
        process = subprocess.Popen(args, stdout=subprocess.PIPE)
        try:
            while 1:
                line = process.stdout.readline()
                if len(line) == 0:
                    print("EOF without finding the expected line")
                    return 1
                print(line.rstrip())
                if "JavaScript is running!" in line:
                    break
        finally:
            process.terminate()

    def in_android_emulator(self, build_type: BuildType):
        avd = "servo-x86"
        target = "i686-linux-android"
        print("Assuming --target " + target)
        self.cross_compile_target = target

        env = self.build_env()
        os.environ["PATH"] = env["PATH"]
        assert self.setup_configuration_for_android_target(target)
        apk = self.get_apk_path(build_type)

        py = path.join(self.context.topdir, "etc", "run_in_headless_android_emulator.py")
        return [py, avd, apk]

    @Command('test-jquery', description='Run the jQuery test suite', category='testing')
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def test_jquery(self, build_type: BuildType):
        return self.jquery_test_runner("test", build_type)

    @Command('test-dromaeo', description='Run the Dromaeo test suite', category='testing')
    @CommandArgument('tests', default=["recommended"], nargs="...", help="Specific tests to run")
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def test_dromaeo(self, tests, build_type: BuildType):
        return self.dromaeo_test_runner(tests, build_type)

    @Command('update-jquery',
             description='Update the jQuery test suite expected results',
             category='testing')
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def update_jquery(self, build_type: BuildType):
        return self.jquery_test_runner("update", build_type)

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
                print("\033[1m" + "{}|{}|{}|{}".format(p.ljust(width_col1), q.ljust(width_col2), r.ljust(width_col3),
                      s.ljust(width_col4)) + "\033[0m" + "\n" + "--------------------------------------------------"
                      + "-------------------------------------------------------------------------")

            for a1, b1, c1, d1 in zip(result['Test'], result['Prev_Time'], result['Cur_Time'], result['Difference(%)']):
                if d1 > 0:
                    print("\033[91m" + "{}|{}|{}|{}".format(a1.ljust(width_col1),
                          str(b1).ljust(width_col2), str(c1).ljust(width_col3), str(d1).ljust(width_col4)) + "\033[0m")
                elif d1 < 0:
                    print("\033[92m" + "{}|{}|{}|{}".format(a1.ljust(width_col1),
                          str(b1).ljust(width_col2), str(c1).ljust(width_col3), str(d1).ljust(width_col4)) + "\033[0m")
                else:
                    print("{}|{}|{}|{}".format(a1.ljust(width_col1), str(b1).ljust(width_col2),
                          str(c1).ljust(width_col3), str(d1).ljust(width_col4)))

    def jquery_test_runner(self, cmd, build_type: BuildType):
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
        bin_path = path.abspath(self.get_binary_path(build_type))

        return call([run_file, cmd, bin_path, base_dir])

    def dromaeo_test_runner(self, tests, build_type: BuildType):
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
        bin_path = path.abspath(self.get_binary_path(build_type))

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
tests/wpt/tests for tests that may be shared
tests/wpt/mozilla/tests for Servo-only tests""" % test_path)
            return 1

        if reference_url is None:
            print("""Reference path %s is not in wpt directories:
tests/wpt/tests for tests that may be shared
tests/wpt/mozilla/tests for Servo-only tests""" % reference_path)
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
            p = wpt.create_parser()
            args = []
            if kwargs["release"]:
                args.append("--release")
            args.append(test_path)
            wpt_kwargs = vars(p.parse_args(args))
            self.context.commands.dispatch("test-wpt", self.context, **wpt_kwargs)
            self.context.commands.dispatch("update-manifest", self.context)

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
        exec(compile(open(run_file).read(), run_file, 'exec'), run_globals)
        return run_globals["update_test_file"](cache_dir)

    @Command('update-webgl',
             description='Update the WebGL conformance suite tests from Khronos repo',
             category='testing')
    @CommandArgument('--version', default='2.0.0',
                     help='WebGL conformance suite version')
    def update_webgl(self, version=None):
        base_dir = path.abspath(path.join(PROJECT_TOPLEVEL_PATH,
                                "tests", "wpt", "mozilla", "tests", "webgl"))
        run_file = path.join(base_dir, "tools", "import-conformance-tests.py")
        dest_folder = path.join(base_dir, "conformance-%s" % version)
        patches_dir = path.join(base_dir, "tools")
        # Clean dest folder if exists
        if os.path.exists(dest_folder):
            shutil.rmtree(dest_folder)

        run_globals = {"__file__": run_file}
        exec(compile(open(run_file).read(), run_file, 'exec'), run_globals)
        return run_globals["update_conformance"](version, dest_folder, None, patches_dir)

    @Command('update-webgpu',
             description='Update the WebGPU conformance test suite',
             category='testing')
    @CommandArgument(
        '--repo', '-r', default="https://github.com/gpuweb/cts",
        help='Repo to vendor cts from')
    @CommandArgument(
        '--checkout', '-c', default="main",
        help='Branch or commit of repo')
    def cts(self, repo="https://github.com/gpuweb/cts", checkout="main"):
        tdir = path.join(self.context.topdir, "tests/wpt/webgpu/tests")
        clone_dir = path.join(tdir, "cts_clone")
        # clone
        res = call(["git", "clone", "-n", repo, "cts_clone"], cwd=tdir)
        if res != 0:
            return res
        # checkout
        res = call(["git", "checkout", checkout], cwd=clone_dir)
        if res != 0:
            return res
        # build
        res = call(["npm", "ci"], cwd=clone_dir)
        if res != 0:
            return res
        res = call(["npm", "run", "wpt"], cwd=clone_dir)
        if res != 0:
            return res
        # https://github.com/gpuweb/cts/pull/2770
        delete(path.join(clone_dir, "out-wpt", "cts-chunked2sec.https.html"))
        cts_html = path.join(clone_dir, "out-wpt", "cts.https.html")
        # patch
        with open(cts_html, 'r') as file:
            filedata = file.read()
        # files are mounted differently
        filedata = filedata.replace('src=/webgpu/common/runtime/wpt.js', 'src=../webgpu/common/runtime/wpt.js')
        # Mark all webgpu tests as long to increase their timeouts. This is needed due to wgpu's slowness.
        # TODO: replace this with more fine grained solution: https://github.com/servo/servo/issues/30999
        filedata = filedata.replace('<meta charset=utf-8>',
                                    '<meta charset=utf-8>\n<meta name="timeout" content="long">')
        # Write the file out again
        with open(cts_html, 'w') as file:
            file.write(filedata)
        # copy
        delete(path.join(tdir, "webgpu"))
        shutil.copytree(path.join(clone_dir, "out-wpt"), path.join(tdir, "webgpu"))
        # update commit
        commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=clone_dir).decode()
        with open(path.join(tdir, "checkout_commit.txt"), 'w') as file:
            file.write(commit)
        # clean up
        delete(clone_dir)
        print("Updating manifest.")
        return self.context.commands.dispatch("update-manifest", self.context)

    @Command('smoketest',
             description='Load a simple page in Servo and ensure that it closes properly',
             category='testing')
    @CommandArgument('params', nargs='...',
                     help="Command-line arguments to be passed through to Servo")
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def smoketest(self, build_type: BuildType, params, with_asan=False):
        # We pass `-f` here so that any thread panic will cause Servo to exit,
        # preventing a panic from hanging execution. This means that these kind
        # of panics won't cause timeouts on CI.
        return self.context.commands.dispatch('run', self.context, build_type=build_type, with_asan=with_asan,
                                              params=params + ['-f', 'tests/html/close-on-load.html'])

    @Command('try', description='Runs try jobs by force pushing to try branch', category='testing')
    @CommandArgument('--remote', '-r', default="origin", help='A git remote to run the try job on')
    @CommandArgument('try_strings', default=["full"], nargs='...',
                     help="A list of try strings specifying what kind of job to run.")
    def try_command(self, remote: str, try_strings: list[str]):
        remote_url = subprocess.check_output(["git", "config", "--get", f"remote.{remote}.url"]).decode().strip()
        if "github.com" not in remote_url:
            print(f"The remote provided ({remote_url}) isn't a GitHub remote.")
            return 1

        try_string = " ".join(try_strings)
        config = servo.try_parser.Config(try_string)
        print(f"Trying on {remote} ({remote_url}) with following configuration:")
        print()
        print(textwrap.indent(config.to_json(indent=2), prefix="  "))
        print()

        # The commit message is composed of both the last commit message and the try string.
        commit_message = subprocess.check_output(["git", "show", "-s", "--format=%s"]).decode().strip()
        commit_message = f"{commit_message} ({try_string})"

        result = call(["git", "commit", "--quiet", "--allow-empty", "-m", commit_message, "-m", f"{config.to_json()}"])
        if result != 0:
            return result

        # From here on out, we need to always clean up the commit we added to the branch.
        try:
            result = call(["git", "push", "--quiet", remote, "--force", "HEAD:try"])
            if result != 0:
                return result

            # TODO: This is a pretty naive approach to turning a GitHub remote URL (either SSH or HTTPS)
            # into a URL to the Actions page. It might be better to create this action with the `gh`
            # tool and get the real URL.
            actions_url = remote_url.replace(".git", "/actions")
            if not actions_url.startswith("https"):
                actions_url = actions_url.replace(':', '/')
                actions_url = actions_url.replace("git@", "")
                actions_url = f"https://{actions_url}"
            print(f"Actions available at: {actions_url}")

        finally:
            # Remove the last commit which only contains the try configuration.
            result = call(["git", "reset", "--quiet", "--soft", "HEAD~1"])
            if result != 0:
                print("Could not clean up try commit. Sorry! Please try to reset to the previous commit.")
            return result
