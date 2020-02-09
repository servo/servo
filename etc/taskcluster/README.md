# Testing Servo on Taskcluster

## In-tree and out-of-tree configuration

Where possible, we prefer keeping Taskcluster-related configuration and code in this directory,
set up CI so that testing of a given git branch uses the version in that branch.
That way, anyone can make changes (such installing a new system dependency
[in a `Dockerfile`](#docker-images)) in the same PR that relies on those changes.

For some things however that is not practical,
or some deployment step that mutates global state is required.
That configuration is split between the [mozilla/community-tc-config] and
[servo/taskcluster-config] repositories,
managed by the Taskcluster team and the Servo team repsectively.

[mozilla/community-tc-config]: https://github.com/mozilla/community-tc-config/blob/master/config/projects.yml
[servo/taskcluster-config]: https://github.com/servo/taskcluster-config/tree/master/config


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

[GitHub integration service]: https://community-tc.services.mozilla.com/docs/manual/using/github
[as a GitHub App]: https://github.com/apps/community-tc-integration/
[`.taskcluster.yml`]: https://community-tc.services.mozilla.com/docs/reference/integrations/taskcluster-github/docs/taskcluster-yml-v1
[task graph]: https://community-tc.services.mozilla.com/docs/manual/using/task-graph


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


## Docker images

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
[artifact]: https://community-tc.services.mozilla.com/docs/manual/using/artifacts


## Log artifacts

Taskcluster automatically save the `stdio` output of a task as an artifact,
and has special support for showing and streaming that output while the task is still running.

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

The [`project:servo:decision-task/base`][base]
and [`project:servo:decision-task/trusted`][trusted] roles
centralize the set of scopes granted to the decision task.
This avoids maintaining them seprately in the `repo:…` roles,
in the `hook-id:…` role,
and in the `taskcluster.yml` file.
Only the `base` role is granted to tasks executed when a pull request is opened.
These tasks are less trusted because they run before the code has been reviewed,
and anyone can open a PR.

Members of the [@servo/taskcluster-admins] GitHub team are granted
the scope `assume:project-admin:servo`, which is necessary to deploy changes
to those roles from the [servo/taskcluster-config] repository.

[Scopes]: https://community-tc.services.mozilla.com/docs/manual/design/apis/hawk/scopes
[web UI]: https://community-tc.services.mozilla.com/
[credentials]: https://community-tc.services.mozilla.com/profile
[Roles]: https://community-tc.services.mozilla.com/docs/manual/design/apis/hawk/roles
[expand]: https://community-tc.services.mozilla.com/docs/reference/platform/taskcluster-auth/docs/roles
[branches]: https://community-tc.services.mozilla.com/auth/roles/repo%3Agithub.com%2Fservo%2Fservo%3Abranch%3A*
[base]: https://community-tc.services.mozilla.com/auth/roles/project%3Aservo%3Adecision-task%2Fbase
[trusted]: https://community-tc.services.mozilla.com/auth/roles/project%3Aservo%3Adecision-task%2Ftrusted
[@servo/taskcluster-admins]: https://github.com/orgs/servo/teams/taskcluster-admins


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
daily tasks are indexed under the [`project.servo.daily`] namespace.

[`project.servo.daily`]: https://tools.taskcluster.net/index/project.servo.daily
[`project-servo/daily`]: https://github.com/servo/taskcluster-config/blob/master/config/hooks.yml
[Hooks service]: https://community-tc.services.mozilla.com/docs/manual/using/scheduled-tasks
[`hook-id:project-servo/daily`]: https://github.com/servo/taskcluster-config/blob/master/config/roles.yml


## Servo’s worker pools

Each task is assigned to a “worker pool”.
Servo has several, for the different environments a task can run in:

* `docker` and `docker-untrusted` provide a Linux environment with full `root` privileges,
  in a Docker container running a [Docker image](#docker-images) of the task’s choice,
  in a short-lived virtual machine,
  on Google Cloud Platform.

  Instances are started automatically as needed
  when the existing capacity is insufficient to execute queued tasks.
  They terminate themselves after being idle without work for a while,
  or unconditionally after a few days.
  Because these workers are short-lived,
  we don’t need to worry about evicting old entries from Cargo’s or rustup’s download cache,
  for example.

  [The Taskcluster team manages][mozilla/community-tc-config]
  the configuration and VM image for these two pools.
  The latter has fewer scopes. It runs automated testing of pull requests
  as soon as they’re opened or updated, before any review.

* `win2016` runs Windows Server 2016 on AWS EC2.
  Like with Docker tasks, workers are short-lived and started automatically.
  The configuration and VM image for this pool
  is [managed by the Servo team][servo/taskcluster-config].

  Tasks run as an unprivileged user.
  Because creating an new the VM image is slow and deploying it mutates global state,
  when a tool does not require system-wide installation
  we prefer having each task obtain it as needed by extracting an archive in a directory.
  See calls of `with_directory_mount` and `with_repacked_msi` in
  [`decision_task.py`](decision_task.py) and [`decisionlib.py`](decisionlib.py).

* `macos` runs, you guessed it, macOS.
  Tasks run on dedicated hardware provided long-term by Macstadium.
  The system-wide configuration of those machines
  is [managed by the Servo team][servo/taskcluster-config] through SaltStack.
  There is a task-owned (but preserved across tasks) install of Homebrew,
  with `Brewfile`s [in this repository](macos/).

  This [Workers] page lists the current state of each macOS worker.
  (A similar page exists for other each worker pools, but as of this writing it has
  [usability issues](https://github.com/taskcluster/taskcluster/issues/1972)
  with short-lived workers.)

[Workers]: https://community-tc.services.mozilla.com/provisioners/proj-servo/worker-types/macos


## Taskcluster − Treeherder integration

See [`treeherder.md`](treeherder.md).


## Self-service, IRC, and Bugzilla

Taskcluster is designed to be “self-service” as much as possible.
Between this repository [servo/taskcluster-config] and [mozilla/community-tc-config],
anyone should be able to submit PRs for any part of the configuration.

Feel free to ask for help on the `#servo` or `#taskcluster` channels on Mozilla IRC.

For issue reports or feature requests on various bits of Taskcluster *software*,
file bugs [in Mozilla’s Bugzilla, under `Taskcluster`][bug].

[bug]: https://bugzilla.mozilla.org/enter_bug.cgi?product=Taskcluster
