# mypy: allow-untyped-defs

import argparse
import logging
import os
import re
import subprocess
import sys

here = os.path.abspath(os.path.dirname(__file__))
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

logger = logging.getLogger()


def build(tag="wpt:local", *args, **kwargs):
    subprocess.check_call(["docker",
                           "build",
                           "--pull",
                           "--tag", tag,
                           here])


def parser_push():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", action="store",
                        help="Tag to use (default is taken from .taskcluster.yml)")
    parser.add_argument("--force", action="store_true",
                        help="Ignore warnings and push anyway")
    return parser


def walk_yaml(root, target):
    rv = []
    if isinstance(root, list):
        for value in root:
            if isinstance(value, (dict, list)):
                rv.extend(walk_yaml(value, target))
    elif isinstance(root, dict):
        for key, value in root.items():
            if isinstance(value, (dict, list)):
                rv.extend(walk_yaml(value, target))
            elif key == target:
                rv.append(value)
    return rv


def read_image_name():
    import yaml
    with open(os.path.join(wpt_root, ".taskcluster.yml")) as f:
        taskcluster_data = yaml.safe_load(f)
    taskcluster_values = set(walk_yaml(taskcluster_data, "image"))
    with open(os.path.join(wpt_root, "tools", "ci", "tc", "tasks", "test.yml")) as f:
        test_data = yaml.safe_load(f)
    tests_value = test_data["components"]["wpt-base"]["image"]
    return taskcluster_values, tests_value


def tag_exists(tag):
    retcode = subprocess.call(["docker", "manifest", "inspect", tag])
    # The command succeeds if the tag exists.
    return retcode != 0


def push(venv, tag=None, force=False, *args, **kwargs):
    taskcluster_tags, tests_tag = read_image_name()

    taskcluster_tag = taskcluster_tags.pop()

    error_log = logger.warning if force else logger.error
    if len(taskcluster_tags) != 0 or tests_tag != taskcluster_tag:
        error_log("Image names in .taskcluster.yml and tools/ci/tc/tasks/test.yml "
                  "don't match.")
        if not force:
            sys.exit(1)
    if tag is not None and tag != taskcluster_tag:
        error_log("Supplied tag doesn't match .taskcluster.yml or "
                  "tools/ci/tc/tasks/test.yml; remember to update before pushing")
        if not force:
            sys.exit(1)
    if tag is None:
        logger.info("Using tag %s from .taskcluster.yml" % taskcluster_tag)
        tag = taskcluster_tag

    tag_re = re.compile(r"ghcr.io/web-platform-tests/wpt:\d+")
    if not tag_re.match(tag):
        error_log("Tag doesn't match expected format ghcr.io/web-platform-tests/wpt:x")
        if not force:
            sys.exit(1)

    if tag_exists(tag):
        # No override for this case
        logger.critical("Tag %s already exists" % tag)
        sys.exit(1)

    build(tag)
    subprocess.check_call(["docker",
                           "push",
                           tag])


def parser_run():
    parser = argparse.ArgumentParser()
    parser.add_argument("--rebuild", action="store_true", help="Force rebuild of image")
    parser.add_argument("--checkout", action="store",
                        help="Revision to checkout in the image. "
                        "If this is not supplied we mount the wpt checkout on the host as "
                        "/home/test/web-platform-tests/")
    parser.add_argument("--privileged", action="store_true",
                        help="Run the image in priviledged mode (required for emulators)")
    parser.add_argument("--tag", action="store", default="wpt:local",
                        help="Docker image tag to use (default wpt:local)")
    return parser


def run(*args, **kwargs):
    if kwargs["rebuild"]:
        build()

    args = ["docker", "run"]
    args.extend(["--security-opt", "seccomp:%s" %
                 os.path.join(wpt_root, "tools", "docker", "seccomp.json")])
    if kwargs["privileged"]:
        args.append("--privileged")
        args.extend(["--device", "/dev/kvm"])
    if kwargs["checkout"]:
        args.extend(["--env", "REF==%s" % kwargs["checkout"]])
    else:
        args.extend(["--mount",
                     "type=bind,source=%s,target=/home/test/web-platform-tests" % wpt_root])
    args.extend(["-it", kwargs["tag"]])

    proc = subprocess.Popen(args)
    proc.wait()
    return proc.returncode
