# Treeherder for Servo

Treeherder is tool for visualizing the status of “trees”,
meaning branches in various source repositories.
It shows each push to the repository with the corresponding commits
as well as the CI jobs that were started for that push.
While it is possible to write other tools that submit job data,
CI integration is easiest with Taskcluster.

* [Production instance](https://treeherder.mozilla.org/)
* [Staging instance](https://treeherder.allizom.org/)
* [Source code](https://github.com/mozilla/treeherder/)


## Trees / repositories / branches

Treeherders knows a about a number of *repostories*.
Mercurial on Mozilla’s servers and git on GitHub are supported.
Despite the name, in the GitHub case
each Treeherder repository maps to one branch in a git repository.
They are configured in the [`repository.json`] file.
As of this writing there are four for `github.com/servo/servo`,
named after the corresponding branch:

[`repository.json`]: https://github.com/mozilla/treeherder/blob/master/treeherder/model/fixtures/repository.json

* [`servo-master`](https://treeherder.mozilla.org/#/jobs?repo=servo-master)
* [`servo-auto`](https://treeherder.mozilla.org/#/jobs?repo=servo-auto)
* [`servo-try`](https://treeherder.mozilla.org/#/jobs?repo=servo-try)
* [`servo-try-taskcluster`](https://treeherder.mozilla.org/#/jobs?repo=servo-try-taskcluster)

In the UI, the “Repos” button near the top right corner allows switching.

`servo-auto` is the relevant one when a pull request is approved with Homu for landing,
since the `auto` branch is where it pushes a merge commit for testing.


## Data flow / how it all works

(This section is mostly useful for future changes or troubleshooting.)

Changes to the Treeherder repository are deployed to Staging
soon (minutes) after they are merged on GitHub,
and to Production manually at some point later.
See [current deployment status](https://whatsdeployed.io/s-dqv).

Once a configuration change with a new repository/branch is deployed,
Treeherder will show it in its UI and start recording push and job data in its database.
This data comes from [Pulse], Mozilla’s shared message queue that coordinates separate services.
The [Pulse Inspector] shows messages as they come (though not in the past),
which can be useful for debugging.
Note that you need to add at least one “Binding”,
or the “Start Listening” button won’t do anything.

[Pulse]: https://wiki.mozilla.org/Auto-tools/Projects/Pulse
[Pulse Inspector]: https://community-tc.services.mozilla.com/pulse-messages


### Push data

When [taskcluster-github] is [enabled] on a repository,
it recieves [webhooks] from GitHub for various events
such as a push to a branch of the repository.

In addition to starting Taskcluster tasks based on `.taskcluster.yml` in the repository,
in [`api.js`] it creates [Pulse messages] corresponding to those events.
Treeherder consumes messages from the `exchange/taskcluster-github/v1/push` exchange
(among others) in [`push_loader.py`].
In Pulse Inspector, these messages for the Servo repository can be seen
by specifying the [`primary.servo.servo`] routing key pattern.

[taskcluster-github]: https://github.com/taskcluster/taskcluster/tree/master/services/github
[enabled]: https://github.com/apps/community-tc-integration/
[webhooks]: https://developer.github.com/webhooks/
[Pulse messages]: https://community-tc.services.mozilla.com/docs/reference/integrations/github/exchanges
[`api.js`]: https://github.com/taskcluster/taskcluster/blob/master/services/github/src/api.js
[`push_loader.py`]: https://github.com/mozilla/treeherder/blob/master/treeherder/etl/push_loader.py
[`primary.servo.servo`]: https://community-tc.services.mozilla.com/pulse-messages?bindings%5B0%5D%5Bexchange%5D=exchange%2Ftaskcluster-github%2Fv1%2Fpush&bindings%5B0%5D%5BroutingKeyPattern%5D=primary.servo.servo


### (Taskcluster) job data

The Taskcluster Queue generates a number of [Pulse messages about tasks].
Each value of the `routes` array in the task definition, with a `route.` prefix prepended,
is additional routing key for those messages.

Treeherder reads those messages
if they have an appropriate route ([see in Pulse inspector][inspector1]),
However, it will drop an incoming message
if the `extra.treeherder` object in the task definition doesn’t conform to the [schema].
Such schema validation errors are logged, but those logs are not easy to access.
Ask on IRC on `#taskcluster`.

Finally, Treeherder reads that latter kind of message in [`job_loader.py`].



[Pulse messages about tasks]: https://community-tc.services.mozilla.com/docs/reference/platform/taskcluster-queue/references/events
[taskcluster-treeherder]: https://github.com/taskcluster/taskcluster-treeherder/
[other messages]: https://community-tc.services.mozilla.com/docs/reference/integrations/taskcluster-treeherder#job-pulse-messages
[schema]: https://schemas.taskcluster.net/treeherder/v1/task-treeherder-config.json
[`job_loader.py`]: https://github.com/mozilla/treeherder/blob/master/treeherder/etl/job_loader.py
[inspector1]: https://tools.taskcluster.net/pulse-inspector?bindings%5B0%5D%5Bexchange%5D=exchange%2Ftaskcluster-queue%2Fv1%2Ftask-defined&bindings%5B0%5D%5BroutingKeyPattern%5D=route.tc-treeherder.%23
