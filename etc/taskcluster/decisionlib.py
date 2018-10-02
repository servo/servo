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

import base64
import datetime
import hashlib
import json
import os
import re
import sys
import taskcluster


class DecisionTask:
    """
    Holds some project-specific configuration and provides higher-level functionality
    on top of the `taskcluster` package a.k.a. `taskcluster-client.py`.
    """

    DOCKER_IMAGE_ARTIFACT_FILENAME = "image.tar.lz4"

    # https://github.com/servo/taskcluster-bootstrap-docker-images#image-builder
    DOCKER_IMAGE_BUILDER_IMAGE = "servobrowser/taskcluster-bootstrap:image-builder@sha256:" \
        "0a7d012ce444d62ffb9e7f06f0c52fedc24b68c2060711b313263367f7272d9d"

    def __init__(self, *, index_prefix="garbage.servo-decisionlib", task_name_template="%s",
                 default_worker_type="github-worker", docker_image_cache_expiry="1 month",
                 routes_for_all_subtasks=None, scopes_for_all_subtasks=None):
        self.task_name_template = task_name_template
        self.index_prefix = index_prefix
        self.default_worker_type = default_worker_type
        self.docker_image_cache_expiry = docker_image_cache_expiry
        self.routes_for_all_subtasks = routes_for_all_subtasks or []
        self.scopes_for_all_subtasks = scopes_for_all_subtasks or []

        # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/features#feature-taskclusterproxy
        self.queue_service = taskcluster.Queue(options={"baseUrl": "http://taskcluster/queue/v1/"})
        self.index_service = taskcluster.Index(options={"baseUrl": "http://taskcluster/index/v1/"})

        self.now = datetime.datetime.utcnow()
        self.found_or_created_indices = {}

    def from_now_json(self, offset):
        """
        Same as `taskcluster.fromNowJSON`, but uses the creation time of `self` for “now”.
        """
        return taskcluster.stringDate(taskcluster.fromNow(offset, dateObj=self.now))

    def find_or_create_task(self, *, index_bucket, index_key, index_expiry, artifacts, **kwargs):
        """
        Find a task indexed in the given bucket (kind, category, …) and cache key,
        on schedule a new one if there isn’t one yet.

        Returns the task ID.
        """
        index_path = "%s.%s.%s" % (self.index_prefix, index_bucket, index_key)

        task_id = self.found_or_created_indices.get(index_path)
        if task_id is not None:
            return task_id

        try:
            result = self.index_service.findTask(index_path)
            task_id = result["taskId"]
        except taskcluster.TaskclusterRestFailure as e:
            if e.status_code == 404:
                task_id = self.create_task(
                    routes=[
                        "index." + index_path,
                    ],
                    extra={
                        "index": {
                            "expires": self.from_now_json(self.docker_image_cache_expiry),
                        },
                    },
                    artifacts=[
                        (artifact, index_expiry)
                        for artifact in artifacts
                    ],
                    **kwargs
                )
            else:
                raise

        self.found_or_created_indices[index_path] = task_id
        return task_id

    def find_or_build_docker_image(self, dockerfile):
        """
        Find a task that built a Docker image based on this `dockerfile`,
        or schedule a new image-building task if needed.

        Returns the task ID.
        """
        dockerfile_contents = expand_dockerfile(dockerfile)
        digest = hashlib.sha256(dockerfile_contents).hexdigest()

        return self.find_or_create_task(
            index_bucket="docker-image",
            index_key=digest,
            index_expiry=self.docker_image_cache_expiry,

            task_name="Docker image: " + image_name(dockerfile),
            script="""
                echo "$DOCKERFILE" | docker build -t taskcluster-built -
                docker save taskcluster-built | lz4 > /%s
            """ % self.DOCKER_IMAGE_ARTIFACT_FILENAME,
            env={
                "DOCKERFILE": dockerfile_contents,
            },
            artifacts=[
                "/" + self.DOCKER_IMAGE_ARTIFACT_FILENAME,
            ],
            max_run_time_minutes=20,
            docker_image=self.DOCKER_IMAGE_BUILDER_IMAGE,
            features={
                "dind": True,  # docker-in-docker
            },
            with_repo=False,
        )

    def create_task(self, *, task_name, script, max_run_time_minutes,
                    docker_image=None, dockerfile=None,  # One of these is required
                    artifacts=None, dependencies=None, env=None, cache=None, scopes=None,
                    routes=None, extra=None, features=None, mounts=None, homedir_path=None,
                    worker_type=None, with_repo=True, sparse_checkout=None):
        """
        Schedule a new task. Returns the new task ID.

        One of `docker_image` or `dockerfile` (but not both) must be given.
        If `dockerfile` is given, the corresponding Docker image is built as needed and cached.

        `with_repo` indicates whether `script` should start in a clone of the git repository.
        """
        # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/environment
        decision_task_id = os.environ["TASK_ID"]

        dependencies = [decision_task_id] + (dependencies or [])

        # Set in .taskcluster.yml
        task_owner = os.environ["TASK_OWNER"]
        task_source = os.environ["TASK_SOURCE"]

        env = env or {}

        if with_repo:
            # Set in .taskcluster.yml
            for k in ["GIT_URL", "GIT_REF", "GIT_SHA"]:
                env[k] = os.environ[k]

        worker_type = worker_type or self.default_worker_type
        if "docker" in worker_type:
            if docker_image and dockerfile:
                raise TypeError("cannot use both `docker_image` or `dockerfile`")
            if not docker_image and not dockerfile:
                raise TypeError("need one of `docker_image` or `dockerfile`")

            if dockerfile:
                image_build_task = self.find_or_build_docker_image(dockerfile)
                dependencies.append(image_build_task)
                docker_image = {
                    "type": "task-image",
                    "taskId": image_build_task,
                    "path": "public/" + self.DOCKER_IMAGE_ARTIFACT_FILENAME,
                }

            if with_repo:
                git = """
                    git init repo
                    cd repo
                    git fetch --depth 1 "$GIT_URL" "$GIT_REF"
                    git reset --hard "$GIT_SHA"
                """
                script = git + script
            command = ["/bin/bash", "--login", "-x", "-e", "-c", deindent(script)]
        else:
            command = [
                "set PATH=%CD%\\{};%PATH%".format(p)
                for p in reversed(homedir_path or [])
            ]
            if with_repo:
                if with_repo:
                    git = """
                        git init repo
                        cd repo
                    """
                    if sparse_checkout:
                        git += """
                            git config core.sparsecheckout true
                            echo %SPARSE_CHECKOUT_BASE64% > .git\\info\\sparse.b64
                            certutil -decode .git\\info\\sparse.b64 .git\\info\\sparse-checkout
                            type .git\\info\\sparse-checkout
                        """
                        env["SPARSE_CHECKOUT_BASE64"] = base64.b64encode(
                            "\n".join(sparse_checkout).encode("utf-8"))
                command.append(deindent(git + """
                    git fetch --depth 1 %GIT_URL% %GIT_REF%
                    git reset --hard %GIT_SHA%
                """))
            command.append(deindent(script))

        worker_payload = {
            "maxRunTime": max_run_time_minutes * 60,
            "command": command,
            "env": env,
        }
        if docker_image:
            worker_payload["image"] = docker_image
        if cache:
            worker_payload["cache"] = cache
        if features:
            worker_payload["features"] = features
        if mounts:
            worker_payload["mounts"] = mounts
        if artifacts:
            if "docker" in worker_type:
                worker_payload["artifacts"] = {
                    "public/" + os.path.basename(path): {
                        "type": "file",
                        "path": path,
                        "expires": self.from_now_json(expires),
                    }
                    for path, expires in artifacts
                }
            else:
                worker_payload["artifacts"] = [
                    {
                        "type": "file",
                        "name": "public/" + os.path.basename(path),
                        "path": path,
                        "expires": self.from_now_json(expires),
                    }
                    for path, expires in artifacts
                ]
        payload = {
            "taskGroupId": decision_task_id,
            "dependencies": dependencies or [],
            "schedulerId": "taskcluster-github",
            "provisionerId": "aws-provisioner-v1",
            "workerType": worker_type,

            "created": self.from_now_json(""),
            "deadline": self.from_now_json("1 day"),
            "metadata": {
                "name": self.task_name_template % task_name,
                "description": "",
                "owner": task_owner,
                "source": task_source,
            },
            "scopes": (scopes or []) + self.scopes_for_all_subtasks,
            "routes": (routes or []) + self.routes_for_all_subtasks,
            "extra": extra or {},
            "payload": worker_payload,
        }

        task_id = taskcluster.slugId().decode("utf8")
        self.queue_service.createTask(task_id, payload)
        print("Scheduled %s" % task_name)
        return task_id


def image_name(dockerfile):
    """
    Guess a short name based on the path `dockerfile`.
    """
    basename = os.path.basename(dockerfile)
    suffix = ".dockerfile"
    if basename == "Dockerfile":
        return os.path.basename(os.path.dirname(os.path.abspath(dockerfile)))
    elif basename.endswith(suffix):
        return basename[:-len(suffix)]
    else:
        return basename


def expand_dockerfile(dockerfile):
    """
    Read the file at path `dockerfile`,
    and transitively expand the non-standard `% include` header if it is present.
    """
    with open(dockerfile, "rb") as f:
        dockerfile_contents = f.read()

    include_marker = b"% include"
    if not dockerfile_contents.startswith(include_marker):
        return dockerfile_contents

    include_line, _, rest = dockerfile_contents.partition(b"\n")
    included = include_line[len(include_marker):].strip().decode("utf8")
    path = os.path.join(os.path.dirname(dockerfile), included)
    return b"\n".join([expand_dockerfile(path), rest])


def deindent(string):
    return re.sub("\n +", " \n ", string).strip()
