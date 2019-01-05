# META: timeout=long

from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import document_hidden, is_fullscreen


def maximize(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/maximize".format(**vars(session)))


def test_no_browsing_context(session, closed_window):
    response = maximize(session)
    assert_error(response, "no such window")


def test_fully_exit_fullscreen(session):
    session.window.fullscreen()
    assert is_fullscreen(session)

    response = maximize(session)
    assert_success(response)
    assert not is_fullscreen(session)


def test_restore_the_window(session):
    session.window.minimize()
    assert document_hidden(session)

    response = maximize(session)
    assert_success(response)


def test_maximize(session):
    before_size = session.window.size

    response = maximize(session)
    assert_success(response)

    assert before_size != session.window.size


def test_payload(session):
    before_size = session.window.size

    response = maximize(session)

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

    assert before_size != session.window.size


def test_maximize_twice_is_idempotent(session):
    first_response = maximize(session)
    assert_success(first_response)
    max_size = session.window.size

    second_response = maximize(session)
    assert_success(second_response)
    assert session.window.size == max_size


def test_maximize_when_resized_to_max_size(session):
    # Determine the largest available window size by first maximising
    # the window and getting the window rect dimensions.
    #
    # Then resize the window to the maximum available size.
    session.end()
    session.window.maximize()
    available = session.window.size
    session.end()

    session.window.size = available

    # In certain window managers a window extending to the full available
    # dimensions of the screen may not imply that the window is maximised,
    # since this is often a special state.  If a remote end expects a DOM
    # resize event, this may not fire if the window has already reached
    # its expected dimensions.
    before = session.window.size
    session.window.maximize()
    assert session.window.size == before
