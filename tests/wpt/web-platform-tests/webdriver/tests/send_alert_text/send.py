import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def send_alert_text(session, text=None):
    return session.transport.send(
        "POST", "session/{session_id}/alert/text".format(**vars(session)),
        {"text": text})


def test_null_response_value(session, url):
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")

    response = send_alert_text(session, "Federer")
    value = assert_success(response)
    assert value is None


@pytest.mark.parametrize("text", [None, {}, [], 42, True])
def test_invalid_input(session, text):
    # 18.4 step 2
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")
    response = send_alert_text(session, text)
    assert_error(response, "invalid argument")


def test_no_browsing_context(session, create_window):
    # 18.4 step 3
    session.window_handle = create_window()
    session.close()

    response = send_alert_text(session, "Federer")
    assert_error(response, "no such window")


def test_no_user_prompt(session):
    # 18.4 step 4
    response = send_alert_text(session, "Federer")
    assert_error(response, "no such alert")


def test_alert_element_not_interactable(session):
    # 18.4 step 5
    session.url = inline("<script>window.alert('Hello');</script>")

    response = send_alert_text(session, "Federer")
    assert_error(response, "element not interactable")


def test_confirm_element_not_interactable(session):
    # 18.4 step 5
    session.url = inline("<script>window.confirm('Hello');</script>")

    response = send_alert_text(session, "Federer")
    assert_error(response, "element not interactable")


def test_send_alert_text(session):
    # 18.4 step 6
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")

    send_response = send_alert_text(session, "Federer")
    assert_success(send_response)

    accept_response = session.transport.send("POST", "session/{session_id}/alert/accept"
                                             .format(session_id=session.session_id))
    assert_success(accept_response)
    assert session.execute_script("return window.result") == "Federer"


def test_send_alert_text_with_whitespace(session):
    # 18.4 step 6
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")

    send_response = send_alert_text(session, " Fed erer ")
    assert_success(send_response)

    accept_response = session.transport.send("POST", "session/{session_id}/alert/accept"
                                             .format(session_id=session.session_id))
    assert_success(accept_response)
    assert session.execute_script("return window.result") == " Fed erer "
