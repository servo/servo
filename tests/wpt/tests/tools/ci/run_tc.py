#!/usr/bin/env python3
# mypy: allow-untyped-defs

"""Wrapper script for running jobs in Taskcluster

This is intended for running test jobs in Taskcluster. The script
takes a two positional arguments which are the name of the test job
and the script to actually run.

The name of the test job is used to determine whether the script should be run
for this push (this is in lieu of having a proper decision task). There are
several ways that the script can be scheduled to run

1. The output of wpt test-jobs includes the job name
2. The job name is included in a job declaration (see below)
3. The string "all" is included in the job declaration
4. The job name is set to "all"

A job declaration is a line appearing in the pull request body (for
pull requests) or first commit message (for pushes) of the form:

tc-jobs: job1,job2,[...]

In addition, there are a number of keyword arguments used to set options for the
environment in which the jobs run. Documentation for these is in the command help.

As well as running the script, the script sets two environment variables;
GITHUB_BRANCH which is the branch that the commits will merge into (if it's a PR)
or the branch that the commits are on (if it's a push), and GITHUB_PULL_REQUEST
which is the string "false" if the event triggering this job wasn't a pull request
or the pull request number if it was. The semantics of these variables are chosen
to match the corresponding TRAVIS_* variables.

Note: for local testing in the Docker image the script ought to still work, but
full functionality requires that the TASK_EVENT environment variable is set to
the serialization of a GitHub event payload.
"""

import argparse
import fnmatch
import json
import os
import subprocess
import sys
import tarfile
import tempfile
import zipfile

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..')))
from tools.wpt.utils import get_download_to_descriptor

root = os.path.abspath(
    os.path.join(os.path.dirname(__file__),
                 os.pardir,
                 os.pardir))


def run(cmd, return_stdout=False, **kwargs):
    print(" ".join(cmd))
    if return_stdout:
        f = subprocess.check_output
        if "encoding" not in kwargs:
            kwargs["encoding"] = "utf-8"
    else:
        f = subprocess.check_call
    return f(cmd, **kwargs)


def start(cmd):
    print(" ".join(cmd))
    subprocess.Popen(cmd)


def get_parser():
    p = argparse.ArgumentParser()
    p.add_argument("--oom-killer",
                   action="store_true",
                   default=False,
                   help="Run userspace OOM killer")
    p.add_argument("--hosts",
                   dest="hosts_file",
                   action="store_true",
                   default=True,
                   help="Setup wpt entries in hosts file")
    p.add_argument("--no-hosts",
                   dest="hosts_file",
                   action="store_false",
                   help="Don't setup wpt entries in hosts file")
    p.add_argument("--browser",
                   action="append",
                   default=[],
                   help="Browsers that will be used in the job")
    p.add_argument("--channel",
                   default=None,
                   choices=["experimental", "canary", "dev", "nightly", "beta", "stable"],
                   help="Chrome browser channel")
    p.add_argument("--xvfb",
                   action="store_true",
                   help="Start xvfb")
    p.add_argument("--install-certificates", action="store_true", default=None,
                   help="Install web-platform.test certificates to UA store")
    p.add_argument("--no-install-certificates", action="store_false", default=None,
                   help="Don't install web-platform.test certificates to UA store")
    p.add_argument("--no-setup-repository", action="store_false", dest="setup_repository",
                   help="Don't run any repository setup steps, instead use the existing worktree. "
                        "This is useful for local testing.")
    p.add_argument("--checkout",
                   help="Revision to checkout before starting job")
    p.add_argument("--ref",
                   help="Git ref for the commit that should be run")
    p.add_argument("--head-rev",
                   help="Commit at the head of the branch when the decision task ran")
    p.add_argument("--merge-rev",
                   help="Provisional merge commit for PR when the decision task ran")
    p.add_argument("script",
                   help="Script to run for the job")
    p.add_argument("script_args",
                   nargs=argparse.REMAINDER,
                   help="Additional arguments to pass to the script")
    return p


