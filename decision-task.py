# coding: utf8

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


def main():
    decision = DecisionTask(
        project_name="Taskcluster experimenfts for Servo",  # Used in task names
        route_prefix="project.servo.servo-taskcluster-experiments",
        docker_image_cache_expiry="1 week",
        worker_type="servo-docker-worker",
    )

    decision.create_task_with_in_tree_dockerfile(
        task_name="servo build task",
        command="./servo-build-task.sh",
        dockerfile=dockerfile("servo-x86_64-linux"),
        max_run_time_minutes=60,
        scopes=CARGO_CACHE_SCOPES,
        cache=CARGO_CACHE,
    )

    build_task = decision.create_task_with_in_tree_dockerfile(
        task_name="build task",
        command="./build-task.sh",
        dockerfile=dockerfile("servo-x86_64-linux"),
        max_run_time_minutes=20,
        scopes=CARGO_CACHE_SCOPES,
        cache=CARGO_CACHE,

        artifacts=[
            ("/repo/something-rust/executable.gz", "1 week"),
        ],
    )

    decision.create_task(
        task_name="run task",
        command="./run-task.sh",
        image="buildpack-deps:bionic-scm",
        max_run_time_minutes=20,
        dependencies=[build_task],
        env={"BUILD_TASK_ID": build_task},
    )


def dockerfile(name):
    return os.path.join(os.path.dirname(__file__), name + ".dockerfile")


if __name__ == "__main__":
    main()
