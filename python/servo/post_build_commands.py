# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json
import os
import os.path as path
import pathlib
import subprocess
from subprocess import CompletedProcess
from shutil import copy2
from typing import Any, Optional, List

import mozdebug

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

import servo.util
import servo.platform

from servo.command_base import (
    CommandBase,
    check_call,
    is_linux,
)
from servo.platform.build_target import is_android

from python.servo.command_base import BuildType

ANDROID_APP_NAME = "org.servo.servoshell"


def read_file(filename: str, if_exists: bool = False) -> str | None:
    if if_exists and not path.exists(filename):
        return None
    with open(filename) as f:
        return f.read()


# Copied from Python 3.3+'s shlex.quote()
def shell_quote(arg: str) -> str:
    # use single quotes, and put single quotes into double quotes
    # the string $'b is then quoted as '$'"'"'b'
    return "'" + arg.replace("'", "'\"'\"'") + "'"


@CommandProvider
class PostBuildCommands(CommandBase):
    @Command("run", description="Run Servo", category="post-build")
    @CommandArgument(
        "--android", action="store_true", default=None, help="Run on an Android device through `adb shell`"
    )
    @CommandArgument("--emulator", action="store_true", help="For Android, run in the only emulated device")
    @CommandArgument("--usb", action="store_true", help="For Android, run in the only USB device")
    @CommandArgument(
        "--debugger",
        action="store_true",
        help="Enable the debugger. Not specifying a "
        "--debugger-cmd option will result in the default "
        "debugger being used. The following arguments "
        "have no effect without this.",
    )
    @CommandArgument("--debugger-cmd", default=None, type=str, help="Name of debugger to use.")
    @CommandArgument("--headless", "-z", action="store_true", help="Launch in headless mode")
    @CommandArgument("--software", "-s", action="store_true", help="Launch with software rendering")
    @CommandArgument("params", nargs="...", help="Command-line arguments to be passed through to Servo")
    @CommandBase.common_command_arguments(binary_selection=True)
    @CommandBase.allow_target_configuration
    def run(
        self,
        servo_binary: str,
        params: list[str],
        debugger: bool = False,
        debugger_cmd: str | None = None,
        headless: bool = False,
        software: bool = False,
        emulator: bool = False,
        usb: bool = False,
    ) -> int | None:
        return self._run(servo_binary, params, debugger, debugger_cmd, headless, software, emulator, usb)

    def _run(
        self,
        servo_binary: str,
        params: list[str],
        debugger: bool = False,
        debugger_cmd: str | None = None,
        headless: bool = False,
        software: bool = False,
        emulator: bool = False,
        usb: bool = False,
    ) -> int | None:
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"
        if software:
            if not is_linux():
                print("Software rendering is only supported on Linux at the moment.")
                return

            env["LIBGL_ALWAYS_SOFTWARE"] = "1"
        os.environ.update(env)

        # Make --debugger-cmd imply --debugger
        if debugger_cmd:
            debugger = True

        if is_android(self.target):
            if debugger:
                print("Android on-device debugging is not supported by mach yet. See")
                print("https://github.com/servo/servo/wiki/Building-for-Android#debugging-on-device")
                return
            script = [
                f"am force-stop {ANDROID_APP_NAME}",
            ]
            json_params = shell_quote(json.dumps(params))
            extra = "-e servoargs " + json_params
            rust_log = env.get("RUST_LOG", None)
            if rust_log:
                extra += " -e servolog " + rust_log
            gst_debug = env.get("GST_DEBUG", None)
            if gst_debug:
                extra += " -e gstdebug " + gst_debug
            script += [
                f"am start {extra} {ANDROID_APP_NAME}/{ANDROID_APP_NAME}.MainActivity",
                "sleep 0.5",
                f"echo Servo PID: $(pidof {ANDROID_APP_NAME})",
                f"logcat --pid=$(pidof {ANDROID_APP_NAME})",
                "exit",
            ]
            args = [self.android_adb_path(env)]
            if emulator and usb:
                print("Cannot run in both emulator and USB at the same time.")
                return 1
            if emulator:
                args += ["-e"]
            if usb:
                args += ["-d"]
            shell = subprocess.Popen(args + ["shell"], stdin=subprocess.PIPE)
            shell.communicate("\n".join(script) + "\n")
            return shell.wait()

        args = [servo_binary]

        if headless:
            args.append("-z")

        # Borrowed and modified from:
        # http://hg.mozilla.org/mozilla-central/file/c9cfa9b91dea/python/mozbuild/mozbuild/mach_commands.py#l883
        if debugger:
            if not debugger_cmd:
                # No debugger name was provided. Look for the default ones on
                # current OS.
                debugger_cmd = mozdebug.get_default_debugger_name(mozdebug.DebuggerSearch.KeepLooking)

            debugger_info = mozdebug.get_debugger_info(debugger_cmd)
            if not debugger_info:
                print("Could not find a suitable debugger in your PATH.")
                return 1

            command = debugger_info.path
            if debugger_cmd == "gdb" or debugger_cmd == "lldb":
                rust_command = "rust-" + debugger_cmd
                try:
                    subprocess.check_call([rust_command, "--version"], env=env, stdout=open(os.devnull, "w"))
                except (OSError, subprocess.CalledProcessError):
                    pass
                else:
                    command = rust_command

            # Prepend the debugger args.
            args = [command] + debugger_info.args + args + params
        else:
            args = args + params

        try:
            check_call(args, env=env)
        except subprocess.CalledProcessError as exception:
            if exception.returncode < 0:
                print(f"Servo was terminated by signal {-exception.returncode}")
            else:
                print(f"Servo exited with non-zero status {exception.returncode}")
            return exception.returncode
        except OSError as exception:
            if exception.errno == 2:
                print("Servo Binary can't be found! Run './mach build' and try again!")
            else:
                raise exception

    @Command("coverage-report", description="Create Servo Code Coverage report.", category="post-build")
    @CommandArgument("params", nargs="...", help="Command-line arguments to be passed through to cargo llvm-cov")
    @CommandBase.common_command_arguments(binary_selection=True, build_type=True, coverage_report=True)
    def coverage_report(self, build_type: BuildType, params: Optional[List[str]] = None, **kwargs: Any) -> int:
        target_dir = servo.util.get_target_dir()
        # See `cargo llvm-cov show-env`. We only export the values required at runtime.
        os.environ["CARGO_LLVM_COV"] = "1"
        os.environ["CARGO_LLVM_COV_SHOW_ENV"] = "1"
        os.environ["CARGO_LLVM_COV_TARGET_DIR"] = target_dir
        profraw_files = [pathlib.Path(target_dir).joinpath(f) for f in os.listdir(target_dir) if f.endswith(".profraw")]
        zero_size_profraw_files = [f for f in profraw_files if os.path.getsize(f) == 0]
        suspicously_small_files = [f for f in profraw_files if 0 < os.path.getsize(f) < 8000000 ]
        filtered_profraw_files = [f for f in profraw_files if os.path.getsize(f) > 0]
        if len(zero_size_profraw_files) > 0:
            print(f"Warning found {len(zero_size_profraw_files)} zero-sized profraw files: {zero_size_profraw_files}")
            print(f"Removing {len(zero_size_profraw_files)} files, keeping {len(filtered_profraw_files)} files.")
            for file in zero_size_profraw_files:
                os.remove(file)
        if len(suspicously_small_files) > 0:
            print(f"Warning found {len(suspicously_small_files)} files with size < 8MB: {suspicously_small_files}")
            for file in suspicously_small_files:
                print(f"Removing {file}")
                os.rename(file, str(file) + ".broken")
        try:
            cargo_llvm_cov_cmd = ["cargo", "llvm-cov", "report", "--target", self.target.triple()]
            cargo_llvm_cov_cmd.extend(build_type.as_cargo_arg())
            cargo_llvm_cov_cmd.extend(params or [])
            subprocess.check_call(cargo_llvm_cov_cmd)
        except subprocess.CalledProcessError as exception:
            if exception.returncode < 0:
                print(f"`cargo llvm-cov` was terminated by signal {-exception.returncode}")
            else:
                print(f"`cargo llvm-cov` exited with non-zero status {exception.returncode}")
            return exception.returncode
        return 0

    @Command("android-emulator", description="Run the Android emulator", category="post-build")
    @CommandArgument("args", nargs="...", help="Command-line arguments to be passed through to the emulator")
    def android_emulator(self, args: list[str] | None = None) -> int:
        if not args:
            args = []
            print("AVDs created by `./mach bootstrap-android` are servo-arm and servo-x86.")
        emulator = self.android_emulator_path(self.build_env())
        return subprocess.call([emulator] + args)

    @Command("rr-record", description="Run Servo whilst recording execution with rr", category="post-build")
    @CommandArgument("params", nargs="...", help="Command-line arguments to be passed through to Servo")
    @CommandBase.common_command_arguments(binary_selection=True)
    def rr_record(self, servo_binary: str, params: list[str] = []) -> None:
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"

        servo_cmd = [servo_binary] + params
        rr_cmd = ["rr", "--fatal-errors", "record"]
        try:
            check_call(rr_cmd + servo_cmd)
        except OSError as e:
            if e.errno == 2:
                print("rr binary can't be found!")
            else:
                raise e

    @Command(
        "rr-replay",
        description="Replay the most recent execution of Servo that was recorded with rr",
        category="post-build",
    )
    def rr_replay(self) -> None:
        try:
            check_call(["rr", "--fatal-errors", "replay"])
        except OSError as e:
            if e.errno == 2:
                print("rr binary can't be found!")
            else:
                raise e

    @Command("doc", description="Generate documentation", category="post-build")
    @CommandArgument("params", nargs="...", help="Command-line arguments to be passed through to cargo doc")
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def doc(self, params: list[str], **kwargs: Any) -> CompletedProcess[bytes] | int | None:
        self.ensure_bootstrapped()

        docs = path.join(servo.util.get_target_dir(), "doc")
        if not path.exists(docs):
            os.makedirs(docs)

        # Document library crates to avoid package name conflict between severoshell
        # and libservo. Besides, main.rs in servoshell is just a stub.
        params.insert(0, "--lib")
        # Documentation build errors shouldn't cause the entire build to fail. This
        # prevents issues with dependencies from breaking our documentation build,
        # with the downside that it hides documentation issues.
        params.insert(0, "--keep-going")

        env = self.build_env()
        env["RUSTC"] = "rustc"
        returncode = self.run_cargo_build_like_command("doc", params, env=env, **kwargs)
        if returncode:
            return returncode

        static = path.join(self.context.topdir, "etc", "doc.servo.org")
        for name in os.listdir(static):
            copy2(path.join(static, name), path.join(docs, name))
