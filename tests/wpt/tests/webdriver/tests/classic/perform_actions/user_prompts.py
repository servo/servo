# META: timeout=long

import pytest
from webdriver import error

from tests.classic.perform_actions.support.refine import get_keys
from tests.support.asserts import assert_error, assert_success, assert_dialog_handled
from tests.support.sync import Poll
from . import perform_actions

actions = [{
    "type": "key",
    "id": "foobar",
    "actions": [
        {"type": "keyDown", "value": "a"},
        {"type": "keyUp", "value": "a"},
    ]
}]


@pytest.fixture
def check_user_prompt_closed_without_exception(session, create_dialog, key_chain, key_reporter):
    def check_user_prompt_closed_without_exception(dialog_type, retval):
        create_dialog(dialog_type, text=dialog_type)

        response = perform_actions(session, actions)
        assert_success(response)

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

        assert get_keys(key_reporter) == "a"

    return check_user_prompt_closed_without_exception


@pytest.fixture
def check_user_prompt_closed_with_exception(session, create_dialog, key_chain, key_reporter):
    def check_user_prompt_closed_with_exception(dialog_type, retval):
        create_dialog(dialog_type, text=dialog_type)

        response = perform_actions(session, actions)
        assert_error(response, "unexpected alert open")

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

        assert get_keys(key_reporter) == ""

    return check_user_prompt_closed_with_exception


@pytest.fixture
def check_user_prompt_not_closed_but_exception(session, create_dialog, key_reporter):
    def check_user_prompt_not_closed_but_exception(dialog_type):
        create_dialog(dialog_type, text=dialog_type)

        response = perform_actions(session, actions)
        assert_error(response, "unexpected alert open")

        assert session.alert.text == dialog_type
        session.alert.dismiss()

        assert get_keys(key_reporter) == ""

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


def test_dismissed_beforeunload(session, url, mouse_chain):
    page_beforeunload = url("/webdriver/tests/support/html/beforeunload.html")
    page_target = url("/webdriver/tests/support/html/default.html")

    session.url = page_beforeunload
    input = session.find.css("input", all=False)
    input.send_keys("bar")

    link = session.find.css("a", all=False)

    mouse_chain.pointer_move(0, 0, origin=link) \
        .click() \
        .perform()

    wait = Poll(
        session,
        timeout=5,
        message="Target page did not load")
    wait.until(lambda s: s.url == page_target)

    # navigation auto-dismissed beforeunload prompt
    with pytest.raises(error.NoSuchAlertException):
        session.alert.text
