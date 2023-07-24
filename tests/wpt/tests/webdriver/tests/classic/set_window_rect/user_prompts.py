# META: timeout=long

import pytest

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success


def set_window_rect(session, rect):
    return session.transport.send(
        "POST", "session/{session_id}/window/rect".format(**vars(session)),
        rect)


@pytest.fixture
def check_user_prompt_closed_without_exception(session, create_dialog):
    def check_user_prompt_closed_without_exception(dialog_type, retval):
        original_rect = session.window.rect

        create_dialog(dialog_type, text=dialog_type)

        response = set_window_rect(session, {
            "x": original_rect["x"] + 10, "y": original_rect["y"] + 10})
        assert_success(response)

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

        assert session.window.rect != original_rect

    return check_user_prompt_closed_without_exception


@pytest.fixture
def check_user_prompt_closed_with_exception(session, create_dialog):
    def check_user_prompt_closed_with_exception(dialog_type, retval):
        original_rect = session.window.rect

        create_dialog(dialog_type, text=dialog_type)

        response = set_window_rect(session, {
            "x": original_rect["x"] + 10, "y": original_rect["y"] + 10})
        assert_error(response, "unexpected alert open")

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

        assert session.window.rect == original_rect

    return check_user_prompt_closed_with_exception


@pytest.fixture
def check_user_prompt_not_closed_but_exception(session, create_dialog):
    def check_user_prompt_not_closed_but_exception(dialog_type):
        original_rect = session.window.rect

        create_dialog(dialog_type, text=dialog_type)

        response = set_window_rect(session, {
            "x": original_rect["x"] + 10, "y": original_rect["y"] + 10})
        assert_error(response, "unexpected alert open")

        assert session.alert.text == dialog_type
        session.alert.dismiss()

        assert session.window.rect == original_rect

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
