# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import sys

import mozlog.commandline

from . import test

SCRIPT_PATH = os.path.abspath(os.path.dirname(__file__))
SERVO_ROOT = os.path.abspath(os.path.join(SCRIPT_PATH, "..", ".."))
WPT_PATH = os.path.join(SERVO_ROOT, "tests", "wpt")
WPT_TOOLS_PATH = os.path.join(WPT_PATH, "tests", "tools")
CERTS_PATH = os.path.join(WPT_TOOLS_PATH, "certs")

sys.path.insert(0, WPT_TOOLS_PATH)
import localpaths  # noqa: F401,E402
import wptrunner.wptcommandline  # noqa: E402


def create_parser():
    parser = wptrunner.wptcommandline.create_parser()
    parser.add_argument('--rr-chaos', default=False, action="store_true",
                        help="Run under chaos mode in rr until a failure is captured")
    parser.add_argument('--pref', default=[], action="append", dest="prefs",
                        help="Pass preferences to servo")
    parser.add_argument('--legacy-layout', '--layout-2013', '--with-layout-2013', default=False,
                        action="store_true", help="Use expected results for the legacy layout engine")
    parser.add_argument('--log-servojson', action="append", type=mozlog.commandline.log_file,
                        help="Servo's JSON logger of unexpected results")
    parser.add_argument('--always-succeed', default=False, action="store_true",
                        help="Always yield exit code of zero")
    parser.add_argument('--no-default-test-types', default=False, action="store_true",
                        help="Run all of the test types provided by wptrunner or specified explicitly by --test-types")
    parser.add_argument('--filter-intermittents', default=None, action="store",
                        help="Filter intermittents against known intermittents "
                             "and save the filtered output to the given file.")
    parser.add_argument('--log-raw-unexpected', default=None, action="store",
                        help="Raw structured log messages for unexpected results."
                             " '--log-raw' Must also be passed in order to use this.")
    return parser


def update_args_for_legacy_layout(kwargs: dict):
    kwargs["test_paths"]["/"].metadata_path = os.path.join(
        WPT_PATH, "meta-legacy-layout"
    )
    kwargs["test_paths"]["/_mozilla/"].metadata_path = os.path.join(
        WPT_PATH, "mozilla", "meta-legacy-layout"
    )
    kwargs["test_paths"]["/_webgl/"].metadata_path = os.path.join(
        WPT_PATH, "webgl", "meta-legacy-layout"
    )


def run_tests():
    return test.run_tests()