def start_userspace_oom_killer():
    # Start userspace OOM killer: https://github.com/rfjakob/earlyoom
    # It will report memory usage every minute and prefer to kill browsers.
    start(["sudo", "earlyoom", "-p", "-r", "60", "--prefer=(chrome|firefox)", "--avoid=python"])


def make_hosts_file():
    run(["sudo", "sh", "-c", "./wpt make-hosts-file >> /etc/hosts"])


def checkout_revision(rev):
    run(["git", "checkout", "--quiet", rev])


def install_certificates():
    run(["sudo", "cp", "tools/certs/cacert.pem",
         "/usr/local/share/ca-certificates/cacert.crt"])
    run(["sudo", "update-ca-certificates"])


def start_dbus():
    # Start system bus
    run(["sudo", "service", "dbus", "start"])
    # Start user bus and set env
    dbus_env = run(["dbus-launch"], return_stdout=True)
    for dbus_env_line in dbus_env.splitlines():
        dbus_env_name, dbus_env_value = dbus_env_line.split("=", 1)
        assert (dbus_env_name.startswith("DBUS_SESSION"))
        os.environ[dbus_env_name] = dbus_env_value
    assert ("DBUS_SESSION_BUS_ADDRESS" in os.environ)


def install_chrome(channel):
    if channel == "canary":
        # Chrome for Testing Canary is installed via --install-browser
        return
    if channel in ("experimental", "dev"):
        deb_archive = "google-chrome-unstable_current_amd64.deb"
    elif channel == "beta":
        deb_archive = "google-chrome-beta_current_amd64.deb"
    elif channel == "stable":
        deb_archive = "google-chrome-stable_current_amd64.deb"
    else:
        raise ValueError("Unrecognized release channel: %s" % channel)

    dest = os.path.join("/tmp", deb_archive)
    deb_url = "https://dl.google.com/linux/direct/%s" % deb_archive
    with open(dest, "wb") as f:
        get_download_to_descriptor(f, deb_url)

    run(["sudo", "apt-get", "-qqy", "update"])
    run(["sudo", "gdebi", "-qn", "/tmp/%s" % deb_archive])


def start_xvfb():
    start(["sudo", "Xvfb", os.environ["DISPLAY"], "-screen", "0",
           "%sx%sx%s" % (os.environ["SCREEN_WIDTH"],
                         os.environ["SCREEN_HEIGHT"],
                         os.environ["SCREEN_DEPTH"])])
    start(["sudo", "fluxbox", "-display", os.environ["DISPLAY"]])


def set_variables(event):
    # Set some variables that we use to get the commits on the current branch
    ref_prefix = "refs/heads/"
    pull_request = "false"
    branch = None
    if "pull_request" in event:
        pull_request = str(event["pull_request"]["number"])
        # Note that this is the branch that a PR will merge to,
        # not the branch name for the PR
        branch = event["pull_request"]["base"]["ref"]
    elif "ref" in event:
        branch = event["ref"]
        if branch.startswith(ref_prefix):
            branch = branch[len(ref_prefix):]

    os.environ["GITHUB_PULL_REQUEST"] = pull_request
    if branch:
        os.environ["GITHUB_BRANCH"] = branch


def task_url(task_id):
    root_url = os.environ['TASKCLUSTER_ROOT_URL']
    if root_url == 'https://taskcluster.net':
        queue_base = "https://queue.taskcluster.net/v1/task"
    else:
        queue_base = root_url + "/api/queue/v1/task"

    return "%s/%s" % (queue_base, task_id)


