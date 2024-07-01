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
import subprocess
from shutil import copy2
from typing import List

import mozdebug

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

import servo.util
import servo.platform

from servo.command_base import (
    BuildType,
    CommandBase,
    check_call,
    is_linux,
)


ANDROID_APP_NAME = "org.servo.servoshell"


def read_file(filename, if_exists=False):
    if if_exists and not path.exists(filename):
        return None
    with open(filename) as f:
        return f.read()


# Copied from Python 3.3+'s shlex.quote()
def shell_quote(arg):
    # use single quotes, and put single quotes into double quotes
    # the string $'b is then quoted as '$'"'"'b'
    return "'" + arg.replace("'", "'\"'\"'") + "'"


@CommandProvider
class PostBuildCommands(CommandBase):
    @Command('run',
             description='Run Servo',
             category='post-build')
    @CommandArgument('--android', action='store_true', default=None,
                     help='Run on an Android device through `adb shell`')
    @CommandArgument('--emulator',
                     action='store_true',
                     help='For Android, run in the only emulated device')
    @CommandArgument('--usb',
                     action='store_true',
                     help='For Android, run in the only USB device')
    @CommandArgument('--debugger', action='store_true',
                     help='Enable the debugger. Not specifying a '
                          '--debugger-cmd option will result in the default '
                          'debugger being used. The following arguments '
                          'have no effect without this.')
    @CommandArgument('--debugger-cmd', default=None, type=str,
                     help='Name of debugger to use.')
    @CommandArgument('--headless', '-z', action='store_true',
                     help='Launch in headless mode')
    @CommandArgument('--software', '-s', action='store_true',
                     help='Launch with software rendering')
    @CommandArgument('--bin', default=None,
                     help='Launch with specific binary')
    @CommandArgument('--nightly', '-n', default=None,
                     help='Specify a YYYY-MM-DD nightly build to run')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to Servo")
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def run(self, params, build_type: BuildType, android=None, debugger=False, debugger_cmd=None,
            headless=False, software=False, bin=None, emulator=False, usb=False, nightly=None, with_asan=False):
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"
        if software:
            if not is_linux():
                print("Software rendering is only supported on Linux at the moment.")
                return

            env['LIBGL_ALWAYS_SOFTWARE'] = "1"
        os.environ.update(env)

        # Make --debugger-cmd imply --debugger
        if debugger_cmd:
            debugger = True

        if android is None:
            android = self.config["build"]["android"]

        if android:
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
                "exit"
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
            shell.communicate(bytes("\n".join(script) + "\n", "utf8"))
            return shell.wait()

        args = [bin or self.get_nightly_binary_path(nightly) or self.get_binary_path(build_type, asan=with_asan)]

        if headless:
            args.append('-z')

        # Borrowed and modified from:
        # http://hg.mozilla.org/mozilla-central/file/c9cfa9b91dea/python/mozbuild/mozbuild/mach_commands.py#l883
        if debugger:
            if not debugger_cmd:
                # No debugger name was provided. Look for the default ones on
                # current OS.
                debugger_cmd = mozdebug.get_default_debugger_name(
                    mozdebug.DebuggerSearch.KeepLooking)

            debugger_info = mozdebug.get_debugger_info(debugger_cmd)
            if not debugger_info:
                print("Could not find a suitable debugger in your PATH.")
                return 1

            command = debugger_info.path
            if debugger_cmd == 'gdb' or debugger_cmd == 'lldb':
                rust_command = 'rust-' + debugger_cmd
                try:
                    subprocess.check_call([rust_command, '--version'], env=env, stdout=open(os.devnull, 'w'))
                except (OSError, subprocess.CalledProcessError):
                    pass
                else:
                    command = rust_command

            # Prepend the debugger args.
            args = ([command] + debugger_info.args + args + params)
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
                print("Servo Binary can't be found! Run './mach build'"
                      " and try again!")
            else:
                raise exception

    @Command('android-emulator',
             description='Run the Android emulator',
             category='post-build')
    @CommandArgument(
        'args', nargs='...',
        help="Command-line arguments to be passed through to the emulator")
    def android_emulator(self, args=None):
        if not args:
            print("AVDs created by `./mach bootstrap-android` are servo-arm and servo-x86.")
        emulator = self.android_emulator_path(self.build_env())
        return subprocess.call([emulator] + args)

    @Command('rr-record',
             description='Run Servo whilst recording execution with rr',
             category='post-build')
    @CommandArgument('--bin', default=None,
                     help='Launch with specific binary')
    @CommandArgument('--nightly', '-n', default=None,
                     help='Specify a YYYY-MM-DD nightly build to run')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to Servo")
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def rr_record(self, build_type: BuildType, bin=None, nightly=None, with_asan=False, params=[]):
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"

        servo_cmd = [bin or self.get_nightly_binary_path(nightly)
                     or self.get_binary_path(build_type, asan=with_asan)] + params
        rr_cmd = ['rr', '--fatal-errors', 'record']
        try:
            check_call(rr_cmd + servo_cmd)
        except OSError as e:
            if e.errno == 2:
                print("rr binary can't be found!")
            else:
                raise e

    @Command('rr-replay',
             description='Replay the most recent execution of Servo that was recorded with rr',
             category='post-build')
    def rr_replay(self):
        try:
            check_call(['rr', '--fatal-errors', 'replay'])
        except OSError as e:
            if e.errno == 2:
                print("rr binary can't be found!")
            else:
                raise e

    @Command('doc',
             description='Generate documentation',
             category='post-build')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to cargo doc")
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def doc(self, params: List[str], **kwargs):
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
