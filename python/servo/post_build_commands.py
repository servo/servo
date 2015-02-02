from __future__ import print_function, unicode_literals

import argparse
import os
import os.path as path
from os import chdir
import subprocess
import SimpleHTTPServer
import SocketServer
import mozdebug
from shutil import copytree, rmtree, ignore_patterns, copy2

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase


def read_file(filename, if_exists=False):
    if if_exists and not path.exists(filename):
        return None
    with open(filename) as f:
        return f.read()


@CommandProvider
class MachCommands(CommandBase):
    @Command('run',
             description='Run Servo',
             category='post-build')
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
    def run(self, params, debug=False, debugger=None):
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"

        args = [path.join("components", "servo", "target", "servo")]

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
            args = ([self.debuggerInfo.path] + self.debuggerInfo.args
                    + args + params)
        else:
            args = args + params

        subprocess.check_call(args, env=env)

    @Command('doc',
             description='Generate documentation',
             category='post-build')
    @CommandArgument(
        'params', nargs='...',
        help="Command-line arguments to be passed through to cargo doc")
    def doc(self, params):
        self.ensure_bootstrapped()
        if not path.exists(path.join(
               self.config["tools"]["rust-root"], "doc")):
            Registrar.dispatch("bootstrap-rust-docs", context=self.context)
        rust_docs = path.join(self.config["tools"]["rust-root"], "doc")
        docs = path.join(
            self.context.topdir, "components", "servo", "target", "doc")
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
            self.servo_crate(), "target", "doc", "servo", "index.html")))
