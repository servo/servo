# coding: utf8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os.path
import decisionlib
from decisionlib import CONFIG, SHARED


def main(task_for):
    if CONFIG.git_ref.startswith("refs/heads/"):
        branch = CONFIG.git_ref[len("refs/heads/"):]
        CONFIG.treeherder_repository_name = "servo-" + (
            branch if not branch.startswith("try-") else "try"
        )

    if task_for == "github-push":
        # FIXME https://github.com/servo/servo/issues/22325 implement these:
        magicleap_dev = linux_arm32_dev = linux_arm64_dev = \
            android_arm32_dev_from_macos = lambda: None

        # FIXME https://github.com/servo/servo/issues/22187
        # In-emulator testing is disabled for now. (Instead we only compile.)
        # This local variable shadows the module-level function of the same name.
        android_x86_wpt = android_x86_release

        # Implemented but disabled for now:
        linux_wpt = lambda: None  # Shadows the existing top-level function

        all_tests = [
            linux_tidy_unit,
            linux_docs,
            windows_unit,
            macos_unit,
            magicleap_dev,
            android_arm32_dev,
            android_arm32_release,
            android_x86_wpt,
            linux_arm32_dev,
            linux_arm64_dev,
            linux_wpt,
            macos_wpt,
        ]
        by_branch_name = {
            "auto": all_tests,
            "try": all_tests,
            "try-taskcluster": [
                # Add functions here as needed, in your push to that branch
            ],
            "master": [
                upload_docs,
            ],

            # The "try-*" keys match those in `servo_try_choosers` in Homu’s config:
            # https://github.com/servo/saltfs/blob/master/homu/map.jinja

            "try-mac": [macos_unit],
            "try-linux": [linux_tidy_unit],
            "try-windows": [windows_unit],
            "try-magicleap": [magicleap_dev],
            "try-arm": [linux_arm32_dev, linux_arm64_dev],
            "try-wpt": [linux_wpt],
            "try-wpt-mac": [macos_wpt],
            "try-wpt-android": [android_x86_wpt],
            "try-android": [
                android_arm32_dev,
                android_arm32_dev_from_macos,
                android_x86_wpt
            ],
        }
        for function in by_branch_name.get(branch, []):
            function()

    elif task_for == "github-pull-request":
        CONFIG.treeherder_repository_name = "servo-prs"
        CONFIG.index_read_only = True

        # We want the merge commit that GitHub creates for the PR.
        # The event does contain a `pull_request.merge_commit_sha` key, but it is wrong:
        # https://github.com/servo/servo/pull/22597#issuecomment-451518810
        CONFIG.git_sha_is_current_head()

        tidy_untrusted()

    # https://tools.taskcluster.net/hooks/project-servo/daily
    elif task_for == "daily":
        daily_tasks_setup()
        with_rust_nightly()
        linux_nightly()


# These are disabled in a "real" decision task,
# but should still run when testing this Python code. (See `mock.py`.)
def mocked_only():
    windows_release()
    android_x86_wpt()
    linux_wpt()
    decisionlib.DockerWorkerTask("Indexed by task definition").find_or_create()


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
    "GSTREAMER_1_0_ROOT_X86_64": "%HOMEDRIVE%%HOMEPATH%\\gst\\gstreamer\\1.0\\x86_64\\",
}
windows_sparse_checkout = [
    "/*",
    "!/tests/wpt/metadata",
    "!/tests/wpt/mozilla",
    "!/tests/wpt/webgl",
    "!/tests/wpt/web-platform-tests",
    "/tests/wpt/web-platform-tests/tools",
]


def tidy_untrusted():
    return (
        decisionlib.DockerWorkerTask("Tidy")
        .with_worker_type("servo-docker-untrusted")
        .with_treeherder("Linux x64", "Tidy")
        .with_max_run_time_minutes(60)
        .with_dockerfile(dockerfile_path("build"))
        .with_env(**build_env, **unix_build_env, **linux_build_env)
        .with_repo()
        .with_script("""
            ./mach test-tidy --no-progress --all
        """)
        .create()
    )


