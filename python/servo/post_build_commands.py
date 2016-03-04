# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals

from glob import glob
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

from servo.command_base import CommandBase, cd, call, check_call


def read_file(filename, if_exists=False):
    if if_exists and not path.exists(filename):
        return None
    with open(filename) as f:
        return f.read()


def find_dep_path_newest(package, bin_path):
    deps_path = path.join(path.split(bin_path)[0], "build")
    with cd(deps_path):
        print(os.getcwd())
        candidates = glob(package + '-*')
    candidates = (path.join(deps_path, c) for c in candidates)
    candidate_times = sorted(((path.getmtime(c), c) for c in candidates), reverse=True)
    if len(candidate_times) > 0:
        return candidate_times[0][1]
    return None


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
    @CommandArgument('--debug', action='store_true',
                     help='Enable the debugger. Not specifying a '
                          '--debugger option will result in the default '
                          'debugger being used. The following arguments '
                          'have no effect without this.')
    @CommandArgument('--debugger', default=None, type=str,
                     help='Name of debugger to use.')
    @CommandArgument('--browserhtml', '-b', action='store_true',
                     help='Launch with Browser.html')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to Servo")
    def run(self, params, release=False, dev=False, android=None, debug=False, debugger=None, browserhtml=False):
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"

        if android is None:
            android = self.config["build"]["android"]

        if android:
            if debug:
                print("Android on-device debugging is not supported by mach yet. See")
                print("https://github.com/servo/servo/wiki/Building-for-Android#debugging-on-device")
                return
            script = [
                "am force-stop com.mozilla.servo",
                "echo servo >/sdcard/servo/android_params"
            ]
            for param in params:
                script += [
                    "echo '%s' >>/sdcard/servo/android_params" % param.replace("'", "\\'")
                ]
            script += [
                "am start com.mozilla.servo/com.mozilla.servo.MainActivity",
                "exit"
            ]
            shell = subprocess.Popen(["adb", "shell"], stdin=subprocess.PIPE)
            shell.communicate("\n".join(script) + "\n")
            return shell.wait()

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

            command = self.debuggerInfo.path
            if debugger == 'gdb' or debugger == 'lldb':
                rustCommand = 'rust-' + debugger
                try:
                    subprocess.check_call([rustCommand, '--version'], env=env, stdout=open(os.devnull, 'w'))
                    command = rustCommand
                except (OSError, subprocess.CalledProcessError):
                    pass

            # Prepend the debugger args.
            args = ([command] + self.debuggerInfo.args +
                    args + params)
        elif browserhtml:
            browserhtml_path = find_dep_path_newest('browserhtml', args[0])
            if browserhtml_path is None:
                print("Could not find browserhtml package; perhaps you haven't built Servo.")
                return 1
            args = args + ['-w', '-b', '--pref', 'dom.mozbrowser.enabled',
                           path.join(browserhtml_path, 'out', 'index.html')]
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

        return call(["cargo", "doc"] + params,
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
    def package(self, release=False, dev=False, debug=False, debugger=None):
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

    @Command('install',
             description='Install Servo (currently, Android only)',
             category='post-build')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Package the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Package the dev build')
    def install(self, release=False, dev=False):
        binary_path = self.get_binary_path(release, dev, android=True)
        if not path.exists(binary_path):
            # TODO: Run the build command automatically?
            print("Servo build not found. Please run `./mach build` to compile Servo.")
            return 1

        apk_path = binary_path + ".apk"
        if not path.exists(apk_path):
            result = Registrar.dispatch("package", context=self.context, release=release, dev=dev)
            if result is not 0:
                return result

        print(["adb", "install", "-r", apk_path])
        return subprocess.call(["adb", "install", "-r", apk_path], env=self.build_env())
