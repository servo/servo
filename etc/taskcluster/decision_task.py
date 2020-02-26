# coding: utf8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os.path
import decisionlib
from decisionlib import CONFIG, SHARED


def main(task_for):
    with decisionlib.make_repo_bundle():
        tasks(task_for)


def tasks(task_for):
    if CONFIG.git_ref.startswith("refs/heads/"):
        branch = CONFIG.git_ref[len("refs/heads/"):]
        CONFIG.treeherder_repository_name = "servo-" + (
            branch if not branch.startswith("try-") else "try"
        )

    # Work around a tc-github bug/limitation:
    # https://bugzilla.mozilla.org/show_bug.cgi?id=1548781#c4
    if task_for.startswith("github"):
        # https://github.com/taskcluster/taskcluster/blob/21f257dc8/services/github/config.yml#L14
        CONFIG.routes_for_all_subtasks.append("statuses")

    # The magicleap build is broken until there's a surfman back end
    magicleap_dev = lambda: None
    magicleap_nightly = lambda: None

    if task_for == "github-push":
        all_tests = [
            linux_tidy_unit,
            linux_docs_check,
            windows_unit,
            windows_arm64,
            windows_uwp_x64,
            macos_unit,
            magicleap_dev,
            linux_wpt,
            linux_wpt_layout_2020,
            linux_release,
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
            "try-linux": [linux_tidy_unit, linux_docs_check, linux_release],
            "try-windows": [windows_unit, windows_arm64, windows_uwp_x64],
            "try-magicleap": [magicleap_dev],
            "try-arm": [windows_arm64],
            "try-wpt": [linux_wpt],
            "try-wpt-2020": [linux_wpt_layout_2020],
            "try-wpt-mac": [macos_wpt],
        }
        for function in by_branch_name.get(branch, []):
            function()

    elif task_for == "github-pull-request":
        CONFIG.treeherder_repository_name = "servo-prs"
        CONFIG.index_read_only = True
        CONFIG.docker_image_build_worker_type = None

        # We want the merge commit that GitHub creates for the PR.
        # The event does contain a `pull_request.merge_commit_sha` key, but it is wrong:
        # https://github.com/servo/servo/pull/22597#issuecomment-451518810
        CONFIG.git_sha_is_current_head()

        linux_tidy_unit_untrusted()

    elif task_for == "try-windows-ami":
        CONFIG.git_sha_is_current_head()
        CONFIG.windows_worker_type = os.environ["NEW_AMI_WORKER_TYPE"]
        windows_unit(cached=False)

    # https://tools.taskcluster.net/hooks/project-servo/daily
    elif task_for == "daily":
        daily_tasks_setup()
        with_rust_nightly()
        linux_nightly()
        windows_nightly()
        macos_nightly()
        update_wpt()
        magicleap_nightly()
        uwp_nightly()


# These are disabled in a "real" decision task,
# but should still run when testing this Python code. (See `mock.py`.)
def mocked_only():
    windows_release()
    magicleap_dev()
    magicleap_nightly()
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
}
linux_build_env = {
    "SHELL": "/bin/dash",  # For SpiderMonkey’s build system
    "CCACHE": "sccache",
    "RUSTC_WRAPPER": "sccache",
    "CC": "clang",
    "CXX": "clang++",
    "SCCACHE_IDLE_TIMEOUT": "1200",
    # https://github.com/servo/servo/issues/24714#issuecomment-552951519
    "SCCACHE_MAX_FRAME_LENGTH": str(100 * 1024 * 1024),  # 100 MiB
}
macos_build_env = {}
windows_build_env = {
    "x86_64": {
        "GSTREAMER_1_0_ROOT_X86_64": "%HOMEDRIVE%%HOMEPATH%\\gst\\gstreamer\\1.0\\x86_64\\",
    },
    "arm64": {
        "PKG_CONFIG_ALLOW_CROSS": "1",
    },
    "all": {
        "PYTHON3": "%HOMEDRIVE%%HOMEPATH%\\python3\\python.exe",
        "LINKER": "lld-link.exe",
        "MOZTOOLS_PATH_PREPEND": "%HOMEDRIVE%%HOMEPATH%\\git\\cmd",
    },
}

