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

## Troubleshooting

SSH into `servo-tc-mac1.servo.org`.
`generic-worker` logs are in `less /Users/worker/stderr.log`.

If the worker seems stuck but nothing seems wrong in the log,
try running `launchctl stop net.generic.worker`.
(It is configured to restart automatically.)
This issue is tracked at
[generic-worker#133](https://github.com/taskcluster/generic-worker/issues/133).


## (Re)deploying a server

* Place an order or file a ticket with MacStadium to get a new hardware or reinstall an OS.

* Change the administrator password to one generated with
  `</dev/urandom tr -d -c 'a-zA-Z' | head -c 8; echo`
  (this short because of VNC),
  and save it in the shared 1Password account.

* Give the public IPv4 address a DNS name through Cloudflare.

* Add a correponding entry in the `config/roster` file.

* Log in through VNC, and run `xcode-select --install`


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