# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

#!/usr/bin/env python
import os
import subprocess
import sys

from mozlog.structured import structuredlog

here = os.path.split(__file__)[0]

sys.path.insert(0, os.path.abspath(os.path.join(here, os.pardir, "harness")))
sys.path.insert(0, os.path.abspath(os.path.join(here, os.pardir, "tests", "tools", "scripts")))

from wptrunner.update import setup_logging, WPTUpdate
from wptrunner.update.base import exit_unclean

from . import updatecommandline
from .update import UpdateRunner

def run_update(logger, **kwargs):
    updater = WPTUpdate(logger, runner_cls=UpdateRunner, **kwargs)
    return updater.run()


if __name__ == "__main__":
    args = updatecommandline.parse_args()
    logger = setup_logging(args, {"mach": sys.stdout})
    assert structuredlog.get_default_logger() is not None


    rv = run_update(logger, **args)
    if rv is exit_unclean:
        sys.exit(1)
    else:
        sys.exit(0)
