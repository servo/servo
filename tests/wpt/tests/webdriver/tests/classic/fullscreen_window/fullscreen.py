from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import is_fullscreen, is_maximized, is_not_maximized


def fullscreen(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/fullscreen".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = fullscreen(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = fullscreen(session)
    assert_success(response)


def test_response_payload(session, screen_size):
    assert not is_fullscreen(session)

    response = fullscreen(session)
    value = assert_success(response)

    assert is_fullscreen(session)

    assert isinstance(value, dict)
    assert isinstance(value.get("x"), int)
    assert isinstance(value.get("y"), int)
    assert isinstance(value.get("width"), int)
    assert isinstance(value.get("height"), int)


def test_fullscreen_from_normal_window(session, screen_size):
    assert not is_fullscreen(session)

    response = fullscreen(session)
    assert_success(response, session.window.rect)

    assert is_fullscreen(session)
    assert session.window.size == screen_size


def test_fullscreen_from_maximized_window(session, screen_size):
    assert not is_fullscreen(session)
    original = session.window.rect

    session.window.maximize()
    assert is_maximized(session, original)

    response = fullscreen(session)
    assert_success(response, session.window.rect)
    assert is_not_maximized(session)

    assert session.window.size == screen_size


def test_fullscreen_twice_is_idempotent(session, screen_size):
    assert not is_fullscreen(session)

    first_response = fullscreen(session)
    assert_success(first_response, session.window.rect)
    assert is_fullscreen(session)
    assert session.window.size == screen_size

    second_response = fullscreen(session)
    assert_success(second_response, session.window.rect)
    assert is_fullscreen(session)
    assert session.window.size == screen_size
