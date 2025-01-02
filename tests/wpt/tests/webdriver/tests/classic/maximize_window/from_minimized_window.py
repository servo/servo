from tests.support.asserts import assert_success
from tests.support.helpers import document_hidden, is_maximized, is_not_maximized


def maximize(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/maximize".format(**vars(session))
    )


# This test is moved to a separate file to not affect other test results
# on Wayland, since at least for Firefox restoring from minimized state
# doesn't work.
def test_restore_from_minimized(session):
    assert is_not_maximized(session)
    original = session.window.rect

    session.window.minimize()
    assert document_hidden(session)
    assert is_not_maximized(session)

    response = maximize(session)
    assert_success(response, session.window.rect)

    assert is_maximized(session, original)
    assert not document_hidden(session)
