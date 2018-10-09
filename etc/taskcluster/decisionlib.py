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
import subprocess
import sys
import taskcluster


# Public API
__all__ = [
    "CONFIG", "SHARED", "Task", "DockerWorkerTask",
    "GenericWorkerTask", "WindowsGenericWorkerTask",
]


class Config:
    """
    Global configuration, for users of the library to modify.
    """
    def __init__(self):
        self.task_name_template = "%s"
        self.index_prefix = "garbage.servo-decisionlib"
        self.scopes_for_all_subtasks = []
        self.routes_for_all_subtasks = []
        self.docker_images_expire_in = "1 month"
        self.repacked_msi_files_expire_in = "1 month"

        # Set by docker-worker:
        # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/environment
        self.decision_task_id = os.environ.get("TASK_ID")

        # Set in the decision task’s payload, such as defined in .taskcluster.yml
        self.task_owner = os.environ.get("TASK_OWNER")
        self.task_source = os.environ.get("TASK_SOURCE")
        self.git_url = os.environ.get("GIT_URL")
        self.git_ref = os.environ.get("GIT_REF")
        self.git_sha = os.environ.get("GIT_SHA")

    def git_sha_is_current_head(self):
        output = subprocess.check_output(["git", "rev-parse", "HEAD"])
        self.git_sha = output.decode("utf8").strip()



class Shared:
    """
    Global shared state.
    """
    def __init__(self):
        self.now = datetime.datetime.utcnow()
        self.found_or_created_indexed_tasks = {}

        # taskclusterProxy URLs:
        # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/features
        self.queue_service = taskcluster.Queue(options={"baseUrl": "http://taskcluster/queue/v1/"})
        self.index_service = taskcluster.Index(options={"baseUrl": "http://taskcluster/index/v1/"})

    def from_now_json(self, offset):
        """
        Same as `taskcluster.fromNowJSON`, but uses the creation time of `self` for “now”.
        """
        return taskcluster.stringDate(taskcluster.fromNow(offset, dateObj=self.now))


CONFIG = Config()
SHARED = Shared()


def chaining(op, attr):
    def method(self, *args, **kwargs):
        op(self, attr, *args, **kwargs)
        return self
    return method


def append_to_attr(self, attr, *args): getattr(self, attr).extend(args)
def prepend_to_attr(self, attr, *args): getattr(self, attr)[0:0] = list(args)
def update_attr(self, attr, **kwargs): getattr(self, attr).update(kwargs)


