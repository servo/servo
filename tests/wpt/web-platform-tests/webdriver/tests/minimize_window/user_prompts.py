import pytest

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success


def minimize(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/minimize".format(**vars(session)))


def is_minimized(session):
    return session.execute_script("return document.hidden")


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_handle_prompt_accept(session, create_dialog, dialog_type, retval):
    assert not is_minimized(session)

    create_dialog(dialog_type, text=dialog_type)

    response = minimize(session)
    assert_success(response)

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert is_minimized(session)


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
def test_handle_prompt_default(session, create_dialog, dialog_type, retval):
    assert not is_minimized(session)

    create_dialog(dialog_type, text=dialog_type)

    response = minimize(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert not is_minimized(session)
