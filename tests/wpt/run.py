# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import argparse
import multiprocessing
import os
import sys

here = os.path.split(__file__)[0]
servo_root = os.path.abspath(os.path.join(here, "..", ".."))

def wpt_path(*args):
    return os.path.join(here, *args)

def servo_path(*args):
    return os.path.join(servo_root, *args)

# Imports
sys.path.append(wpt_path("harness"))
from wptrunner import wptrunner, wptcommandline

def run_tests(paths=None, **kwargs):
    if paths is None:
        paths = {}
    set_defaults(paths, kwargs)
    wptrunner.setup_logging(kwargs, {"mach": sys.stdout})
    success = wptrunner.run_tests(**kwargs)
    return 0 if success else 1

def set_defaults(paths, kwargs):
    if kwargs.get("product") is None:
        kwargs["product"] = "servo"

    if kwargs.get("config") is None and "config" in paths:
        kwargs["config"] = paths["config"]

    if kwargs.get("include_manifest") is None and "include_manifest" in paths:
        kwargs["include_manifest"] = paths["include_manifest"]

    if kwargs.get("binary") is None:
        bin_dir = "release" if kwargs.get("release") else "debug"
        bin_path = servo_path("target", bin_dir, "servo")

        kwargs["binary"] = bin_path

    if kwargs.get("processes") is None:
        kwargs["processes"] = multiprocessing.cpu_count()

    wptcommandline.check_args(kwargs)

def main(paths=None):
    parser = wptcommandline.create_parser()
    kwargs = vars(parser.parse_args())
    return run_tests(paths, **kwargs)
