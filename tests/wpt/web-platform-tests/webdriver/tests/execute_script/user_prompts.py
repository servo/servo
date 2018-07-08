# META: timeout=long

import pytest

from webdriver import error

from tests.support.asserts import assert_success


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session.transport.send(
        "POST", "/session/{session_id}/execute/sync".format(
            session_id=session.session_id),
        body)


def test_handle_prompt_accept(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})

    response = execute_script(session, "window.alert('Hello');")
    assert_success(response, None)

    session.title
    with pytest.raises(error.NoSuchAlertException):
        session.alert.accept()


def test_handle_prompt_dismiss(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "dismiss"})}})

    response = execute_script(session, "window.alert('Hello');")
    assert_success(response, None)

    session.title
    with pytest.raises(error.NoSuchAlertException):
        session.alert.dismiss()


def test_handle_prompt_dismiss_and_notify(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "dismiss and notify"})}})

    response = execute_script(session, "window.alert('Hello');")
    assert_success(response, None)

    with pytest.raises(error.UnexpectedAlertOpenException):
        session.title
    with pytest.raises(error.NoSuchAlertException):
        session.alert.dismiss()


def test_handle_prompt_accept_and_notify(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept and notify"})}})

    response = execute_script(session, "window.alert('Hello');")
    assert_success(response, None)

    with pytest.raises(error.UnexpectedAlertOpenException):
        session.title
    with pytest.raises(error.NoSuchAlertException):
        session.alert.accept()


def test_handle_prompt_ignore(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "ignore"})}})

    response = execute_script(session, "window.alert('Hello');")
    assert_success(response, None)

    with pytest.raises(error.UnexpectedAlertOpenException):
        session.title
    session.alert.dismiss()


def test_handle_prompt_default(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})

    response = execute_script(session, "window.alert('Hello');")
    assert_success(response, None)

    with pytest.raises(error.UnexpectedAlertOpenException):
        session.title
    with pytest.raises(error.NoSuchAlertException):
        session.alert.dismiss()


def test_handle_prompt_twice(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})

    response = execute_script(session, "window.alert('Hello');window.alert('Bye');")
    assert_success(response, None)

    session.alert.dismiss()
    # The first alert has been accepted by the user prompt handler, the second one remains.
    # FIXME: this is how browsers currently work, but the spec should clarify if this is the
    #        expected behavior, see https://github.com/w3c/webdriver/issues/1153.
    assert session.alert.text == "Bye"
    session.alert.dismiss()
