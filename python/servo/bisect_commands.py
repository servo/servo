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
import time
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
class BisectCommands(CommandBase):
    @Command('bisect',
             description='Run Servo using downloaded nightlies',
             category='bisect')
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
    def bisect(self, params, release=False, dev=False, android=None, debug=False, debugger=None, browserhtml=False,
            headless=False, software=False):
        # Run the command
        SCRIPT_PATH = os.path.split(__file__)[0]
        command_to_run = [os.path.abspath(os.path.join(SCRIPT_PATH, "..", "..", "mach"))]
        command_to_run.extend(params)
        print("Wrapping bisect around command {}".format(command_to_run))
        start = time.time()
        proc = subprocess.Popen(command_to_run)
        output = proc.communicate()
        if proc.returncode:
            print("Bisection on nightly {} was bad, trying older version".format("FOOBAR"))
        else:
            print("Bisection on nightly {} was good, trying newer version".format("FOOBAR11234"))
        end = time.time()
        print("Bisect completed in {} seconds.".format(round(end-start,2)))