class Task:
    def __init__(self, name):
        self.name = name
        self.description = ""
        self.scheduler_id = "taskcluster-github"
        self.provisioner_id = "aws-provisioner-v1"
        self.worker_type = "github-worker"
        self.deadline_in = "1 day"
        self.expires_in = "1 year"
        self.index_and_artifacts_expire_in = self.expires_in
        self.dependencies = []
        self.scopes = []
        self.routes = []
        self.extra = {}

    with_description = chaining(setattr, "description")
    with_scheduler_id = chaining(setattr, "scheduler_id")
    with_provisioner_id = chaining(setattr, "provisioner_id")
    with_worker_type = chaining(setattr, "worker_type")
    with_deadline_in = chaining(setattr, "deadline_in")
    with_expires_in = chaining(setattr, "expires_in")
    with_index_and_artifacts_expire_in = chaining(setattr, "index_and_artifacts_expire_in")

    with_dependencies = chaining(append_to_attr, "dependencies")
    with_scopes = chaining(append_to_attr, "scopes")
    with_routes = chaining(append_to_attr, "routes")

    with_extra = chaining(update_attr, "extra")

    def build_worker_payload(self):  # pragma: no cover
        raise NotImplementedError

    def create(self):
        worker_payload = self.build_worker_payload()

        assert CONFIG.decision_task_id
        assert CONFIG.task_owner
        assert CONFIG.task_source
        queue_payload = {
            "taskGroupId": CONFIG.decision_task_id,
            "dependencies": [CONFIG.decision_task_id] + self.dependencies,
            "schedulerId": self.scheduler_id,
            "provisionerId": self.provisioner_id,
            "workerType": self.worker_type,

            "created": SHARED.from_now_json(""),
            "deadline": SHARED.from_now_json(self.deadline_in),
            "expires": SHARED.from_now_json(self.expires_in),
            "metadata": {
                "name": CONFIG.task_name_template % self.name,
                "description": self.description,
                "owner": CONFIG.task_owner,
                "source": CONFIG.task_source,
            },

            "payload": worker_payload,
        }
        scopes = self.scopes + CONFIG.scopes_for_all_subtasks
        routes = self.routes + CONFIG.routes_for_all_subtasks
        if any(r.startswith("index.") for r in routes):
            self.extra.setdefault("index", {})["expires"] = \
                SHARED.from_now_json(self.index_and_artifacts_expire_in)
        dict_update_if_truthy(
            queue_payload,
            scopes=scopes,
            routes=routes,
            extra=self.extra,
        )

        task_id = taskcluster.slugId().decode("utf8")
        SHARED.queue_service.createTask(task_id, queue_payload)
        print("Scheduled %s" % self.name)
        return task_id

    def find_or_create(self, index_path=None):
        if not index_path:
            worker_type = self.worker_type
            index_by = json.dumps([worker_type, self.build_worker_payload()]).encode("utf-8")
            index_path = "by-task-definition." + hashlib.sha256(index_by).hexdigest()
        index_path = "%s.%s" % (CONFIG.index_prefix, index_path)

        task_id = SHARED.found_or_created_indexed_tasks.get(index_path)
        if task_id is not None:
            return task_id

        try:
            task_id = SHARED.index_service.findTask(index_path)["taskId"]
        except taskcluster.TaskclusterRestFailure as e:
            if e.status_code != 404:  # pragma: no cover
                raise
            self.routes.append("index." + index_path)
            task_id = self.create()

        SHARED.found_or_created_indexed_tasks[index_path] = task_id
        return task_id


class GenericWorkerTask(Task):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.max_run_time_minutes = 30
        self.env = {}
        self.mounts = []
        self.artifacts = []

    with_max_run_time_minutes = chaining(setattr, "max_run_time_minutes")
    with_mounts = chaining(append_to_attr, "mounts")
    with_env = chaining(update_attr, "env")

    def build_command(self):  # pragma: no cover
        raise NotImplementedError

    def build_worker_payload(self):
        worker_payload = {
            "command": self.build_command(),
            "maxRunTime": self.max_run_time_minutes * 60
        }
        return dict_update_if_truthy(
            worker_payload,
            env=self.env,
            mounts=self.mounts,
            artifacts=[
                {
                    "type": type_,
                    "path": path,
                    "name": "public/" + url_basename(path),
                    "expires": SHARED.from_now_json(self.index_and_artifacts_expire_in),
                }
                for type_, path in self.artifacts
            ],
        )

    def with_artifacts(self, *paths, type="file"):
        self.artifacts.extend((type, path) for path in paths)
        return self

    def _mount_content(self, url_or_artifact_name, task_id, sha256):
        if task_id:
            content = {"taskId": task_id, "artifact": url_or_artifact_name}
        else:
            content = {"url": url_or_artifact_name}
        if sha256:
            content["sha256"] = sha256
        return content

    def with_file_mount(self, url_or_artifact_name, task_id=None, sha256=None, path=None):
        return self.with_mounts({
            "file": path or url_basename(url_or_artifact_name),
            "content": self._mount_content(url_or_artifact_name, task_id, sha256),
        })

    def with_directory_mount(self, url_or_artifact_name, task_id=None, sha256=None, path=None):
        supported_formats = ["rar", "tar.bz2", "tar.gz", "zip"]
        for fmt in supported_formats:
            suffix = "." + fmt
            if url_or_artifact_name.endswith(suffix):
                return self.with_mounts({
                    "directory": path or url_basename(url_or_artifact_name[:-len(suffix)]),
                    "content": self._mount_content(url_or_artifact_name, task_id, sha256),
                    "format": fmt,
                })
        raise ValueError(
            "%r does not appear to be in one of the supported formats: %r"
            % (url_or_artifact_name, ", ".join(supported_formats))
        )  # pragma: no cover


