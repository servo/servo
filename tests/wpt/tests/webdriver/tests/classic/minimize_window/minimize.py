# META: timeout=long

# Longer timeout required due to a bug in Chrome:
# https://bugs.chromium.org/p/chromedriver/issues/detail?id=4642#c4

from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import document_hidden, is_fullscreen, is_maximized


def minimize(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/minimize".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = minimize(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = minimize(session)
    assert_success(response)


def test_response_payload(session):
    assert not document_hidden(session)

    response = minimize(session)
    value = assert_success(response, session.window.rect)

    assert document_hidden(session)

    assert isinstance(value, dict)
    assert isinstance(value.get("x"), int)
    assert isinstance(value.get("y"), int)
    assert isinstance(value.get("width"), int)
    assert isinstance(value.get("height"), int)


def test_restore_from_fullscreen(session):
    assert not document_hidden(session)

    session.window.fullscreen()
    assert is_fullscreen(session)
    assert not document_hidden(session)

    response = minimize(session)
    assert_success(response, session.window.rect)
    assert not is_fullscreen(session)
    assert document_hidden(session)


def test_restore_from_maximized(session):
    assert not document_hidden(session)

    session.window.maximize()
    assert is_maximized(session)
    assert not document_hidden(session)

    response = minimize(session)
    assert_success(response, session.window.rect)
    assert not is_maximized(session)
    assert document_hidden(session)


def test_minimize_from_normal_window(session):
    assert not document_hidden(session)

    response = minimize(session)
    assert_success(response, session.window.rect)
    assert document_hidden(session)


def test_minimize_twice_is_idempotent(session):
    assert not document_hidden(session)

    first_response = minimize(session)
    assert_success(first_response, session.window.rect)
    assert document_hidden(session)

    second_response = minimize(session)
    assert_success(second_response, session.window.rect)
    assert document_hidden(session)
