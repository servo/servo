import json
import os
from io import open

import jsone
import mock
import pytest
import requests
import sys
import yaml
from jsonschema import validate

from tools.ci.tc import decision

here = os.path.dirname(__file__)
root = os.path.abspath(os.path.join(here, "..", "..", "..", ".."))


def data_path(filename):
    return os.path.join(here, "..", "testdata", filename)


@pytest.mark.xfail(sys.version_info.major == 2,
                   reason="taskcluster library has an encoding bug "
                   "https://github.com/taskcluster/json-e/issues/338")
def test_verify_taskcluster_yml():
    """Verify that the json-e in the .taskcluster.yml is valid"""
    with open(os.path.join(root, ".taskcluster.yml"), encoding="utf8") as f:
        template = yaml.safe_load(f)

    events = [("pr_event.json", "github-pull-request", "Pull Request"),
              ("master_push_event.json", "github-push", "Push to master")]

    for filename, tasks_for, title in events:
        with open(data_path(filename), encoding="utf8") as f:
            event = json.load(f)

        context = {"tasks_for": tasks_for,
                   "event": event,
                   "as_slugid": lambda x: x}

        jsone.render(template, context)


def test_verify_payload():
    """Verify that the decision task produces tasks with a valid payload"""
    from tools.ci.tc.decision import decide

    create_task_schema = requests.get(
        "https://raw.githubusercontent.com/taskcluster/taskcluster/blob/master/services/queue/schemas/v1/create-task-request.yml")
    create_task_schema = yaml.safe_load(create_task_schema.content)

    payload_schema = requests.get("https://raw.githubusercontent.com/taskcluster/docker-worker/master/schemas/v1/payload.json").json()

    jobs = ["lint",
            "manifest_upload",
            "resources_unittest",
            "tools_unittest",
            "wpt_integration",
            "wptrunner_infrastructure",
            "wptrunner_unittest"]

    for filename in ["pr_event.json", "master_push_event.json"]:
        with open(data_path(filename), encoding="utf8") as f:
            event = json.load(f)

        with mock.patch("tools.ci.tc.decision.get_fetch_rev", return_value=(None, event["after"], None)):
            with mock.patch("tools.ci.tc.decision.get_run_jobs", return_value=set(jobs)):
                task_id_map = decide(event)
        for name, (task_id, task_data) in task_id_map.items():
            try:
                validate(instance=task_data, schema=create_task_schema)
                validate(instance=task_data["payload"], schema=payload_schema)
            except Exception as e:
                print("Validation failed for task '%s':\n%s" % (name, json.dumps(task_data, indent=2)))
                raise e


