# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import run
import sys

paths = {"include_manifest": run.wpt_path("include_css.ini"),
         "config": run.wpt_path("config_css.ini")}


def run_tests(**kwargs):
    return run.run_tests(paths=paths, **kwargs)


def main():
    return run.main(paths)

if __name__ == "__main__":
    sys.exit(0 if main() else 1)
