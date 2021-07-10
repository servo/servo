#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

''''set -e
cd "$(dirname $0)"
exec ../../python/_virtualenv/bin/python "$(basename $0)"
'''

try:
    import jsone
except ImportError:
    import sys
    sys.exit("pip install git+https://github.com/taskcluster/json-e")

import yaml
import json

template = yaml.load(open("../../.taskcluster.yml").read().decode("utf8"))
repo = dict(
    repository=dict(
        clone_url="https://github.com/servo/servo.git",
    ),
)
contexts = [
    dict(
        tasks_for="github-release",
        event=repo,
    ),
    dict(
        tasks_for="github-pull-request",
        event=dict(
            action="comment",
            **repo
        ),
    ),
    dict(
        tasks_for="github-push",
        event=dict(
            ref="refs/heads/master",
            compare="https://github.com/servo/servo/compare/1753cda...de09c8f",
            after="de09c8fb6ef87dec5932d5fab4adcb421d291a54",
            pusher=dict(
                name="bors-servo",
            ),
            **repo
        ),
    ),
    dict(
        tasks_for="github-pull-request",
        event=dict(
            action="synchronize",
            pull_request=dict(
                number=22583,
                url="https://github.com/servo/servo/pull/22583",
                head=dict(
                    sha="51a422c9ef47420eb69c802643b7686bdb498652",
                ),
                merge_commit_sha="876fcf7a5fe971a9ac0a4ce117906c552c08c095",
            ),
            sender=dict(
                login="jdm",
            ),
            **repo
        ),
    ),
]
for context in contexts:
    print(context["tasks_for"])
    print(json.dumps(jsone.render(template, context), indent=2))
