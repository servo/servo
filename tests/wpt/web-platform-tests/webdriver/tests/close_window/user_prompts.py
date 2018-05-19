# META: timeout=long

from tests.support.asserts import assert_error, assert_dialog_handled
from tests.support.fixtures import create_dialog, create_window
from tests.support.inline import inline


def close(session):
    return session.transport.send(
        "DELETE", "session/{session_id}/window".format(**vars(session)))


def test_handle_prompt_dismiss_and_notify():
    """TODO"""


def test_handle_prompt_accept_and_notify():
    """TODO"""


def test_handle_prompt_ignore():
    """TODO"""


def test_handle_prompt_accept(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})
    original_handle = session.window_handle

    session.window_handle = create_window(session)()
    session.url = inline("<title>WD doc title</title>")

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")
    response = close(session)
    assert response.status == 200

    # Asserting that the dialog was handled requires valid top-level browsing
    # context, so we must switch to the original window.
    session.window_handle = original_handle
    assert_dialog_handled(session, "dismiss #1")

    session.window_handle = create_window(session)()
    session.url = inline("<title>WD doc title</title>")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")
    response = close(session)
    assert response.status == 200

    # Asserting that the dialog was handled requires valid top-level browsing
    # context, so we must switch to the original window.
    session.window_handle = original_handle
    assert_dialog_handled(session, "dismiss #2")

    session.window_handle = create_window(session)()
    session.url = inline("<title>WD doc title</title>")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")
    response = close(session)
    assert response.status == 200

    # Asserting that the dialog was handled requires valid top-level browsing
    # context, so we must switch to the original window.
    session.window_handle = original_handle
    assert_dialog_handled(session, "dismiss #3")


def test_handle_prompt_missing_value(session, create_dialog, create_window):
    session.window_handle = create_window()

    session.url = inline("<title>WD doc title</title>")
    create_dialog("alert", text="dismiss #1", result_var="dismiss1")

    response = close(session)

    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog("confirm", text="dismiss #2", result_var="dismiss2")

    response = close(session)
    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog("prompt", text="dismiss #3", result_var="dismiss3")

    response = close(session)
    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")
