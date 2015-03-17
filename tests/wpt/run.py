# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import sys, os, argparse

here = os.path.split(__file__)[0]
servo_root = os.path.abspath(os.path.join(here, "..", ".."))

def wptsubdir(*args):
    return os.path.join(here, *args)

# Imports
sys.path.append(wptsubdir("web-platform-tests"))
from wptrunner import wptrunner, wptcommandline

def run_tests(**kwargs):
    wptrunner.setup_logging(kwargs, {"raw": sys.stdout})
    return wptrunner.run_tests(**kwargs)

def set_defaults(args):
    args.include_manifest = args.include_manifest if args.include_manifest else wptsubdir("include.ini")
    args.product = "servo"
    rv = vars(args)
    wptcommandline.check_args(rv)
    return rv

def main():
    parser = wptcommandline.create_parser()
    args = parser.parse_args()
    kwargs = set_defaults(args)
    return run_tests(**kwargs)

if __name__ == "__main__":
    sys.exit(0 if main() else 1)
