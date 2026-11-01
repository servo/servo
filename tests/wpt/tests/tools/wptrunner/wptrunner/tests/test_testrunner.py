# mypy: allow-untyped-calls

from collections import deque
from queue import SimpleQueue
from unittest.mock import Mock

import pytest

from .. import testloader, testrunner, wpttest
from ..browsers.base import Browser


@pytest.fixture
def manager() -> testrunner.TestRunnerManager:
    current_test = wpttest.TestharnessTest("/", "/", "/current.html", [], None)
    next_test = wpttest.Test262Test("/", "/", "/next.html", [], None)
    current_group = testloader.TestGroup(
        deque(), "", "testharness", testloader.GroupMetadata("/current"))
    next_group = testloader.TestGroup(
        deque([next_test]), "", "test262", testloader.GroupMetadata("/next"))
    test_queue = testloader.WriteQueue()
    test_queue.put(next_group)
    reader = test_queue.to_read()

    implementation = testrunner.TestImplementation(
        Mock(), {"timeout_multiplier": 1}, Mock(), {})
    manager = testrunner.TestRunnerManager(
        "test", 0, reader, {("", "test262"): implementation},
        testrunner.StopFlag(1), restart_on_unexpected=False,
        restart_on_new_group=False, recording=Mock())
    manager.logger = Mock()
    manager.remote_queue = SimpleQueue()
    manager.test_source = testrunner.TestSource(manager.logger, reader)
    manager.state = testrunner.RunningState(current_group, current_test)
    manager.run_count = 1
    manager.executor_implementation = testrunner.ExecutorImplementation(
        Mock(), {"timeout_multiplier": 1}, Mock(), {})

    browser = Mock(spec=Browser)
    browser.restart_on_test_type_change.return_value = False
    browser.executor_browser.return_value = (Mock(), {})
    browser.check_crash.return_value = False
    manager.browser = testrunner.BrowserManager(manager.logger, browser, SimpleQueue())
    return manager


@pytest.mark.parametrize("status", ["CRASH", "EXTERNAL-TIMEOUT", "INTERNAL-ERROR"])
def test_restart_after_error_before_test_type_change(
    manager: testrunner.TestRunnerManager, status: str
) -> None:
    test = manager.state.test
    state = manager.test_ended(test, (test.make_result(status, None), []))

    assert isinstance(state, testrunner.RestartingState)
    assert state.test_group is manager.test_source.current_group
    assert state.test.id == "/next.html"
    assert manager.remote_queue.empty()


def test_switch_executor_after_success(manager: testrunner.TestRunnerManager) -> None:
    test = manager.state.test
    state = manager.test_ended(test, (test.make_result("OK", None), []))

    assert isinstance(state, testrunner.SwitchingExecutorState)
    assert state.test_group is manager.test_source.current_group
    assert state.test.id == "/next.html"
    command, args = manager.remote_queue.get_nowait()
    assert command == "switch_executor"
    assert args == (manager.executor_implementation,)
    assert manager.remote_queue.empty()
