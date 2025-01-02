from tests.support.asserts import assert_success
from tests.support.helpers import (
    document_hidden,
    is_fullscreen,
)


def fullscreen(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/fullscreen".format(**vars(session))
    )


# This test is moved to a separate file to not affect other test results
# on Wayland, since at least for Firefox restoring from minimized state
# doesn't work.
def test_fullscreen_from_minimized_window(session, screen_size):
    assert not document_hidden(session)

    session.window.minimize()
    assert document_hidden(session)
    assert not is_fullscreen(session)

    response = fullscreen(session)
    assert_success(response, session.window.rect)
    assert not document_hidden(session)
    assert is_fullscreen(session)

    assert session.window.size == screen_size
