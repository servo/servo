# coding: utf8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os.path
from decisionlib import *


def main(task_for, mock=False):
    if task_for == "github-push":
        if CONFIG.git_ref in ["refs/heads/auto", "refs/heads/try", "refs/heads/try-taskcluster"]:
            CONFIG.treeherder_repo_name = "servo/servo-" + CONFIG.git_ref.split("/")[-1]

            linux_tidy_unit()
            android_arm32()
            android_x86()
            windows_unit()
            macos_unit()

            # These are disabled in a "real" decision task,
            # but should still run when testing this Python code. (See `mock.py`.)
            if mock:
                windows_release()
                linux_wpt()
                linux_build_task("Indexed by task definition").find_or_create()

    # https://tools.taskcluster.net/hooks/project-servo/daily
    elif task_for == "daily":
        daily_tasks_setup()
        with_rust_nightly()
        android_arm32()

    else:  # pragma: no cover
        raise ValueError("Unrecognized $TASK_FOR value: %r", task_for)


ping_on_daily_task_failure = "SimonSapin, nox, emilio"
build_artifacts_expire_in = "1 week"
build_dependencies_artifacts_expire_in = "1 month"
log_artifacts_expire_in = "1 year"

build_env = {
    "RUST_BACKTRACE": "1",
    "RUSTFLAGS": "-Dwarnings",
    "CARGO_INCREMENTAL": "0",
}
unix_build_env = {
    "CCACHE": "sccache",
    "RUSTC_WRAPPER": "sccache",
    "SCCACHE_IDLE_TIMEOUT": "1200",
}
linux_build_env = {
    "SHELL": "/bin/dash",  # For SpiderMonkey’s build system
}
macos_build_env = {}
windows_build_env = {
    "LIB": "%HOMEDRIVE%%HOMEPATH%\\gst\\gstreamer\\1.0\\x86_64\\lib;%LIB%",
}
windows_sparse_checkout = [
    "/*",
    "!/tests/wpt/metadata",
    "!/tests/wpt/mozilla",
    "!/tests/wpt/webgl",
    "!/tests/wpt/web-platform-tests",
    "/tests/wpt/web-platform-tests/tools",
]


def linux_tidy_unit():
    return (
        linux_build_task("Tidy + dev build + unit")
        .with_treeherder("Linux x64")
        .with_script("""
            ./mach test-tidy --no-progress --all
            ./mach build --dev
            ./mach test-unit
            ./mach package --dev
            ./mach test-tidy --no-progress --self-test
            ./etc/memory_reports_over_time.py --test
            ./etc/taskcluster/mock.py
            ./etc/ci/lockfile_changed.sh
            ./etc/ci/check_no_panic.sh
        """).create()
    )


def macos_unit():
    return (
        macos_build_task("Dev build + unit tests")
        .with_treeherder("macOS x64")
        .with_script("""
            ./mach build --dev
            ./mach test-unit
            ./mach package --dev
            ./etc/ci/lockfile_changed.sh
        """).create()
    )


def with_rust_nightly():
    return linux_build_task("Linux x64: with Rust Nightly").with_script("""
        echo "nightly" > rust-toolchain
        ./mach build --dev
        ./mach test-unit
    """).create()


def android_arm32():
    return (
        android_build_task("Release build")
        .with_treeherder("Android ARMv7")
        .with_script("./mach build --android --release")
        .with_artifacts(
            "/repo/target/android/armv7-linux-androideabi/release/servoapp.apk",
            "/repo/target/android/armv7-linux-androideabi/release/servoview.aar",
        )
        .find_or_create("build.android_armv7_release." + CONFIG.git_sha)
    )


def android_x86():
    build_task = (
        android_build_task("Release build")
        .with_treeherder("Android x86")
        .with_script("./mach build --target i686-linux-android --release")
        .with_artifacts(
            "/repo/target/android/i686-linux-android/release/servoapp.apk",
            "/repo/target/android/i686-linux-android/release/servoview.aar",
        )
        .find_or_create("build.android_x86_release." + CONFIG.git_sha)
    )
    return (
        DockerWorkerTask("WPT")
        .with_treeherder("Android x86")
        .with_provisioner_id("proj-servo")
        .with_worker_type("docker-worker-kvm")
        .with_capabilities(privileged=True)
        .with_scopes("project:servo:docker-worker-kvm:capability:privileged")
        .with_dockerfile(dockerfile_path("run-android-emulator"))
        .with_repo()
        .with_curl_artifact_script(build_task, "servoapp.apk", "target/android/i686-linux-android/release")
        .with_script("""
            ./mach bootstrap-android --accept-all-licences --emulator-x86
            ./mach test-android-startup --release
            ./mach test-wpt-android --release \
                /_mozilla/mozilla/DOMParser.html \
                /_mozilla/mozilla/webgl/context_creation_error.html
        """)
        .create()
    )


