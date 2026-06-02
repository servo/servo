# META: timeout=long

# Longer timeout required due to a bug in Chrome:
# https://bugs.chromium.org/p/chromedriver/issues/detail?id=4642#c4

from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import (
    document_hidden,
    is_fullscreen,
    is_maximized,
    is_not_maximized
)


def maximize(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/maximize".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = maximize(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = maximize(session)
    assert_success(response)


def test_response_payload(session):
    assert is_not_maximized(session)
    original = session.window.rect

    response = maximize(session)
    value = assert_success(response, session.window.rect)

    assert is_maximized(session, original)

    assert isinstance(value, dict)
    assert isinstance(value.get("x"), int)
    assert isinstance(value.get("y"), int)
    assert isinstance(value.get("width"), int)
    assert isinstance(value.get("height"), int)


def test_fully_exit_fullscreen(session):
    assert is_not_maximized(session)
    original = session.window.rect

    session.window.fullscreen()
    assert is_fullscreen(session)

    response = maximize(session)
    assert_success(response, session.window.rect)

    assert is_maximized(session, original)
    assert not document_hidden(session)


def test_maximize_from_normal_window(session):
    assert is_not_maximized(session)
    original = session.window.rect

    response = maximize(session)
    assert_success(response, session.window.rect)

    assert is_maximized(session, original)
    assert not document_hidden(session)


def test_maximize_with_window_already_at_maximum_size(session, available_screen_size):
    assert is_not_maximized(session)
    original = session.window.rect

    # Resize the window to the maximum available size.
    session.window.size = available_screen_size
    assert session.window.size == available_screen_size

    # In certain window managers a window extending to the full available
    # dimensions of the screen may not imply that the window is maximised,
    # since this is often a special state.  If a remote end expects a DOM
    # resize event, this may not fire if the window has already reached
    # its expected dimensions.
    response = maximize(session)
    assert_success(response, session.window.rect)

    assert is_maximized(session, original)
    assert not document_hidden(session)


def test_maximize_twice_is_idempotent(session):
    assert is_not_maximized(session)
    original = session.window.rect

    first_response = maximize(session)
    assert_success(first_response, session.window.rect)

    assert is_maximized(session, original)
    assert not document_hidden(session)

    second_response = maximize(session)
    assert_success(second_response, session.window.rect)

    assert is_maximized(session, original)
    assert not document_hidden(session)
