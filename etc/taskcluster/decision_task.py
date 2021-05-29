# coding: utf8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os.path
import decisionlib
import functools
from decisionlib import CONFIG, SHARED
from urllib.request import urlopen


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

    if task_for == "github-push":
        all_tests = [
            linux_tidy_unit,
            linux_docs_check,
            windows_unit,
        ]
        by_branch_name = {
            "auto": all_tests,
            "try": all_tests,
            "try-taskcluster": [
                # Add functions here as needed, in your push to that branch
            ],
            "master": [
                layout_2020_regressions_report,
            ],

            # The "try-*" keys match those in `servo_try_choosers` in Homu’s config:
            # https://github.com/servo/saltfs/blob/master/homu/map.jinja

            "try-mac": [],
            "try-linux": [linux_tidy_unit, linux_docs_check],
            "try-windows": [windows_unit],
            "try-arm": [],
            "try-wpt": [],
            "try-wpt-2020": [],
            "try-wpt-mac": [],
            "test-wpt": [],
        }
        by_branch_name["try-windows-rdp"] = [
            functools.partial(f, rdp=True) for f in by_branch_name["try-windows"]
        ]

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
        windows_unit(cached=False, rdp=True)

    # https://tools.taskcluster.net/hooks/project-servo/daily
    elif task_for == "daily":
        daily_tasks_setup()
        update_wpt()


ping_on_daily_task_failure = "SimonSapin, nox, emilio"
build_artifacts_expire_in = "1 week"
build_dependencies_artifacts_expire_in = "1 month"
log_artifacts_expire_in = "1 year"

build_env = {
    "RUSTFLAGS": "-Dwarnings",
    "CARGO_INCREMENTAL": "0",
}
unix_build_env = {
}
linux_build_env = {
    "RUST_BACKTRACE": "1", # https://github.com/servo/servo/issues/26192
    "SHELL": "/bin/dash",  # For SpiderMonkey’s build system
    "CCACHE": "sccache",
    "RUSTC_WRAPPER": "sccache",
    "CC": "clang",
    "CXX": "clang++",
    "SCCACHE_IDLE_TIMEOUT": "1200",
    # https://github.com/servo/servo/issues/24714#issuecomment-552951519
    "SCCACHE_MAX_FRAME_LENGTH": str(100 * 1024 * 1024),  # 100 MiB
}
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
        "CC": "clang-cl.exe",
        "CXX": "clang-cl.exe",
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
            python3 ./mach test-tidy --no-progress --all
            python3 ./mach test-tidy --no-progress --self-test
            python3 ./mach bootstrap-gstreamer
            python3 ./mach build --dev
            python3 ./mach test-unit

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
            python3 ./mach test-tidy --no-progress --all
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
        .find_or_create("linux_unit." + CONFIG.tree_hash())
    )


def linux_docs_check():
    return (
        linux_build_task("Check")
        .with_treeherder("Linux x64", "Check")
        .with_script('RUSTDOCFLAGS="--disable-minification" python3 ./mach doc')
        # Because `rustdoc` needs metadata of dependency crates,
        # `cargo doc` does almost all of the work that `cargo check` does.
        # Therefore, when running them in this order the second command does very little
        # and should finish quickly.
        # The reverse order would not increase the total amount of work to do,
        # but would reduce the amount of parallelism available.
        .with_script("python3 ./mach check")
        .find_or_create("check." + CONFIG.tree_hash())
    )


def layout_2020_regressions_report():
    return (
        linux_task("Layout 2020 regressions report")
        .with_treeherder("Linux x64", "RegressionsReport")
        .with_dockerfile(dockerfile_path("base"))
        .with_repo_bundle()
        .with_script(
            "python3 tests/wpt/reftests-report/gen.py %s %s"
            % (CONFIG.tree_hash(), CONFIG.git_sha)
        )
        .with_index_and_artifacts_expire_in(log_artifacts_expire_in)
        .with_artifacts("/repo/tests/wpt/reftests-report/report.html")
        .with_index_at("layout-2020-regressions-report")
        .create()
    )