def windows_unit():
    return (
        windows_build_task("Build + unit tests")
        .with_treeherder("Windows x64", "debug")
        .with_script(
            # Not necessary as this would be done at the start of `build`,
            # but this allows timing it separately.
            "mach fetch",

            "mach build --dev",
            "mach test-unit",
            "mach package --dev",
        )
        .with_artifacts("repo/target/debug/msi/Servo.exe",
                        "repo/target/debug/msi/Servo.zip")
        .find_or_create("build.windows_x64_dev." + CONFIG.git_sha)
    )


def windows_release():
    return (
        windows_build_task("Release build")
        .with_treeherder("Windows x64")
        .with_script("mach build --release",
                     "mach package --release")
        .with_artifacts("repo/target/release/msi/Servo.exe",
                        "repo/target/release/msi/Servo.zip")
        .find_or_create("build.windows_x64_release." + CONFIG.git_sha)
    )


def linux_wpt():
    release_build_task = linux_release_build()
    total_chunks = 2
    for i in range(total_chunks):
        this_chunk = i + 1
        wpt_chunk(release_build_task, total_chunks, this_chunk)


def linux_release_build():
    return (
        linux_build_task("Release build")
        .with_treeherder("Linux x64")
        .with_script("""
            ./mach build --release --with-debug-assertions -p servo
            ./etc/ci/lockfile_changed.sh
            tar -czf /target.tar.gz \
                target/release/servo \
                target/release/build/osmesa-src-*/output \
                target/release/build/osmesa-src-*/out/lib/gallium
        """)
        .with_artifacts("/target.tar.gz")
        .find_or_create("build.linux_x64_release." + CONFIG.git_sha)
    )


def wpt_chunk(release_build_task, total_chunks, this_chunk):
    task = (
        linux_task("WPT chunk %s / %s" % (this_chunk, total_chunks))
        .with_treeherder("Linux x64", "WPT %s" % this_chunk)
        .with_dockerfile(dockerfile_path("run"))
        .with_repo()
        .with_curl_artifact_script(release_build_task, "target.tar.gz")
        .with_script("tar -xzf target.tar.gz")
        .with_index_and_artifacts_expire_in(log_artifacts_expire_in)
        .with_max_run_time_minutes(60)
        .with_env(TOTAL_CHUNKS=total_chunks, THIS_CHUNK=this_chunk)
        .with_script("""
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
        """)
        # FIXME: --reporter-api default
        # IndexError: list index out of range
        # File "/repo/python/servo/testing_commands.py", line 533, in filter_intermittents
        #   pull_request = int(last_merge.split(' ')[4][1:])
    )
    if this_chunk == 1:
        task.name += " + extra"
        task.extra["treeherder"]["symbol"] += "+"
        task.with_script("""
            ./mach test-wpt-failure
            ./mach test-wpt --release --binary-arg=--multiprocess --processes 24 \
                --log-raw test-wpt-mp.log \
                --log-errorsummary wpt-mp-errorsummary.log \
                eventsource
        """)
    task.with_artifacts(*[
        "/repo/" + word
        for script in task.scripts
        for word in script.split()
        if word.endswith(".log")
    ])
    task.create()


def daily_tasks_setup():
    # ':' is not accepted in an index namepspace:
    # https://docs.taskcluster.net/docs/reference/core/taskcluster-index/references/api
    now = SHARED.now.strftime("%Y-%m-%d_%H-%M-%S")
    index_path = "%s.daily.%s" % (CONFIG.index_prefix, now)
    # Index this task manually rather than with a route,
    # so that it is indexed even if it fails.
    SHARED.index_service.insertTask(index_path, {
        "taskId": CONFIG.decision_task_id,
        "rank": 0,
        "data": {},
        "expires": SHARED.from_now_json(log_artifacts_expire_in),
    })

    # Unlike when reacting to a GitHub event,
    # the commit hash is not known until we clone the repository.
    CONFIG.git_sha_is_current_head()

    # On failure, notify a few people on IRC
    # https://docs.taskcluster.net/docs/reference/core/taskcluster-notify/docs/usage
    notify_route = "notify.irc-channel.#servo.on-failed"
    CONFIG.routes_for_all_subtasks.append(notify_route)
    CONFIG.scopes_for_all_subtasks.append("queue:route:" + notify_route)
    CONFIG.task_name_template = "Servo daily: %s. On failure, ping: " + ping_on_daily_task_failure


