# -*- coding: utf-8 -*-

import json
import time

import pytest

# flake8: noqa


@pytest.mark.parametrize(
    "logs,expected",
    (
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo", "test_bar", "test_baz"],
                        "manifestB": ["test_something"],
                    },
                ),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "SKIP"),
                ("test_start", "test_bar"),
                ("test_end", "test_bar", "OK"),
                ("test_start", "test_something"),
                ("test_end", "test_something", "OK"),
                ("test_start", "test_baz"),
                ("test_end", "test_baz", "PASS", "FAIL"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"test": "test_baz", "subtest": null, "group": "manifestA", "status": "PASS", "expected": "FAIL", "message": null, "stack": null, "modifiers": "", "known_intermittent": [], "action": "test_result", "line": 8}
                {"group": "manifestA", "status": "ERROR", "duration": 20, "action": "group_result", "line": 9}
                {"group": "manifestB", "status": "OK", "duration": 10, "action": "group_result", "line": 9}
            """.strip(),
            id="basic",
        ),
        pytest.param(
            [
                ("suite_start", {"manifest": ["test_foo"]}),
                ("test_start", "test_foo"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifest"], "action": "test_groups", "line": 0}
                {"group": "manifest", "status": null, "duration": 0, "action": "group_result", "line": 2}
            """.strip(),
            id="missing_test_end",
        ),
        pytest.param(
            [
                ("suite_start", {"manifest": ["test_foo"]}),
                ("test_start", "test_foo"),
                ("test_status", "test_foo", "subtest", "PASS"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifest"], "action": "test_groups", "line": 0}
                {"group": "manifest", "status": "ERROR", "duration": null, "action": "group_result", "line": 3}
            """.strip(),
            id="missing_test_end_with_test_status_ok",
            marks=pytest.mark.xfail,  # status is OK but should be ERROR
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo", "test_bar", "test_baz"],
                        "manifestB": ["test_something"],
                    },
                ),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "SKIP"),
                ("test_start", "test_bar"),
                ("test_end", "test_bar", "CRASH"),
                ("test_start", "test_something"),
                ("test_end", "test_something", "OK"),
                ("test_start", "test_baz"),
                ("test_end", "test_baz", "FAIL", "FAIL"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"test": "test_bar", "subtest": null, "group": "manifestA", "status": "CRASH", "expected": "OK", "message": null, "stack": null, "modifiers": "", "known_intermittent": [], "action": "test_result", "line": 4}
                {"group": "manifestA", "status": "ERROR", "duration": 20, "action": "group_result", "line": 9}
                {"group": "manifestB", "status": "OK", "duration": 10, "action": "group_result", "line": 9}
            """.strip(),
            id="crash_and_group_status",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo", "test_bar", "test_baz"],
                        "manifestB": ["test_something"],
                    },
                ),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "SKIP"),
                ("test_start", "test_bar"),
                ("test_end", "test_bar", "OK"),
                ("test_start", "test_something"),
                ("test_end", "test_something", "OK"),
                ("test_start", "test_baz"),
                ("test_status", "test_baz", "subtest", "FAIL", "FAIL"),
                ("test_end", "test_baz", "OK"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"group": "manifestA", "status": "OK", "duration": 29, "action": "group_result", "line": 10}
                {"group": "manifestB", "status": "OK", "duration": 10, "action": "group_result", "line": 10}
            """.strip(),
            id="fail_expected_fail",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo", "test_bar", "test_baz"],
                        "manifestB": ["test_something"],
                    },
                ),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "SKIP"),
                ("test_start", "test_bar"),
                ("test_end", "test_bar", "OK"),
                ("test_start", "test_something"),
                ("test_end", "test_something", "OK"),
                ("test_start", "test_baz"),
                ("test_status", "test_baz", "Test timed out", "FAIL", "PASS"),
                ("test_status", "test_baz", "", "TIMEOUT", "PASS"),
                ("crash", "", "signature", "manifestA"),
                ("test_end", "test_baz", "OK"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"test": "test_baz", "subtest": "Test timed out", "group": "manifestA", "status": "FAIL", "expected": "PASS", "message": null, "stack": null, "modifiers": "", "known_intermittent": [], "action": "test_result", "line": 8}
                {"test": "test_baz", "subtest": "", "group": "manifestA", "status": "TIMEOUT", "expected": "PASS", "message": null, "stack": null, "modifiers": "", "known_intermittent": [], "action": "test_result", "line": 9}
                {"test": "manifestA", "group": "manifestA", "signature": "signature", "stackwalk_stderr": null, "stackwalk_stdout": null, "action": "crash", "line": 10}
                {"group": "manifestA", "status": "ERROR", "duration": 49, "action": "group_result", "line": 12}
                {"group": "manifestB", "status": "OK", "duration": 10, "action": "group_result", "line": 12}
            """.strip(),
            id="timeout_and_crash",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo", "test_bar", "test_baz"],
                        "manifestB": ["test_something"],
                    },
                ),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "SKIP"),
                ("test_start", "test_bar"),
                ("test_end", "test_bar", "CRASH", "CRASH"),
                ("test_start", "test_something"),
                ("test_end", "test_something", "OK"),
                ("test_start", "test_baz"),
                ("test_end", "test_baz", "FAIL", "FAIL"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"group": "manifestA", "status": "OK", "duration": 20, "action": "group_result", "line": 9}
                {"group": "manifestB", "status": "OK", "duration": 10, "action": "group_result", "line": 9}
            """.strip(),
            id="crash_expected_crash",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo", "test_bar", "test_baz"],
                        "manifestB": ["test_something"],
                    },
                ),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "SKIP"),
                ("test_start", "test_bar"),
                ("test_end", "test_bar", "OK"),
                ("test_start", "test_something"),
                ("test_end", "test_something", "OK"),
                ("test_start", "test_baz"),
                ("test_end", "test_baz", "FAIL", "FAIL"),
                ("crash", "", "", "manifestA"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"test": "manifestA", "group": "manifestA", "signature": "", "stackwalk_stderr": null, "stackwalk_stdout": null, "action": "crash", "line": 9}
                {"group": "manifestA", "status": "ERROR", "duration": 20, "action": "group_result", "line": 10}
                {"group": "manifestB", "status": "OK", "duration": 10, "action": "group_result", "line": 10}
            """.strip(),
            id="assertion_crash_on_shutdown",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo", "test_bar", "test_baz"],
                        "manifestB": ["test_something"],
                    },
                ),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "SKIP"),
                ("test_start", "test_bar"),
                ("test_end", "test_bar", "OK"),
                ("test_start", "test_something"),
                ("test_end", "test_something", "OK"),
                ("test_start", "test_baz"),
                ("test_end", "test_baz", "FAIL"),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"test": "test_baz", "group": "manifestA", "status": "FAIL", "expected": "OK", "subtest": null, "message": null, "stack": null, "modifiers": "", "known_intermittent": [], "action": "test_result", "line": 8}
            """.strip(),
            id="timeout_no_group_status",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo", "test_bar", "test_baz"],
                        "manifestB": ["test_something"],
                    },
                ),
                ("group_start", "manifestA"),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "SKIP"),
                ("test_start", "test_bar"),
                ("test_end", "test_bar", "OK"),
                ("test_start", "test_baz"),
                ("test_end", "test_baz", "PASS", "FAIL"),
                ("group_end", "manifestA"),
                ("group_start", "manifestB"),
                ("test_start", "test_something"),
                ("test_end", "test_something", "OK"),
                ("group_end", "manifestB"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"test": "test_baz", "subtest": null, "group": "manifestA", "status": "PASS", "expected": "FAIL", "message": null, "stack": null, "modifiers": "", "known_intermittent": [], "action": "test_result", "line": 7}
                {"group": "manifestA", "status": "ERROR", "duration": 70, "action": "group_result", "line": 13}
                {"group": "manifestB", "status": "OK", "duration": 30, "action": "group_result", "line": 13}
            """.strip(),
            id="group_start_end_duration",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo", "test_bar"],
                    },
                ),
                ("group_start", "manifestA"),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "SKIP"),
                ("test_start", "test_bar"),
                ("test_end", "test_bar", "SKIP"),
                ("group_end", "manifestA"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA"], "action": "test_groups", "line": 0}
                {"group": "manifestA", "status": "SKIP", "duration": 0, "action": "group_result", "line": 7}
            """.strip(),
            id="group_start_end_all_skipped",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_foo"],
                        "manifestB": ["test_something"],
                    },
                ),
                ("group_start", "parallel"),
                ("test_start", "test_foo"),
                ("test_end", "test_foo", "OK"),
                ("test_start", "test_something"),
                ("test_end", "test_something", "OK"),
                ("group_end", "parallel"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"group": "manifestA", "status": "OK", "duration": 10, "action": "group_result", "line": 7}
                {"group": "manifestB", "status": "OK", "duration": 10, "action": "group_result", "line": 7}
            """.strip(),
            id="non_manifest_group_names_ignored",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_a1", "test_a2"],
                        "manifestB": ["test_b1"],
                    },
                ),
                ("group_start", "parallel", {"threads": 2}),
                ("test_start", "test_a1"),
                ("test_end", "test_a1", "OK"),
                ("test_start", "test_b1"),
                ("test_end", "test_b1", "OK"),
                ("group_end", "parallel"),
                ("group_start", "sequential"),
                ("test_start", "test_a2"),
                ("test_end", "test_a2", "OK"),
                ("group_end", "sequential"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA", "manifestB"], "action": "test_groups", "line": 0}
                {"group": "manifestA", "status": "OK", "duration": 14, "action": "group_result", "line": 11}
                {"group": "manifestB", "status": "OK", "duration": 5, "action": "group_result", "line": 11}
            """.strip(),
            id="parallel_threads_divide_time",
        ),
        pytest.param(
            [
                (
                    "suite_start",
                    {
                        "manifestA": ["test_a1", "test_a2"],
                    },
                ),
                ("test_start", "test_a1"),
                ("test_start", "test_a2"),
                ("test_end", "test_a2", "SKIP"),
                ("test_end", "test_a1", "OK"),
                ("suite_end",),
            ],
            """
                {"groups": ["manifestA"], "action": "test_groups", "line": 0}
                {"group": "manifestA", "status": "OK", "duration": 30, "action": "group_result", "line": 5}
            """.strip(),
            id="parallel_skip_does_not_clear_other_test_time",
        ),
    ),
)
def test_errorsummary(monkeypatch, get_logger, logs, expected):
    ts = {"ts": 0.0}  # need to use dict since 'nonlocal' doesn't exist on PY2

    def fake_time():
        ts["ts"] += 0.01
        return ts["ts"]

    monkeypatch.setattr(time, "time", fake_time)
    logger = get_logger("errorsummary")

    for log in logs:
        getattr(logger, log[0])(*log[1:])

    buf = logger.handlers[0].stream
    result = buf.getvalue()
    print("Dumping result for copy/paste:")
    print(result)

    expected = expected.split("\n")
    for i, line in enumerate(result.split("\n")):
        if not line:
            continue

        data = json.loads(line)
        assert data == json.loads(expected[i])


if __name__ == "__main__":
    import mozunit
    mozunit.main()
