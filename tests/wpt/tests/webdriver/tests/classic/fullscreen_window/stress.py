# META: timeout=long

# Longer timeout required due to a bug in Chrome:
# https://bugs.chromium.org/p/chromedriver/issues/detail?id=4642#c4

import pytest

from tests.support.asserts import assert_success
from tests.support.helpers import is_fullscreen


def fullscreen_window(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/fullscreen".format(**vars(session)))


@pytest.mark.parametrize("i", range(5))
def test_stress(session, i):
    assert not is_fullscreen(session)
    response = fullscreen_window(session)
    assert_success(response)
    assert is_fullscreen(session)
