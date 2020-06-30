# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals

import json
import os
import os.path as path
import subprocess
from shutil import copytree, rmtree, copy2

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import (
    CommandBase,
    check_call, check_output, BIN_SUFFIX,
    is_linux,
)


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
    @CommandArgument('--release', '-r', action='store_true',
                     help='Run the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Run the dev build')
    @CommandArgument('--android', action='store_true', default=None,
                     help='Run on an Android device through `adb shell`')
    @CommandArgument('--emulator',
                     action='store_true',
                     help='For Android, run in the only emulated device')
    @CommandArgument('--usb',
                     action='store_true',
                     help='For Android, run in the only USB device')
    @CommandArgument('--debug', action='store_true',
                     help='Enable the debugger. Not specifying a '
                          '--debugger option will result in the default '
                          'debugger being used. The following arguments '
                          'have no effect without this.')
    @CommandArgument('--debugger', default=None, type=str,
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
    def run(self, params, release=False, dev=False, android=None, debug=False, debugger=None,
            headless=False, software=False, bin=None, emulator=False, usb=False, nightly=None):
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"

        # Make --debugger imply --debug
        if debugger:
            debug = True

        if android is None:
            android = self.config["build"]["android"]

        if android:
            if debug:
                print("Android on-device debugging is not supported by mach yet. See")
                print("https://github.com/servo/servo/wiki/Building-for-Android#debugging-on-device")
                return
            script = [
                "am force-stop org.mozilla.servo",
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
                "am start " + extra + " org.mozilla.servo/org.mozilla.servo.MainActivity",
                "sleep 0.5",
                "echo Servo PID: $(pidof org.mozilla.servo)",
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
            shell.communicate("\n".join(script) + "\n")
            return shell.wait()

        args = [bin or self.get_nightly_binary_path(nightly) or self.get_binary_path(release, dev)]

        if headless:
            args.append('-z')

        if software:
            if not is_linux():
                print("Software rendering is only supported on Linux at the moment.")
                return

            env['LIBGL_ALWAYS_SOFTWARE'] = "1"

        # Borrowed and modified from:
        # http://hg.mozilla.org/mozilla-central/file/c9cfa9b91dea/python/mozbuild/mozbuild/mach_commands.py#l883
        if debug:
            import mozdebug
            if not debugger:
                # No debugger name was provided. Look for the default ones on
                # current OS.
                debugger = mozdebug.get_default_debugger_name(
                    mozdebug.DebuggerSearch.KeepLooking)

            self.debuggerInfo = mozdebug.get_debugger_info(debugger)
            if not self.debuggerInfo:
                print("Could not find a suitable debugger in your PATH.")
                return 1

            command = self.debuggerInfo.path
            if debugger == 'gdb' or debugger == 'lldb':
                rustCommand = 'rust-' + debugger
                try:
                    subprocess.check_call([rustCommand, '--version'], env=env, stdout=open(os.devnull, 'w'))
                except (OSError, subprocess.CalledProcessError):
                    pass
                else:
                    command = rustCommand

            # Prepend the debugger args.
            args = ([command] + self.debuggerInfo.args
                    + args + params)
        else:
            args = args + params

        try:
            check_call(args, env=env)
        except subprocess.CalledProcessError as e:
            print("Servo exited with return value %d" % e.returncode)
            return e.returncode
        except OSError as e:
            if e.errno == 2:
                print("Servo Binary can't be found! Run './mach build'"
                      " and try again!")
            else:
                raise e

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
    @CommandArgument('--release', '-r', action='store_true',
                     help='Use release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Use dev build')
    @CommandArgument('--bin', default=None,
                     help='Launch with specific binary')
    @CommandArgument('--nightly', '-n', default=None,
                     help='Specify a YYYY-MM-DD nightly build to run')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to Servo")
    def rr_record(self, release=False, dev=False, bin=None, nightly=None, params=[]):
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"

        servo_cmd = [bin or self.get_nightly_binary_path(nightly)
                     or self.get_binary_path(release, dev)] + params
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
    @CommandBase.build_like_command_arguments
    def doc(self, params, features, target=None, android=False, magicleap=False,
            media_stack=None, **kwargs):
        self.ensure_bootstrapped(rustup_components=["rust-docs"])
        rustc_path = check_output(
            ["rustup" + BIN_SUFFIX, "which", "--toolchain", self.rust_toolchain(), "rustc"])
        assert path.basename(path.dirname(rustc_path)) == "bin"
        toolchain_path = path.dirname(path.dirname(rustc_path))
        rust_docs = path.join(toolchain_path, "share", "doc", "rust", "html")

        docs = path.join(self.get_target_dir(), "doc")
        if not path.exists(docs):
            os.makedirs(docs)

        if read_file(path.join(docs, "version_info.html"), if_exists=True) != \
                read_file(path.join(rust_docs, "version_info.html")):
            print("Copying Rust documentation.")
            # copytree doesn't like the destination already existing.
            for name in os.listdir(rust_docs):
                if not name.startswith('.'):
                    full_name = path.join(rust_docs, name)
                    destination = path.join(docs, name)
                    if path.isdir(full_name):
                        if path.exists(destination):
                            rmtree(destination)
                        copytree(full_name, destination)
                    else:
                        copy2(full_name, destination)

        features = features or []

        target, android = self.pick_target_triple(target, android, magicleap)

        features += self.pick_media_stack(media_stack, target)

        env = self.build_env(target=target, is_build=True, features=features)

        returncode = self.run_cargo_build_like_command("doc", params, features=features, env=env, **kwargs)
        if returncode:
            return returncode

        static = path.join(self.context.topdir, "etc", "doc.servo.org")
        for name in os.listdir(static):
            copy2(path.join(static, name), path.join(docs, name))

    @Command('browse-doc',
             description='Generate documentation and open it in a web browser',
             category='post-build')
    def serve_docs(self):
        self.doc([])
        import webbrowser
        webbrowser.open("file://" + path.abspath(path.join(
            self.get_target_dir(), "doc", "servo", "index.html")))