def linux_tidy_unit():
    return (
        linux_build_task("Tidy + dev build + unit tests")
        .with_treeherder("Linux x64", "Tidy+Unit")
        .with_script("""
            ./mach test-tidy --no-progress --all
            ./mach build --dev
            ./mach test-unit
            ./mach package --dev
            ./mach build --dev --libsimpleservo
            ./mach build --dev --no-default-features --features default-except-unstable
            ./mach test-tidy --no-progress --self-test

            ./etc/memory_reports_over_time.py --test
            ./etc/taskcluster/mock.py
            ./etc/ci/lockfile_changed.sh
            ./etc/ci/check_no_panic.sh
        """)
        .find_or_create("linux_tidy_unit." + CONFIG.git_sha)
    )

def linux_docs():
    return (
        linux_build_task("Docs")
        .with_treeherder("Linux x64", "Doc")
        .with_script("""
            ./mach doc
            cd target/doc
            git init
            time git add .
            git -c user.name="Taskcluster" -c user.email="" \
                commit -q -m "Rebuild Servo documentation"
            git bundle create docs.bundle HEAD
        """)
        .with_artifacts("/repo/target/doc/docs.bundle")
        .find_or_create("docs." + CONFIG.git_sha)
    )

def upload_docs():
    docs_build_task_id = decisionlib.Task.find("docs." + CONFIG.git_sha)
    return (
        linux_task("Upload docs to GitHub Pages")
        .with_treeherder("Linux x64", "DocUpload")
        .with_dockerfile(dockerfile_path("base"))
        .with_curl_artifact_script(docs_build_task_id, "docs.bundle")
        .with_features("taskclusterProxy")
        .with_scopes("secrets:get:project/servo/doc.servo.org")
        .with_env(PY="""if 1:
            import urllib, json
            url = "http://taskcluster/secrets/v1/secret/project/servo/doc.servo.org"
            token = json.load(urllib.urlopen(url))["secret"]["token"]
            open("/root/.git-credentials", "w").write("https://git:%s@github.com/" % token)
        """)
        .with_script("""
            python -c "$PY"
            git init --bare
            git config credential.helper store
            git fetch --quiet docs.bundle
            git push --force https://github.com/servo/doc.servo.org FETCH_HEAD:gh-pages
        """)
        .create()
    )


def macos_unit():
    return (
        macos_build_task("Dev build + unit tests")
        .with_treeherder("macOS x64", "Unit")
        .with_script("""
            ./mach build --dev
            ./mach test-unit
            ./mach package --dev
            ./etc/ci/lockfile_changed.sh
        """)
        .find_or_create("macos_unit." + CONFIG.git_sha)
    )


def with_rust_nightly():
    modified_build_env = dict(build_env)
    flags = modified_build_env.pop("RUSTFLAGS").split(" ")
    flags.remove("-Dwarnings")
    if flags:  # pragma: no cover
        modified_build_env["RUSTFLAGS"] = " ".join(flags)

    return (
        linux_build_task("with Rust Nightly", build_env=modified_build_env)
        .with_treeherder("Linux x64", "RustNightly")
        .with_script("""
            echo "nightly" > rust-toolchain
            ./mach build --dev
            ./mach test-unit
        """)
        .create()
    )


def android_arm32_dev():
    return (
        android_build_task("Dev build")
        .with_treeherder("Android ARMv7")
        .with_script("""
            ./mach build --android --dev
            ./etc/ci/lockfile_changed.sh
            python ./etc/ci/check_dynamic_symbols.py
        """)
        .find_or_create("android_arm32_dev." + CONFIG.git_sha)
    )


