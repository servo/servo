# mypy: ignore-errors

import json
import os
from unittest import mock

import httpx
import jsone
import pytest
import yaml
from jsonschema import validate

from tools.ci.tc import decision

here = os.path.dirname(__file__)
root = os.path.abspath(os.path.join(here, "..", "..", "..", ".."))


def data_path(filename):
    return os.path.join(here, "..", "testdata", filename)


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


@pytest.mark.parametrize("event_path,expected",
                         [("pr_event.json",
                           frozenset(["lint", "wpt-chrome-dev-stability"])),
                          ("pr_event_tests_affected.json", frozenset(["lint"]))]
                         )
def test_exclude_users(event_path, expected):
    """Verify that tasks excluded by the PR submitter are properly excluded"""
    tasks = {
        "lint": {
            "commands": "wpt example"
        },
        "wpt-chrome-dev-stability": {
            "commands": "wpt example",
            "exclude-users": ["chromium-wpt-export-bot"]
        }
    }
    with open(data_path(event_path), encoding="utf8") as f:
        event = json.load(f)
        decision.filter_excluded_users(tasks, event)
        assert set(tasks) == expected


def test_sink_task_depends():
    """Verify that sink task only depends on required tasks"""
    from tools.ci.tc.decision import decide
    files_changed = {"layout-instability/clip-negative-bottom-margin.html",
                     "layout-instability/composited-element-movement.html"}
    with open(data_path("pr_event_tests_affected.json"), encoding="utf8") as f:
        event = json.load(f)
        with mock.patch("tools.ci.tc.decision.get_fetch_rev", return_value=(None, None, None)):
            with mock.patch("tools.wpt.testfiles.repo_files_changed",
                            return_value=files_changed):
                # Ensure we don't exclude the Chrome jobs
                event["pull_request"]["user"]["login"] = "test"
                task_id_map = decide(event)

        sink_task = task_id_map["sink-task"][1]

        assert set(sink_task["dependencies"]) == (
            set(task_id for (task_name, (task_id, _)) in task_id_map.items()
                if task_name not in {"sink-task", "wpt-chrome-dev-stability"}))


def test_verify_payload():
    """Verify that the decision task produces tasks with a valid payload"""
    from tools.ci.tc.decision import decide

    r = httpx.get("https://community-tc.services.mozilla.com/schemas/queue/v1/create-task-request.json")
    r.raise_for_status()
    create_task_schema = r.json()

    r = httpx.get("https://community-tc.services.mozilla.com/references/schemas/docker-worker/v1/payload.json")
    r.raise_for_status()
    payload_schema = r.json()

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
                print(f"Validation failed for task '{name}':\n{json.dumps(task_data, indent=2)}")
                raise e


