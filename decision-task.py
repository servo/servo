# coding: utf8

import os
import datetime
import taskcluster

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
        "owner": os.environ["DECISION_TASK_OWNER"],
        "source": os.environ["DECISION_TASK_SOURCE"],
    },
    "payload": {
        "maxRunTime": 600,
        "image": "buildpack-deps:bionic-scm",
        "command": [
            "/bin/bash",
            "--login",
            "-c",
            """
                git clone %(DECISION_TASK_CLONE_URL)s repo &&
                cd repo &&
                git checkout %(DECISION_TASK_COMMIT_SHA)s &&
                python2.7 child-task.py
            """ % os.environ,
        ],
    },
}
# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/features#feature-taskclusterproxy
queue = taskcluster.Queue(options={"baseUrl": "http://taskcluster/queue/v1/"})
result = queue.createTask(task_id, payload)
print("task %s created…? %r" % (task_id, result))
