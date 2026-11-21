import pytest

from tests.support.classic.asserts import assert_success
from tests.support.classic.helpers import wait_for_new_handle
from tests.support.sync import Poll


def get_window_handles(session):
    return session.transport.send(
        "GET", "session/{session_id}/window/handles".format(**vars(session)))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_popup_with_opener(session, inline, domain):
    original_handle = session.window_handle
    session.url = inline("<p>opener</p>")
    popup_url = inline("<p>popup</p>", domain=domain)

    handles_before = session.handles
    session.execute_script("window.open(arguments[0])", args=[popup_url])
    popup_handle = wait_for_new_handle(session, handles_before)

    # Make sure that the popup has loaded. A browser may put a cross-site popup in a separate process,
    # and keep a proxy for each window in the other's process. Neither must make a window appear twice.
    session.window_handle = popup_handle
    Poll(session, timeout=5, message="The popup did not load").until(lambda s: s.url == popup_url)
    session.window_handle = original_handle

    response = get_window_handles(session)
    value = assert_success(response)

    assert len(value) == 2
    assert original_handle in value
    assert popup_handle in value
