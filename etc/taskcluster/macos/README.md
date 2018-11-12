# macOS

This is the configuration for the `proj-servo/macos` worker type.
These macOS workers are configured with SaltStack in [agentless] mode.

[agentless]: https://docs.saltstack.com/en/getstarted/ssh/index.html

Either run `./salt-ssh`
to automatically install `salt-ssh` in `mach`’s existing Python virtualenv,
or install `salt-ssh` through some other mean and run in from this directory.

```sh
cd etc/taskcluster/macos
./salt-ssh '*' test.ping
./salt-ssh '*' state.apply test=True
```

## Servers

Servers are provisioned manually from MacStadium.
The `config/roster` file lists them by DNS name.


## Taskcluster secrets

This SaltStack configuration has a custom module that uses Taskcluster’s
[secrets service](https://tools.taskcluster.net/secrets/).
These secrets include an [authentication token](
You’ll need to authenticate with a Taskcluster client ID
that has scope `secrets:get:project/servo/*`.
This should be the case if you’re a Servo project administrator (the `project-admin:servo` role).


## Worker’s client ID

Workers are configured to authenticate with client ID
[`project/servo/worker/macos/1`](
https://tools.taskcluster.net/auth/clients/project%2Fservo%2Fworker%macos%2F1).
This client has the scopes required to run tasks for this worker type.