# coding: utf8

"""
Project-independent library for Taskcluster decision tasks
"""

import hashlib
import json
import os
import re
import sys
import taskcluster


# Used in task names
PROJECT_NAME = "Taskcluster experiments for Servo"

DOCKER_IMAGE_CACHE_EXPIRY = "1 week"

# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/environment
DECISION_TASK_ID = os.environ["TASK_ID"]

# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/features#feature-taskclusterproxy
QUEUE = taskcluster.Queue(options={"baseUrl": "http://taskcluster/queue/v1/"})
INDEX = taskcluster.Index(options={"baseUrl": "http://taskcluster/index/v1/"})

# https://github.com/servo/taskcluster-bootstrap-docker-images#image-builder
DOCKER_IMAGE_BUILDER_IMAGE = "servobrowser/taskcluster-bootstrap:image-builder@sha256:" \
    "0a7d012ce444d62ffb9e7f06f0c52fedc24b68c2060711b313263367f7272d9d"

DOCKER_IMAGE_ARTIFACT_FILENAME = "image.tar.lz4"

REPO = os.path.dirname(__file__)


class DecisionTask:
    def create_task_with_in_tree_dockerfile(self, *, image, **kwargs):
        image_build_task = self.build_image(image)
        kwargs.setdefault("dependencies", []).append(image_build_task)
        image = {
            "type": "task-image",
            "taskId": image_build_task,
            "path": "public/" + DOCKER_IMAGE_ARTIFACT_FILENAME,
        }
        return self.create_task(image=image, **kwargs)


    def build_image(self, image_name):
        with open(os.path.join(REPO, image_name + ".dockerfile"), "rb") as f:
            dockerfile = f.read()
        digest = hashlib.sha256(dockerfile).hexdigest()
        route = "project.servo.servo-taskcluster-experiments.docker-image." + digest

        try:
            result = INDEX.findTask(route)
            return result["taskId"]
        except taskcluster.TaskclusterRestFailure as e:
            if e.status_code != 404:
                raise

        image_build_task = self.create_task(
            task_name="docker image build task for image: " + image_name,
            command="""
                echo "$DOCKERFILE" | docker build -t taskcluster-built -
                docker save taskcluster-built | lz4 > /%s
            """ % DOCKER_IMAGE_ARTIFACT_FILENAME,
            env={
                "DOCKERFILE": dockerfile,
            },
            artifacts=[
                (
                    DOCKER_IMAGE_ARTIFACT_FILENAME,
                    "/" + DOCKER_IMAGE_ARTIFACT_FILENAME,
                    DOCKER_IMAGE_CACHE_EXPIRY
                ),
            ],
            max_run_time_minutes=20,
            image=DOCKER_IMAGE_BUILDER_IMAGE,
            features={
                "dind": True,  # docker-in-docker
            },
            with_repo=False,
            routes=[
                "index." + route,
            ],
            extra={
                "index": {
                    "expires": taskcluster.fromNowJSON(DOCKER_IMAGE_CACHE_EXPIRY),
                },
            },
        )
        return image_build_task


    def create_task(self, *, task_name, command, image, max_run_time_minutes,
                    artifacts=None, dependencies=None, env=None, cache=None, scopes=None,
                    routes=None, extra=None, features=None,
                    with_repo=True):
        env = env or {}

        if with_repo:
            for k in ["GITHUB_EVENT_CLONE_URL", "GITHUB_EVENT_COMMIT_SHA"]:
                env.setdefault(k, os.environ[k])

            command = """
                    git clone $GITHUB_EVENT_CLONE_URL repo
                    cd repo
                    git checkout $GITHUB_EVENT_COMMIT_SHA
                """ + command

        payload = {
            "taskGroupId": DECISION_TASK_ID,
            "dependencies": [DECISION_TASK_ID] + (dependencies or []),
            "schedulerId": "taskcluster-github",
            "provisionerId": "aws-provisioner-v1",
            "workerType": "servo-docker-worker",

            "created": taskcluster.fromNowJSON(""),
            "deadline": taskcluster.fromNowJSON("1 day"),
            "metadata": {
                "name": "%s: %s" % (PROJECT_NAME, task_name),
                "description": "",
                "owner": os.environ["GITHUB_EVENT_OWNER"],
                "source": os.environ["GITHUB_EVENT_SOURCE"],
            },
            "scopes": scopes or [],
            "routes": routes or [],
            "extra": extra or {},
            "payload": {
                "cache": cache or {},
                "maxRunTime": max_run_time_minutes * 60,
                "image": image,
                "command": [
                    "/bin/bash",
                    "--login",
                    "-x",
                    "-e",
                    "-c",
                    deindent(command)
                ],
                "env": env,
                "artifacts": {
                    "public/" + artifact_name: {
                        "type": "file",
                        "path": path,
                        "expires": taskcluster.fromNowJSON(expires),
                    }
                    for artifact_name, path, expires in artifacts or []
                },
                "features": features or {},
            },
        }

        task_id = taskcluster.slugId().decode("utf8")
        QUEUE.createTask(task_id, payload)
        print("Scheduled %s: %s" % (task_name, task_id))
        return task_id


def deindent(string):
    return re.sub("\n +", "\n ", string)
