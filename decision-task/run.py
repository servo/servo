# coding: utf8

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "vendored"))

import json
import taskcluster

event = json.loads(os.environ["GITHUB_EVENT"])
print("GitHub event:\n%s\n" % json.dumps(event, sort_keys=True, indent=4, separators=(',', ': ')))

task_id = taskcluster.slugId()
payload = {
    "taskGroupId": os.environ["DECISION_TASK_ID"],
    "dependencies": [os.environ["DECISION_TASK_ID"]],
    "schedulerId": "taskcluster-github",  # FIXME: can we avoid hard-coding this?
    "provisionerId": "aws-provisioner-v1",
    "workerType": "github-worker",
    "created": taskcluster.fromNowJSON(""),
    "deadline": taskcluster.fromNowJSON("1 hour"),
    "metadata": {
        "name": "Taskcluster experiments for Servo: Child task",
        "description": "",
        "owner": event["pusher"]["name"] + "@users.noreply.github.com",
        "source": event["compare"],
    },
    "payload": {
        "maxRunTime": 600,
        "image": "buildpack-deps:bionic",
        "command": [
            "/bin/bash",
            "--login",
            "-c",
            """
                git clone {event[repository][clone_url]} repo &&
                cd repo &&
                git checkout {event[after]} &&
                ./child-task.sh
            """.format(event=event),
        ],
        "artifacts": {
            "public/executable.gz": {
                "type": "file",
                "path": "/repo/something-rust/something-rust.gz",
                "expires": taskcluster.fromNowJSON("1 week"),
            },
        },
    },
}
# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/features#feature-taskclusterproxy
queue = taskcluster.Queue(options={"baseUrl": "http://taskcluster/queue/v1/"})
queue.createTask(task_id, payload)
print("new task scheduled: " + task_id)
