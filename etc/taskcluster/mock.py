#!/usr/bin/env bash

# Copyright 2018 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

''''set -e
python3 -m coverage run $0
python3 -m coverage report -m --fail-under 100
exit

Run the decision task with fake Taskcluster APIs, to catch Python errors before pushing.
'''

import os
import sys
from unittest.mock import MagicMock


class TaskclusterRestFailure(Exception):
    status_code = 404


class Index:
    __init__ = insertTask = lambda *_, **__: None

    def findTask(self, path):
        if decision_task.CONFIG.git_ref == "refs/heads/master":
            return {"taskId": "<from index>"}
        raise TaskclusterRestFailure


stringDate = str
slugId = b"<new id>".lower
sys.exit = Queue = fromNow = MagicMock()
sys.modules["taskcluster"] = sys.modules[__name__]
sys.dont_write_bytecode = True
os.environ.update(**{k: k for k in "TASK_ID TASK_OWNER TASK_SOURCE GIT_URL GIT_SHA".split()})
os.environ["GIT_REF"] = "refs/heads/auto"
os.environ["TASKCLUSTER_ROOT_URL"] = "https://community-tc.services.mozilla.com"
os.environ["TASKCLUSTER_PROXY_URL"] = "http://taskcluster"
os.environ["NEW_AMI_WORKER_TYPE"] = "-"
import decision_task  # noqa: E402
decision_task.decisionlib.subprocess = MagicMock()

print("\n# Push:")
decision_task.main("github-push")

print("\n# Push with hot caches:")
decision_task.main("github-push")

print("\n# Push to master:")
decision_task.CONFIG.git_ref = "refs/heads/master"
decision_task.main("github-push")

print("\n# Daily:")
decision_task.main("daily")

print("\n# Try AMI:")
decision_task.main("try-windows-ami")

print("\n# PR:")
decision_task.main("github-pull-request")

print()
