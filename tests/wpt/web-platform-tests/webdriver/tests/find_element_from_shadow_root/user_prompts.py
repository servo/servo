# META: timeout=long

import pytest

from tests.support.asserts import (
    assert_error,
    assert_same_element,
    assert_success,
    assert_dialog_handled,
)


def find_element(session, shadow_id, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/shadow/{shadow_id}/element".format(
            session_id=session.session_id,
            shadow_id=shadow_id),
        {"using": using, "value": value})


@pytest.fixture
def check_user_prompt_closed_without_exception(session, create_dialog, inline, get_shadow_page):
    def check_user_prompt_closed_without_exception(dialog_type, retval):
        session.url = inline(get_shadow_page("<div><p>bar</p><div>"))
        outer_element = session.find.css("custom-shadow-element", all=False)
        shadow_root = outer_element.shadow_root
        inner_element = session.execute_script("return arguments[0].shadowRoot.querySelector('p')",
                                               args=(outer_element,))

        create_dialog(dialog_type, text=dialog_type)

        response = find_element(session, shadow_root.id, "css selector", "p")
        value = assert_success(response)

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

        assert_same_element(session, value, inner_element)

    return check_user_prompt_closed_without_exception


@pytest.fixture
def check_user_prompt_closed_with_exception(session, create_dialog, inline, get_shadow_page):
    def check_user_prompt_closed_with_exception(dialog_type, retval):
        session.url = inline(get_shadow_page("<div><p>bar</p><div>"))
        outer_element = session.find.css("custom-shadow-element", all=False)
        shadow_root = outer_element.shadow_root

        create_dialog(dialog_type, text=dialog_type)

        response = find_element(session, shadow_root.id, "css selector", "p")
        assert_error(response, "unexpected alert open")

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    return check_user_prompt_closed_with_exception


@pytest.fixture
def check_user_prompt_not_closed_but_exception(session, create_dialog, inline, get_shadow_page):
    def check_user_prompt_not_closed_but_exception(dialog_type):
        session.url = inline(get_shadow_page("<div><p>bar</p><div>"))
        outer_element = session.find.css("custom-shadow-element", all=False)
        shadow_root = outer_element.shadow_root

        create_dialog(dialog_type, text=dialog_type)

        response = find_element(session, shadow_root.id, "css selector", "p")
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
