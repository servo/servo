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
sys.path.append(wptsubdir("web-platform-tests", "tools", "scripts"))
from wptrunner import wptrunner, wptcommandline

def manifest_path(**kwargs):
    return wptsubdir("csswg-metadata", "MANIFEST.json") if kwargs['csswg'] else wptsubdir("metadata", "MANIFEST.json")

def update_manifest(args):
    subdir = wptsubdir("csswg-test") if args.csswg else wptsubdir("web-platform-tests")
    import manifest
    if args.csswg:
        manifest._repo_root = wptsubdir("csswg-test")
    opts = {
        'tests_root': subdir,
        'path': manifest_path(**vars(args)),
        'rebuild': False,
        'experimental_include_local_changes': True,
        'url_base': '/',
    }
    manifest.update_from_cli(**opts)
    return True

def run_tests(**kwargs):
    if not os.path.isfile(manifest_path(**kwargs)):
        raise Exception("Manifest not found. Please use --update-manifest in WPTARGS to create one")
    wptrunner.setup_logging(kwargs, {"raw": sys.stdout})
    return wptrunner.run_tests(**kwargs)

def default_manifest(args):
    return wptsubdir("csswg-include.ini") if args.csswg else wptsubdir("include.ini")

def set_defaults(args):
    args.include_manifest = args.include_manifest if args.include_manifest else default_manifest(args)
    args.product = "servo"
    rv = vars(args)
    wptcommandline.check_args(rv)
    return rv

def main():
    parser = wptcommandline.create_parser()
    parser.add_argument('--update-manifest', dest='update_manifest', action='store_true')
    parser.add_argument('--csswg', dest='csswg', action='store_true')
    args = parser.parse_args()
    if args.update_manifest:
        return update_manifest(args)
    kwargs = set_defaults(args)
    return run_tests(**kwargs)

if __name__ == "__main__":
    sys.exit(0 if main() else 1)
