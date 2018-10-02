# coding: utf8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import hashlib
import json
import os.path
import subprocess
from decisionlib import DecisionTask


def main():
    task_for = os.environ["TASK_FOR"]

    if task_for == "github-push":
        linux_tidy_unit()
        #linux_wpt()
        android_arm32()
        windows_dev()

    # https://tools.taskcluster.net/hooks/project-servo/daily
    elif task_for == "daily":
        daily_tasks_setup()
        with_rust_nightly()
        android_arm32()

    else:
        raise ValueError("Unrecognized $TASK_FOR value: %r", task_for)


ping_on_daily_task_failure = "SimonSapin, nox, emilio"
build_artifacts_expiry = "1 week"
build_dependencies_artifacts_expiry = "1 month"
log_artifacts_expiry = "1 year"

build_env = {
    "RUST_BACKTRACE": "1",
    "RUSTFLAGS": "-Dwarnings",
    "CARGO_INCREMENTAL": "0",
    "SCCACHE_IDLE_TIMEOUT": "1200",
    "CCACHE": "sccache",
    "RUSTC_WRAPPER": "sccache",
    "SHELL": "/bin/dash",  # For SpiderMonkey’s build system
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
            python3 ./etc/taskcluster/mock.py
            ./etc/ci/lockfile_changed.sh
            ./etc/ci/check_no_panic.sh
        """,
        **linux_build_kwargs
    )


def with_rust_nightly():
    return decision.create_task(
        task_name="Linux x86_64: with Rust Nightly",
        script="""
            echo "nightly" > rust-toolchain
            ./mach build --dev
            ./mach test-unit
        """,
        **linux_build_kwargs
    )


def android_arm32():
    return decision.find_or_create_task(
        index_bucket="build.android_armv7_release",
        index_key=os.environ["GIT_SHA"],  # Set in .taskcluster.yml
        index_expiry=build_artifacts_expiry,

        task_name="Android ARMv7: build",
        # file: NDK parses $(file $SHELL) to tell x86_64 from x86
        # wget: servo-media-gstreamer’s build script
        script="""
            apt-get install -y --no-install-recommends openjdk-8-jdk-headless file wget
            ./etc/ci/bootstrap-android-and-accept-licences.sh
            ./mach build --android --release
        """,
        artifacts=[
            "/repo/target/armv7-linux-androideabi/release/servoapp.apk",
            "/repo/target/armv7-linux-androideabi/release/servoview.aar",
        ],
        **linux_build_kwargs
    )


def windows_dev():
    python2_task_definition = dict(
        task_name="Windows x86_64: repackage Python 2",
        worker_type="servo-win2016",
        with_repo=False,
        script="""
            lessmsi x python2.msi python2\\
            cd python2\\SourceDir
            7za a python2.zip *
        """,
        mounts=[
            {
                "file": "python2.msi",
                "content": {
                    "url": "https://www.python.org/ftp/python/2.7.15/python-2.7.15.amd64.msi",
                    "sha256": "5e85f3c4c209de98480acbf2ba2e71a907fd5567a838ad4b6748c76deb286ad7",
                }
            },
            {
                "directory": "lessmsi",
                "format": "zip",
                "content": {
                    "url": "https://github.com/activescott/lessmsi/releases/download/" +
                           "v1.6.1/lessmsi-v1.6.1.zip",
                    "sha256": "540b8801e08ec39ba26a100c855898f455410cecbae4991afae7bb2b4df026c7",
                }
            },
            {
                "directory": "7zip",
                "format": "zip",
                "content": {
                    "url": "https://www.7-zip.org/a/7za920.zip",
                    "sha256": "2a3afe19c180f8373fa02ff00254d5394fec0349f5804e0ad2f6067854ff28ac",
                }
            }
        ],
        homedir_path=[
            "lessmsi",
            "7zip",
        ],
        artifacts=[
            "python2/SourceDir/python2.zip",
        ],
        max_run_time_minutes=20,
    )
    index_by = json.dumps(python2_task_definition).encode("utf-8")
    python2_task = decision.find_or_create_task(
        index_bucket="by-task-definition",
        index_key=hashlib.sha256(index_by).hexdigest(),
        index_expiry=build_artifacts_expiry,
        **python2_task_definition
    )

    return decision.create_task(
        task_name="Windows x86_64: clone only (for now)",
        worker_type="servo-win2016",
        script="""
            ..\\rustup-init.exe --default-toolchain none -y
            python -m ensurepip
            pip install virtualenv==16.0.0
            python mach --help
        """,
        mounts=[
            {
                "directory": "git",
                "format": "zip",
                "content": {
                    "url": "https://github.com/git-for-windows/git/releases/download/" +
                           "v2.19.0.windows.1/MinGit-2.19.0-64-bit.zip",
                    "sha256": "424d24b5fc185a9c5488d7872262464f2facab4f1d4693ea8008196f14a3c19b",
                }
            },
            {
                "directory": "python2",
                "format": "zip",
                "content": {
                    "taskId": python2_task,
                    "artifact": "public/python2.zip",
                }
            },
            {
                "file": "rustup-init.exe",
                "content": {
                    "url": "https://static.rust-lang.org/rustup/archive/" +
                           "1.13.0/i686-pc-windows-gnu/rustup-init.exe",
                    "sha256": "43072fbe6b38ab38cd872fa51a33ebd781f83a2d5e83013857fab31fc06e4bf0",
                }
            }
        ],
        homedir_path=[
            "git\\cmd",
            "python2",
            "python2\\Scripts",
            ".cargo\\bin",
        ],
        dependencies=[python2_task],
        sparse_checkout=[
            "/*",
            "!/tests/wpt/metadata",
            "!/tests/wpt/mozilla",
            "!/tests/wpt/webgl",
            "!/tests/wpt/web-platform-tests",
            "/tests/wpt/web-platform-tests/tools",
        ],
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
        index_bucket="build.linux_x86-64_release",
        index_key=os.environ["GIT_SHA"],  # Set in .taskcluster.yml
        index_expiry=build_artifacts_expiry,

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
        **linux_build_kwargs
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
        ./etc/taskcluster/curl-artifact.sh ${BUILD_TASK_ID} target.tar.gz | tar -xz
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


def daily_tasks_setup():
    # ':' is not accepted in an index namepspace:
    # https://docs.taskcluster.net/docs/reference/core/taskcluster-index/references/api
    now = decision.now.strftime("%Y-%m-%d_%H-%M-%S")
    index_path = "%s.daily.%s" % (decision.index_prefix, now)
    # Index this task manually rather than with a route,
    # so that it is indexed even if it fails.
    decision.index_service.insertTask(index_path, {
        "taskId": os.environ["TASK_ID"],
        "rank": 0,
        "data": {},
        "expires": decision.from_now_json(log_artifacts_expiry),
    })

    # Unlike when reacting to a GitHub event,
    # the commit hash is not known until we clone the repository.
    os.environ["GIT_SHA"] = \
        subprocess.check_output(["git", "rev-parse", "HEAD"]).decode("utf8").strip()

    # On failure, notify a few people on IRC
    # https://docs.taskcluster.net/docs/reference/core/taskcluster-notify/docs/usage
    notify_route = "notify.irc-channel.#servo.on-failed"
    decision.routes_for_all_subtasks.append(notify_route)
    decision.scopes_for_all_subtasks.append("queue:route:" + notify_route)
    decision.task_name_template = "Servo daily: %s. On failure, ping: " + ping_on_daily_task_failure


def dockerfile_path(name):
    return os.path.join(os.path.dirname(__file__), "docker", name + ".dockerfile")


decision = DecisionTask(
    task_name_template="Servo: %s",
    index_prefix="project.servo.servo",
    default_worker_type="servo-docker-worker",
    docker_image_cache_expiry=build_dependencies_artifacts_expiry,
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
    "env": build_env,
}
linux_build_kwargs = dict(**build_kwargs, **{
    "worker_type": "servo-docker-worker",
    "dockerfile": dockerfile_path("build"),
    "scopes": cache_scopes,
    "cache": build_caches,
})


if __name__ == "__main__":
    main()
