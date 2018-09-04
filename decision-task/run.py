# coding: utf8

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "vendored"))

import json

import taskcluster
# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/features#feature-taskclusterproxy
queue = taskcluster.Queue(options={"baseUrl": "http://taskcluster/queue/v1/"})


def create_task(name, command, artifacts=None, dependencies=None, env=None, cache=None, scopes=None):
    env = env or {}
    for k in ["GITHUB_EVENT_CLONE_URL", "GITHUB_EVENT_COMMIT_SHA"]:
        env.setdefault(k, os.environ[k])

    task_id = taskcluster.slugId()
    payload = {
        "taskGroupId": os.environ["DECISION_TASK_ID"],
        "dependencies": [os.environ["DECISION_TASK_ID"]] + (dependencies or []),
        "schedulerId": "taskcluster-github",
        "provisionerId": "aws-provisioner-v1",
        "workerType": "github-worker",
        "created": taskcluster.fromNowJSON(""),
        "deadline": taskcluster.fromNowJSON("1 hour"),
        "metadata": {
            "name": "Taskcluster experiments for Servo: " + name,
            "description": "",
            "owner": os.environ["GITHUB_EVENT_OWNER"],
            "source": os.environ["GITHUB_EVENT_SOURCE"],
        },
        "scopes": scopes or [],
        "payload": {
            "cache": cache or {},
            "maxRunTime": 600,
            "image": "buildpack-deps:bionic",
            "command": [
                "/bin/bash",
                "--login",
                "-c",
                """
                    git clone $GITHUB_EVENT_CLONE_URL repo &&
                    cd repo &&
                    git checkout $GITHUB_EVENT_COMMIT_SHA &&
                """ + command
            ],
            "env": env,
            "artifacts": {
                "public/" + artifact_name: {
                    "type": "file",
                    "path": path,
                    "expires": taskcluster.fromNowJSON("1 week"),
                }
                for artifact_name, path in artifacts or []
            },
        },
    }
    queue.createTask(task_id, payload)
    print("Scheduled %s: %s" % (name, task_id))
    return task_id

build_task = create_task(
    "build task",
    "./build-task.sh",
    artifacts=[("executable.gz", "/repo/something-rust/something-rust.gz")],

    # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/caches
    # For this to be allowed, I created role
    # "repo:github.com/servo/servo-taskcluster-experiments:branch:master"
    # with scope "assume:project:servo:grants/cargo-caches"
    # at https://tools.taskcluster.net/auth/roles/
    scopes=[
        "docker-worker:cache:cargo-registry-cache",
        "docker-worker:cache:cargo-git-cache",
    ],
    cache={
        "cargo-registry-cache": "/root/.cargo/registry",
        "cargo-git-cache": "/root/.cargo/git",
    },
)

create_task(
    "run task",
    "./run-task.sh",
    dependencies=[build_task],
    env={"BUILD_TASK_ID": build_task},
)
