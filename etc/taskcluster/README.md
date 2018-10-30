# Testing Servo on Taskcluster

## Homu

When a pull request is reviewed and the appropriate command is given,
[Homu] creates a merge commit of `master` and the PR’s branch, and pushes it to the `auto` branch.
One or more CI system (through their own means) get notified of this push by GitHub,
start testing the merge commit, and use the [GitHub Status API] to report results.

Through a [Webhook], Homu gets notified of changes to these statues.
If all of the required statuses are reported successful,
Homu pushes its merge commit to the `master` branch
and goes on to testing the next pull request in its queue.

[Homu]: https://github.com/servo/servo/wiki/Homu
[GitHub Status API]: https://developer.github.com/v3/repos/statuses/
[Webhook]: https://developer.github.com/webhooks/


## Taskcluster − GitHub integration

Taskcluster is very flexible and not necessarily tied to GitHub,
but it does have an optional [GitHub integration service] that you can enable
on a repository [as a GitHub App].
When enabled, this service gets notified for every push, pull request, or GitHub release.
It then schedules some tasks based on reading [`.taskcluster.yml`] in the corresponding commit.

This file contains templates for creating one or more tasks,
but the logic it can support is fairly limited.
So a common pattern is to have it only run a single initial task called a *decision task*
that can have complex logic based on code and data in the repository
to build an arbitrary [task graph].

[GitHub integration service]: https://docs.taskcluster.net/docs/manual/using/github
[as a GitHub App]: https://github.com/apps/taskcluster
[`.taskcluster.yml`]: https://docs.taskcluster.net/docs/reference/integrations/taskcluster-github/docs/taskcluster-yml-v1
[task graph]: https://docs.taskcluster.net/docs/manual/using/task-graph


## Servo’s decision task

This repository’s [`.taskcluster.yml`][tc.yml] schedules a single task
that runs the Python 3 script [`etc/taskcluster/decision_task.py`](decision_task.py).
It is called a *decision task* as it is responsible for deciding what other tasks to schedule.

The Docker image that runs the decision task
is hosted on Docker Hub at [`servobrowser/taskcluster-bootstrap`][hub].
It is built by [Docker Hub automated builds] based on a `Dockerfile`
in the [`taskcluster-bootstrap-docker-images`] GitHub repository.
Hopefully, this image does not need to be modified often
as it only needs to clone the repository and run Python.

[tc.yml]: ../../../.taskcluster.yml
[hub]: https://hub.docker.com/r/servobrowser/taskcluster-bootstrap/
[Docker Hub automated builds]: https://docs.docker.com/docker-hub/builds/
[`taskcluster-bootstrap-docker-images`]: https://github.com/servo/taskcluster-bootstrap-docker-images/


## In-tree Docker images

[Similar to Firefox][firefox], Servo’s decision task supports running other tasks
in Docker images built on-demand, based on `Dockerfile`s in the main repository.
Modifying a `Dockerfile` and relying on those new changes
can be done in the same pull request or commit.

To avoid rebuilding images on every pull request,
they are cached based on a hash of the source `Dockerfile`.
For now, to support this hashing, we make `Dockerfile`s be self-contained (with one exception).
Images are built without a [context],
so instructions like [`COPY`] cannot be used because there is nothing to copy from.
The exception is that the decision task adds support for a non-standard include directive:
when a `Dockerfile` first line is `% include` followed by a filename,
that line is replaced with the content of that file.

For example,
[`etc/taskcluster/docker/build.dockerfile`](docker/build.dockerfile) starts like so:

```Dockerfile
% include base.dockerfile

RUN \
    apt-get install -qy --no-install-recommends \
# […]
```

[firefox]: https://firefox-source-docs.mozilla.org/taskcluster/taskcluster/docker-images.html
[context]: https://docs.docker.com/engine/reference/commandline/build/#extended-description
[`COPY`]: https://docs.docker.com/engine/reference/builder/#copy


## Build artifacts

[web-platform-tests] (WPT) is large enough that running all of a it takes a long time.
So it supports *chunking*,
such as multiple chunks of the test suite can be run in parallel on different machines.
As of this writing,
Servo’s current Buildbot setup for this has each machine start by compiling its own copy of Servo.
On Taskcluster with a decision task,
we can have a single build task save its resulting binary executable as an [artifact],
together with multiple testing tasks that each depend on the build task
(wait until it successfully finishes before they can start)
and start by downloading the artifact that was saved earlier.

The logic for all this is in [`decision_task.py`](decision_task.py)
and can be modified in any pull request.

[web-platform-tests]: https://github.com/web-platform-tests/wpt
[artifact]: https://docs.taskcluster.net/docs/manual/using/artifacts


## Log artifacts

Taskcluster automatically save the `stdio` output of a task as an artifact,
and as special support for seeing and streaming that output while the task is still running.

Servo’s decision task additionally looks for `*.log` arguments to its tasks’s commands,
assumes they instruct a program to create a log file with that name,
and saves those log files as individual artifacts.

For example, WPT tasks have a `filtered-wpt-errorsummary.log` artifact
that is typically the most relevant output when such a task fails.


## Scopes and roles

[Scopes] are what Taskcluster calls permissions.
They control access to everything.

Anyone logged in in the [web UI] has (access to) a set of scopes,
which is visible on the [credentials] page
(reachable from clicking on one’s own name on the top-right of any page).

