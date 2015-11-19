# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals

import os
import os.path as path
import subprocess
from shutil import copytree, rmtree, copy2

from mach.registrar import Registrar

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd


def read_file(filename, if_exists=False):
    if if_exists and not path.exists(filename):
        return None
    with open(filename) as f:
        return f.read()


@CommandProvider
class PostBuildCommands(CommandBase):
    @Command('run',
             description='Run Servo',
             category='post-build')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Run the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Run the dev build')
    @CommandArgument('--android', action='store_true',
                     help='Run on an Android device through `adb shell`')
    @CommandArgument('--debug', action='store_true',
                     help='Enable the debugger. Not specifying a '
                          '--debugger option will result in the default '
                          'debugger being used. The following arguments '
                          'have no effect without this.')
    @CommandArgument('--debugger', default=None, type=str,
                     help='Name of debugger to use.')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to Servo")
    def run(self, params, release=False, dev=False, android=False, debug=False, debugger=None):
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"

        if android:
            if debug:
                print("Android on-device debugging is not supported by mach yet. See")
                print("https://github.com/servo/servo/wiki/Building-for-Android#debugging-on-device")
                return
            if params:
                url = params[0]
            else:
                url = 'http://mozilla.org/'
            subprocess.Popen(["adb", "shell"], stdin=subprocess.PIPE).communicate('''
                am force-stop com.mozilla.servo
                export SERVO_URL='%s'
                am start com.mozilla.servo/com.mozilla.servo.MainActivity
                exit
            ''' % url.replace('\'', '\\\''))
            return

        args = [self.get_binary_path(release, dev)]

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

            # Prepend the debugger args.
            args = ([self.debuggerInfo.path] + self.debuggerInfo.args +
                    args + params)
        else:
            args = args + params

        try:
            subprocess.check_call(args, env=env)
        except subprocess.CalledProcessError as e:
            print("Servo exited with return value %d" % e.returncode)
            return e.returncode
        except OSError as e:
            if e.errno == 2:
                print("Servo Binary can't be found! Run './mach build'"
                      " and try again!")
            else:
                raise e

    @Command('rr-record',
             description='Run Servo whilst recording execution with rr',
             category='post-build')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Use release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Use dev build')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to Servo")
    def rr_record(self, release=False, dev=False, params=[]):
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"

        servo_cmd = [self.get_binary_path(release, dev)] + params
        rr_cmd = ['rr', '--fatal-errors', 'record']
        try:
            subprocess.check_call(rr_cmd + servo_cmd)
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
            subprocess.check_call(['rr', '--fatal-errors', 'replay'])
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
    def doc(self, params):
        self.ensure_bootstrapped()
        if not path.exists(path.join(self.config["tools"]["rust-root"], "doc")):
            Registrar.dispatch("bootstrap-rust-docs", context=self.context)
        rust_docs = path.join(self.config["tools"]["rust-root"], "doc")
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

        return subprocess.call(["cargo", "doc"] + params,
                               env=self.build_env(), cwd=self.servo_crate())

    @Command('browse-doc',
             description='Generate documentation and open it in a web browser',
             category='post-build')
    def serve_docs(self):
        self.doc([])
        import webbrowser
        webbrowser.open("file://" + path.abspath(path.join(
            self.get_target_dir(), "doc", "servo", "index.html")))

    @Command('package',
             description='Package Servo (currently, Android APK only)',
             category='post-build')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Package the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Package the dev build')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to Servo")
    def package(self, params, release=False, dev=False, debug=False, debugger=None):
        env = self.build_env()
        binary_path = self.get_binary_path(release, dev, android=True)

        if dev:
            env["NDK_DEBUG"] = "1"
            env["ANT_FLAVOR"] = "debug"
            dev_flag = "-d"
        else:
            env["ANT_FLAVOR"] = "release"
            dev_flag = ""

        target_dir = os.path.dirname(binary_path)
        output_apk = "{}.apk".format(binary_path)
        try:
            with cd(path.join("support", "android", "build-apk")):
                subprocess.check_call(["cargo", "run", "--", dev_flag, "-o", output_apk, "-t", target_dir,
                                       "-r", self.get_top_dir()], env=env)
        except subprocess.CalledProcessError as e:
            print("Packaging Android exited with return value %d" % e.returncode)
            return e.returncode
