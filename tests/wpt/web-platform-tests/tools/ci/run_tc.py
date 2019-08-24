#!/usr/bin/env python

"""Wrapper script for running jobs in TaskCluster

This is intended for running test jobs in TaskCluster. The script
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
import json
import os
import re
import subprocess
import sys
try:
    from urllib2 import urlopen
except ImportError:
    # Python 3 case
    from urllib.request import urlopen


root = os.path.abspath(
    os.path.join(os.path.dirname(__file__),
                 os.pardir,
                 os.pardir))


def run(cmd, return_stdout=False, **kwargs):
    print(" ".join(cmd))
    if return_stdout:
        f = subprocess.check_output
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
                   choices=["experimental", "dev", "nightly", "beta", "stable"],
                   help="Chrome browser channel")
    p.add_argument("--xvfb",
                   action="store_true",
                   help="Start xvfb")
    p.add_argument("--checkout",
                   help="Revision to checkout before starting job")
    p.add_argument("job",
                   help="Name of the job associated with the current event")
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
    subprocess.check_call(["sudo", "sh", "-c", "./wpt make-hosts-file >> /etc/hosts"])


def checkout_revision(rev):
    subprocess.check_call(["git", "checkout", "--quiet", rev])


def install_chrome(channel):
    if channel in ("experimental", "dev", "nightly"):
        deb_archive = "google-chrome-unstable_current_amd64.deb"
    elif channel == "beta":
        deb_archive = "google-chrome-beta_current_amd64.deb"
    elif channel == "stable":
        deb_archive = "google-chrome-stable_current_amd64.deb"
    else:
        raise ValueError("Unrecognized release channel: %s" % channel)

    dest = os.path.join("/tmp", deb_archive)
    resp = urlopen("https://dl.google.com/linux/direct/%s" % deb_archive)
    with open(dest, "w") as f:
        f.write(resp.read())

    run(["sudo", "apt-get", "-qqy", "update"])
    run(["sudo", "gdebi", "-qn", "/tmp/%s" % deb_archive])


def start_xvfb():
    start(["sudo", "Xvfb", os.environ["DISPLAY"], "-screen", "0",
           "%sx%sx%s" % (os.environ["SCREEN_WIDTH"],
                         os.environ["SCREEN_HEIGHT"],
                         os.environ["SCREEN_DEPTH"])])
    start(["sudo", "fluxbox", "-display", os.environ["DISPLAY"]])


def get_extra_jobs(event):
    body = None
    jobs = set()
    if "commits" in event:
        body = event["commits"][0]["message"]
    elif "pull_request" in event:
        body = event["pull_request"]["body"]

    if not body:
        return jobs

    regexp = re.compile(r"\s*tc-jobs:(.*)$")

    for line in body.splitlines():
        m = regexp.match(line)
        if m:
            items = m.group(1)
            for item in items.split(","):
                jobs.add(item.strip())
            break
    return jobs


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


def include_job(job):
    # Special case things that unconditionally run on pushes,
    # assuming a higher layer is filtering the required list of branches
    if (os.environ["GITHUB_PULL_REQUEST"] == "false" and
        job == "run-all"):
        return True

    jobs_str = run([os.path.join(root, "wpt"),
                    "test-jobs"], return_stdout=True)
    print(jobs_str)
    return job in set(jobs_str.splitlines())


def setup_environment(args):
    if args.hosts_file:
        make_hosts_file()

    if "chrome" in args.browser:
        assert args.channel is not None
        install_chrome(args.channel)

    if args.xvfb:
        start_xvfb()

    if args.oom_killer:
        start_userspace_oom_killer()

    if args.checkout:
        checkout_revision(args.checkout)


def setup_repository():
    if os.environ.get("GITHUB_PULL_REQUEST", "false") != "false":
        parents = run(["git", "show", "--no-patch", "--format=%P", "task_head"], return_stdout=True).strip().split()
        if len(parents) == 2:
            base_head = parents[0]
            pr_head = parents[1]

            run(["git", "branch", "base_head", base_head])
            run(["git", "branch", "pr_head", pr_head])
        else:
            print("ERROR: Pull request HEAD wasn't a 2-parent merge commit; "
                  "expected to test the merge of PR into the base")
            commit = run(["git", "show", "--no-patch", "--format=%H", "task_head"], return_stdout=True).strip()
            print("HEAD: %s" % commit)
            print("Parents: %s" % ", ".join(parents))
            sys.exit(1)

    branch = os.environ.get("GITHUB_BRANCH")
    if branch:
        # Ensure that the remote base branch exists
        # TODO: move this somewhere earlier in the task
        run(["git", "fetch", "--quiet", "origin", "%s:%s" % (branch, branch)])


def main():
    args = get_parser().parse_args()
    try:
        event = json.loads(os.environ["TASK_EVENT"])
    except KeyError:
        print("WARNING: Missing TASK_EVENT environment variable")
        # For example under local testing
        event = {}

    if event:
        set_variables(event)

    setup_repository()

    extra_jobs = get_extra_jobs(event)

    job = args.job

    print("Job %s" % job)

    run_if = [(lambda: job == "all", "job set to 'all'"),
              (lambda:"all" in extra_jobs, "Manually specified jobs includes 'all'"),
              (lambda:job in extra_jobs, "Manually specified jobs includes '%s'" % job),
              (lambda:include_job(job), "CI required jobs includes '%s'" % job)]

    for fn, msg in run_if:
        if fn():
            print(msg)
            break
    else:
        print("Job not scheduled for this push")
        return

    # Run the job
    setup_environment(args)
    os.chdir(root)
    cmd = [args.script] + args.script_args
    print(" ".join(cmd))
    sys.exit(subprocess.call(cmd))


if __name__ == "__main__":
    main()
