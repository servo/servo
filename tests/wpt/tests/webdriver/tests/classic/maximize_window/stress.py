# META: timeout=long

# Longer timeout required due to a bug in Chrome:
# https://bugs.chromium.org/p/chromedriver/issues/detail?id=4642#c4

import time

import pytest

from tests.support.asserts import assert_success


def maximize_window(session):
    response = session.transport.send(
        "POST", "session/{session_id}/window/maximize".format(**vars(session)))
    rect = assert_success(response)
    return (rect["width"], rect["height"])


@pytest.mark.parametrize("i", range(5))
def test_stress(session, i):
    """
    Without defining the heuristics of each platform WebDriver runs on,
    the best we can do is to test that maximization occurs synchronously.

    Not all systems and window managers support maximizing the window,
    but they are expected to do their best.  The minimum requirement
    is that the maximized window is larger than its original size.

    To ensure the maximization happened synchronously, we test
    that the size hasn't changed after a short amount of time,
    using a thread suspend.  This is not ideal, but the best we
    can do given the level of platform ambiguity implied by WebDriver.
    """
    session.window.size = (100, 100)
    session.window.position = (0, 0)
    original_size = session.window.size

    size_after_maximize = maximize_window(session)
    assert size_after_maximize > original_size

    t_end = time.time() + 3
    while time.time() < t_end:
        assert session.window.size == size_after_maximize
        time.sleep(.1)
