# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import argparse
import json
import logging
import os
import os.path as path
import re
import shutil
import subprocess
import sys
import textwrap
from argparse import ArgumentParser
from pathlib import Path
from typing import Any, List, Optional

import tidy
import wpt
import wpt.manifestupdate
import wpt.run
import wpt.update
from mach.decorators import (
    Command,
    CommandArgument,
    CommandProvider,
)

import servo.devtools_tests
import servo.try_parser
from servo.command_base import BuildType, CommandBase, call, check_call
from servo.post_build_commands import PostBuildCommands
from servo.util import delete

SCRIPT_PATH = os.path.split(__file__)[0]
PROJECT_TOPLEVEL_PATH = os.path.abspath(os.path.join(SCRIPT_PATH, "..", ".."))
WEB_PLATFORM_TESTS_PATH = os.path.join("tests", "wpt", "tests")
SERVO_TESTS_PATH = os.path.join("tests", "wpt", "mozilla", "tests")

# Servo depends on several `rustfmt` options that are unstable. These are still
# supported by stable `rustfmt` if they are passed as these command-line arguments.
UNSTABLE_RUSTFMT_ARGUMENTS = [
    "--config",
    "unstable_features=true",
    "--config",
    "binop_separator=Back",
    "--config",
    "imports_granularity=Module",
    "--config",
    "group_imports=StdExternalCrate",
]

# Listing these globs manually is a work-around for very slow `taplo` invocation
# on MacOS machines. If `taplo` runs fast without the globs on MacOS, this
# can be removed.
TOML_GLOBS = [
    "*.toml",
    ".cargo/*.toml",
    "components/*/*.toml",
    "components/shared/*/*.toml",
    "ports/*/*.toml",
    "support/*/*.toml",
]


def format_toml_files_with_taplo(check_only: bool = True) -> int:
    taplo = shutil.which("taplo")
    if taplo is None:
        print("Could not find `taplo`. Run `./mach bootstrap` or `cargo install taplo-cli --locked`")
        return 1

    if check_only:
        return call([taplo, "fmt", "--check", *TOML_GLOBS], env={"RUST_LOG": "error"})
    else:
        return call([taplo, "fmt", *TOML_GLOBS], env={"RUST_LOG": "error"})


def format_python_files_with_ruff(check_only: bool = True) -> int:
    ruff = shutil.which("ruff")
    if ruff is None:
        print("Could not find `ruff`. Run `./mach bootstrap`")
        return 1

    if check_only:
        return call([ruff, "format", "--check", "--quiet"])
    else:
        return call([ruff, "format", "--quiet"])


def format_with_rustfmt(check_only: bool = True) -> int:
    maybe_check_only = ["--check"] if check_only else []
    result = call(["cargo", "fmt", "--", *UNSTABLE_RUSTFMT_ARGUMENTS, *maybe_check_only])
    if result != 0:
        return result

    return call(
        [
            "cargo",
            "fmt",
            "--manifest-path",
            "support/crown/Cargo.toml",
            "--",
            *UNSTABLE_RUSTFMT_ARGUMENTS,
            *maybe_check_only,
        ]
    )


def pretty_print_json_decode_error(error: json.decoder.JSONDecodeError) -> None:
    print(f"json decode error: {error}")
    # Print section around that character that raised the json decode error.
    snippet_radius = 25
    start, stop = max(0, error.pos - snippet_radius), error.pos + snippet_radius
    snippet = error.doc[start:stop]
    print("```")
    prefix = "... " if start != 0 else ""
    suffix = " ..." if stop < len(error.doc) else ""
    print(prefix, snippet, suffix, sep="")
    # If the snippet is multiline, this won't work as expected, but it's a best effort.
    right_justification = snippet_radius + 1 + len(suffix)
    print("^ Offending character".rjust(right_justification))
    print("```")


