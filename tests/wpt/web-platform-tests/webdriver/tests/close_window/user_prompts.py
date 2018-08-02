# META: timeout=long
import pytest

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success


def close(session):
    return session.transport.send(
        "DELETE", "session/{session_id}/window".format(**vars(session)))


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_accept(session, create_dialog, create_window, dialog_type):
    original_handle = session.window_handle
    new_handle = create_window()
    session.window_handle = new_handle

    create_dialog(dialog_type, text=dialog_type)

    response = close(session)
    assert_success(response)

    # Asserting that the dialog was handled requires valid top-level browsing
    # context, so we must switch to the original window.
    session.window_handle = original_handle

    # retval not testable for confirm and prompt because window is gone
    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=None)

    assert new_handle not in session.handles


def test_handle_prompt_accept_and_notify():
    """TODO"""


def test_handle_prompt_dismiss():
    """TODO"""


def test_handle_prompt_dismiss_and_notify():
    """TODO"""


def test_handle_prompt_ignore():
    """TODO"""


@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_handle_prompt_default(session, create_dialog, create_window, dialog_type, retval):
    new_handle = create_window()
    session.window_handle = new_handle

    create_dialog(dialog_type, text=dialog_type)

    response = close(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert new_handle in session.handles