@pytest.mark.parametrize("event_path,is_pr,files_changed,expected", [
    ("master_push_event.json", False, None,
     {'download-firefox-nightly',
      'wpt-firefox-nightly-testharness-1',
      'wpt-firefox-nightly-testharness-2',
      'wpt-firefox-nightly-testharness-3',
      'wpt-firefox-nightly-testharness-4',
      'wpt-firefox-nightly-testharness-5',
      'wpt-firefox-nightly-testharness-6',
      'wpt-firefox-nightly-testharness-7',
      'wpt-firefox-nightly-testharness-8',
      'wpt-firefox-nightly-testharness-9',
      'wpt-firefox-nightly-testharness-10',
      'wpt-firefox-nightly-testharness-11',
      'wpt-firefox-nightly-testharness-12',
      'wpt-firefox-nightly-testharness-13',
      'wpt-firefox-nightly-testharness-14',
      'wpt-firefox-nightly-testharness-15',
      'wpt-firefox-nightly-testharness-16',
      'wpt-chrome-dev-testharness-1',
      'wpt-chrome-dev-testharness-2',
      'wpt-chrome-dev-testharness-3',
      'wpt-chrome-dev-testharness-4',
      'wpt-chrome-dev-testharness-5',
      'wpt-chrome-dev-testharness-6',
      'wpt-chrome-dev-testharness-7',
      'wpt-chrome-dev-testharness-8',
      'wpt-chrome-dev-testharness-9',
      'wpt-chrome-dev-testharness-10',
      'wpt-chrome-dev-testharness-11',
      'wpt-chrome-dev-testharness-12',
      'wpt-chrome-dev-testharness-13',
      'wpt-chrome-dev-testharness-14',
      'wpt-chrome-dev-testharness-15',
      'wpt-chrome-dev-testharness-16',
      'wpt-firefox-nightly-reftest-1',
      'wpt-firefox-nightly-reftest-2',
      'wpt-firefox-nightly-reftest-3',
      'wpt-firefox-nightly-reftest-4',
      'wpt-firefox-nightly-reftest-5',
      'wpt-chrome-dev-reftest-1',
      'wpt-chrome-dev-reftest-2',
      'wpt-chrome-dev-reftest-3',
      'wpt-chrome-dev-reftest-4',
      'wpt-chrome-dev-reftest-5',
      'wpt-firefox-nightly-wdspec-1',
      'wpt-chrome-dev-wdspec-1',
      'lint'}),
    ("pr_event.json", True, {".taskcluster.yml",".travis.yml","tools/ci/start.sh"},
     {'lint',
      'tools/ unittests (Python 2)',
      'tools/ unittests (Python 3)',
      'tools/wpt/ tests',
      'resources/ tests',
      'infrastructure/ tests'}),
    # More tests are affected in the actual PR but it shouldn't affect the scheduled tasks
    ("pr_event_tests_affected.json", True, {"layout-instability/clip-negative-bottom-margin.html",
                                            "layout-instability/composited-element-movement.html"},
     {'download-firefox-nightly',
      'wpt-firefox-nightly-stability',
      'wpt-firefox-nightly-results',
      'wpt-firefox-nightly-results-without-changes',
      'wpt-chrome-dev-stability',
      'wpt-chrome-dev-results',
      'wpt-chrome-dev-results-without-changes',
      'lint'}),
    ("epochs_daily_push_event.json", False, None,
     {'download-firefox-stable',
      'wpt-chrome-stable-reftest-1',
      'wpt-chrome-stable-reftest-2',
      'wpt-chrome-stable-reftest-3',
      'wpt-chrome-stable-reftest-4',
      'wpt-chrome-stable-reftest-5',
      'wpt-chrome-stable-testharness-1',
      'wpt-chrome-stable-testharness-10',
      'wpt-chrome-stable-testharness-11',
      'wpt-chrome-stable-testharness-12',
      'wpt-chrome-stable-testharness-13',
      'wpt-chrome-stable-testharness-14',
      'wpt-chrome-stable-testharness-15',
      'wpt-chrome-stable-testharness-16',
      'wpt-chrome-stable-testharness-2',
      'wpt-chrome-stable-testharness-3',
      'wpt-chrome-stable-testharness-4',
      'wpt-chrome-stable-testharness-5',
      'wpt-chrome-stable-testharness-6',
      'wpt-chrome-stable-testharness-7',
      'wpt-chrome-stable-testharness-8',
      'wpt-chrome-stable-testharness-9',
      'wpt-chrome-stable-wdspec-1',
      'wpt-firefox-stable-reftest-1',
      'wpt-firefox-stable-reftest-2',
      'wpt-firefox-stable-reftest-3',
      'wpt-firefox-stable-reftest-4',
      'wpt-firefox-stable-reftest-5',
      'wpt-firefox-stable-testharness-1',
      'wpt-firefox-stable-testharness-10',
      'wpt-firefox-stable-testharness-11',
      'wpt-firefox-stable-testharness-12',
      'wpt-firefox-stable-testharness-13',
      'wpt-firefox-stable-testharness-14',
      'wpt-firefox-stable-testharness-15',
      'wpt-firefox-stable-testharness-16',
      'wpt-firefox-stable-testharness-2',
      'wpt-firefox-stable-testharness-3',
      'wpt-firefox-stable-testharness-4',
      'wpt-firefox-stable-testharness-5',
      'wpt-firefox-stable-testharness-6',
      'wpt-firefox-stable-testharness-7',
      'wpt-firefox-stable-testharness-8',
      'wpt-firefox-stable-testharness-9',
      'wpt-firefox-stable-wdspec-1',
      'wpt-webkitgtk_minibrowser-nightly-reftest-1',
      'wpt-webkitgtk_minibrowser-nightly-reftest-2',
      'wpt-webkitgtk_minibrowser-nightly-reftest-3',
      'wpt-webkitgtk_minibrowser-nightly-reftest-4',
      'wpt-webkitgtk_minibrowser-nightly-reftest-5',
      'wpt-webkitgtk_minibrowser-nightly-testharness-1',
      'wpt-webkitgtk_minibrowser-nightly-testharness-10',
      'wpt-webkitgtk_minibrowser-nightly-testharness-11',
      'wpt-webkitgtk_minibrowser-nightly-testharness-12',
      'wpt-webkitgtk_minibrowser-nightly-testharness-13',
      'wpt-webkitgtk_minibrowser-nightly-testharness-14',
      'wpt-webkitgtk_minibrowser-nightly-testharness-15',
      'wpt-webkitgtk_minibrowser-nightly-testharness-16',
      'wpt-webkitgtk_minibrowser-nightly-testharness-2',
      'wpt-webkitgtk_minibrowser-nightly-testharness-3',
      'wpt-webkitgtk_minibrowser-nightly-testharness-4',
      'wpt-webkitgtk_minibrowser-nightly-testharness-5',
      'wpt-webkitgtk_minibrowser-nightly-testharness-6',
      'wpt-webkitgtk_minibrowser-nightly-testharness-7',
      'wpt-webkitgtk_minibrowser-nightly-testharness-8',
      'wpt-webkitgtk_minibrowser-nightly-testharness-9',
      'wpt-webkitgtk_minibrowser-nightly-wdspec-1'})
])
def test_schedule_tasks(event_path, is_pr, files_changed, expected):
    with mock.patch("tools.ci.tc.decision.get_fetch_rev", return_value=(None, None, None)):
        with mock.patch("tools.wpt.testfiles.repo_files_changed",
                        return_value=files_changed):
            with open(data_path(event_path), encoding="utf8") as event_file:
                event = json.load(event_file)
                scheduled = decision.decide(event)
                assert set(scheduled.keys()) == expected