windows_sparse_checkout = [
    "/*",
    "!/tests/wpt/metadata",
    "!/tests/wpt/mozilla",
    "!/tests/wpt/webgl",
    "!/tests/wpt/web-platform-tests",
    "/tests/wpt/web-platform-tests/tools",
]


def linux_tidy_unit_untrusted():
    return (
        decisionlib.DockerWorkerTask("Tidy + dev build + unit tests")
        .with_worker_type("docker-untrusted")
        .with_treeherder("Linux x64", "Tidy+Unit")
        .with_max_run_time_minutes(60)
        .with_dockerfile(dockerfile_path("build"))
        .with_env(**build_env, **unix_build_env, **linux_build_env)
        .with_repo_bundle()
        .with_script("""
            ./mach test-tidy --no-progress --all
            ./mach test-tidy --no-progress --self-test
            ./mach bootstrap-gstreamer
            ./mach build --dev
            ./mach test-unit

            ./etc/ci/lockfile_changed.sh
            ./etc/memory_reports_over_time.py --test
            ./etc/ci/check_no_panic.sh
        """)
        .create()
    )


def linux_tidy_unit():
    return (
        linux_build_task("Tidy + dev build + unit tests")
        .with_treeherder("Linux x64", "Tidy+Unit")
        .with_max_run_time_minutes(75)
        .with_script("""
            ./mach test-tidy --no-progress --all
            python3 ./mach test-tidy --no-progress --all --no-wpt
            python3 ./mach build --dev
            python3 ./mach test-unit
            python3 ./mach package --dev
            python3 ./mach build --dev --features refcell_backtrace
            python3 ./mach build --dev --features layout-2020
            python3 ./mach build --dev --libsimpleservo
            python3 ./mach build --dev -p servo-gst-plugin
            python3 ./mach build --dev --media-stack=dummy
            python3 ./mach test-tidy --no-progress --self-test

            ./etc/memory_reports_over_time.py --test
            ./etc/taskcluster/mock.py
            ./etc/ci/lockfile_changed.sh
            ./etc/ci/check_no_panic.sh
        """)
        .find_or_create("linux_unit." + CONFIG.task_id())
    )


def linux_docs_check():
    return (
        linux_build_task("Docs + check")
        .with_treeherder("Linux x64", "Doc+Check")
        .with_script("""
            RUSTDOCFLAGS="--disable-minification" ./mach doc
            (
                cd target/doc
                git init
                git add .
                git -c user.name="Taskcluster" -c user.email="" \
                    commit -q -m "Rebuild Servo documentation"
                git bundle create docs.bundle HEAD
            )

        """
        # Because `rustdoc` needs metadata of dependency crates,
        # `cargo doc` does almost all of the work that `cargo check` does.
        # Therefore, when running them in this order the second command does very little
        # and should finish quickly.
        # The reverse order would not increase the total amount of work to do,
        # but would reduce the amount of parallelism available.
        """
            ./mach check
        """)
        .with_artifacts("/repo/target/doc/docs.bundle")
        .find_or_create("docs." + CONFIG.task_id())
    )