def android_arm32_release():
    return (
        android_build_task("Release build")
        .with_treeherder("Android ARMv7", "Release")
        .with_script("./mach build --android --release")
        .with_artifacts(
            "/repo/target/android/armv7-linux-androideabi/release/servoapp.apk",
            "/repo/target/android/armv7-linux-androideabi/release/servoview.aar",
        )
        .find_or_create("build.android_armv7_release." + CONFIG.git_sha)
    )


def android_x86_release():
    return (
        android_build_task("Release build")
        .with_treeherder("Android x86", "Release")
        .with_script("./mach build --target i686-linux-android --release")
        .with_artifacts(
            "/repo/target/android/i686-linux-android/release/servoapp.apk",
            "/repo/target/android/i686-linux-android/release/servoview.aar",
        )
        .find_or_create("build.android_x86_release." + CONFIG.git_sha)
    )


def android_x86_wpt():
    build_task = android_x86_release()
    return (
        linux_task("WPT")
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
        .find_or_create("android_x86_release." + CONFIG.git_sha)
    )


def windows_unit():
    return (
        windows_build_task("Dev build + unit tests")
        .with_treeherder("Windows x64", "Unit")
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
        .with_treeherder("Windows x64", "Release")
        .with_script("mach build --release",
                     "mach package --release")
        .with_artifacts("repo/target/release/msi/Servo.exe",
                        "repo/target/release/msi/Servo.zip")
        .find_or_create("build.windows_x64_release." + CONFIG.git_sha)
    )


def linux_nightly():
    return (
        linux_build_task("Nightly build and upload")
        .with_treeherder("Linux x64", "Nightly")
        .with_features("taskclusterProxy")
        .with_scopes("secrets:get:project/servo/s3-upload")
        .with_env(PY=r"""if 1:
            import urllib, json
            url = "http://taskcluster/secrets/v1/secret/project/servo/s3-upload"
            secret = json.load(urllib.urlopen(url))["secret"]
            open("/root/.aws/credentials", "w").write(secret["credentials_file"])
        """)
        # Not reusing the build made for WPT because it has debug assertions
        .with_script("""
            ./mach build --release
            ./mach package --release
            mkdir /root/.aws
            python -c "$PY"
            ./mach upload-nightly linux
        """)
        .with_artifacts("/repo/target/release/servo-tech-demo.tar.gz")
        .find_or_create("build.linux_x64_nightly" + CONFIG.git_sha)
    )


def linux_wpt():
    release_build_task = (
        linux_build_task("Release build, with debug assertions")
        .with_treeherder("Linux x64", "Release+A")
        .with_script("""
            ./mach build --release --with-debug-assertions -p servo
            ./etc/ci/lockfile_changed.sh
            tar -czf /target.tar.gz \
                target/release/servo \
                target/release/build/osmesa-src-*/output \
                target/release/build/osmesa-src-*/out/lib/gallium
        """)
        .with_artifacts("/target.tar.gz")
        .find_or_create("build.linux_x64_release~assertions" + CONFIG.git_sha)
    )
    def linux_run_task(name):
        return linux_task(name).with_dockerfile(dockerfile_path("run"))
    wpt_chunks("Linux x64", linux_run_task, release_build_task, repo_dir="/repo",
               total_chunks=2, processes=24)


def macos_wpt():
    build_task = (
        macos_build_task("Release build")
        .with_treeherder("macOS x64", "Release")
        .with_script("""
            ./mach build --release
            ./etc/ci/lockfile_changed.sh
            tar -czf target.tar.gz \
                target/release/servo \
                target/release/build/osmesa-src-*/output \
                target/release/build/osmesa-src-*/out/src/gallium/targets/osmesa/.libs \
                target/release/build/osmesa-src-*/out/src/mapi/shared-glapi/.libs
        """)
        .with_artifacts("repo/target.tar.gz")
        .find_or_create("build.macos_x64_release." + CONFIG.git_sha)
    )
    def macos_run_task(name):
        return macos_task(name).with_python2()
    wpt_chunks("macOS x64", macos_run_task, build_task, repo_dir="repo",
               total_chunks=6, processes=4, chunks=[1])