@pytest.mark.parametrize("event_path,is_pr,files_changed,expected", [
    ("master_push_event.json", False, None,
     ['download-firefox-nightly',
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
      'wpt-chrome-canary-testharness-1',
      'wpt-chrome-canary-testharness-2',
      'wpt-chrome-canary-testharness-3',
      'wpt-chrome-canary-testharness-4',
      'wpt-chrome-canary-testharness-5',
      'wpt-chrome-canary-testharness-6',
      'wpt-chrome-canary-testharness-7',
      'wpt-chrome-canary-testharness-8',
      'wpt-chrome-canary-testharness-9',
      'wpt-chrome-canary-testharness-10',
      'wpt-chrome-canary-testharness-11',
      'wpt-chrome-canary-testharness-12',
      'wpt-chrome-canary-testharness-13',
      'wpt-chrome-canary-testharness-14',
      'wpt-chrome-canary-testharness-15',
      'wpt-chrome-canary-testharness-16',
      'wpt-firefox-nightly-reftest-1',
      'wpt-firefox-nightly-reftest-2',
      'wpt-firefox-nightly-reftest-3',
      'wpt-firefox-nightly-reftest-4',
      'wpt-firefox-nightly-reftest-5',
      'wpt-firefox-nightly-reftest-6',
      'wpt-chrome-canary-reftest-1',
      'wpt-chrome-canary-reftest-2',
      'wpt-chrome-canary-reftest-3',
      'wpt-chrome-canary-reftest-4',
      'wpt-chrome-canary-reftest-5',
      'wpt-chrome-canary-reftest-6',
      'wpt-firefox-nightly-wdspec-1',
      'wpt-firefox-nightly-wdspec-2',
      'wpt-chrome-canary-wdspec-1',
      'wpt-chrome-canary-wdspec-2',
      'wpt-firefox-nightly-crashtest-1',
      'wpt-chrome-canary-crashtest-1',
      'wpt-firefox-nightly-print-reftest-1',
      'wpt-chrome-canary-print-reftest-1',
      'lint']),
    ("pr_event.json", True, {".taskcluster.yml", ".travis.yml", "tools/ci/start.sh"},
     ['lint',
      'tools/ unittests (Python 3.8)',
      'tools/ unittests (Python 3.12)',
      'tools/ integration tests (Python 3.8)',
      'tools/ integration tests (Python 3.12)',
      'resources/ tests (Python 3.8)',
      'resources/ tests (Python 3.12)',
      'download-firefox-nightly',
      'infrastructure/ tests',
      'sink-task']),
    # More tests are affected in the actual PR but it shouldn't affect the scheduled tasks
    ("pr_event_tests_affected.json", True, {"layout-instability/clip-negative-bottom-margin.html",
                                            "layout-instability/composited-element-movement.html"},
     ['download-firefox-nightly',
      'wpt-firefox-nightly-stability',
      'wpt-firefox-nightly-results',
      'wpt-firefox-nightly-results-without-changes',
      'wpt-chrome-dev-results',
      'wpt-chrome-dev-results-without-changes',
      'lint',
      'sink-task']),
    ("pr_event_tests_affected.json", True, {"resources/testharness.js"},
     ['lint',
      'resources/ tests (Python 3.8)',
      'resources/ tests (Python 3.12)',
      'download-firefox-nightly',
      'infrastructure/ tests',
      'sink-task']),
    ("epochs_daily_push_event.json", False, None,
     ['download-firefox-stable',
      'wpt-firefox-stable-testharness-1',
      'wpt-firefox-stable-testharness-2',
      'wpt-firefox-stable-testharness-3',
      'wpt-firefox-stable-testharness-4',
      'wpt-firefox-stable-testharness-5',
      'wpt-firefox-stable-testharness-6',
      'wpt-firefox-stable-testharness-7',
      'wpt-firefox-stable-testharness-8',
      'wpt-firefox-stable-testharness-9',
      'wpt-firefox-stable-testharness-10',
      'wpt-firefox-stable-testharness-11',
      'wpt-firefox-stable-testharness-12',
      'wpt-firefox-stable-testharness-13',
      'wpt-firefox-stable-testharness-14',
      'wpt-firefox-stable-testharness-15',
      'wpt-firefox-stable-testharness-16',
      'wpt-chromium-nightly-testharness-1',
      'wpt-chromium-nightly-testharness-2',
      'wpt-chromium-nightly-testharness-3',
      'wpt-chromium-nightly-testharness-4',
      'wpt-chromium-nightly-testharness-5',
      'wpt-chromium-nightly-testharness-6',
      'wpt-chromium-nightly-testharness-7',
      'wpt-chromium-nightly-testharness-8',
      'wpt-chromium-nightly-testharness-9',
      'wpt-chromium-nightly-testharness-10',
      'wpt-chromium-nightly-testharness-11',
      'wpt-chromium-nightly-testharness-12',
      'wpt-chromium-nightly-testharness-13',
      'wpt-chromium-nightly-testharness-14',
      'wpt-chromium-nightly-testharness-15',
      'wpt-chromium-nightly-testharness-16',
      'wpt-chrome-stable-testharness-1',
      'wpt-chrome-stable-testharness-2',
      'wpt-chrome-stable-testharness-3',
      'wpt-chrome-stable-testharness-4',
      'wpt-chrome-stable-testharness-5',
      'wpt-chrome-stable-testharness-6',
      'wpt-chrome-stable-testharness-7',
      'wpt-chrome-stable-testharness-8',
      'wpt-chrome-stable-testharness-9',
      'wpt-chrome-stable-testharness-10',
      'wpt-chrome-stable-testharness-11',
      'wpt-chrome-stable-testharness-12',
      'wpt-chrome-stable-testharness-13',
      'wpt-chrome-stable-testharness-14',
      'wpt-chrome-stable-testharness-15',
      'wpt-chrome-stable-testharness-16',
      'wpt-webkitgtk_minibrowser-nightly-testharness-1',
      'wpt-webkitgtk_minibrowser-nightly-testharness-2',
      'wpt-webkitgtk_minibrowser-nightly-testharness-3',
      'wpt-webkitgtk_minibrowser-nightly-testharness-4',
      'wpt-webkitgtk_minibrowser-nightly-testharness-5',
      'wpt-webkitgtk_minibrowser-nightly-testharness-6',
      'wpt-webkitgtk_minibrowser-nightly-testharness-7',
      'wpt-webkitgtk_minibrowser-nightly-testharness-8',
      'wpt-webkitgtk_minibrowser-nightly-testharness-9',
      'wpt-webkitgtk_minibrowser-nightly-testharness-10',
      'wpt-webkitgtk_minibrowser-nightly-testharness-11',
      'wpt-webkitgtk_minibrowser-nightly-testharness-12',
      'wpt-webkitgtk_minibrowser-nightly-testharness-13',
      'wpt-webkitgtk_minibrowser-nightly-testharness-14',
      'wpt-webkitgtk_minibrowser-nightly-testharness-15',
      'wpt-webkitgtk_minibrowser-nightly-testharness-16',
      'wpt-servo-nightly-testharness-1',
      'wpt-servo-nightly-testharness-2',
      'wpt-servo-nightly-testharness-3',
      'wpt-servo-nightly-testharness-4',
      'wpt-servo-nightly-testharness-5',
      'wpt-servo-nightly-testharness-6',
      'wpt-servo-nightly-testharness-7',
      'wpt-servo-nightly-testharness-8',
      'wpt-servo-nightly-testharness-9',
      'wpt-servo-nightly-testharness-10',
      'wpt-servo-nightly-testharness-11',
      'wpt-servo-nightly-testharness-12',
      'wpt-servo-nightly-testharness-13',
      'wpt-servo-nightly-testharness-14',
      'wpt-servo-nightly-testharness-15',
      'wpt-servo-nightly-testharness-16',
      'wpt-firefox_android-nightly-testharness-1',
      'wpt-firefox_android-nightly-testharness-2',
      'wpt-firefox_android-nightly-testharness-3',
      'wpt-firefox_android-nightly-testharness-4',
      'wpt-firefox_android-nightly-testharness-5',
      'wpt-firefox_android-nightly-testharness-6',
      'wpt-firefox_android-nightly-testharness-7',
      'wpt-firefox_android-nightly-testharness-8',
      'wpt-firefox_android-nightly-testharness-9',
      'wpt-firefox_android-nightly-testharness-10',
      'wpt-firefox_android-nightly-testharness-11',
      'wpt-firefox_android-nightly-testharness-12',
      'wpt-firefox_android-nightly-testharness-13',
      'wpt-firefox_android-nightly-testharness-14',
      'wpt-firefox_android-nightly-testharness-15',
      'wpt-firefox_android-nightly-testharness-16',
      'wpt-firefox_android-nightly-testharness-17',
      'wpt-firefox_android-nightly-testharness-18',
      'wpt-firefox_android-nightly-testharness-19',
      'wpt-firefox_android-nightly-testharness-20',
      'wpt-firefox_android-nightly-testharness-21',
      'wpt-firefox_android-nightly-testharness-22',
      'wpt-firefox_android-nightly-testharness-23',
      'wpt-firefox_android-nightly-testharness-24',
      'wpt-firefox-stable-reftest-1',
      'wpt-firefox-stable-reftest-2',
      'wpt-firefox-stable-reftest-3',
      'wpt-firefox-stable-reftest-4',
      'wpt-firefox-stable-reftest-5',
      'wpt-firefox-stable-reftest-6',
      'wpt-chromium-nightly-reftest-1',
      'wpt-chromium-nightly-reftest-2',
      'wpt-chromium-nightly-reftest-3',
      'wpt-chromium-nightly-reftest-4',
      'wpt-chromium-nightly-reftest-5',
      'wpt-chromium-nightly-reftest-6',
      'wpt-chrome-stable-reftest-1',
      'wpt-chrome-stable-reftest-2',
      'wpt-chrome-stable-reftest-3',
      'wpt-chrome-stable-reftest-4',
      'wpt-chrome-stable-reftest-5',
      'wpt-chrome-stable-reftest-6',
      'wpt-webkitgtk_minibrowser-nightly-reftest-1',
      'wpt-webkitgtk_minibrowser-nightly-reftest-2',
      'wpt-webkitgtk_minibrowser-nightly-reftest-3',
      'wpt-webkitgtk_minibrowser-nightly-reftest-4',
      'wpt-webkitgtk_minibrowser-nightly-reftest-5',
      'wpt-webkitgtk_minibrowser-nightly-reftest-6',
      'wpt-servo-nightly-reftest-1',
      'wpt-servo-nightly-reftest-2',
      'wpt-servo-nightly-reftest-3',
      'wpt-servo-nightly-reftest-4',
      'wpt-servo-nightly-reftest-5',
      'wpt-servo-nightly-reftest-6',
      'wpt-firefox_android-nightly-reftest-1',
      'wpt-firefox_android-nightly-reftest-2',
      'wpt-firefox_android-nightly-reftest-3',
      'wpt-firefox_android-nightly-reftest-4',
      'wpt-firefox_android-nightly-reftest-5',
      'wpt-firefox_android-nightly-reftest-6',
      'wpt-firefox-stable-wdspec-1',
      'wpt-firefox-stable-wdspec-2',
      'wpt-chromium-nightly-wdspec-1',
      'wpt-chromium-nightly-wdspec-2',
      'wpt-chrome-stable-wdspec-1',
      'wpt-chrome-stable-wdspec-2',
      'wpt-webkitgtk_minibrowser-nightly-wdspec-1',
      'wpt-webkitgtk_minibrowser-nightly-wdspec-2',
      'wpt-servo-nightly-wdspec-1',
      'wpt-servo-nightly-wdspec-2',
      'wpt-firefox_android-nightly-wdspec-1',
      'wpt-firefox_android-nightly-wdspec-2',
      'wpt-firefox-stable-crashtest-1',
      'wpt-chromium-nightly-crashtest-1',
      'wpt-chrome-stable-crashtest-1',
      'wpt-webkitgtk_minibrowser-nightly-crashtest-1',
      'wpt-servo-nightly-crashtest-1',
      'wpt-firefox_android-nightly-crashtest-1',
      'wpt-firefox-stable-print-reftest-1',
      'wpt-chromium-nightly-print-reftest-1',
      'wpt-chrome-stable-print-reftest-1'])
])
def test_schedule_tasks(event_path, is_pr, files_changed, expected):
    with mock.patch("tools.ci.tc.decision.get_fetch_rev", return_value=(None, None, None)):
        with mock.patch("tools.wpt.testfiles.repo_files_changed",
                        return_value=files_changed):
            with open(data_path(event_path), encoding="utf8") as event_file:
                event = json.load(event_file)
                scheduled = decision.decide(event)
                print(list(scheduled.keys()))
                assert list(scheduled.keys()) == expected
