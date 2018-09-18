# coding: utf8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


import os.path
from decisionlib import DecisionTask


# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/caches
CARGO_CACHE_SCOPES = [
    "docker-worker:cache:cargo-registry-cache",
    "docker-worker:cache:cargo-git-cache",
]

CARGO_CACHE = {
    "cargo-registry-cache": "/root/.cargo/registry",
    "cargo-git-cache": "/root/.cargo/git",
}

BUILD_ENV = {
    "RUST_BACKTRACE": "1",
    "RUSTFLAGS": "-Dwarnings",
    "CARGO_INCREMENTAL": "0",
    "SCCACHE_IDLE_TIMEOUT": "1200",
}


def main():
    decision = DecisionTask(
        project_name="Servo",  # Used in task names
        route_prefix="project.servo.servo",
        worker_type="servo-docker-worker",
    )

    # FIXME: remove this before merging in servo/servo
    os.environ["GIT_URL"] = "https://github.com/SimonSapin/servo"
    os.environ["GIT_REF"] = "refs/heads/taskcluster-experiments-20180918"
    os.environ["GIT_SHA"] = "605d74c59b6de7ae2b535d42fde40405a96b67e0"
    decision.docker_image_cache_expiry = "1 week"
    decision.route_prefix = "project.servo.servo-taskcluster-experiments"
    # ~

    decision.create_task_with_in_tree_dockerfile(
        task_name="Linux x86_64: tidy + dev build + unit tests",
        command="""
            sccache --version
            ./mach test-tidy --no-progress --all
            #./mach build --dev
            #./mach test-unit
            #./mach test-tidy --no-progress --self-test
        """,
        env=BUILD_ENV,
        dockerfile=dockerfile("build-x86_64-linux"),
        max_run_time_minutes=60,
        scopes=CARGO_CACHE_SCOPES,
        cache=CARGO_CACHE,
    )


def dockerfile(name):
    return os.path.join(os.path.dirname(__file__), name + ".dockerfile")


if __name__ == "__main__":
    main()
