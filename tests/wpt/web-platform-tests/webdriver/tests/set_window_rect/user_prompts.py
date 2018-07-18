import pytest

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success


def set_window_rect(session, rect):
    return session.transport.send(
        "POST", "session/{session_id}/window/rect".format(**vars(session)),
        rect)


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_accept(session, create_dialog, dialog_type):
    original = session.window.rect

    create_dialog(dialog_type, text="dialog")

    response = set_window_rect(session, {"x": original["x"], "y": original["y"]})
    assert_success(response)

    assert_dialog_handled(session, expected_text="dialog")


def test_handle_prompt_accept_and_notify():
    """TODO"""


def test_handle_prompt_dismiss():
    """TODO"""


def test_handle_prompt_dismiss_and_notify():
    """TODO"""


def test_handle_prompt_ignore():
    """TODO"""


@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_default(session, create_dialog, dialog_type):
    original = session.window.rect

    create_dialog(dialog_type, text="dialog")

    response = set_window_rect(session, {"x": original["x"],
                                         "y": original["y"]})
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text="dialog")
