# coding: utf8

import hashlib
import json
import os
import re
import sys
import taskcluster


def main():
    build_task = create_task_with_in_tree_dockerfile(
        "build task",
        "./build-task.sh",
        image="servo-x86_64-linux",

        artifacts=[
            ("executable.gz", "/repo/something-rust/something-rust.gz", "1 week"),
        ],

        # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/caches
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
        image="buildpack-deps:bionic-scm",
        dependencies=[build_task],
        env={"BUILD_TASK_ID": build_task},
    )


# https://github.com/servo/taskcluster-bootstrap-docker-images#image-builder
IMAGE_BUILDER_IMAGE = "servobrowser/taskcluster-bootstrap:image-builder@sha256:" \
    "0a7d012ce444d62ffb9e7f06f0c52fedc24b68c2060711b313263367f7272d9d"

# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/environment
DECISION_TASK_ID = os.environ["TASK_ID"]

# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/features#feature-taskclusterproxy
QUEUE = taskcluster.Queue(options={"baseUrl": "http://taskcluster/queue/v1/"})
INDEX = taskcluster.Index(options={"baseUrl": "http://taskcluster/index/v1/"})

IMAGE_ARTIFACT_FILENAME = "image.tar.lz4"

REPO = os.path.dirname(__file__)


def create_task_with_in_tree_dockerfile(name, command, image, **kwargs):
    image_build_task = build_image(image)
    kwargs.setdefault("dependencies", []).append(image_build_task)
    image = {
        "type": "task-image",
        "taskId": image_build_task,
        "path": "public/" + IMAGE_ARTIFACT_FILENAME,
    }
    return create_task(name, command, image, **kwargs)


def build_image(name):
    with open(os.path.join(REPO, name + ".dockerfile"), "rb") as f:
        dockerfile = f.read()
    digest = hashlib.sha256(dockerfile).hexdigest()
    route = "index.project.servo.servo-taskcluster-experiments.docker-image." + digest

    try:
        result = INDEX.findTask(route)
        return result["taskId"]
    except taskcluster.TaskclusterRestFailure as e:
        if e.status_code != 404:
            raise
        print("404 when looking up route", route, e, vars(e))

    image_build_task = create_task(
        "docker image build task for image: " + name,
        """
            echo "$DOCKERFILE" | docker build -t taskcluster-built -
            docker save taskcluster-built | lz4 > /%s
        """ % IMAGE_ARTIFACT_FILENAME,
        env={
            "DOCKERFILE": dockerfile,
        },
        artifacts=[
            (IMAGE_ARTIFACT_FILENAME, "/" + IMAGE_ARTIFACT_FILENAME, "1 week"),
        ],
        image=IMAGE_BUILDER_IMAGE,
        features={
            "dind": True,  # docker-in-docker
        },
        with_repo=False,
        routes=[route],
    )
    return image_build_task


def create_task(name, command, image, artifacts=None, dependencies=None, env=None, cache=None,
                scopes=None, routes=None, features=None, with_repo=True):
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
        "deadline": taskcluster.fromNowJSON("1 hour"),
        "metadata": {
            "name": "Taskcluster experiments for Servo: " + name,
            "description": "",
            "owner": os.environ["GITHUB_EVENT_OWNER"],
            "source": os.environ["GITHUB_EVENT_SOURCE"],
        },
        "scopes": scopes or [],
        "routes": routes or [],
        "payload": {
            "cache": cache or {},
            "maxRunTime": 3600,
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
    print("Scheduled %s: %s" % (name, task_id))
    return task_id


def deindent(string):
    return re.sub("\n +", "\n ", string)


if __name__ == "__main__":
    main()