def upload_docs():
    docs_build_task_id = decisionlib.Task.find("docs." + CONFIG.task_id())
    return (
        linux_task("Upload docs to GitHub Pages")
        .with_treeherder("Linux x64", "DocUpload")
        .with_dockerfile(dockerfile_path("base"))
        .with_curl_artifact_script(docs_build_task_id, "docs.bundle")
        .with_features("taskclusterProxy")
        .with_scopes("secrets:get:project/servo/doc.servo.org")
        .with_env(PY="""if 1:
            import urllib, json, os
            root_url = os.environ["TASKCLUSTER_PROXY_URL"]
            url = root_url + "/api/secrets/v1/secret/project/servo/doc.servo.org"
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
            ./mach build --dev --verbose
            ./mach test-unit
            ./mach package --dev
            ./etc/ci/lockfile_changed.sh
        """)
        .find_or_create("macos_unit." + CONFIG.task_id())
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


def appx_artifact(debug):
    return '/'.join([
        'repo',
        'support',
        'hololens',
        'AppPackages',
        'ServoApp',
        'ServoApp_1.0.0.0_%sTest.zip' % ('Debug_' if debug else ''),
    ])


def windows_arm64():
    return (
        windows_build_task("UWP dev build", arch="arm64", package=False)
        .with_treeherder("Windows arm64", "UWP-Dev")
        .with_script(
            "python mach build --dev --target=aarch64-uwp-windows-msvc",
            "python mach package --dev --target aarch64-uwp-windows-msvc --uwp=arm64",
        )
        .with_artifacts(appx_artifact(debug=True))
        .find_or_create("build.windows_uwp_arm64_dev." + CONFIG.task_id())
    )


def windows_uwp_x64():
    return (
        windows_build_task("UWP dev build", package=False)
        .with_treeherder("Windows x64", "UWP-Dev")
        .with_script(
            "python mach build --dev --target=x86_64-uwp-windows-msvc",
            "python mach package --dev --target=x86_64-uwp-windows-msvc --uwp=x64",
            "python mach test-tidy --force-cpp --no-wpt",
        )
        .with_artifacts(appx_artifact(debug=True))
        .find_or_create("build.windows_uwp_x64_dev." + CONFIG.task_id())
    )


def uwp_nightly():
    return (
        windows_build_task("Nightly UWP build and upload", package=False)
        .with_treeherder("Windows x64", "UWP-Nightly")
        .with_features("taskclusterProxy")
        .with_scopes("secrets:get:project/servo/s3-upload-credentials")
        .with_script(
            "python mach build --release --target=x86_64-uwp-windows-msvc",
            "python mach build --release --target=aarch64-uwp-windows-msvc",
            "mach package --release --target=x86_64-uwp-windows-msvc --uwp=x64 --uwp=arm64",
            "mach upload-nightly uwp --secret-from-taskcluster",
        )
        .with_artifacts(appx_artifact(debug=False))
        .with_max_run_time_minutes(3 * 60)
        .find_or_create("build.windows_uwp_nightlies." + CONFIG.task_id())
    )


def windows_unit(cached=True):
    task = (
        windows_build_task("Dev build + unit tests")
        .with_treeherder("Windows x64", "Unit")
        .with_script(
            # Not necessary as this would be done at the start of `build`,
            # but this allows timing it separately.
            "mach fetch",

            "mach build --dev",
            "mach test-unit",
            "mach smoketest --angle",
            "mach package --dev",
            "mach build --dev --libsimpleservo",
            # The GStreamer plugin currently doesn't support Windows
            # https://github.com/servo/servo/issues/25353
            # "mach build --dev -p servo-gst-plugin",

        )
        .with_artifacts("repo/target/debug/msi/Servo.exe",
                        "repo/target/debug/msi/Servo.zip")
    )
    if cached:
        return task.find_or_create("build.windows_x64_dev." + CONFIG.task_id())
    else:
        return task.create()


def windows_release():
    return (
        windows_build_task("Release build")
        .with_treeherder("Windows x64", "Release")
        .with_script("mach build --release",
                     "mach package --release")
        .with_artifacts("repo/target/release/msi/Servo.exe",
                        "repo/target/release/msi/Servo.zip")
        .find_or_create("build.windows_x64_release." + CONFIG.task_id())
    )


def windows_nightly():
    return (
        windows_build_task("Nightly build and upload")
        .with_treeherder("Windows x64", "Nightly")
        .with_features("taskclusterProxy")
        .with_scopes("secrets:get:project/servo/s3-upload-credentials")
        .with_script("mach fetch",
                     "mach build --release",
                     "mach package --release",
                     "mach upload-nightly windows-msvc --secret-from-taskcluster")
        .with_artifacts("repo/target/release/msi/Servo.exe",
                        "repo/target/release/msi/Servo.zip")
        .find_or_create("build.windows_x64_nightly." + CONFIG.task_id())
    )


def linux_nightly():
    return (
        linux_build_task("Nightly build and upload")
        .with_treeherder("Linux x64", "Nightly")
        .with_features("taskclusterProxy")
        .with_scopes("secrets:get:project/servo/s3-upload-credentials")
        # Not reusing the build made for WPT because it has debug assertions
        .with_script(
            "./mach build --release",
            "./mach package --release",
            "./mach upload-nightly linux --secret-from-taskcluster",
        )
        .with_artifacts("/repo/target/release/servo-tech-demo.tar.gz")
        .find_or_create("build.linux_x64_nightly" + CONFIG.task_id())
    )


def linux_release():
    return (
        linux_build_task("Release build")
        .with_treeherder("Linux x64", "Release")
        .with_script(
            "./mach build --release",
            "./mach package --release",
        )
        .find_or_create("build.linux_x64_release" + CONFIG.task_id())
    )


def macos_nightly():
    return (
        macos_build_task("Nightly build and upload")
        .with_treeherder("macOS x64", "Nightly")
        .with_features("taskclusterProxy")
        .with_scopes(
            "secrets:get:project/servo/s3-upload-credentials",
            "secrets:get:project/servo/github-homebrew-token",
        )
        .with_script(
            "./mach build --release",
            "./mach package --release",
            "./mach upload-nightly mac --secret-from-taskcluster",
        )
        .with_artifacts("repo/target/release/servo-tech-demo.dmg")
        .find_or_create("build.mac_x64_nightly." + CONFIG.task_id())
    )


def update_wpt():
    build_task = linux_release_build_with_debug_assertions(layout_2020=False)
    return (
        linux_task("WPT update")
        .with_treeherder("Linux x64", "WPT-update")
        .with_dockerfile(dockerfile_path("wpt-update"))
        .with_features("taskclusterProxy")
        .with_scopes("secrets:get:project/servo/wpt-sync")
        .with_index_and_artifacts_expire_in(log_artifacts_expire_in)
        .with_max_run_time_minutes(8 * 60)
        # Not using the bundle, pushing the new changes to the git remote requires a full repo.
        .with_repo()
        .with_curl_artifact_script(build_task, "target.tar.gz")
        .with_script("""
            tar -xzf target.tar.gz
            # Use `cat` to force wptrunner’s non-interactive mode
            ./etc/ci/update-wpt-checkout fetch-and-update-expectations | cat
            ./etc/ci/update-wpt-checkout open-pr
            ./etc/ci/update-wpt-checkout cleanup
        """)
        .create()
    )


def macos_release_build_with_debug_assertions(priority=None):
    return (
        macos_build_task("Release build, with debug assertions")
        .with_treeherder("macOS x64", "Release+A")
        .with_priority(priority)
        .with_script("\n".join([
            "./mach build --release --verbose --with-debug-assertions",
            "./etc/ci/lockfile_changed.sh",
            "tar -czf target.tar.gz" +
            " target/release/servo" +
            " target/release/build/osmesa-src-*/output" +
            " target/release/build/osmesa-src-*/out/src/gallium/targets/osmesa/.libs" +
            " target/release/build/osmesa-src-*/out/src/mapi/shared-glapi/.libs",
        ]))
        .with_artifacts("repo/target.tar.gz")
        .find_or_create("build.macos_x64_release_w_assertions." + CONFIG.task_id())
    )


def linux_release_build_with_debug_assertions(layout_2020):
    if layout_2020:
        name_prefix = "Layout 2020 "
        build_args = "--with-layout-2020"
        index_key_suffix = "_2020"
        treeherder_prefix = "2020-"
    else:
        name_prefix = ""
        build_args = ""
        index_key_suffix = ""
        treeherder_prefix = ""
    return (
        linux_build_task(name_prefix + "Release build, with debug assertions")
        .with_treeherder("Linux x64", treeherder_prefix + "Release+A")
        .with_script("""
            time ./mach rustc -V
            time ./mach fetch
            ./mach build --release --with-debug-assertions %s -p servo
            ./etc/ci/lockfile_changed.sh
            tar -czf /target.tar.gz \
                target/release/servo \
                target/release/build/osmesa-src-*/output \
                target/release/build/osmesa-src-*/out/lib/gallium
            sccache --show-stats
        """ % build_args)
        .with_artifacts("/target.tar.gz")
        .find_or_create("build.linux_x64%s_release_w_assertions.%s" % (
            index_key_suffix,
            CONFIG.task_id(),
        ))
    )


def macos_wpt():
    priority = "high" if CONFIG.git_ref == "refs/heads/auto" else None
    build_task = macos_release_build_with_debug_assertions(priority=priority)
    def macos_run_task(name):
        task = macos_task(name).with_python2().with_python3() \
            .with_repo_bundle(alternate_object_dir="/var/cache/servo.git/objects")
        return with_homebrew(task, ["etc/taskcluster/macos/Brewfile"])
    wpt_chunks(
        "macOS x64",
        macos_run_task,
        build_task,
        repo_dir="repo",
        total_chunks=20,
        processes=8,
    )


def linux_wpt():
    linux_wpt_common(total_chunks=4, layout_2020=False)


def linux_wpt_layout_2020():
    linux_wpt_common(total_chunks=2, layout_2020=True)


def linux_wpt_common(total_chunks, layout_2020):
    release_build_task = linux_release_build_with_debug_assertions(layout_2020)
    def linux_run_task(name):
        return linux_task(name).with_dockerfile(dockerfile_path("run")).with_repo_bundle()
    wpt_chunks("Linux x64", linux_run_task, release_build_task, repo_dir="/repo",
               processes=20, total_chunks=total_chunks, layout_2020=layout_2020)


def wpt_chunks(platform, make_chunk_task, build_task, total_chunks, processes,
               repo_dir, chunks="all", layout_2020=False):
    if layout_2020:
        start = 1  # Skip the "extra" WPT testing, a.k.a. chunk 0
        name_prefix = "Layout 2020 "
        job_id_prefix = "2020-"
        args = "--layout-2020"
    else:
        start = 0
        name_prefix = ""
        job_id_prefix = ""
        args = ""

    if chunks == "all":
        chunks = range(start, total_chunks + 1)
    for this_chunk in chunks:
        task = (
            make_chunk_task("{}WPT chunk {:0{width}} / {}".format(
                name_prefix,
                this_chunk,
                total_chunks,
                width=len(str(total_chunks)),
            ))
            .with_treeherder(
                platform,
                "WPT-%s" % this_chunk,
                group_symbol=job_id_prefix + "WPT",
                group_name=name_prefix + "web-platform-tests"
            )
            .with_curl_artifact_script(build_task, "target.tar.gz")
            .with_script("tar -xzf target.tar.gz")
            .with_index_and_artifacts_expire_in(log_artifacts_expire_in)
            .with_max_run_time_minutes(90)
            .with_env(
                TOTAL_CHUNKS=str(total_chunks),
                THIS_CHUNK=str(this_chunk),
                PROCESSES=str(processes),
                WPT_ARGS=args,
                GST_DEBUG="3",
            )
        )
        # `test-wpt` is piped into `cat` so that stdout is not a TTY
        # and wptrunner does not use "interactive mode" formatting:
        # https://github.com/servo/servo/issues/22438
        if this_chunk == 0:
            task.with_script("""
                ./mach test-wpt-failure
                time python2 ./mach test-wpt --release --binary-arg=--multiprocess \
                    --processes $PROCESSES \
                    --log-raw test-wpt-mp.log \
                    --log-errorsummary wpt-mp-errorsummary.log \
                    eventsource \
                    | cat
                time env PYTHONIOENCODING=utf-8 python3 ./mach test-wpt --release --binary-arg=--multiprocess \
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
                time ./mach test-wpt --release --processes $PROCESSES --product=servodriver \
                    --headless --log-raw test-bluetooth.log \
                    --log-errorsummary bluetooth-errorsummary.log \
                    bluetooth \
                    | cat
                time ./mach test-wpt --release --processes $PROCESSES --timeout-multiplier=4 \
                    --headless --log-raw test-wdspec.log \
                    --log-servojson wdspec-jsonsummary.log \
                    --always-succeed \
                    webdriver \
                    | cat
                ./mach filter-intermittents \
                    wdspec-jsonsummary.log \
                    --log-intermittents intermittents.log \
                    --log-filteredsummary filtered-wdspec-errorsummary.log \
                    --tracker-api default \
                    --reporter-api default
            """)
        else:
            task.with_script("""
                ./mach test-wpt \
                    --release \
                    $WPT_ARGS \
                    --processes $PROCESSES \
                    --total-chunks "$TOTAL_CHUNKS" \
                    --this-chunk "$THIS_CHUNK" \
                    --log-raw test-wpt.log \
                    --log-servojson wpt-jsonsummary.log \
                    --always-succeed \
                    | cat
                ./mach filter-intermittents \
                    wpt-jsonsummary.log \
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
        task.find_or_create("%s_%swpt_%s.%s" % (
            platform.replace(" ", "_").lower(),
            job_id_prefix.replace("-", "_"),
            this_chunk,
            CONFIG.task_id(),
        ))


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
        .with_worker_type("docker")
        .with_treeherder_required()
    )


def windows_task(name):
    return (
        decisionlib.WindowsGenericWorkerTask(name)
        .with_worker_type(CONFIG.windows_worker_type)
        .with_treeherder_required()
    )


def macos_task(name):
    return (
        decisionlib.MacOsGenericWorkerTask(name)
        .with_provisioner_id("proj-servo")
        .with_worker_type(CONFIG.macos_worker_type)
        .with_treeherder_required()
    )


def linux_build_task(name, *, build_env=build_env):
    task = (
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
        .with_repo_bundle()
        .with_script("./mach bootstrap-gstreamer")
    )
    return task


def windows_build_task(name, package=True, arch="x86_64"):
    hashes = {
        "devel": {
            "x86_64": "c136cbfb0330041d52fe6ec4e3e468563176333c857f6ed71191ebc37fc9d605",
        },
        "non-devel": {
            "x86_64": "0744a8ef2a4ba393dacb7927d741df871400a85bab5aecf7905f63bf52c405e4",
        },
    }
    prefix = {
        "x86_64": "msvc",
    }
    version = "1.16.0"
    task = (
        windows_task(name)
        .with_max_run_time_minutes(90)
        .with_env(
            **build_env,
            **windows_build_env[arch],
            **windows_build_env["all"]
        )
        .with_repo_bundle(sparse_checkout=windows_sparse_checkout)
        .with_python2()
        .with_directory_mount(
            "https://www.python.org/ftp/python/3.7.3/python-3.7.3-embed-amd64.zip",
            sha256="6de14c9223226cf0cd8c965ecb08c51d62c770171a256991b4fddc25188cfa8e",
            path="python3",
        )
        .with_rustup()
    )
    if arch in hashes["non-devel"] and arch in hashes["devel"]:
        task = (
            task.with_repacked_msi(
                url=("https://gstreamer.freedesktop.org/data/pkg/windows/" +
                     "%s/gstreamer-1.0-%s-%s-%s.msi" % (version, prefix[arch], arch, version)),
                sha256=hashes["non-devel"][arch],
                path="gst",
            )
            .with_repacked_msi(
                url=("https://gstreamer.freedesktop.org/data/pkg/windows/" +
                     "%s/gstreamer-1.0-devel-%s-%s-%s.msi" % (version, prefix[arch], arch, version)),
                sha256=hashes["devel"][arch],
                path="gst",
            )
        )
    if package:
        task = (
            task
            .with_directory_mount(
                "https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311-binaries.zip",
                sha256="37f0a533b0978a454efb5dc3bd3598becf9660aaf4287e55bf68ca6b527d051d",
                path="wix",
            )
            .with_path_from_homedir("wix")
        )
    return task


def with_homebrew(task, brewfiles):
    for brewfile in brewfiles:
        task.with_script("time brew bundle install --verbose --no-upgrade --file=" + brewfile)
    return task


def macos_build_task(name):
    build_task = (
        macos_task(name)
        # Stray processes eating CPU can slow things down:
        # https://github.com/servo/servo/issues/24735
        .with_max_run_time_minutes(60 * 2)
        .with_env(**build_env, **unix_build_env, **macos_build_env)
        .with_repo_bundle(alternate_object_dir="/var/cache/servo.git/objects")
        .with_python2()
        .with_rustup()
        .with_index_and_artifacts_expire_in(build_artifacts_expire_in)
        # Debugging for surprising generic-worker behaviour
        .with_early_script("ls")
        .with_script("ls target || true")
    )
    return (
        with_homebrew(build_task, [
            "etc/taskcluster/macos/Brewfile",
            "etc/taskcluster/macos/Brewfile-build",
        ])
        .with_script("""
            export OPENSSL_INCLUDE_DIR="$(brew --prefix openssl)/include"
            export OPENSSL_LIB_DIR="$(brew --prefix openssl)/lib"
            export PKG_CONFIG_PATH="$(brew --prefix libffi)/lib/pkgconfig/"
            export PKG_CONFIG_PATH="$(brew --prefix zlib)/lib/pkgconfig/:$PKG_CONFIG_PATH"
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


def magicleap_build_task(name, build_type):
    return (
        macos_build_task(name)
        .with_treeherder("MagicLeap aarch64", build_type)
        .with_directory_mount(
            "https://servo-deps.s3.amazonaws.com/magicleap/macos-sdk-v0.20.0%2Bndk19c.tar.gz",
            sha256="d5890cc7612694d79e60247a5d5fe4d8bdeb797095f098d56f3069be33426781",
            path="magicleap"
        )
        .with_directory_mount(
            "https://servo-deps.s3.amazonaws.com/magicleap/ServoCICert-expires-2020-08-25.zip",
            sha256="33f9d07b89c206e671f6a5020e52265b131e83aede8fa474be323a8e3345d760",
            path="magicleap"
        )
        # Early script in order to run with the initial $PWD
        .with_early_script("""
            export MAGICLEAP_SDK="$PWD/magicleap/v0.20.0+ndk19c"
            export MLCERT="$PWD/magicleap/servocimlcert.cert"
        """)
        .with_script("""
            unset OPENSSL_INCLUDE_DIR
            unset OPENSSL_LIB_DIR
            export HOST_CC=$(brew --prefix llvm)/bin/clang
            export HOST_CXX=$(brew --prefix llvm)/bin/clang++
        """)
    )


def magicleap_dev():
    return (
        magicleap_build_task("Dev build", "Dev")
        .with_script("""
            ./mach build --magicleap --dev
            env -u DYLD_LIBRARY_PATH ./mach package --magicleap --dev
        """)
        .find_or_create("build.magicleap_dev." + CONFIG.task_id())
    )


def magicleap_nightly():
    return (
        magicleap_build_task("Nightly build and upload", "Release")
        .with_features("taskclusterProxy")
        .with_scopes("secrets:get:project/servo/s3-upload-credentials")
        .with_script("""
            ./mach build --magicleap --release
            env -u DYLD_LIBRARY_PATH ./mach package --magicleap --release
            ./mach upload-nightly magicleap --secret-from-taskcluster
        """)
        .with_artifacts("repo/target/magicleap/aarch64-linux-android/release/Servo.mpk")
        .find_or_create("build.magicleap_nightly." + CONFIG.task_id())
    )


CONFIG.task_name_template = "Servo: %s"
CONFIG.docker_images_expire_in = build_dependencies_artifacts_expire_in
CONFIG.repacked_msi_files_expire_in = build_dependencies_artifacts_expire_in
CONFIG.index_prefix = "project.servo"
CONFIG.default_provisioner_id = "proj-servo"
CONFIG.docker_image_build_worker_type = "docker"

CONFIG.windows_worker_type = "win2016"
CONFIG.macos_worker_type = "macos"

if __name__ == "__main__":  # pragma: no cover
    main(task_for=os.environ["TASK_FOR"])
