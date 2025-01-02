import pytest
from webdriver import error

from tests.support.asserts import assert_error, assert_success


def close(session):
    return session.transport.send(
        "DELETE", "session/{session_id}/window".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = close(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, url):
    new_handle = session.new_window()

    session.url = url("/webdriver/tests/support/html/frames.html")

    subframe = session.find.css("#sub-frame", all=False)
    session.switch_frame(subframe)

    frame = session.find.css("#delete-frame", all=False)
    session.switch_frame(frame)

    button = session.find.css("#remove-parent", all=False)
    button.click()

    response = close(session)
    handles = assert_success(response)
    assert handles == [new_handle]


def test_close_browsing_context(session):
    original_handles = session.handles

    new_handle = session.new_window()
    session.window_handle = new_handle

    response = close(session)
    handles = assert_success(response, original_handles)
    assert session.handles == original_handles
    assert new_handle not in handles


@pytest.mark.parametrize("type", ["tab", "window"])
def test_close_browsing_context_with_accepted_beforeunload_prompt(session, url, type):
    original_handles = session.handles

    new_handle = session.new_window(type_hint=type)
    session.window_handle = new_handle

    session.url = url("/webdriver/tests/support/html/beforeunload.html")

    element = session.find.css("input", all=False)
    element.send_keys("bar")

    response = close(session)
    handles = assert_success(response, original_handles)
    assert session.handles == original_handles
    assert new_handle not in handles

    # A beforeunload prompt has to be automatically accepted
    with pytest.raises(error.NoSuchWindowException):
        session.alert.text


def test_close_last_browsing_context(session):
    assert len(session.handles) == 1
    response = close(session)

    assert_success(response, [])

    # With no more open top-level browsing contexts, the session is closed.
    session.session_id = None


def test_element_usage_after_closing_browsing_context(session, inline):
    session.url = inline("<p id='a'>foo")
    session.find.css("p", all=False)
    first = session.window_handle

    second = session.new_window(type_hint="tab")
    session.window_handle = second

    session.url = inline("<p id='b'>other")
    b = session.find.css("p", all=False)

    session.window_handle = first
    response = close(session)
    assert_success(response)
    assert len(session.handles) == 1

    session.window_handle = second
    assert b.attribute("id") == "b"
