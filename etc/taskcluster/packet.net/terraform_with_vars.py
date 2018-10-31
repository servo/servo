#!/usr/bin/python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import sys
import base64
import subprocess

import tc


def main(*args):
    tc.check()
    ssh_key = tc.secret("project/servo/ssh-keys/docker-worker-kvm")
    tc_creds = tc.secret("project/servo/tc-client/worker/docker-worker-kvm/1")
    win2016 = tc.api("awsProvisioner", "workerType", "servo-win2016")
    files_by_desc = {f.get("description"): f for f in win2016["secrets"]["files"]}

    def decode(description):
        f = files_by_desc[description]
        assert f["encoding"] == "base64"
        return base64.b64decode(f["content"])

    terraform_vars = dict(
        ssh_pub_key=ssh_key["public"],
        ssh_priv_key=ssh_key["private"],
        taskcluster_client_id=tc_creds["client_id"],
        taskcluster_access_token=tc_creds["access_token"],
        packet_api_key=tc.packet_auth_token(),
        ssl_certificate=decode("SSL certificate for livelog"),
        cert_key=decode("SSL key for livelog"),
    )
    env = dict(os.environ)
    env["PACKET_AUTH_TOKEN"] = terraform_vars["packet_api_key"]
    env.update({"TF_VAR_" + k: v for k, v in terraform_vars.items()})
    sys.exit(subprocess.call(["terraform"] + list(args), env=env))


if __name__ == "__main__":
    main(*sys.argv[1:])
