# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os
import sys
import json
import base64
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


def livelog():
    win2016 = api("awsProvisioner", "workerType", "servo-win2016")
    files = win2016["secrets"]["files"]
    assert all(f["encoding"] == "base64" for f in files)
    files = {f.get("description"): f["content"] for f in files}
    cert = files["SSL certificate for livelog"]
    key = files["SSL key for livelog"]
    return {
        "livelog_cert_base64": cert,
        "livelog_key_base64": key,
        "livelog_cert": base64.b64decode(cert),
        "livelog_key": base64.b64decode(key),
        "livelog_secret": win2016["secrets"]["generic-worker"]["config"]["livelogSecret"],
    }


def packet_auth_token():
    return secret("project/servo/packet.net-api-key")["key"]


def secret(name):
    return api("secrets", "get", name)["secret"]


def api(*args):
    args = ["taskcluster", "api"] + list(args)
    output = subprocess.check_output(args)
    if output:
        return json.loads(output)
