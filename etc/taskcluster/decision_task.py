# coding: utf8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os.path
import decisionlib
import functools
from decisionlib import CONFIG, SHARED
from urllib.request import urlopen


def main(task_for):
    with decisionlib.make_repo_bundle():
        tasks(task_for)


def tasks(task_for):
    if CONFIG.git_ref.startswith("refs/heads/"):
        branch = CONFIG.git_ref[len("refs/heads/"):]
        CONFIG.treeherder_repository_name = "servo-" + (
            branch if not branch.startswith("try-") else "try"
        )

    # Work around a tc-github bug/limitation:
    # https://bugzilla.mozilla.org/show_bug.cgi?id=1548781#c4
    if task_for.startswith("github"):
        # https://github.com/taskcluster/taskcluster/blob/21f257dc8/services/github/config.yml#L14
        CONFIG.routes_for_all_subtasks.append("statuses")

    if task_for == "github-push":
        all_tests = []
        by_branch_name = {
            "auto": all_tests,
            "try": all_tests,
            "try-taskcluster": [
                # Add functions here as needed, in your push to that branch
            ],
            "master": [],

            # The "try-*" keys match those in `servo_try_choosers` in Homu’s config:
            # https://github.com/servo/saltfs/blob/master/homu/map.jinja

            "try-mac": [],
            "try-linux": [],
            "try-windows": [],
            "try-arm": [],
            "try-wpt": [],
            "try-wpt-2020": [],
            "try-wpt-mac": [],
            "test-wpt": [],
        }

    elif task_for == "github-pull-request":
        CONFIG.treeherder_repository_name = "servo-prs"
        CONFIG.index_read_only = True
        CONFIG.docker_image_build_worker_type = None

        # We want the merge commit that GitHub creates for the PR.
        # The event does contain a `pull_request.merge_commit_sha` key, but it is wrong:
        # https://github.com/servo/servo/pull/22597#issuecomment-451518810
        CONFIG.git_sha_is_current_head()

    elif task_for == "try-windows-ami":
        CONFIG.git_sha_is_current_head()
        CONFIG.windows_worker_type = os.environ["NEW_AMI_WORKER_TYPE"]

    # https://tools.taskcluster.net/hooks/project-servo/daily
    elif task_for == "daily":
        daily_tasks_setup()


ping_on_daily_task_failure = "SimonSapin, nox, emilio"
build_artifacts_expire_in = "1 week"
build_dependencies_artifacts_expire_in = "1 month"
log_artifacts_expire_in = "1 year"


def daily_tasks_setup():
    # Unlike when reacting to a GitHub push event,
    # the commit hash is not known until we clone the repository.
    CONFIG.git_sha_is_current_head()

    # On failure, notify a few people on IRC
    # https://docs.taskcluster.net/docs/reference/core/taskcluster-notify/docs/usage
    notify_route = "notify.irc-channel.#servo.on-failed"
    CONFIG.routes_for_all_subtasks.append(notify_route)
    CONFIG.scopes_for_all_subtasks.append("queue:route:" + notify_route)
    CONFIG.task_name_template = "Servo daily: %s. On failure, ping: " + ping_on_daily_task_failure


CONFIG.task_name_template = "Servo: %s"
CONFIG.docker_images_expire_in = build_dependencies_artifacts_expire_in
CONFIG.repacked_msi_files_expire_in = build_dependencies_artifacts_expire_in
CONFIG.index_prefix = "project.servo"
CONFIG.default_provisioner_id = "proj-servo"
CONFIG.docker_image_build_worker_type = "docker"

CONFIG.windows_worker_type = "win2016"

if __name__ == "__main__":  # pragma: no cover
    main(task_for=os.environ["TASK_FOR"])