class WindowsGenericWorkerTask(GenericWorkerTask):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.scripts = []

    with_script = chaining(append_to_attr, "scripts")
    with_early_script = chaining(prepend_to_attr, "scripts")

    def build_command(self):
        return [deindent(s) for s in self.scripts]

    def with_path_from_homedir(self, *paths):
        for p in paths:
            self.with_early_script("set PATH=%HOMEDRIVE%%HOMEPATH%\\{};%PATH%".format(p))
        return self

    def with_repo(self, sparse_checkout=None):
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
            self.env["SPARSE_CHECKOUT_BASE64"] = base64.b64encode(
                "\n".join(sparse_checkout).encode("utf-8"))
        git += """
            git fetch --depth 1 %GIT_URL% %GIT_REF%
            git reset --hard %GIT_SHA%
        """
        return self \
        .with_git() \
        .with_script(git) \
        .with_env(**git_env())

    def with_git(self):
        return self \
        .with_path_from_homedir("git\\cmd") \
        .with_directory_mount(
            "https://github.com/git-for-windows/git/releases/download/" +
                "v2.19.0.windows.1/MinGit-2.19.0-64-bit.zip",
            sha256="424d24b5fc185a9c5488d7872262464f2facab4f1d4693ea8008196f14a3c19b",
            path="git",
        )

    def with_rustup(self):
        return self \
        .with_path_from_homedir(".cargo\\bin") \
        .with_early_script(
            "%HOMEDRIVE%%HOMEPATH%\\rustup-init.exe --default-toolchain none -y"
        ) \
        .with_file_mount(
            "https://static.rust-lang.org/rustup/archive/" +
                "1.13.0/i686-pc-windows-gnu/rustup-init.exe",
            sha256="43072fbe6b38ab38cd872fa51a33ebd781f83a2d5e83013857fab31fc06e4bf0",
        )

    def with_repacked_msi(self, url, sha256, path):
        repack_task = (
            WindowsGenericWorkerTask("MSI repack: " + url)
            .with_worker_type(self.worker_type)
            .with_max_run_time_minutes(20)
            .with_file_mount(url, sha256=sha256, path="input.msi")
            .with_directory_mount(
                "https://github.com/activescott/lessmsi/releases/download/" +
                    "v1.6.1/lessmsi-v1.6.1.zip",
                sha256="540b8801e08ec39ba26a100c855898f455410cecbae4991afae7bb2b4df026c7",
                path="lessmsi"
            )
            .with_directory_mount(
                "https://www.7-zip.org/a/7za920.zip",
                sha256="2a3afe19c180f8373fa02ff00254d5394fec0349f5804e0ad2f6067854ff28ac",
                path="7zip",
            )
            .with_path_from_homedir("lessmsi", "7zip")
            .with_script("""
                lessmsi x input.msi extracted\\
                cd extracted\\SourceDir
                7za a repacked.zip *
            """)
            .with_artifacts("extracted/SourceDir/repacked.zip")
            .with_index_and_artifacts_expire_in(CONFIG.repacked_msi_files_expire_in)
            .find_or_create("repacked-msi." + sha256)
        )
        return self \
        .with_dependencies(repack_task) \
        .with_directory_mount("public/repacked.zip", task_id=repack_task, path=path)

    def with_python2(self):
        return self \
        .with_repacked_msi(
            "https://www.python.org/ftp/python/2.7.15/python-2.7.15.amd64.msi",
            sha256="5e85f3c4c209de98480acbf2ba2e71a907fd5567a838ad4b6748c76deb286ad7",
            path="python2"
        ) \
        .with_early_script("""
            python -m ensurepip
            pip install virtualenv==16.0.0
        """) \
        .with_path_from_homedir("python2", "python2\\Scripts")



