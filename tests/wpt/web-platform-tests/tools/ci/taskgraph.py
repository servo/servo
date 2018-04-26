import argparse
import copy
import os
import six

import yaml


here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))


task_template = {
    "provisionerId": "{{ taskcluster.docker.provisionerId }}",
    "workerType": "{{ taskcluster.docker.workerType }}",
    "extra": {
        "github": {
            "events": ["push"],
            "branches": ["master"],
        },
    },
    "payload": {
        "maxRunTime": 5400,
        "image": "harjgam/web-platform-tests:0.6",
        "command":[
            "/bin/bash",
            "--login",
            "-c",
            """>-
            ~/start.sh &&
            cd /home/test/web-platform-tests &&
            git fetch {{event.head.repo.url}} &&
            git config advice.detachedHead false &&
            git checkout {{event.head.sha}} &&
            %(command)s"""],
        "artifacts": {
            "public/results": {
                "path": "/home/test/artifacts",
                "type": "directory"
            }
        }
    },
    "metadata": {
        "name": "wpt-%(browser_name)s-%(suite)s-%(chunk)s",
        "description": "",
        "owner": "{{ event.head.user.email }}",
        "source": "{{ event.head.repo.url }}",
    }
}


file_template = {
    "version": 0,
    "tasks": [],
    "allowPullRequests": "collaborators"
}

suites = {
    "testharness": {"chunks": 12},
    "reftest": {"chunks": 6},
    "wdspec": {"chunks": 1}
}

browsers = {
    "firefox": {"name": "firefox-nightly"},
    "chrome": {"name": "chrome-dev"}
}


def get_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dest",
                        action="store",
                        default=wpt_root,
                        help="Directory to write the .taskcluster.yml file to")
    return parser


def fill(template, data):
    rv = {}
    for key, value in template.iteritems():
        rv[key] = fill_inner(value, data)
    return rv


def fill_inner(value, data):
    if isinstance(value, six.string_types):
        return value % data
    elif isinstance(value, dict):
        return fill(value, data)
    elif isinstance(value, list):
        return [fill_inner(item, data) for item in value]
    elif isinstance(value, (float,) + six.integer_types):
        return value
    else:
        raise ValueError


def run(venv, *args, **kwargs):
    if not os.path.isdir(kwargs["dest"]):
        raise ValueError("Invalid directory %s" % kwargs["dest"])

    task_config = copy.deepcopy(file_template)
    for browser, browser_props in browsers.iteritems():
        for suite, suite_props in suites.iteritems():
            total_chunks = suite_props.get("chunks", 1)
            for chunk in six.moves.xrange(1, total_chunks + 1):
                data = {
                    "suite": suite,
                    "chunk": chunk,
                    "browser_name": browser_props["name"],
                    "command": ("./tools/ci/ci_taskcluster.sh %s %s %s %s" %
                                (browser, suite, chunk, total_chunks))
                }

                task_data = fill(task_template, data)
                task_config["tasks"].append(task_data)

    with open(os.path.join(kwargs["dest"], ".taskcluster.yml"), "w") as f:
        f.write("""# GENERATED FILE DO NOT EDIT
# To regenerate this file run ./wpt make-tasks
""")
        yaml.dump(task_config, f)
