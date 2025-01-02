# META: timeout=long

import pytest
from webdriver import error

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


@pytest.fixture
def check_beforeunload_implicitly_accepted(session, url):
    def check_beforeunload_implicitly_accepted():
        page_beforeunload = url(
            "/webdriver/tests/support/html/beforeunload.html")
        page_target = url("/webdriver/tests/support/html/default.html")

        session.url = page_beforeunload
        input = session.find.css("input", all=False)
        input.send_keys("bar")

        link = session.find.css("a", all=False)
        response = element_click(session, link)
        assert_success(response)

        assert session.url == page_target

        # navigation auto-dismissed beforeunload prompt
        with pytest.raises(error.NoSuchAlertException):
            session.alert.text

    return check_beforeunload_implicitly_accepted


@pytest.fixture
def check_user_prompt_closed_without_exception(session, create_dialog, inline):
    def check_user_prompt_closed_without_exception(dialog_type, retval):
        session.url = inline("<input type=text>")
        element = session.find.css("input", all=False)

        create_dialog(dialog_type, text=dialog_type)

        response = element_click(session, element)
        assert_success(response)

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

        assert session.active_element == element

    return check_user_prompt_closed_without_exception


@pytest.fixture
def check_user_prompt_closed_with_exception(session, create_dialog, inline):
    def check_user_prompt_closed_with_exception(dialog_type, retval):
        session.url = inline("<input type=text>")
        element = session.find.css("input", all=False)

        create_dialog(dialog_type, text=dialog_type)

        response = element_click(session, element)
        assert_error(response, "unexpected alert open")

        assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

        assert session.active_element != element

    return check_user_prompt_closed_with_exception


@pytest.fixture
def check_user_prompt_not_closed_but_exception(session, create_dialog, inline):
    def check_user_prompt_not_closed_but_exception(dialog_type):
        session.url = inline("<input type=text>")
        element = session.find.css("input", all=False)

        create_dialog(dialog_type, text=dialog_type)

        response = element_click(session, element)
        assert_error(response, "unexpected alert open")

        assert session.alert.text == dialog_type
        session.alert.dismiss()

        assert session.active_element != element

    return check_user_prompt_not_closed_but_exception


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("beforeunload", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_accept(
    check_beforeunload_implicitly_accepted,
    check_user_prompt_closed_without_exception,
    dialog_type,
    retval
):
    if dialog_type == "beforeunload":
        check_beforeunload_implicitly_accepted()
    else:
        check_user_prompt_closed_without_exception(dialog_type, retval)


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("beforeunload", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_accept_and_notify(
    check_beforeunload_implicitly_accepted,
    check_user_prompt_closed_with_exception,
    dialog_type,
    retval
):
    if dialog_type == "beforeunload":
        check_beforeunload_implicitly_accepted()
    else:
        check_user_prompt_closed_with_exception(dialog_type, retval)


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("beforeunload", None),
    ("confirm", False),
    ("prompt", None),
])
def test_dismiss(
    check_beforeunload_implicitly_accepted,
    check_user_prompt_closed_without_exception,
    dialog_type,
    retval,
):
    if dialog_type == "beforeunload":
        check_beforeunload_implicitly_accepted()
    else:
        check_user_prompt_closed_without_exception(dialog_type, retval)


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("beforeunload", None),
    ("confirm", False),
    ("prompt", None),
])
def test_dismiss_and_notify(
    check_beforeunload_implicitly_accepted,
    check_user_prompt_closed_with_exception, dialog_type,
    retval
):
    if dialog_type == "beforeunload":
        check_beforeunload_implicitly_accepted()
    else:
        check_user_prompt_closed_with_exception(dialog_type, retval)


@pytest.mark.capabilities({"unhandledPromptBehavior": "ignore"})
@pytest.mark.parametrize("dialog_type", ["alert", "beforeunload", "confirm", "prompt"])
def test_ignore(
    check_beforeunload_implicitly_accepted,
    check_user_prompt_not_closed_but_exception,
    dialog_type
):
    if dialog_type == "beforeunload":
        check_beforeunload_implicitly_accepted()
    else:
        check_user_prompt_not_closed_but_exception(dialog_type)


@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("beforeunload", None),
    ("confirm", False),
    ("prompt", None),
])
def test_default(
    check_beforeunload_implicitly_accepted,
    check_user_prompt_closed_with_exception,
    dialog_type,
    retval
):
    if dialog_type == "beforeunload":
        check_beforeunload_implicitly_accepted()
    else:
        check_user_prompt_closed_with_exception(dialog_type, retval)
