# coding: utf8

# Copyright 2018 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

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
            task_name="docker image build task for image: " + image_name(dockerfile),
            script="""
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
            docker_image=self.DOCKER_IMAGE_BUILDER_IMAGE,
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

    def create_task(self, *, task_name, script, max_run_time_minutes,
                    docker_image=None, dockerfile=None,  # One of these is required
                    artifacts=None, dependencies=None, env=None, cache=None, scopes=None,
                    routes=None, extra=None, features=None,
                    with_repo=True):
        if docker_image and dockerfile:
            raise TypeError("cannot use both `docker_image` or `dockerfile`")
        if not docker_image and not dockerfile:
            raise TypeError("need one of `docker_image` or `dockerfile`")

        # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/environment
        decision_task_id = os.environ["TASK_ID"]

        dependencies = [decision_task_id] + (dependencies or [])

        if dockerfile:
            image_build_task = self.find_or_build_image(dockerfile)
            dependencies.append(image_build_task)
            docker_image = {
                "type": "task-image",
                "taskId": image_build_task,
                "path": "public/" + self.DOCKER_IMAGE_ARTIFACT_FILENAME,
            }

        # Set in .taskcluster.yml
        task_owner = os.environ["TASK_OWNER"]
        task_source = os.environ["TASK_SOURCE"]

        env = env or {}

        if with_repo:
            # Set in .taskcluster.yml
            for k in ["GIT_URL", "GIT_REF", "GIT_SHA"]:
                env[k] = os.environ[k]

            script = """
                    git init repo
                    cd repo
                    git fetch --depth 1 "$GIT_URL" "$GIT_REF"
                    git reset --hard "$GIT_SHA"
                """ + script

        payload = {
            "taskGroupId": decision_task_id,
            "dependencies": dependencies or [],
            "schedulerId": "taskcluster-github",
            "provisionerId": "aws-provisioner-v1",
            "workerType": self.worker_type,

            "created": self.from_now_json(""),
            "deadline": self.from_now_json("1 day"),
            "metadata": {
                "name": "%s: %s" % (self.project_name, task_name),
                "description": "",
                "owner": task_owner,
                "source": task_source,
            },
            "scopes": scopes or [],
            "routes": routes or [],
            "extra": extra or {},
            "payload": {
                "cache": cache or {},
                "maxRunTime": max_run_time_minutes * 60,
                "image": docker_image,
                "command": [
                    "/bin/bash",
                    "--login",
                    "-x",
                    "-e",
                    "-c",
                    deindent(script)
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


def image_name(dockerfile):
    basename = os.path.basename(dockerfile)
    suffix = ".dockerfile"
    if basename == "Dockerfile":
        return os.path.basename(os.path.dirname(os.path.abspath(dockerfile)))
    elif basename.endswith(suffix):
        return basename[:-len(suffix)]
    else:
        return basename


def deindent(string):
    return re.sub("\n +", " \n ", string)
