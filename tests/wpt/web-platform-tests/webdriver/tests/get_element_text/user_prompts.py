# META: timeout=long

import pytest

from tests.support.asserts import assert_error, assert_success, assert_dialog_handled
from tests.support.inline import inline


def get_element_text(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/text".format(
            session_id=session.session_id,
            element_id=element_id))


@pytest.fixture
def check_user_prompt_closed_without_exception(session, create_dialog):
    def check_user_prompt_closed_without_exception(dialog_type, retval):
        session.url = inline("<p id=foo>bar</p>")
        element = session.find.css("#foo", all=False)

        create_dialog(dialog_type, text=dialog_type)

        response = get_element_text(session, element.id)
        assert_success(response, "bar")

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    return check_user_prompt_closed_without_exception


@pytest.fixture
def check_user_prompt_closed_with_exception(session, create_dialog):
    def check_user_prompt_closed_with_exception(dialog_type, retval):
        session.url = inline("<p id=foo>bar</p>")
        element = session.find.css("#foo", all=False)

        create_dialog(dialog_type, text=dialog_type)

        response = get_element_text(session, element.id)
        assert_error(response, "unexpected alert open")

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    return check_user_prompt_closed_with_exception


@pytest.fixture
def check_user_prompt_not_closed_but_exception(session, create_dialog):
    def check_user_prompt_not_closed_but_exception(dialog_type):
        session.url = inline("<p id=foo>bar</p>")
        element = session.find.css("#foo", all=False)

        create_dialog(dialog_type, text=dialog_type)

        response = get_element_text(session, element.id)
        assert_error(response, "unexpected alert open")

        assert session.alert.text == dialog_type
        session.alert.dismiss()

    return check_user_prompt_not_closed_but_exception


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_accept(check_user_prompt_closed_without_exception, dialog_type, retval):
    check_user_prompt_closed_without_exception(dialog_type, retval)


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_accept_and_notify(check_user_prompt_closed_with_exception, dialog_type, retval):
    check_user_prompt_closed_with_exception(dialog_type, retval)


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_dismiss(check_user_prompt_closed_without_exception, dialog_type, retval):
    check_user_prompt_closed_without_exception(dialog_type, retval)


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_dismiss_and_notify(check_user_prompt_closed_with_exception, dialog_type, retval):
    check_user_prompt_closed_with_exception(dialog_type, retval)


@pytest.mark.capabilities({"unhandledPromptBehavior": "ignore"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_ignore(check_user_prompt_not_closed_but_exception, dialog_type):
    check_user_prompt_not_closed_but_exception(dialog_type)


@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_default(check_user_prompt_closed_with_exception, dialog_type, retval):
    check_user_prompt_closed_with_exception(dialog_type, retval)
