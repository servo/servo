#!/usr/bin/python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os
import sys
import subprocess

import tc


def main(*args):
    tc.check()
    ssh_key = tc.secret("project/servo/ssh-keys/docker-worker-kvm")
    tc_creds = tc.secret("project/servo/tc-client/worker/docker-worker-kvm/1")
    livelog = tc.livelog()

    terraform_vars = dict(
        ssh_pub_key=ssh_key["public"],
        ssh_priv_key=ssh_key["private"],
        taskcluster_client_id=tc_creds["client_id"],
        taskcluster_access_token=tc_creds["access_token"],
        packet_api_key=tc.packet_auth_token(),
        ssl_certificate=livelog["livelog_cert_base64"],
        cert_key=livelog["livelog_key_base64"],
    )
    env = dict(os.environ)
    env["PACKET_AUTH_TOKEN"] = terraform_vars["packet_api_key"]
    env.update({"TF_VAR_" + k: v for k, v in terraform_vars.items()})
    cwd = os.path.abspath(os.path.dirname(__file__))
    sys.exit(subprocess.call(["terraform"] + list(args), env=env, cwd=cwd))


if __name__ == "__main__":
    main(*sys.argv[1:])
