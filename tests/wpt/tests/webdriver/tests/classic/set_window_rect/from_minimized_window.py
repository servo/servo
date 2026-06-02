from tests.support.asserts import assert_success
from tests.support.helpers import document_hidden


def set_window_rect(session, rect):
    return session.transport.send(
        "POST", "session/{session_id}/window/rect".format(**vars(session)), rect
    )


# This test is moved to a separate file to not affect other test results
# on Wayland, since at least for Firefox restoring from minimized state
# doesn't work.
def test_restore_from_minimized(session):
    assert not document_hidden(session)

    original = session.window.rect
    target_rect = {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"] + 50,
        "height": original["height"] + 50,
    }

    session.window.minimize()
    assert document_hidden(session)

    response = set_window_rect(session, target_rect)
    value = assert_success(response, session.window.rect)

    assert not document_hidden(session)
    assert value == target_rect
