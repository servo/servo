import pytest

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success
from tests.support.inline import inline


def get_current_url(session):
    return session.transport.send("GET", "session/%s/url" % session.session_id)


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_handle_prompt_accept(session, create_dialog, dialog_type, retval):
    session.url = inline("<p id=1>")
    expected_url = session.url

    create_dialog(dialog_type, text=dialog_type)

    response = get_current_url(session)
    assert_success(response, expected_url)

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)


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
    session.url = inline("<p id=1>")

    create_dialog(dialog_type, text=dialog_type)

    response = get_current_url(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)
