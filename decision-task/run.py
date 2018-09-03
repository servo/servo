# coding: utf8

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "vendored"))

import json
import taskcluster

event = json.loads(os.environ["GITHUB_EVENT"])
print("GitHub event:\n%s\n" % json.dumps(event, sort_keys=True, indent=4, separators=(',', ': ')))

# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/features#feature-taskclusterproxy
queue = taskcluster.Queue(options={"baseUrl": "http://taskcluster/queue/v1/"})

command_prefix = """
    git clone {event[repository][clone_url]} repo &&
    cd repo &&
    git checkout {event[after]} &&
    """.format(event=event)

def create_task(name, command, artifacts=None, dependencies=None, env=None, cache=None, scopes=None):
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
            "owner": event["pusher"]["name"] + "@users.noreply.github.com",
            "source": event["compare"],
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
                command_prefix + command
            ],
            "env": env or {},
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
