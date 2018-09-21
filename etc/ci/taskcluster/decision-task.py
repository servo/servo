# coding: utf8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


import os.path
from decisionlib import DecisionTask


def main():
    linux_tidy_unit()
    linux_wpt()


build_artifacts_expiry = "1 week"
log_artifacts_expiry = "1 year"

build_env = {
    "RUST_BACKTRACE": "1",
    "RUSTFLAGS": "-Dwarnings",
    "CARGO_INCREMENTAL": "0",
    "SCCACHE_IDLE_TIMEOUT": "1200",
    "CCACHE": "sccache",
    "RUSTC_WRAPPER": "sccache",
}


def linux_tidy_unit():
    return decision.create_task(
        task_name="Linux x86_64: tidy + dev build + unit tests",
        script="""
            ./mach test-tidy --no-progress --all
            ./mach build --dev
            ./mach test-unit
            ./mach package --dev
            ./mach test-tidy --no-progress --self-test
            python2.7 ./etc/memory_reports_over_time.py --test
            ./etc/ci/lockfile_changed.sh
            ./etc/ci/check_no_panic.sh
        """,
        **build_kwargs
    )


def linux_wpt():
    release_build_task = linux_release_build()
    total_chunks = 2
    for i in range(total_chunks):
        this_chunk = i + 1
        wpt_chunk(release_build_task, total_chunks, this_chunk, extra=(this_chunk == 1))


def linux_release_build():
    return decision.find_or_create_task(
        route_bucket="build.linux_x86-64_release",
        route_key=os.environ["GIT_SHA"],  # Set in .taskcluster.yml
        route_expiry=build_artifacts_expiry,

        task_name="Linux x86_64: release build",
        script="""
            ./mach build --release --with-debug-assertions -p servo
            ./etc/ci/lockfile_changed.sh
            tar -czf /target.tar.gz \
                target/release/servo \
                target/release/build/osmesa-src-*/output \
                target/release/build/osmesa-src-*/out/lib/gallium
        """,
        artifacts=[
            "/target.tar.gz",
        ],
        **build_kwargs
    )


def wpt_chunk(release_build_task, total_chunks, this_chunk, extra):
    if extra:
        name_extra = " + extra"
        script_extra = """
            ./mach test-wpt-failure
            ./mach test-wpt --release --binary-arg=--multiprocess --processes 24 \
                --log-raw test-wpt-mp.log \
                --log-errorsummary wpt-mp-errorsummary.log \
                eventsource
        """
    else:
        name_extra = ""
        script_extra = ""
    script = """
        ./mach test-wpt \
            --release \
            --processes 24 \
            --total-chunks "$TOTAL_CHUNKS" \
            --this-chunk "$THIS_CHUNK" \
            --log-raw test-wpt.log \
            --log-errorsummary wpt-errorsummary.log \
            --always-succeed
        ./mach filter-intermittents\
            wpt-errorsummary.log \
            --log-intermittents intermittents.log \
            --log-filteredsummary filtered-wpt-errorsummary.log \
            --tracker-api default
    """
    # FIXME: --reporter-api default
    # IndexError: list index out of range
    # File "/repo/python/servo/testing_commands.py", line 533, in filter_intermittents
    #   pull_request = int(last_merge.split(' ')[4][1:])
    create_run_task(
        build_task=release_build_task,
        task_name="Linux x86_64: WPT chunk %s / %s%s" % (this_chunk, total_chunks, name_extra),
        script=script_extra + script,
        env={
            "TOTAL_CHUNKS": total_chunks,
            "THIS_CHUNK": this_chunk,
        },
    )


def create_run_task(*, build_task, script, **kwargs):
    fetch_build = """
        ./etc/ci/taskcluster/curl-artifact.sh ${BUILD_TASK_ID} target.tar.gz | tar -xz
    """
    kwargs.setdefault("env", {})["BUILD_TASK_ID"] = build_task
    kwargs.setdefault("dependencies", []).append(build_task)
    kwargs.setdefault("artifacts", []).extend(
        ("/repo/" + word, log_artifacts_expiry)
        for word in script.split() if word.endswith(".log")
    )
    return decision.create_task(
        script=fetch_build + script,
        max_run_time_minutes=60,
        dockerfile=dockerfile_path("run"),
        **kwargs
    )


def dockerfile_path(name):
    return os.path.join(os.path.dirname(__file__), "docker", name + ".dockerfile")


decision = DecisionTask(
    project_name="Servo",  # Used in task names
    route_prefix="project.servo.servo",
    worker_type="servo-docker-worker",
)

# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/caches
cache_scopes = [
    "docker-worker:cache:cargo-*",
]
build_caches = {
    "cargo-registry-cache": "/root/.cargo/registry",
    "cargo-git-cache": "/root/.cargo/git",
    "cargo-rustup": "/root/.rustup",
    "cargo-sccache": "/root/.cache/sccache",
}
build_kwargs = {
    "max_run_time_minutes": 60,
    "dockerfile": dockerfile_path("build"),
    "env": build_env,
    "scopes": cache_scopes,
    "cache": build_caches,
}


if __name__ == "__main__":
    main()
