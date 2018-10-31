# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import sys
import json
import subprocess


def check():
    try:
        subprocess.check_output(["taskcluster", "version"])
    except FileNotFoundError:  # noqa: F821
        sys.exit("taskcluster CLI tool not available. Install it from "
                 "https://github.com/taskcluster/taskcluster-cli#installation")

    if "TASKCLUSTER_CLIENT_ID" not in os.environ or "TASKCLUSTER_ACCESS_TOKEN" not in os.environ:
        sys.exit("Taskcluster API credentials not available. Run this command and try again:\n\n"
                 "eval `taskcluster signin`\n")


def packet_auth_token():
    return secret("project/servo/packet.net-api-key")["key"]


def secret(name):
    return api("secrets", "get", name)["secret"]


def api(*args):
    args = ["taskcluster", "api"] + list(args)
    output = subprocess.check_output(args)
    if output:
        return json.loads(output)
