import pytest

from webdriver import error


# 15.2 Executing Script

def test_handle_prompt_accept(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})
    session.execute_script("window.alert('Hello');")
    with pytest.raises(error.NoSuchAlertException):
        session.alert.accept()


def test_handle_prompt_dismiss(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "dismiss"})}})
    session.execute_script("window.alert('Hello');")
    with pytest.raises(error.NoSuchAlertException):
        session.alert.dismiss()


def test_handle_prompt_dismiss_and_notify(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "dismiss and notify"})}})
    with pytest.raises(error.UnexpectedAlertOpenException):
        session.execute_script("window.alert('Hello');")
    with pytest.raises(error.NoSuchAlertException):
        session.alert.dismiss()


def test_handle_prompt_accept_and_notify(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept and notify"})}})
    with pytest.raises(error.UnexpectedAlertOpenException):
        session.execute_script("window.alert('Hello');")
    with pytest.raises(error.NoSuchAlertException):
        session.alert.accept()


def test_handle_prompt_ignore(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "ignore"})}})
    with pytest.raises(error.UnexpectedAlertOpenException):
        session.execute_script("window.alert('Hello');")
    session.alert.dismiss()


def test_handle_prompt_default(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})
    with pytest.raises(error.UnexpectedAlertOpenException):
        session.execute_script("window.alert('Hello');")
    with pytest.raises(error.NoSuchAlertException):
        session.alert.dismiss()


def test_handle_prompt_twice(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})
    session.execute_script("window.alert('Hello');window.alert('Bye');")
    # The first alert has been accepted by the user prompt handler, the second one remains.
    # FIXME: this is how browsers currently work, but the spec should clarify if this is the
    #        expected behavior, see https://github.com/w3c/webdriver/issues/1153.
    assert session.alert.text == "Bye"
    session.alert.dismiss()
