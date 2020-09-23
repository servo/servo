import pytest
from webdriver import error

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


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


def test_close_browsing_context_with_dismissed_beforeunload_prompt(session):
    original_handles = session.handles

    new_handle = session.new_window()
    session.window_handle = new_handle

    session.url = inline("""
      <input type="text">
      <script>
        window.addEventListener("beforeunload", function (event) {
          event.preventDefault();
        });
      </script>
    """)

    session.find.css("input", all=False).send_keys("foo")

    response = close(session)
    handles = assert_success(response, original_handles)
    assert session.handles == original_handles
    assert new_handle not in handles

    # A beforeunload prompt has to be automatically dismissed
    with pytest.raises(error.NoSuchWindowException):
        session.alert.text


def test_close_last_browsing_context(session):
    assert len(session.handles) == 1
    response = close(session)

    assert_success(response, [])

    # With no more open top-level browsing contexts, the session is closed.
    session.session_id = None