A running task has a set of scopes allowing it access to various functionality and APIs.
It can grant those scopes (and at most only thoses) to sub-tasks that it schedules
(if it has the scope allowing it to schedule new tasks in the first place).

[Roles] represent each a set of scopes.
They can be granted to… things,
and then configured separately to modify what scopes they [expand] to.

For example, when Taskcluster-GitHub schedules tasks based on the `.taskcluster.yml` file
in a push to the `auto` branch of this repository,
those tasks are granted the scope `assume:repo:github.com/servo/servo:branch:auto`.
Scopes that start with `assume:` are special,
they expand to the scopes defined in the matching roles.
In this case, the [`repo:github.com/servo/servo:branch:*`][branches] role matches.

Servo admins have scope `auth:update-role:repo:github.com/servo/*` which allows them
to edit that role in the web UI and grant more scopes to these tasks
(if that person has the new scope themselves).

The [`project:servo:decision-task/base`][base]
and [`project:servo:decision-task/trusted`][trusted] roles
centralize the set of scopes granted to the decision task.
This avoids maintaining them seprately in the `repo:…` roles,
in the `hook-id:…` role,
and in the `taskcluster.yml` file.
Only the `base` role is granted to tasks executed when a pull request is opened.
These tasks are less trusted because they run before the code has been reviewed,
and anyone can open a PR.

[Scopes]: https://docs.taskcluster.net/docs/manual/design/apis/hawk/scopes
[web UI]: https://tools.taskcluster.net/
[credentials]: https://tools.taskcluster.net/credentials
[Roles]: https://docs.taskcluster.net/docs/manual/design/apis/hawk/roles
[expand]: https://docs.taskcluster.net/docs/reference/platform/taskcluster-auth/docs/roles
[branches]: https://tools.taskcluster.net/auth/roles/repo%3Agithub.com%2Fservo%2Fservo%3Abranch%3A*
[base]: https://tools.taskcluster.net/auth/roles/project%3Aservo%3Adecision-task%2Fbase
[trusted]: https://tools.taskcluster.net/auth/roles/project%3Aservo%3Adecision-task%2Ftrusted


## Daily tasks

The [`project-servo/daily`] hook in Taskcluster’s [Hooks service]
is used to run some tasks automatically ever 24 hours.
In this case as well we use a decision task.
The `decision_task.py` script can differenciate this from a GitHub push
based on the `$TASK_FOR` environment variable.
Daily tasks can also be triggered manually.

Scopes available to the daily decision task need to be both requested in the hook definition
and granted through the [`hook-id:project-servo/daily`] role.

Because they do not have something similar to GitHub statuses that link to them,
daily tasks are indexed under the [`project.servo.servo.daily`] namespace.

[`project.servo.servo.daily`]: https://tools.taskcluster.net/index/project.servo.servo.daily

[`project-servo/daily`]: https://tools.taskcluster.net/hooks/project-servo/daily
[Hooks service]: https://docs.taskcluster.net/docs/manual/using/scheduled-tasks
[`hook-id:project-servo/daily`]: https://tools.taskcluster.net/auth/roles/hook-id%3Aproject-servo%2Fdaily


## AWS EC2 workers

As of this writing, Servo on Taskcluster can only use the `servo-docker-worker` worker type.
Tasks scheduled with this worker type run in a Linux environment,
in a Docker container, on an AWS EC2 virtual machine.

These machines are short-lived “spot instances”.
They are started automatically as needed by the [AWS provisioner]
when the existing capacity is insufficient to execute queued tasks.
They terminate themselves after being idle without work for a while,
or unconditionally after a few days.
Because these workers are short-lived,
we don’t need to worry about evicting old entries from Cargo’s or rustup’s download cache,
for example.

Servo admins can view and edit the [worker type definition] which configures the provisioner,
in particular with the types of EC2 instances to be used.

[AWS provisioner]: https://docs.taskcluster.net/docs/reference/integrations/aws-provisioner/references/api
[worker type definition]: https://tools.taskcluster.net/aws-provisioner/servo-docker-worker/view


## Self-service, Bugzilla, and IRC

Taskcluster is designed to be “self-service” as much as possible,
with features like in-tree `.taskcluster.yml`
or the web UI for modifying the worker type definitions.
However some changes like adding a new worker type still require Taskcluster admin access.
For those, file requests on Bugzilla under [Taskcluster :: Service Request][req].

For asking for help less formally, try the `#servo` or `#taskcluster` channels on Mozilla IRC.

[req]: https://bugzilla.mozilla.org/enter_bug.cgi?product=Taskcluster&component=Service%20Request


## Configuration recap

We try to keep as much as possible of our Taskcluster configuration in this repository.
To modify those, submit a pull request.

* The [`.taskcluster.yml`][tc.yml] file,
  for starting decision tasks in reaction to GitHub events
* The [`etc/ci/decision_task.py`](decision_task.py) file,
  defining what other tasks to schedule

However some configuration needs to be handled separately.
Modifying those requires Servo-project-level administrative access.

* The [`aws-provisioner/servo-docker-worker`][worker type definition] worker type definition,
  for EC2 instances configuration
* The [`project-servo/daily`] hook definition,
  for starting daily decision tasks
* The [`hook-id:project-servo/daily`] role,
  for scopes granted to those tasks
* The [`repo:github.com/servo/servo:branch:*`][branches] role,
  for scopes granted to tasks responding to a GitHub push to the repository (includin by Homu)
