#!/usr/bin/python3

# Copyright 2018 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

"""
Run the decision task with fake Taskcluster APIs, to catch Python errors before pushing.
"""

import os
import sys
from unittest.mock import MagicMock


class TaskclusterRestFailure(Exception):
    status_code = 404


class Index:
    def __init__(self, options):
        pass

    def findTask(self, _):
        raise TaskclusterRestFailure


Queue = stringDate = fromNow = slugId = os.environ = MagicMock()
sys.modules["taskcluster"] = sys.modules[__name__]
sys.dont_write_bytecode = True
exec(open("decision-task.py").read())
