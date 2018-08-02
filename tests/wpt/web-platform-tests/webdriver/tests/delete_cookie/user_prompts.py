import pytest

from webdriver.error import NoSuchCookieException

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success


def delete_cookie(session, name):
    return session.transport.send("DELETE", "/session/%s/cookie/%s" % (session.session_id, name))


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_handle_prompt_accept(session, create_cookie, create_dialog, dialog_type, retval):
    create_cookie("foo", value="bar", path="/common/blank.html")

    create_dialog(dialog_type, text=dialog_type)

    response = delete_cookie(session, "foo")
    assert_success(response)

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    with pytest.raises(NoSuchCookieException):
        assert session.cookies("foo")


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
def test_handle_prompt_default(session, create_cookie, create_dialog, dialog_type, retval):
    cookie = create_cookie("foo", value="bar", path="/common/blank.html")

    create_dialog(dialog_type, text=dialog_type)

    response = delete_cookie(session, "foo")
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert session.cookies("foo") == cookie
