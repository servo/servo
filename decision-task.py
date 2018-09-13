# coding: utf8

from decisionlib import *


def main():
    build_task = create_task_with_in_tree_dockerfile(
        task_name="build task",
        command="./build-task.sh",
        image="servo-x86_64-linux",
        max_run_time_minutes=20,

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
        task_name="run task",
        command="./run-task.sh",
        image="buildpack-deps:bionic-scm",
        max_run_time_minutes=20,
        dependencies=[build_task],
        env={"BUILD_TASK_ID": build_task},
    )


if __name__ == "__main__":
    main()
