# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import sys, os, argparse

here = os.path.split(__file__)[0]
servo_root = os.path.abspath(os.path.join(here, "..", "..", ".."))

def wptsubdir(*args):
    return os.path.join(here, *args)

# Imports
sys.path.append(wptsubdir("web-platform-tests"))
sys.path.append(wptsubdir("web-platform-tests", "tools", "scripts"))
from wptrunner import wptrunner, wptcommandline
import manifest

def update_manifest():
    manifest.update_manifest(wptsubdir("web-platform-tests"),
                             rebuild=False,
                             experimental_include_local_changes=True,
                             path=wptsubdir("metadata", "MANIFEST.json"))
    return True

def run_tests(**kwargs):
    if not os.path.isfile(wptsubdir("metadata", "MANIFEST.json")):
        raise Exception("Manifest not found. Please use --update-manifest in WPTARGS to create one")
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
    parser.add_argument('--update-manifest', dest='update_manifest', action='store_true')
    args = parser.parse_args()
    if args.update_manifest:
        return update_manifest()
    kwargs = set_defaults(args)
    return run_tests(**kwargs)

if __name__ == "__main__":
    sys.exit(0 if main() else 1)
