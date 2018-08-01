# META: timeout=long

import pytest

from webdriver.error import StaleElementReferenceException

from tests.support.inline import inline
from tests.support.asserts import assert_dialog_handled, assert_error, assert_success


def refresh(session):
    return session.transport.send(
        "POST", "session/{session_id}/refresh".format(**vars(session)))


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_accept(session, create_dialog, dialog_type):
    session.url = inline("<div id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(dialog_type, text=dialog_type)

    response = refresh(session)
    assert_success(response)

    # retval not testable for confirm and prompt because window has been reloaded
    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=None)

    with pytest.raises(StaleElementReferenceException):
        element.property("id")


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_handle_prompt_accept_and_notify(session, create_dialog, dialog_type, retval):
    session.url = inline("<div id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(dialog_type, text=dialog_type)

    response = refresh(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert element.property("id") == "foo"


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_handle_prompt_dismiss(session, create_dialog, dialog_type):
    session.url = inline("<div id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(dialog_type, text=dialog_type)

    response = refresh(session)
    assert_success(response)

    # retval not testable for confirm and prompt because window has been reloaded
    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=None)

    with pytest.raises(StaleElementReferenceException):
        element.property("id")


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_handle_prompt_dissmiss_and_notify(session, create_dialog, dialog_type, retval):
    session.url = inline("<div id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(dialog_type, text=dialog_type)

    response = refresh(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert element.property("id") == "foo"


def test_handle_prompt_ignore():
    """TODO"""


@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_handle_prompt_default(session, create_dialog, dialog_type, retval):
    session.url = inline("<div id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(dialog_type, text=dialog_type)

    response = refresh(session)
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert element.property("id") == "foo"