def dockerfile_path(name):
    return os.path.join(os.path.dirname(__file__), "docker", name + ".dockerfile")


def linux_task(name):
    return DockerWorkerTask(name).with_worker_type("servo-docker-worker")


def windows_task(name):
    return WindowsGenericWorkerTask(name).with_worker_type("servo-win2016")


def macos_task(name):
    return (
        MacOsGenericWorkerTask(name)
        .with_provisioner_id("proj-servo")
        .with_worker_type("macos")
    )


def linux_build_task(name):
    return (
        linux_task(name)
        # https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/caches
        .with_scopes("docker-worker:cache:servo-*")
        .with_caches(**{
            "servo-cargo-registry": "/root/.cargo/registry",
            "servo-cargo-git": "/root/.cargo/git",
            "servo-rustup": "/root/.rustup",
            "servo-sccache": "/root/.cache/sccache",
            "servo-gradle": "/root/.gradle",
        })
        .with_index_and_artifacts_expire_in(build_artifacts_expire_in)
        .with_max_run_time_minutes(60)
        .with_dockerfile(dockerfile_path("build"))
        .with_env(**build_env, **unix_build_env, **linux_build_env)
        .with_repo()
        .with_index_and_artifacts_expire_in(build_artifacts_expire_in)
    )


def android_build_task(name):
    return (
        linux_build_task(name)
        # file: NDK parses $(file $SHELL) to tell x64 host from x86
        # wget: servo-media-gstreamer’s build script
        .with_script("""
            apt-get update -q
            apt-get install -y --no-install-recommends openjdk-8-jdk-headless file wget
            ./mach bootstrap-android --accept-all-licences --build
        """)
    )


def windows_build_task(name):
    return (
        windows_task(name)
        .with_max_run_time_minutes(60)
        .with_env(**build_env, **windows_build_env)
        .with_repo(sparse_checkout=windows_sparse_checkout)
        .with_python2()
        .with_rustup()
        .with_repacked_msi(
            url="https://gstreamer.freedesktop.org/data/pkg/windows/" +
                "1.14.3/gstreamer-1.0-devel-x86_64-1.14.3.msi",
            sha256="b13ea68c1365098c66871f0acab7fd3daa2f2795b5e893fcbb5cd7253f2c08fa",
            path="gst",
        )
        .with_directory_mount(
            "https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311-binaries.zip",
            sha256="37f0a533b0978a454efb5dc3bd3598becf9660aaf4287e55bf68ca6b527d051d",
            path="wix",
        )
        .with_path_from_homedir("wix")
    )


def macos_build_task(name):
    return (
        macos_task(name)
        # Allow long runtime in case the cache expired for all those Homebrew dependencies
        .with_max_run_time_minutes(60 * 2)
        .with_env(**build_env, **unix_build_env, **macos_build_env)
        .with_repo()
        .with_python2()
        .with_rustup()
        .with_script("""
            mkdir -p "$HOME/homebrew"
            export PATH="$HOME/homebrew/bin:$PATH"
            which brew || curl -L https://github.com/Homebrew/brew/tarball/master \
                | tar xz --strip 1 -C "$HOME/homebrew"

            time brew bundle install --no-upgrade --file=etc/taskcluster/macos/Brewfile
            export OPENSSL_INCLUDE_DIR="$(brew --prefix openssl)/include"
            export OPENSSL_LIB_DIR="$(brew --prefix openssl)/lib"
        """)

        .with_directory_mount(
            "https://github.com/mozilla/sccache/releases/download/"
                "0.2.7/sccache-0.2.7-x86_64-apple-darwin.tar.gz",
            sha256="f86412abbbcce2d3f23e7d33305469198949f5cf807e6c3258c9e1885b4cb57f",
            path="sccache",
        )
        # Early script in order to run with the initial $PWD
        .with_early_script("""
            export PATH="$PWD/sccache/sccache-0.2.7-x86_64-apple-darwin:$PATH"
        """)
        # sccache binaries requires OpenSSL 1.1 and are not compatible with 1.0.
        # "Late" script in order to run after Homebrew is installed.
        .with_script("""
            time brew install openssl@1.1
            export DYLD_LIBRARY_PATH="$HOME/homebrew/opt/openssl@1.1/lib"
        """)
    )


CONFIG.task_name_template = "Servo: %s"
CONFIG.index_prefix = "project.servo.servo"
CONFIG.docker_image_buil_worker_type = "servo-docker-worker"
CONFIG.docker_images_expire_in = build_dependencies_artifacts_expire_in
CONFIG.repacked_msi_files_expire_in = build_dependencies_artifacts_expire_in


if __name__ == "__main__":  # pragma: no cover
    main(task_for=os.environ["TASK_FOR"])
