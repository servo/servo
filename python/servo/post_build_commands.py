from __future__ import print_function, unicode_literals

import os.path as path
from os import chdir
import subprocess
import SimpleHTTPServer
import SocketServer
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
             category='post-build',
             allow_all_args=True)
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to Servo")
    def run(self, params):
        env = self.build_env()
        env["RUST_BACKTRACE"] = "1"
        subprocess.check_call([path.join("target", "servo")] + params,
                              env=env)

    @Command('doc',
             description='Generate documentation',
             category='post-build',
             allow_all_args=True)
    @CommandArgument(
        'params', default=None, nargs='...',
        help="Command-line arguments to be passed through to cargo doc")
    def doc(self, params):
        self.ensure_bootstrapped()
        return subprocess.call(["cargo", "doc"] + params,
                               env=self.build_env())

    @Command('serve-docs',
             description='Locally serve Servo and Rust documentation',
             category='post-build',
             allow_all_args=True)
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