def download_artifacts(artifacts):
    artifact_list_by_task = {}
    for artifact in artifacts:
        base_url = task_url(artifact["task"])
        if artifact["task"] not in artifact_list_by_task:
            with tempfile.TemporaryFile() as f:
                get_download_to_descriptor(f, base_url + "/artifacts")
                f.seek(0)
                artifacts_data = json.load(f)
            artifact_list_by_task[artifact["task"]] = artifacts_data

        artifacts_data = artifact_list_by_task[artifact["task"]]
        print("DEBUG: Got artifacts %s" % artifacts_data)
        found = False
        for candidate in artifacts_data["artifacts"]:
            print("DEBUG: candidate: %s glob: %s" % (candidate["name"], artifact["glob"]))
            if fnmatch.fnmatch(candidate["name"], artifact["glob"]):
                found = True
                print("INFO: Fetching aritfact %s from task %s" % (candidate["name"], artifact["task"]))
                file_name = candidate["name"].rsplit("/", 1)[1]
                url = base_url + "/artifacts/" + candidate["name"]
                dest_path = os.path.expanduser(os.path.join("~", artifact["dest"], file_name))
                dest_dir = os.path.dirname(dest_path)
                if not os.path.exists(dest_dir):
                    os.makedirs(dest_dir)
                with open(dest_path, "wb") as f:
                    get_download_to_descriptor(f, url)

                if artifact.get("extract"):
                    unpack(dest_path)
        if not found:
            print("WARNING: No artifact found matching %s in task %s" % (artifact["glob"], artifact["task"]))


def unpack(path):
    dest = os.path.dirname(path)
    if tarfile.is_tarfile(path):
        run(["tar", "-xf", path], cwd=os.path.dirname(path))
    elif zipfile.is_zipfile(path):
        with zipfile.ZipFile(path) as archive:
            archive.extractall(dest)
    else:
        print("ERROR: Don't know how to extract %s" % path)
        raise Exception


def setup_environment(args):
    if "TASK_ARTIFACTS" in os.environ:
        artifacts = json.loads(os.environ["TASK_ARTIFACTS"])
        download_artifacts(artifacts)

    if args.hosts_file:
        make_hosts_file()

    if args.install_certificates:
        install_certificates()

    if "chrome" in args.browser:
        assert args.channel is not None
        install_chrome(args.channel)

    # These browsers use dbus for various features.
    if any(b in args.browser for b in ["chrome", "webkitgtk_minibrowser"]):
        start_dbus()

    if args.xvfb:
        start_xvfb()

    if args.oom_killer:
        start_userspace_oom_killer()


