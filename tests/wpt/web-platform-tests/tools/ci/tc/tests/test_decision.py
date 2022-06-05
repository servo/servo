# mypy: allow-untyped-defs

from unittest import mock

import pytest

from tools.ci.tc import decision


@pytest.mark.parametrize("run_jobs,tasks,expected", [
    ([], {"task-no-schedule-if": {}}, ["task-no-schedule-if"]),
    ([], {"task-schedule-if-no-run-job": {"schedule-if": {}}}, []),
    (["job"],
     {"job-present": {"schedule-if": {"run-job": ["other-job", "job"]}}},
     ["job-present"]),
    (["job"], {"job-missing": {"schedule-if": {"run-job": ["other-job"]}}}, []),
    (["all"], {"job-all": {"schedule-if": {"run-job": ["other-job"]}}}, ["job-all"]),
    (["job"],
     {"job-1": {"schedule-if": {"run-job": ["job"]}},
      "job-2": {"schedule-if": {"run-job": ["other-job"]}}},
     ["job-1"]),
])
def test_filter_schedule_if(run_jobs, tasks, expected):
    with mock.patch("tools.ci.tc.decision.get_run_jobs",
                    return_value=run_jobs) as get_run_jobs:
        assert (decision.filter_schedule_if({}, tasks) ==
                {name: tasks[name] for name in expected})
        get_run_jobs.call_count in (0, 1)


@pytest.mark.parametrize("msg,expected", [
    ("Some initial line\n\ntc-jobs:foo,bar", {"foo", "bar"}),
    ("Some initial line\n\ntc-jobs:foo, bar", {"foo", "bar"}),
    ("tc-jobs:foo, bar   \nbaz", {"foo", "bar"}),
    ("tc-jobs:all", {"all"}),
    ("", set()),
    ("tc-jobs:foo\ntc-jobs:bar", {"foo"})])
@pytest.mark.parametrize("event", [
    {"commits": [{"message": "<message>"}]},
    {"pull_request": {"body": "<message>"}}
])
def test_extra_jobs_pr(msg, expected, event):
    def sub(obj):
        """Copy obj, except if it's a string with the value <message>
        replace it with the value of the msg argument"""
        if isinstance(obj, dict):
            return {key: sub(value) for (key, value) in obj.items()}
        elif isinstance(obj, list):
            return [sub(value) for value in obj]
        elif obj == "<message>":
            return msg
        return obj

    event = sub(event)

    assert decision.get_extra_jobs(event) == expected
