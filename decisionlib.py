# coding: utf8

"""
Project-independent library for Taskcluster decision tasks
"""

import datetime
import hashlib
import json
import os
import re
import sys
import taskcluster


class DecisionTask:
    DOCKER_IMAGE_ARTIFACT_FILENAME = "image.tar.lz4"

    # https://github.com/servo/taskcluster-bootstrap-docker-images#image-builder
    DOCKER_IMAGE_BUILDER_IMAGE = "servobrowser/taskcluster-bootstrap:image-builder@sha256:" \
        "0a7d012ce444d62ffb9e7f06f0c52fedc24b68c2060711b313263367f7272d9d"

    def __init__(self, project_name, *, route_prefix,
                 worker_type="github-worker", docker_image_cache_expiry="1 year"):
        self.project_name = project_name
        self.route_prefix = route_prefix
        self.worker_type = worker_type
        self.docker_image_cache_expiry = docker_image_cache_expiry

        # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/features#feature-taskclusterproxy
        self.queue_service = taskcluster.Queue(options={"baseUrl": "http://taskcluster/queue/v1/"})
        self.index_service = taskcluster.Index(options={"baseUrl": "http://taskcluster/index/v1/"})

        self.now = datetime.datetime.utcnow()
        self.built_images = {}

    def from_now_json(self, offset):
        return taskcluster.stringDate(taskcluster.fromNow(offset, dateObj=self.now))

    def create_task_with_in_tree_dockerfile(self, *, dockerfile, **kwargs):
        image_build_task = self.find_or_build_image(dockerfile)
        kwargs.setdefault("dependencies", []).append(image_build_task)
        image = {
            "type": "task-image",
            "taskId": image_build_task,
            "path": "public/" + self.DOCKER_IMAGE_ARTIFACT_FILENAME,
        }
        return self.create_task(image=image, **kwargs)

    def find_or_build_image(self, dockerfile):
        image_build_task = self.built_images.get(dockerfile)
        if image_build_task is None:
            image_build_task = self._find_or_build_image(dockerfile)
            self.built_images[dockerfile] = image_build_task
        return image_build_task

    def _find_or_build_image(self, dockerfile):
        with open(dockerfile, "rb") as f:
            dockerfile_contents = f.read()
        digest = hashlib.sha256(dockerfile_contents).hexdigest()
        route = "%s.docker-image.%s" % (self.route_prefix, digest)

        try:
            result = self.index_service.findTask(route)
            return result["taskId"]
        except taskcluster.TaskclusterRestFailure as e:
            if e.status_code != 404:
                raise

        return self.create_task(
            task_name="docker image build task for image: " + self.image_name(dockerfile),
            command="""
                echo "$DOCKERFILE" | docker build -t taskcluster-built -
                docker save taskcluster-built | lz4 > /%s
            """ % self.DOCKER_IMAGE_ARTIFACT_FILENAME,
            env={
                "DOCKERFILE": dockerfile_contents,
            },
            artifacts=[
                ("/" + self.DOCKER_IMAGE_ARTIFACT_FILENAME, self.docker_image_cache_expiry),
            ],
            max_run_time_minutes=20,
            image=self.DOCKER_IMAGE_BUILDER_IMAGE,
            features={
                "dind": True,  # docker-in-docker
            },
            with_repo=False,
            routes=[
                "index." + route,
            ],
            extra={
                "index": {
                    "expires": self.from_now_json(self.docker_image_cache_expiry),
                },
            },
        )

    def image_name(self, dockerfile):
        basename = os.path.basename(dockerfile)
        suffix = ".dockerfile"
        if basename == "Dockerfile":
            return os.path.basename(os.path.dirname(os.path.abspath(dockerfile)))
        elif basename.endswith(suffix):
            return basename[:-len(suffix)]
        else:
            return basename

    def create_task(self, *, task_name, command, image, max_run_time_minutes,
                    artifacts=None, dependencies=None, env=None, cache=None, scopes=None,
                    routes=None, extra=None, features=None,
                    with_repo=True):
        # Set in .taskcluster.yml
        commit_sha = os.environ["GITHUB_EVENT_COMMIT_SHA"]
        clone_url = os.environ["GITHUB_EVENT_CLONE_URL"]
        source = os.environ["GITHUB_EVENT_SOURCE"]
        owner = os.environ["GITHUB_EVENT_OWNER"]

        env = env or {}

        if with_repo:
            env["GITHUB_EVENT_COMMIT_SHA"] = commit_sha
            env["GITHUB_EVENT_CLONE_URL"] = clone_url

            command = """
                    git clone --depth 1 $GITHUB_EVENT_CLONE_URL repo
                    cd repo
                    git checkout $GITHUB_EVENT_COMMIT_SHA
                """ + command

        # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/environment
        decision_task_id = os.environ["TASK_ID"]

        payload = {
            "taskGroupId": decision_task_id,
            "dependencies": [decision_task_id] + (dependencies or []),
            "schedulerId": "taskcluster-github",
            "provisionerId": "aws-provisioner-v1",
            "workerType": self.worker_type,

            "created": self.from_now_json(""),
            "deadline": self.from_now_json("1 day"),
            "metadata": {
                "name": "%s: %s" % (self.project_name, task_name),
                "description": "",
                "owner": owner,
                "source": source,
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
                    "public/" + os.path.basename(path): {
                        "type": "file",
                        "path": path,
                        "expires": self.from_now_json(expires),
                    }
                    for path, expires in artifacts or []
                },
                "features": features or {},
            },
        }

        task_id = taskcluster.slugId().decode("utf8")
        self.queue_service.createTask(task_id, payload)
        print("Scheduled %s: %s" % (task_name, task_id))
        return task_id


def deindent(string):
    return re.sub("\n +", "\n ", string)
