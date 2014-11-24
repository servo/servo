from __future__ import print_function, unicode_literals

import argparse
import os.path as path
from os import chdir
import subprocess
import SimpleHTTPServer
import SocketServer
import mozdebug
from shutil import copytree, rmtree, ignore_patterns

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase


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

        args = [path.join("target", "servo")]

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
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to cargo doc")
    def doc(self, params):
        self.ensure_bootstrapped()
        return subprocess.call(["cargo", "doc"] + params,
                               env=self.build_env())

    @Command('serve-docs',
             description='Locally serve Servo and Rust documentation',
             category='post-build')
    @CommandArgument(
        'port', default=8888, nargs='?', type=int, metavar='PORT',
        help="Port to serve documentation at (default is 8888)")
    def serve_docs(self, port):
        self.doc([])
        servedir = path.join("target", "serve-docs")
        docdir = path.join("target", "doc")

        rmtree(servedir, True)
        copytree(docdir, servedir, ignore=ignore_patterns('.*'))

        rustdocs = path.join("rust", self.rust_snapshot_path(), "doc")
        copytree(rustdocs, path.join(servedir, "rust"), ignore=ignore_patterns('.*'))

        chdir(servedir)
        Handler = SimpleHTTPServer.SimpleHTTPRequestHandler

        httpd = SocketServer.TCPServer(("", port), Handler)

        print("serving at port", port)
        httpd.serve_forever()
