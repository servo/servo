# macOS

Servo’s macOS workers for Taskcluster are configured with
SaltStack in [agentless] mode.

[agentless]: https://docs.saltstack.com/en/getstarted/ssh/index.html

Either run `./salt-ssh`
to automatically install `salt-ssh` in `mach`’s existing Python virtualenv,
or install `salt-ssh` through some other mean and run in from this directory.

```sh
cd etc/taskcluster/macos
./salt-ssh '*' test.ping
./salt-ssh '*' state.apply test=True
```

## Worker’s client ID

`project/servo/worker/macos/1`