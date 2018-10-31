# docker-worker on Packet.net

This is the configuration for the `proj-servo/docker-worker-kvm` worker type.
It is similar to `aws-provisioner/docker-worker`,
except that it runs on a server from Packet.net.
This server is “real” non-virtualized hardware,
so that Intel VT-x instructions are available and we can run KVM.
KVM is required for the Android emulator’s CPU acceleration,
which in turn is required to run OpenGL ES 3 (not just 2) in the guest system.

## Setup

* [Install Terraform](https://www.terraform.io/downloads.html)
* [Install taskcluster-cli](https://github.com/taskcluster/taskcluster-cli/#installation)
* Run ``eval `taskcluster signin` `` (once per open terminal/shell)
* Run `./terraform_with_vars.py init` (once per checkout of the Servo repository)

## List running servers

* Run `./list_devices.py`

## (Re)deploying a server

* Run `./terraform_with_vars.py plan`
* If the plan looks good, run `./terraform_with_vars.py apply`
* Watch the new server being installed. Terraform should finish in 15~20 minutes.

## Taskcluster secrets

`terraform_with_vars.py` uses Taskcluster’s
[secrets service](https://tools.taskcluster.net/secrets/).
These secrets include an [authentication token](
https://app.packet.net/projects/e3d0d8be-9e4c-4d39-90af-38660eb70544/settings/api-keys)
for Packet.net’s API.
You’ll need to authenticate with a Taskcluster client ID
that has scope `secrets:get:project/servo/*`.
This should be the case if you’re a Servo project administrator (the `project-admin:servo` role).

## Worker’s client ID

Workers are configured to authenticate with client ID
[project/servo/worker/docker-worker-kvm/1](
https://tools.taskcluster.net/auth/clients/project%2Fservo%2Fworker%2Fdocker-worker-kvm%2F1).
This client has the scopes required to run docker-worker
as well as for tasks that we run on this worker type.