def windows_unit(cached=True, rdp=False):
    task = (
        windows_build_task("Dev build + unit tests", rdp=rdp)
        .with_treeherder("Windows x64", "Unit")
        .with_script(
            # Not necessary as this would be done at the start of `build`,
            # but this allows timing it separately.
            "python mach fetch",

            "python mach build --dev",

            "python mach test-unit",
            # Running the TC task with administrator privileges breaks the
            # smoketest for unknown reasons.
            #"python mach smoketest --angle",

            "python mach package --dev",
            "python mach build --dev --libsimpleservo",
            # The GStreamer plugin currently doesn't support Windows
            # https://github.com/servo/servo/issues/25353
            # "mach build --dev -p servo-gst-plugin",

        )
        .with_artifacts("repo/target/debug/msi/Servo.exe",
                        "repo/target/debug/msi/Servo.zip")
    )
    if cached:
        return task.find_or_create("build.windows_x64_dev." + CONFIG.tree_hash())
    else:
        return task.create()


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


def linux_release_build_with_debug_assertions(layout_2020):
    if layout_2020:
        name_prefix = "Layout 2020 "      # pragma: no cover
        build_args = "--with-layout-2020" # pragma: no cover
        index_key_suffix = "_2020"        # pragma: no cover
        treeherder_prefix = "2020-"       # pragma: no cover
    else:
        name_prefix = ""
        build_args = ""
        index_key_suffix = ""
        treeherder_prefix = ""
    return (
        linux_build_task(name_prefix + "Release build, with debug assertions")
        .with_treeherder("Linux x64", treeherder_prefix + "Release+A")
        .with_script("""
            time python3 ./mach rustc -V
            time python3 ./mach fetch
            python3 ./mach build --release --with-debug-assertions %s -p servo
            ./etc/ci/lockfile_changed.sh
            tar -czf /target.tar.gz \
                target/release/servo \
                resources
            sccache --show-stats
        """ % build_args)
        .with_artifacts("/target.tar.gz")
        .find_or_create("build.linux_x64%s_release_w_assertions.%s" % (
            index_key_suffix,
            CONFIG.tree_hash(),
        ))
    )

def daily_tasks_setup():
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
        .with_script("python3 ./mach bootstrap-gstreamer")
    )
    return task


def windows_build_task(name, package=True, arch="x86_64", rdp=False):
    hashes = {
        "devel": {
            "x86_64": "bd444f3ff9d828f93ba5db0ef511d648d238fff50c4435ccefc7b3e9b2bea3b9",
        },
        "non-devel": {
            "x86_64": "f33fff17a558a433b9c4cf7bd9a338a3d0867fa2d5ee1ee33d249b6a55e8a297",
        },
    }
    prefix = {
        "x86_64": "msvc",
    }
    version = "1.16.2"
    task = (
        windows_task(name)
        .with_max_run_time_minutes(90)
        .with_env(
            **build_env,
            **windows_build_env[arch],
            **windows_build_env["all"]
        )
        .with_repo_bundle(sparse_checkout=windows_sparse_checkout)
        .with_python3()
        .with_rustup()
    )
    if arch in hashes["non-devel"] and arch in hashes["devel"]:
        task.with_repacked_msi(
            url=("https://gstreamer.freedesktop.org/data/pkg/windows/" +
                 "%s/gstreamer-1.0-%s-%s-%s.msi" % (version, prefix[arch], arch, version)),
            sha256=hashes["non-devel"][arch],
            path="gst",
        )
        task.with_repacked_msi(
            url=("https://gstreamer.freedesktop.org/data/pkg/windows/" +
                 "%s/gstreamer-1.0-devel-%s-%s-%s.msi" % (version, prefix[arch], arch, version)),
            sha256=hashes["devel"][arch],
            path="gst",
        )
    if package:
        task.with_directory_mount(
            "https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311-binaries.zip",
            sha256="37f0a533b0978a454efb5dc3bd3598becf9660aaf4287e55bf68ca6b527d051d",
            path="wix",
        )
        task.with_path_from_homedir("wix")
    if rdp:
        task.with_rdp_info(artifact_name="project/servo/rdp-info")
    return task


CONFIG.task_name_template = "Servo: %s"
CONFIG.docker_images_expire_in = build_dependencies_artifacts_expire_in
CONFIG.repacked_msi_files_expire_in = build_dependencies_artifacts_expire_in
CONFIG.index_prefix = "project.servo"
CONFIG.default_provisioner_id = "proj-servo"
CONFIG.docker_image_build_worker_type = "docker"

CONFIG.windows_worker_type = "win2016"

if __name__ == "__main__":  # pragma: no cover
    main(task_for=os.environ["TASK_FOR"])
