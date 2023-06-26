# META: timeout=long

import pytest

from tests.support.asserts import assert_success


def set_timeouts(session, timeouts):
    return session.transport.send(
        "POST", "session/{session_id}/timeouts".format(**vars(session)),
        timeouts)


@pytest.fixture
def check_user_prompt_not_closed(session, create_dialog):
    def check_user_prompt_not_closed(dialog_type):
        create_dialog(dialog_type, text=dialog_type)

        response = set_timeouts(session, {"script": 100})
        assert_success(response)

        assert session.alert.text == dialog_type
        session.alert.dismiss()

        assert session.timeouts.script == 100

    return check_user_prompt_not_closed


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_accept(check_user_prompt_not_closed, dialog_type):
    check_user_prompt_not_closed(dialog_type)


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept and notify"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_accept_and_notify(check_user_prompt_not_closed, dialog_type):
    check_user_prompt_not_closed(dialog_type)


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_dismiss(check_user_prompt_not_closed, dialog_type):
    check_user_prompt_not_closed(dialog_type)


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss and notify"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_dismiss_and_notify(check_user_prompt_not_closed, dialog_type):
    check_user_prompt_not_closed(dialog_type)


@pytest.mark.capabilities({"unhandledPromptBehavior": "ignore"})
@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_ignore(check_user_prompt_not_closed, dialog_type):
    check_user_prompt_not_closed(dialog_type)


@pytest.mark.parametrize("dialog_type", ["alert", "confirm", "prompt"])
def test_default(check_user_prompt_not_closed, dialog_type):
    check_user_prompt_not_closed(dialog_type)
