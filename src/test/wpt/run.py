# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

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

def ensure_manifest():
    if not os.path.isfile(wptsubdir("metadata", "MANIFEST.json")):
        opts = argparse.Namespace(rebuild=False, experimental_include_local_changes=True,
                                  path=wptsubdir("metadata", "MANIFEST.json"))
        manifest.update_manifest(wptsubdir("web-platform-tests"), opts)

def run_tests(**kwargs):
    wptrunner.setup_logging(kwargs, {"raw": sys.stdout})
    return wptrunner.run_tests(**kwargs)

def set_defaults(args):
    args.metadata_root = args.metadata_root if args.metadata_root else wptsubdir("metadata")
    args.tests_root = args.tests_root if args.tests_root else wptsubdir("web-platform-tests")
    args.include = args.include if args.include else ["/dom"]
    args.binary = args.binary if args.binary else os.path.join(servo_root, "build", "servo")
    args.product = "servo"
    return vars(args)

def main():
    ensure_manifest()
    parser = wptcommandline.create_parser(False)
    args = parser.parse_args()
    kwargs = set_defaults(args)
    return run_tests(**kwargs)

if __name__ == "__main__":
    sys.exit(0 if main() else 1)
