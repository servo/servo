import pytest

from tests.support.asserts import assert_error, assert_dialog_handled, assert_success
from tests.support.inline import inline


def is_element_selected(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/selected".format(
            session_id=session.session_id,
            element_id=element_id))


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_accept(session, create_dialog, dialog_type):
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(dialog_type, text="dialog")

    response = is_element_selected(session, element.id)
    assert_success(response, False)

    assert_dialog_handled(session, expected_text="dialog")


def test_handle_prompt_accept_and_notify():
    """TODO"""


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_dismiss(session, create_dialog, dialog_type):
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(dialog_type, text="dialog")

    response = is_element_selected(session, element.id)
    assert_success(response, False)

    assert_dialog_handled(session, expected_text="dialog")


def test_handle_prompt_dismiss_and_notify():
    """TODO"""


def test_handle_prompt_ignore():
    """TODO"""


@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_default(session, create_dialog, dialog_type):
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(dialog_type, text="dialog")

    response = is_element_selected(session, element.id)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text="dialog")