class DockerWorkerTask(Task):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.docker_image = "ubuntu:bionic-20180821"
        self.max_run_time_minutes = 30
        self.scripts = []
        self.env = {}
        self.caches = {}
        self.features = {}
        self.artifacts = []

    with_docker_image = chaining(setattr, "docker_image")
    with_max_run_time_minutes = chaining(setattr, "max_run_time_minutes")
    with_artifacts = chaining(append_to_attr, "artifacts")
    with_script = chaining(append_to_attr, "scripts")
    with_early_script = chaining(prepend_to_attr, "scripts")
    with_caches = chaining(update_attr, "caches")
    with_env = chaining(update_attr, "env")

    def build_worker_payload(self):
        worker_payload = {
            "image": self.docker_image,
            "maxRunTime": self.max_run_time_minutes * 60,
            "command": [
                "/bin/bash", "--login", "-x", "-e", "-c",
                deindent("\n".join(self.scripts))
            ],
        }
        return dict_update_if_truthy(
            worker_payload,
            env=self.env,
            cache=self.caches,
            features=self.features,
            artifacts={
                "public/" + url_basename(path): {
                    "type": "file",
                    "path": path,
                    "expires": SHARED.from_now_json(self.index_and_artifacts_expire_in),
                }
                for path in self.artifacts
            },
        )

    def with_features(self, *names):
        self.features.update({name: True for name in names})
        return self

    def with_repo(self):
        return self \
        .with_env(**git_env()) \
        .with_early_script("""
            git init repo
            cd repo
            git fetch --depth 1 "$GIT_URL" "$GIT_REF"
            git reset --hard "$GIT_SHA"
        """)

    def with_dockerfile(self, dockerfile):
        basename = os.path.basename(dockerfile)
        suffix = ".dockerfile"
        assert basename.endswith(suffix)
        image_name = basename[:-len(suffix)]

        dockerfile_contents = expand_dockerfile(dockerfile)
        digest = hashlib.sha256(dockerfile_contents).hexdigest()

        image_build_task = (
            DockerWorkerTask("Docker image: " + image_name)
            .with_worker_type(self.worker_type)
            .with_max_run_time_minutes(30)
            .with_index_and_artifacts_expire_in(CONFIG.docker_images_expire_in)
            .with_features("dind")
            .with_env(DOCKERFILE=dockerfile_contents)
            .with_artifacts("/image.tar.lz4")
            .with_script("""
                echo "$DOCKERFILE" | docker build -t taskcluster-built -
                docker save taskcluster-built | lz4 > /image.tar.lz4
            """)
            .with_docker_image(
                # https://github.com/servo/taskcluster-bootstrap-docker-images#image-builder
                "servobrowser/taskcluster-bootstrap:image-builder@sha256:" \
                "0a7d012ce444d62ffb9e7f06f0c52fedc24b68c2060711b313263367f7272d9d"
            )
            .find_or_create("docker-image." + digest)
        )

        return self \
        .with_dependencies(image_build_task) \
        .with_docker_image({
            "type": "task-image",
            "path": "public/image.tar.lz4",
            "taskId": image_build_task,
        })


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


def git_env():
    assert CONFIG.git_url
    assert CONFIG.git_ref
    assert CONFIG.git_sha
    return {
        "GIT_URL": CONFIG.git_url,
        "GIT_REF": CONFIG.git_ref,
        "GIT_SHA": CONFIG.git_sha,
    }

def dict_update_if_truthy(d, **kwargs):
    for key, value in kwargs.items():
        if value:
            d[key] = value
    return d


def deindent(string):
    return re.sub("\n +", "\n ", string).strip()


def url_basename(url):
    return url.rpartition("/")[-1]