@CommandProvider
class MachCommands(CommandBase):
    DEFAULT_RENDER_MODE = "cpu"
    HELP_RENDER_MODE = "Value can be 'cpu', 'gpu' or 'both' (default " + DEFAULT_RENDER_MODE + ")"

    def __init__(self, context: Any) -> None:
        CommandBase.__init__(self, context)
        if not hasattr(self.context, "built_tests"):
            self.context.built_tests = False

    @Command("test-perf", description="Run the page load performance test", category="testing")
    @CommandArgument("--base", default=None, help="the base URL for testcases")
    @CommandArgument("--date", default=None, help="the datestamp for the data")
    @CommandArgument("--submit", "-a", default=False, action="store_true", help="submit the data to perfherder")
    def test_perf(self, base: str | None = None, date: str | None = None, submit: bool = False) -> int:
        env = self.build_env()
        cmd = ["bash", "test_perf.sh"]
        if base:
            cmd += ["--base", base]
        if date:
            cmd += ["--date", date]
        if submit:
            cmd += ["--submit"]
        return call(cmd, env=env, cwd=path.join("etc", "ci", "performance"))

    @Command("test-unit", description="Run unit tests", category="testing")
    @CommandArgument("test_name", nargs=argparse.REMAINDER, help="Only run tests that match this pattern or file path")
    @CommandArgument("--package", "-p", default=None, help="Specific package to test")
    @CommandArgument("--bench", default=False, action="store_true", help="Run in bench mode")
    @CommandArgument(
        "--nocapture", default=False, action="store_true", help="Run tests with nocapture ( show test stdout )"
    )
    @CommandArgument("--code-coverage", default=False, action="store_true", help="Run in code coverage mode")
    @CommandArgument("--llvm-cov-option", default=None, action="append", help="Additional options for llvm-cov")
    @CommandArgument("--nextest-profile", default=None, help="Specify the Nextest profile to use")
    @CommandArgument("params", nargs="...", help="Command-line arguments to be passed through to Cargo nextest")
    @CommandBase.common_command_arguments(build_configuration=True, build_type=True)
    def test_unit(
        self,
        build_type: BuildType,
        test_name: list[str] | None = None,
        params: list[str] | None = None,
        package: str | None = None,
        bench: bool = False,
        code_coverage: bool = False,
        llvm_cov_option: Optional[List[str]] = None,
        nocapture: bool = False,
        nextest_profile: str | None = None,
        **kwargs: Any,
    ) -> int:
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
            "compositing_traits",
            "constellation",
            "devtools",
            "fonts",
            "hyper_serde",
            "layout",
            "layout_api",
            "libservo",
            "metrics",
            "net",
            "net_traits",
            "pixels",
            "script_traits",
            "script_bindings",
            "selectors",
            "servo_config",
            "servoshell",
            "servo_url",
            "storage",
            "storage_traits",
            "xpath",
        ]
        if not packages:
            packages = set(os.listdir(path.join(self.context.topdir, "tests", "unit"))) - set([".DS_Store"])
            packages |= set(self_contained_tests)

        in_crate_packages = []
        for crate in self_contained_tests:
            try:
                packages.remove(crate)
                in_crate_packages += [crate]
            except KeyError:
                pass

        # Return if there is nothing to do.
        if len(packages) == 0 and len(in_crate_packages) == 0:
            return 0

        args: list[str] = params or []

        if build_type.is_release():
            args += ["--release"]
        elif build_type.is_dev():
            pass  # there is no argument for debug
        else:
            args += ["--cargo-profile", build_type.profile]

        if nextest_profile is not None:
            args += ["--profile", nextest_profile]

        for crate in packages:
            args += ["-p", "%s_tests" % crate]
        for crate in in_crate_packages:
            args += ["-p", crate]
        args += test_patterns

        if nocapture:
            args += ["--nocapture"]
        args += ["--no-fail-fast"]

        env = self.build_env()

        crown_cargo_command: List[str] = ["cargo"]
        cargo_command: str
        if bench:
            cargo_command = "bench"
            if code_coverage:
                print(
                    "Error: Invalid argument combination for `./mach test-unit`. "
                    "`--bench` and `--code-coverage` are mutually exclusive."
                )
                exit(1)
        elif code_coverage:
            cargo_llvm_cov_options: List[str] = llvm_cov_option or []
            crown_cargo_command.extend(["llvm-cov", "nextest"])
            crown_cargo_command.extend(cargo_llvm_cov_options)
            cargo_command = "llvm-cov"
            args.insert(0, "nextest")
            args.extend(cargo_llvm_cov_options)
        else:
            crown_cargo_command.extend(["nextest", "run"])
            cargo_command = "nextest"
            args.insert(0, "run")
        result = call(crown_cargo_command, cwd="support/crown")
        if result != 0:
            return result
        result = self.run_cargo_build_like_command(cargo_command, args, env=env, **kwargs)
        assert isinstance(result, int)
        return result

    @Command("test-tidy", description="Run the source code tidiness check", category="testing")
    @CommandArgument(
        "--all",
        default=False,
        action="store_true",
        dest="all_files",
        help="Check all files, and run the WPT lint in tidy, even if unchanged",
    )
    @CommandArgument("--no-progress", default=False, action="store_true", help="Don't show progress for tidy")
    @CommandArgument(
        "--github-annotations",
        default=False,
        action="store_true",
        help="Emit tidy warnings in the Github Actions annotations format",
    )
    def test_tidy(self, all_files: bool, no_progress: bool, github_annotations: bool) -> int:
        tidy_failed = tidy.scan(not all_files, not no_progress, github_annotations)

        print("\r ➤  Checking formatting of Rust files...")
        rustfmt_failed = format_with_rustfmt(check_only=True)

        print("\r ➤  Checking formatting of python files...")
        ruff_format_failed = format_python_files_with_ruff()

        print("\r ➤  Checking formatting of toml files...")
        taplo_failed = format_toml_files_with_taplo()

        format_failed = rustfmt_failed or ruff_format_failed or taplo_failed
        tidy_failed = format_failed or tidy_failed
        print()
        if tidy_failed:
            print("\r ❌ test-tidy reported errors.")
        else:
            print("\r ✅ test-tidy reported no errors.")

        if format_failed:
            print("Run `./mach fmt` to fix the formatting")

        return tidy_failed

    @Command("test-scripts", description="Run tests for all build and support scripts.", category="testing")
    @CommandArgument("--verbose", "-v", default=False, action="store_true", help="Enable verbose output")
    @CommandArgument("--very-verbose", "-vv", default=False, action="store_true", help="Enable very verbose output")
    @CommandArgument(
        "--all", "-a", default=False, action="store_true", help="Run all script tests, even the slow ones."
    )
    @CommandArgument(
        "tests", default=None, nargs="...", help="Specific WebIDL tests to run, relative to the tests directory"
    )
    def test_scripts(self, verbose: bool, very_verbose: bool, all: bool, tests: list[str]) -> int:
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

        print("Running devtools parser tests...")
        if shutil.which("tshark"):
            try:
                result = subprocess.run(
                    ["etc/devtools_parser.py", "--json", "--read-file", "etc/devtools_parser_test.pcap"],
                    check=True,
                    capture_output=True,
                )
                expected = open("etc/devtools_parser_test.json", "rb").read()
                actual = result.stdout
                assert actual == expected, f"Incorrect output!\nExpected: {repr(expected)}\nActual:   {repr(actual)}"
                print("OK")
            except subprocess.CalledProcessError as e:
                print(f"Process failed with exit status {e.returncode}: {e.cmd}", file=sys.stderr)
                print(f"stdout: {repr(e.stdout)}", file=sys.stderr)
                print(f"stderr: {repr(e.stderr)}", file=sys.stderr)
                raise e
        else:
            print("SKIP: Install tshark manually")

        if all or tests:
            print("Running WebIDL tests...")

            test_file_dir = path.abspath(path.join(PROJECT_TOPLEVEL_PATH, "third_party", "WebIDL"))
            # For the `import WebIDL` in runtests.py
            sys.path.insert(0, test_file_dir)
            run_file = path.abspath(path.join(test_file_dir, "runtests.py"))
            run_globals: dict[str, Any] = {"__file__": run_file}
            exec(compile(open(run_file).read(), run_file, "exec"), run_globals)
            passed = run_globals["run_tests"](tests, verbose or very_verbose) and passed

        return 0 if passed else 1

    @Command("test-devtools", description="Run tests for devtools.", category="testing")
    @CommandArgument("test_names", nargs=argparse.REMAINDER, help="Only run tests that match these patterns")
    @CommandBase.common_command_arguments(binary_selection=True)
    def test_devtools(self, servo_binary: str, test_names: list[str], **kwargs: Any) -> int:
        print("Running devtools tests...")
        passed = servo.devtools_tests.run_tests(SCRIPT_PATH, servo_binary, test_names)
        return 0 if passed else 1

    @Command(
        "test-wpt-failure",
        description="Run the tests harness that verifies that the test failures are reported correctly",
        category="testing",
        parser=wpt.create_parser,
    )
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def test_wpt_failure(self, build_type: BuildType, **kwargs: Any) -> bool:
        kwargs["pause_after_test"] = False
        kwargs["include"] = ["infrastructure/failing-test.html"]
        return not self._test_wpt(build_type=build_type, **kwargs)

    @Command(
        "test-wpt", description="Run the regular web platform test suite", category="testing", parser=wpt.create_parser
    )
    @CommandBase.common_command_arguments(binary_selection=True)
    def test_wpt(self, servo_binary: str, **kwargs: Any) -> int:
        return self._test_wpt(servo_binary, **kwargs)

    @CommandBase.allow_target_configuration
    def _test_wpt(self, servo_binary: str, **kwargs: Any) -> int:
        # TODO(mrobinson): Why do we pass the wrong binary path in when running WPT on Android?
        return_value = wpt.run.run_tests(servo_binary, **kwargs)
        return return_value if not kwargs["always_succeed"] else 0

    @Command(
        "update-manifest",
        description="Run test-wpt --manifest-update SKIP_TESTS to regenerate MANIFEST.json",
        category="testing",
        parser=wpt.manifestupdate.create_parser,
    )
    def update_manifest(self, **kwargs: Any) -> int:
        return wpt.manifestupdate.update(check_clean=False)

    @Command("fmt", description="Format Rust, Python, and TOML files", category="testing")
    def format_code(self) -> int:
        result = format_python_files_with_ruff(check_only=False)
        if result != 0:
            return result

        result = format_toml_files_with_taplo(check_only=False)
        if result != 0:
            return result

        return format_with_rustfmt(check_only=False)

    @Command(
        "update-wpt", description="Update the web platform tests", category="testing", parser=wpt.update.create_parser
    )
    def update_wpt(self, **kwargs: Any) -> int:
        patch = kwargs.get("patch", False)
        if not patch and kwargs["sync"]:
            print("Are you sure you don't want a patch?")
            return 1
        return wpt.update.update_tests(**kwargs)

    @Command("test-ohos-wpt", description="Run a single WPT test on OHOS device using WebDriver", category="testing")
    @CommandArgument("--test", required=True, help="Path to WPT test (relative to tests/wpt/tests/)")
    @CommandArgument("--webdriver-port", type=int, default=7000, help="WebDriver server port on OHOS device")
    @CommandArgument("--wpt-server-port", type=int, default=8000, help="WPT server port on desktop")
    @CommandArgument("--verbose", action="store_true", help="Enable verbose logging")
    def test_ohos_wpt(self, **kwargs: Any) -> int:
        """Run a single WPT test on OHOS device."""
        script_path = Path(__file__).parent.parent / "wpt" / "ohos_webdriver_test.py"

        cmd = [
            sys.executable,
            str(script_path),
            "--test",
            kwargs["test"],
            "--webdriver-port",
            str(kwargs["webdriver_port"]),
            "--wpt-server-port",
            str(kwargs["wpt_server_port"]),
        ]

        if kwargs.get("verbose"):
            cmd.append("--verbose")

        print(f"Running OHOS WPT test: {kwargs['test']}")
        print("Make sure:")
        print("1. OHOS device is connected and running Servo with WebDriver enabled")
        print("2. WPT server is running on desktop")
        print("3. HDC is available in PATH")

        return subprocess.call(cmd)

    @Command("test-jquery", description="Run the jQuery test suite", category="testing")
    @CommandBase.common_command_arguments(binary_selection=True)
    def test_jquery(self, servo_binary: str) -> int:
        return self.jquery_test_runner("test", servo_binary)

    @Command("test-dromaeo", description="Run the Dromaeo test suite", category="testing")
    @CommandArgument("tests", default=["recommended"], nargs="...", help="Specific tests to run")
    @CommandArgument("--bmf-output", default=None, help="Specify BMF JSON output file")
    @CommandBase.common_command_arguments(build_type=True, binary_selection=True)
    def test_dromaeo(
        self, tests: list[str], build_type: BuildType, servo_binary: str, bmf_output: str | None = None, **kwargs: Any
    ) -> None:
        return self.dromaeo_test_runner(tests, servo_binary, bmf_output, build_type.profile)

    @Command("test-speedometer", description="Run servo's speedometer", category="testing")
    @CommandArgument("--bmf-output", default=None, help="Specify BMF JSON output file")
    @CommandBase.common_command_arguments(build_type=True, binary_selection=True)
    def test_speedometer(
        self, build_type: BuildType, servo_binary: str, bmf_output: str | None = None, **kwargs: Any
    ) -> None:
        return self.speedometer_runner(servo_binary, bmf_output, build_type.profile)

    @Command("update-jquery", description="Update the jQuery test suite expected results", category="testing")
    @CommandBase.common_command_arguments(binary_selection=True)
    def update_jquery(self, servo_binary: str) -> int:
        return self.jquery_test_runner("update", servo_binary)

    @Command(
        "compare_dromaeo", description="Compare outputs of two runs of ./mach test-dromaeo command", category="testing"
    )
    @CommandArgument(
        "params", default=None, nargs="...", help=" filepaths of output files of two runs of dromaeo test "
    )
    def compare_dromaeo(self, params: list[str]) -> None:
        prev_op_filename = params[0]
        cur_op_filename = params[1]
        result = {"Test": [], "Prev_Time": [], "Cur_Time": [], "Difference(%)": []}
        with open(prev_op_filename, "r") as prev_op, open(cur_op_filename, "r") as cur_op:
            l1 = prev_op.readline()
            l2 = cur_op.readline()

            while l1.find("[dromaeo] Saving...") and l2.find("[dromaeo] Saving..."):
                l1 = prev_op.readline()
                l2 = cur_op.readline()

            reach = 3
            while reach > 0:
                l1 = prev_op.readline()
                l2 = cur_op.readline()
                reach -= 1

            while True:
                l1 = prev_op.readline()
                l2 = cur_op.readline()
                if not l1:
                    break
                result["Test"].append(str(l1).split("|")[0].strip())
                result["Prev_Time"].append(float(str(l1).split("|")[1].strip()))
                result["Cur_Time"].append(float(str(l2).split("|")[1].strip()))
                a = float(str(l1).split("|")[1].strip())
                b = float(str(l2).split("|")[1].strip())
                result["Difference(%)"].append(((b - a) / a) * 100)

            width_col1 = max([len(x) for x in result["Test"]])
            width_col2 = max([len(str(x)) for x in result["Prev_Time"]])
            width_col3 = max([len(str(x)) for x in result["Cur_Time"]])
            width_col4 = max([len(str(x)) for x in result["Difference(%)"]])

            for p, q, r, s in zip(["Test"], ["First Run"], ["Second Run"], ["Difference(%)"]):
                print(
                    "\033[1m"
                    + "{}|{}|{}|{}".format(
                        p.ljust(width_col1), q.ljust(width_col2), r.ljust(width_col3), s.ljust(width_col4)
                    )
                    + "\033[0m"
                    + "\n"
                    + "--------------------------------------------------"
                    + "-------------------------------------------------------------------------"
                )

            for a1, b1, c1, d1 in zip(result["Test"], result["Prev_Time"], result["Cur_Time"], result["Difference(%)"]):
                if d1 > 0:
                    print(
                        "\033[91m"
                        + "{}|{}|{}|{}".format(
                            a1.ljust(width_col1),
                            str(b1).ljust(width_col2),
                            str(c1).ljust(width_col3),
                            str(d1).ljust(width_col4),
                        )
                        + "\033[0m"
                    )
                elif d1 < 0:
                    print(
                        "\033[92m"
                        + "{}|{}|{}|{}".format(
                            a1.ljust(width_col1),
                            str(b1).ljust(width_col2),
                            str(c1).ljust(width_col3),
                            str(d1).ljust(width_col4),
                        )
                        + "\033[0m"
                    )
                else:
                    print(
                        "{}|{}|{}|{}".format(
                            a1.ljust(width_col1),
                            str(b1).ljust(width_col2),
                            str(c1).ljust(width_col3),
                            str(d1).ljust(width_col4),
                        )
                    )

    def jquery_test_runner(self, cmd: str, binary: str) -> int:
        base_dir = path.abspath(path.join("tests", "jquery"))
        jquery_dir = path.join(base_dir, "jquery")
        run_file = path.join(base_dir, "run_jquery.py")

        # Clone the jQuery repository if it doesn't exist
        if not os.path.isdir(jquery_dir):
            check_call(["git", "clone", "-b", "servo", "--depth", "1", "https://github.com/servo/jquery", jquery_dir])

        # Run pull in case the jQuery repo was updated since last test run
        check_call(["git", "-C", jquery_dir, "pull"])

        # Check that a release servo build exists
        bin_path = path.abspath(binary)

        return call([run_file, cmd, bin_path, base_dir])

    def dromaeo_test_runner(self, tests: list[str], binary: str, bmf_output: str | None, profile: str) -> None:
        base_dir = path.abspath(path.join("tests", "dromaeo"))
        dromaeo_dir = path.join(base_dir, "dromaeo")
        run_file = path.join(base_dir, "run_dromaeo.py")
        if bmf_output:
            bmf_output = path.abspath(bmf_output)
        else:
            bmf_output = ""

        # Clone the Dromaeo repository if it doesn't exist
        if not os.path.isdir(dromaeo_dir):
            check_call(
                ["git", "clone", "-b", "servo", "--depth", "1", "https://github.com/notriddle/dromaeo", dromaeo_dir]
            )

        # Run pull in case the Dromaeo repo was updated since last test run
        check_call(["git", "-C", dromaeo_dir, "pull"])

        # Compile test suite
        check_call(["make", "-C", dromaeo_dir, "web"])

        # Check that a release servo build exists
        bin_path = path.abspath(binary)

        args = [run_file, "|".join(tests), bin_path, base_dir]
        if bmf_output is not None:
            args.append(bmf_output)
            args.append(profile)

        return check_call(args)

    def speedometer_to_bmf(self, speedometer: dict[str, Any], bmf_output: str, profile: str | None = None) -> None:
        output = dict()
        profile = "" if profile is None else profile + "/"

        def parse_speedometer_result(result: dict[str, Any]) -> None:
            if result["unit"] == "ms":
                output[profile + f"Speedometer/{result['name']}"] = {
                    "latency": {  # speedometer has ms we need to convert to ns
                        "value": float(result["mean"]) * 1000000.0,
                        "lower_value": float(result["min"]) * 1000000.0,
                        "upper_value": float(result["max"]) * 1000000.0,
                    }
                }
            elif result["unit"] == "score":
                output[profile + f"Speedometer/{result['name']}"] = {
                    "score": {
                        "value": float(result["mean"]),
                        "lower_value": float(result["min"]),
                        "upper_value": float(result["max"]),
                    }
                }
            else:
                raise Exception("Unknown unit!")

            for child in result["children"]:
                parse_speedometer_result(child)

        for v in speedometer.values():
            parse_speedometer_result(v)
        with open(bmf_output, "w", encoding="utf-8") as f:
            json.dump(output, f, indent=4)

    def speedometer_runner(self, binary: str, bmf_output: str | None, profile: str) -> None:
        output = subprocess.check_output(
            [
                binary,
                "https://servospeedometer.netlify.app?headless=1",
                "--pref",
                "dom_allow_scripts_to_close_windows",
                "--window-size=1100x900",
                "--headless",
            ],
            encoding="utf-8",
            timeout=120,
        )
        try:
            speedometer = json.loads(output)
        except json.decoder.JSONDecodeError as e:
            pretty_print_json_decode_error(e)
            print("Error: Failed to parse speedometer results")
            print("This can happen if other log messages are printed while running servo...")
            exit(1)

        print(f"Score: {speedometer['Score']['mean']} ± {speedometer['Score']['delta']}")

        if bmf_output:
            self.speedometer_to_bmf(speedometer, bmf_output, profile)

    @Command(
        "update-net-cookies",
        description="Update the net unit tests with cookie tests from http-state",
        category="testing",
    )
    def update_net_cookies(self) -> int:
        cache_dir = path.join(self.config["tools"]["cache-dir"], "tests")
        run_file = path.abspath(
            path.join(PROJECT_TOPLEVEL_PATH, "components", "net", "tests", "cookie_http_state_utils.py")
        )
        run_globals: dict[str, Any] = {"__file__": run_file}
        exec(compile(open(run_file).read(), run_file, "exec"), run_globals)
        return run_globals["update_test_file"](cache_dir)

    @Command(
        "update-webgl", description="Update the WebGL conformance suite tests from Khronos repo", category="testing"
    )
    @CommandArgument("--version", default="2.0.0", help="WebGL conformance suite version")
    def update_webgl(self, version: str | None = None) -> None:
        base_dir = path.abspath(path.join(PROJECT_TOPLEVEL_PATH, "tests", "wpt", "mozilla", "tests", "webgl"))
        run_file = path.join(base_dir, "tools", "import-conformance-tests.py")
        dest_folder = path.join(base_dir, "conformance-%s" % version)
        patches_dir = path.join(base_dir, "tools")
        # Clean dest folder if exists
        if os.path.exists(dest_folder):
            shutil.rmtree(dest_folder)

        run_globals: dict[str, Any] = {"__file__": run_file}
        exec(compile(open(run_file).read(), run_file, "exec"), run_globals)
        return run_globals["update_conformance"](version, dest_folder, None, patches_dir)

    @Command("update-webgpu", description="Update the WebGPU conformance test suite", category="testing")
    @CommandArgument("--repo", "-r", default="https://github.com/gpuweb/cts", help="Repo to vendor cts from")
    @CommandArgument("--checkout", "-c", default="main", help="Branch or commit of repo")
    def cts(self, repo: str = "https://github.com/gpuweb/cts", checkout: str = "main") -> int:
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
        with open(cts_html, "r") as file:
            filedata = file.read()
        # files are mounted differently
        filedata = filedata.replace("src=/webgpu/common/runtime/wpt.js", "src=../webgpu/common/runtime/wpt.js")
        # Mark all webgpu tests as long to increase their timeouts. This is needed due to wgpu's slowness.
        # TODO: replace this with more fine grained solution: https://github.com/servo/servo/issues/30999
        filedata = filedata.replace(
            "<meta charset=utf-8>", '<meta charset=utf-8>\n<meta name="timeout" content="long">'
        )
        # Write the file out again
        with open(cts_html, "w") as file:
            file.write(filedata)
        logger = path.join(clone_dir, "out-wpt", "common/internal/logging/test_case_recorder.js")
        with open(logger, "r") as file:
            filedata = file.read()
        filedata.replace("info(ex) {", "info(ex) {return;")
        with open(logger, "w") as file:
            file.write(filedata)
        # copy
        delete(path.join(tdir, "webgpu"))
        shutil.copytree(path.join(clone_dir, "out-wpt"), path.join(tdir, "webgpu"))
        # update commit
        commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=clone_dir).decode()
        with open(path.join(tdir, "checkout_commit.txt"), "w") as file:
            file.write(commit)
        # clean up
        delete(clone_dir)
        print("Updating manifest.")
        return self.context.commands.dispatch("update-manifest", self.context)

    @Command(
        "smoketest", description="Load a simple page in Servo and ensure that it closes properly", category="testing"
    )
    @CommandArgument("params", nargs="...", help="Command-line arguments to be passed through to Servo")
    @CommandBase.common_command_arguments(binary_selection=True)
    def smoketest(self, servo_binary: str, params: list[str], **kwargs: Any) -> int | None:
        # We pass `-f` here so that any thread panic will cause Servo to exit,
        # preventing a panic from hanging execution. This means that these kind
        # of panics won't cause timeouts on CI.
        return PostBuildCommands(self.context)._run(servo_binary, params + ["-f", "tests/html/close-on-load.html"])

    @Command("try", description="Runs try jobs by force pushing to try branch", category="testing")
    @CommandArgument("--remote", "-r", default="origin", help="A git remote to run the try job on")
    @CommandArgument(
        "try_strings", default=["full"], nargs="...", help="A list of try strings specifying what kind of job to run."
    )
    def try_command(self, remote: str, try_strings: list[str]) -> int:
        if subprocess.check_output(["git", "diff", "--cached", "--name-only"]).strip():
            print("Cannot run `try` with staged and uncommited changes. ")
            print("Please either commit or stash them before running `try`.")
            return 1

        remote_url = subprocess.check_output(["git", "config", "--get", f"remote.{remote}.url"]).decode().strip()
        if "github.com" not in remote_url:
            print(f"The remote provided ({remote_url}) isn't a GitHub remote.")
            return 1

        try_string = " ".join(try_strings)
        try:
            config = servo.try_parser.Config(try_string)
        except ValueError as err:
            print(f"Failed to parse `try` config because of {err}")
            return 1
        print(f"Trying on {remote} ({remote_url}) with following configuration:")
        print()
        print(textwrap.indent(config.to_json(indent=2), prefix="  "))
        print()

        # The commit message is composed of both the last commit message and the try string.
        commit_message = subprocess.check_output(["git", "show", "-s", "--format=%s"]).decode().strip()
        commit_message = f"{commit_message} ({try_string})"

        result = call(
            [
                "git",
                "commit",
                "--no-verify",
                "--quiet",
                "--allow-empty",
                "-m",
                commit_message,
                "-m",
                f"{config.to_json()}",
            ]
        )
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
                actions_url = actions_url.replace(":", "/")
                actions_url = actions_url.replace("git@", "")
                actions_url = f"https://{actions_url}"
            print(f"Actions available at: {actions_url}")

        finally:
            # Remove the last commit which only contains the try configuration.
            result = call(["git", "reset", "--quiet", "--soft", "HEAD~1"])
            if result != 0:
                print("Could not clean up try commit. Sorry! Please try to reset to the previous commit.")
            return result


def create_parser_create() -> ArgumentParser:
    import argparse

    p = argparse.ArgumentParser()
    p.add_argument("--no-editor", action="store_true", help="Don't try to open the test in an editor")
    p.add_argument("-e", "--editor", action="store", help="Editor to use")
    p.add_argument(
        "--no-run", action="store_true", help="Don't try to update the wpt manifest or open the test in a browser"
    )
    p.add_argument("--release", action="store_true", help="Run with a release build of servo")
    p.add_argument(
        "--long-timeout",
        action="store_true",
        help="Test should be given a long timeout (typically 60s rather than 10s,but varies depending on environment)",
    )
    p.add_argument("--overwrite", action="store_true", help="Allow overwriting an existing test file")
    (
        p.add_argument(
            "-r", "--reftest", action="store_true", help="Create a reftest rather than a testharness (js) test"
        ),
    )
    p.add_argument("-ref", "--reference", dest="ref", help="Path to the reference file")
    p.add_argument("--mismatch", action="store_true", help="Create a mismatch reftest")
    p.add_argument("--wait", action="store_true", help="Create a reftest that waits until takeScreenshot() is called")
    p.add_argument("path", action="store", help="Path to the test file")
    return p
