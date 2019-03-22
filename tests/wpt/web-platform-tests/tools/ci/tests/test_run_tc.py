import pytest

from six import iteritems

from tools.ci import run_tc


@pytest.mark.parametrize("msg,expected", [
    ("Some initial line\n\ntc-jobs:foo,bar", set(["foo", "bar"])),
    ("Some initial line\n\ntc-jobs:foo, bar", set(["foo", "bar"])),
    ("tc-jobs:foo, bar   \nbaz", set(["foo", "bar"])),
    ("tc-jobs:all", set(["all"])),
    ("", set()),
    ("tc-jobs:foo\ntc-jobs:bar", set(["foo"]))])
@pytest.mark.parametrize("event", [
    {"commits": [{"message": "<message>"}]},
    {"pull_request": {"body": "<message>"}}
])
def test_extra_jobs_pr(msg, expected, event):
    def sub(obj):
        """Copy obj, except if it's a string with the value <message>
        replace it with the value of the msg argument"""
        if isinstance(obj, dict):
            return {key: sub(value) for (key, value) in iteritems(obj)}
        elif isinstance(obj, list):
            return [sub(value) for value in obj]
        elif obj == "<message>":
            return msg
        return obj

    event = sub(event)

    assert run_tc.get_extra_jobs(event) == expected