def wpt_chunks(platform, make_chunk_task, build_task, total_chunks, processes,
               repo_dir, chunks="all"):
    if chunks == "all":
        chunks = [n + 1 for n in range(total_chunks)]
    for this_chunk in chunks:
        task = (
            make_chunk_task("WPT chunk %s / %s" % (this_chunk, total_chunks))
            .with_treeherder(platform, "WPT-%s" % this_chunk)
            .with_repo()
            .with_curl_artifact_script(build_task, "target.tar.gz")
            .with_script("tar -xzf target.tar.gz")
            .with_index_and_artifacts_expire_in(log_artifacts_expire_in)
            .with_max_run_time_minutes(60)
            .with_env(
                TOTAL_CHUNKS=str(total_chunks),
                THIS_CHUNK=str(this_chunk),
                PROCESSES=str(processes),
            )
        )
        if this_chunk == chunks[-1]:
            task.name += " + extra"
            task.extra["treeherder"]["symbol"] += "+"
            task.with_script("""
                ./mach test-wpt-failure
                time ./mach test-wpt --release --binary-arg=--multiprocess \
                    --processes $PROCESSES \
                    --log-raw test-wpt-mp.log \
                    --log-errorsummary wpt-mp-errorsummary.log \
                    eventsource \
                    | cat
                time ./mach test-wpt --release --product=servodriver --headless  \
                    tests/wpt/mozilla/tests/mozilla/DOMParser.html \
                    tests/wpt/mozilla/tests/css/per_glyph_font_fallback_a.html \
                    tests/wpt/mozilla/tests/css/img_simple.html \
                    tests/wpt/mozilla/tests/mozilla/secure.https.html \
                    | cat
            """)
        # `test-wpt` is piped into `cat` so that stdout is not a TTY
        # and wptrunner does not use "interactive mode" formatting:
        # https://github.com/servo/servo/issues/22438
        task.with_script("""
            ./mach test-wpt \
                --release \
                --processes $PROCESSES \
                --total-chunks "$TOTAL_CHUNKS" \
                --this-chunk "$THIS_CHUNK" \
                --log-raw test-wpt.log \
                --log-errorsummary wpt-errorsummary.log \
                --always-succeed \
                | cat
            ./mach filter-intermittents \
                wpt-errorsummary.log \
                --log-intermittents intermittents.log \
                --log-filteredsummary filtered-wpt-errorsummary.log \
                --tracker-api default \
                --reporter-api default
        """)
        task.with_artifacts(*[
            "%s/%s" % (repo_dir, word)
            for script in task.scripts
            for word in script.split()
            if word.endswith(".log")
        ])
        platform_id = platform.replace(" ", "_").lower()
        task.find_or_create("%s_wpt_%s.%s" % (platform_id, this_chunk, CONFIG.git_sha))


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

    # Unlike when reacting to a GitHub push event,
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
    return (
        decisionlib.DockerWorkerTask(name)
        .with_worker_type("servo-docker-worker")
        .with_treeherder_required()
    )


def windows_task(name):
    return (
        decisionlib.WindowsGenericWorkerTask(name)
        .with_worker_type("servo-win2016")
        .with_treeherder_required()
    )



def macos_task(name):
    return (
        decisionlib.MacOsGenericWorkerTask(name)
        .with_provisioner_id("proj-servo")
        .with_worker_type("macos")
        .with_treeherder_required()
    )


def linux_build_task(name, *, build_env=build_env):
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
                "1.14.3/gstreamer-1.0-x86_64-1.14.3.msi",
            sha256="f4f20c713766ed6718b914b9ae57ed993a59ffe194e6ef530c8547508b4484d8",
            path="gst",
        )
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
