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

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import (
    CommandBase,
    check_call, check_output, BIN_SUFFIX,
    is_linux, is_windows, is_macosx, set_osmesa_env,
    get_browserhtml_path,
)


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
    @CommandArgument('--headless', '-z', action='store_true',
                     help='Launch in headless mode')
    @CommandArgument('--software', '-s', action='store_true',
                     help='Launch with software rendering')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to Servo")
    def run(self, params, release=False, dev=False, android=None, debug=False, debugger=None, browserhtml=False,
            headless=False, software=False):
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
                "am force-stop com.mozilla.servo",
                "echo servo >/sdcard/Android/data/com.mozilla.servo/files/android_params"
            ]
            for param in params:
                script += [
                    "echo '%s' >>/sdcard/Android/data/com.mozilla.servo/files/android_params"
                    % param.replace("'", "\\'")
                ]
            script += [
                "am start com.mozilla.servo/com.mozilla.servo.MainActivity",
                "exit"
            ]
            shell = subprocess.Popen(["adb", "shell"], stdin=subprocess.PIPE)
            shell.communicate("\n".join(script) + "\n")
            return shell.wait()

        args = [self.get_binary_path(release, dev)]

        if browserhtml:
            browserhtml_path = get_browserhtml_path(args[0])
            if is_macosx():
                # Enable borderless on OSX
                args = args + ['-b']
            elif is_windows():
                # Convert to a relative path to avoid mingw -> Windows path conversions
                browserhtml_path = path.relpath(browserhtml_path, os.getcwd())

            args = args + ['--pref', 'dom.mozbrowser.enabled',
                           '--pref', 'dom.forcetouch.enabled',
                           '--pref', 'shell.builtin-key-shortcuts.enabled=false',
                           path.join(browserhtml_path, 'index.html')]

        if headless:
            set_osmesa_env(args[0], env)
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
            args = ([command] + self.debuggerInfo.args +
                    args + params)
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
        env = os.environ.copy()
        env["RUSTUP_TOOLCHAIN"] = self.toolchain()
        rustc_path = check_output(["rustup" + BIN_SUFFIX, "which", "rustc"], env=env)
        assert path.basename(path.dirname(rustc_path)) == "bin"
        toolchain_path = path.dirname(path.dirname(rustc_path))
        rust_docs = path.join(toolchain_path, "share", "doc", "rust", "html")

        self.ensure_bootstrapped()
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

        return self.call_rustup_run(
            ["cargo", "doc", "--manifest-path", self.servo_manifest()] + params,
            env=self.build_env()
        )

    @Command('browse-doc',
             description='Generate documentation and open it in a web browser',
             category='post-build')
    def serve_docs(self):
        self.doc([])
        import webbrowser
        webbrowser.open("file://" + path.abspath(path.join(
            self.get_target_dir(), "doc", "servo", "index.html")))
