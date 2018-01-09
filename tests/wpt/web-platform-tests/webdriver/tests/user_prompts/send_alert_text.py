import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline

def send_alert_text(session, body=None):
    return session.transport.send("POST", "session/{session_id}/alert/text"
                                  .format(session_id=session.session_id), body)


# 18.4 Send Alert Text

@pytest.mark.parametrize("text", [None, {}, [], 42, True])
def test_invalid_input(session, text):
    # 18.4 step 2
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")
    response = send_alert_text(session, {"text": text})
    assert_error(response, "invalid argument")


def test_no_browsing_context(session, create_window):
    # 18.4 step 3
    session.window_handle = create_window()
    session.close()
    body = {"text": "Federer"}
    response = send_alert_text(session, body)
    assert_error(response, "no such window")


def test_no_user_prompt(session):
    # 18.4 step 4
    body = {"text": "Federer"}
    response = send_alert_text(session, body)
    assert_error(response, "no such alert")


def test_alert_element_not_interactable(session):
    # 18.4 step 5
    session.url = inline("<script>window.alert('Hello');</script>")
    body = {"text": "Federer"}
    response = send_alert_text(session, body)
    assert_error(response, "element not interactable")


def test_confirm_element_not_interactable(session):
    # 18.4 step 5
    session.url = inline("<script>window.confirm('Hello');</script>")
    body = {"text": "Federer"}
    response = send_alert_text(session, body)
    assert_error(response, "element not interactable")


def test_send_alert_text(session):
    # 18.4 step 6
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")
    body = {"text": "Federer"}
    send_response = send_alert_text(session, body)
    assert_success(send_response)
    accept_response = session.transport.send("POST", "session/{session_id}/alert/accept"
                                             .format(session_id=session.session_id))
    assert_success(accept_response)
    assert session.execute_script("return window.result") == "Federer"


def test_send_alert_text_with_whitespace(session):
    # 18.4 step 6
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")
    body = {"text": " Fed erer "}
    send_response = send_alert_text(session, body)
    assert_success(send_response)
    accept_response = session.transport.send("POST", "session/{session_id}/alert/accept"
                                             .format(session_id=session.session_id))
    assert_success(accept_response)
    assert session.execute_script("return window.result") == " Fed erer "
