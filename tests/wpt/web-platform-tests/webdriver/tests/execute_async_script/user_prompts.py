# META: timeout=long

import pytest

from tests.support.asserts import assert_dialog_handled, assert_error, assert_success


def execute_async_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session.transport.send(
        "POST", "/session/{session_id}/execute/async".format(**vars(session)),
        body)


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_handle_prompt_accept(session, create_dialog, dialog_type, retval):
    create_dialog(dialog_type, text=dialog_type)

    response = execute_async_script(session, "window.result = 1; arguments[0](1);")
    assert_success(response, 1)

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert session.execute_script("return window.result;") == 1


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", True),
    ("prompt", ""),
])
def test_handle_prompt_accept_and_notify(session, create_dialog, dialog_type, retval):
    create_dialog(dialog_type, text=dialog_type)

    response = execute_async_script(session, "window.result = 1; arguments[0](1);")
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert session.execute_script("return window.result;") is None


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_handle_prompt_dismiss(session, create_dialog, dialog_type, retval):
    create_dialog(dialog_type, text=dialog_type)

    response = execute_async_script(session, "window.result = 1; arguments[0](1);")
    assert_success(response, 1)

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert session.execute_script("return window.result;") == 1


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss and notify"})
@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_handle_prompt_dissmiss_and_notify(session, create_dialog, dialog_type, retval):
    create_dialog(dialog_type, text=dialog_type)

    response = execute_async_script(session, "window.result = 1; arguments[0](1);")
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert session.execute_script("return window.result;") is None


def test_handle_prompt_ignore():
    """TODO"""


@pytest.mark.parametrize("dialog_type, retval", [
    ("alert", None),
    ("confirm", False),
    ("prompt", None),
])
def test_handle_prompt_default(session, create_dialog, dialog_type, retval):
    create_dialog(dialog_type, text=dialog_type)

    response = execute_async_script(session, "window.result = 1; arguments[0](1);")
    assert_error(response, "unexpected alert open")

    assert_dialog_handled(session, expected_text=dialog_type, expected_retval=retval)

    assert session.execute_script("return window.result;") is None