def setup_repository(args):
    is_pr = os.environ.get("GITHUB_PULL_REQUEST", "false") != "false"

    # Initially task_head points at the same commit as the ref we want to test.
    # However that may not be the same commit as we actually want to test if
    # the branch changed since the decision task ran. The branch may have
    # changed because someone has pushed more commits (either to the PR
    # or later commits to the branch), or because someone has pushed to the
    # base branch for the PR.
    #
    # In that case we take a different approach depending on whether this is a
    # PR or a push to a branch.
    # If this is a push to a branch, and the original commit is still fetchable,
    # we try to fetch that (it may not be in the case of e.g. a force push).
    # If it's not fetchable then we fail the run.
    # For a PR we are testing the provisional merge commit. If that's changed it
    # could be that the PR branch was updated or the base branch was updated. In the
    # former case we fail the run because testing an old commit is a waste of
    # resources. In the latter case we assume it's OK to use the current merge
    # instead of the one at the time the decision task ran.

    if args.ref:
        if is_pr:
            assert args.ref.endswith("/merge")
            expected_head = args.merge_rev
        else:
            expected_head = args.head_rev

        task_head = run(["git", "rev-parse", "task_head"], return_stdout=True).strip()

        if task_head != expected_head:
            if not is_pr:
                try:
                    run(["git", "fetch", "origin", expected_head])
                    run(["git", "reset", "--hard", expected_head])
                except subprocess.CalledProcessError:
                    print("CRITICAL: task_head points at %s, expected %s and "
                          "unable to fetch expected commit.\n"
                          "This may be because the branch was updated" % (task_head, expected_head))
                    sys.exit(1)
            else:
                # Convert the refs/pulls/<id>/merge to refs/pulls/<id>/head
                head_ref = args.ref.rsplit("/", 1)[0] + "/head"
                try:
                    remote_head = run(["git", "ls-remote", "origin", head_ref],
                                      return_stdout=True).split("\t")[0]
                except subprocess.CalledProcessError:
                    print("CRITICAL: Failed to read remote ref %s" % head_ref)
                    sys.exit(1)
                if remote_head != args.head_rev:
                    print("CRITICAL: task_head points at %s, expected %s. "
                          "This may be because the branch was updated" % (task_head, expected_head))
                    sys.exit(1)
                print("INFO: Merge commit changed from %s to %s due to base branch changes. "
                      "Running task anyway." % (expected_head, task_head))

    if os.environ.get("GITHUB_PULL_REQUEST", "false") != "false":
        parents = run(["git", "rev-parse", "task_head^@"],
                      return_stdout=True).strip().split()
        if len(parents) == 2:
            base_head = parents[0]
            pr_head = parents[1]

            run(["git", "branch", "base_head", base_head])
            run(["git", "branch", "pr_head", pr_head])
        else:
            print("ERROR: Pull request HEAD wasn't a 2-parent merge commit; "
                  "expected to test the merge of PR into the base")
            commit = run(["git", "rev-parse", "task_head"],
                         return_stdout=True).strip()
            print("HEAD: %s" % commit)
            print("Parents: %s" % ", ".join(parents))
            sys.exit(1)

    branch = os.environ.get("GITHUB_BRANCH")
    if branch:
        # Ensure that the remote base branch exists
        # TODO: move this somewhere earlier in the task
        run(["git", "fetch", "--quiet", "origin", "%s:%s" % (branch, branch)])

    checkout_rev = args.checkout if args.checkout is not None else "task_head"
    checkout_revision(checkout_rev)

    refs = run(["git", "for-each-ref", "refs/heads"], return_stdout=True)
    print("INFO: git refs:\n%s" % refs)
    print("INFO: checked out commit:\n%s" % run(["git", "rev-parse", "HEAD"],
                                                return_stdout=True))


def fetch_event_data():
    try:
        task_id = os.environ["TASK_ID"]
    except KeyError:
        print("WARNING: Missing TASK_ID environment variable")
        # For example under local testing
        return None

    with tempfile.TemporaryFile() as f:
        get_download_to_descriptor(f, task_url(task_id))
        f.seek(0)
        task_data = json.load(f)
    event_data = task_data.get("extra", {}).get("github_event")
    if event_data is not None:
        return json.loads(event_data)


def include_job(job):
    # Only for supporting pre decision-task PRs
    # Special case things that unconditionally run on pushes,
    # assuming a higher layer is filtering the required list of branches
    if "GITHUB_PULL_REQUEST" not in os.environ:
        return True

    if (os.environ["GITHUB_PULL_REQUEST"] == "false" and
        job == "run-all"):
        return True

    jobs_str = run([os.path.join(root, "wpt"),
                    "test-jobs"], return_stdout=True)
    print(jobs_str)
    return job in set(jobs_str.splitlines())


def main():
    args = get_parser().parse_args()

    if "TASK_EVENT" in os.environ:
        event = json.loads(os.environ["TASK_EVENT"])
    else:
        event = fetch_event_data()

    if event:
        set_variables(event)

    if args.setup_repository:
        setup_repository(args)

    # Hack for backwards compatibility
    if args.script in ["run-all", "lint", "update_built", "tools_unittest",
                       "wpt_integration", "resources_unittest",
                       "wptrunner_infrastructure", "stability", "affected_tests"]:
        job = args.script
        if not include_job(job):
            return
        args.script = args.script_args[0]
        args.script_args = args.script_args[1:]

    # Run the job
    setup_environment(args)
    os.chdir(root)
    cmd = [args.script] + args.script_args
    print(" ".join(cmd))
    sys.exit(subprocess.call(cmd))


if __name__ == "__main__":
    main()  # type: ignore
