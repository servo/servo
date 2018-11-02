from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import is_fullscreen


def fullscreen(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/fullscreen".format(**vars(session)))


def test_no_browsing_context(session, closed_window):
    response = fullscreen(session)
    assert_error(response, "no such window")


def test_fullscreen(session):
    response = fullscreen(session)
    assert_success(response)

    assert is_fullscreen(session)


def test_payload(session):
    response = fullscreen(session)

    assert response.status == 200
    assert isinstance(response.body["value"], dict)

    value = response.body["value"]
    assert "width" in value
    assert "height" in value
    assert "x" in value
    assert "y" in value
    assert isinstance(value["width"], int)
    assert isinstance(value["height"], int)
    assert isinstance(value["x"], int)
    assert isinstance(value["y"], int)


def test_fullscreen_twice_is_idempotent(session):
    assert not is_fullscreen(session)

    first_response = fullscreen(session)
    assert_success(first_response)
    assert is_fullscreen(session)

    second_response = fullscreen(session)
    assert_success(second_response)
    assert is_fullscreen(session)
