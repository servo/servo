import pytest

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def send_alert_text(session, text=None):
    return session.transport.send(
        "POST", "session/{session_id}/alert/text".format(**vars(session)),
        {"text": text})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/alert/text".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session, url):
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")

    response = send_alert_text(session, "Federer")
    value = assert_success(response)
    assert value is None


def test_no_browsing_context(session, closed_window):
    response = send_alert_text(session, "foo")
    assert_error(response, "no such window")


@pytest.mark.parametrize("text", [None, {}, [], 42, True])
def test_invalid_input(session, text):
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")
    response = send_alert_text(session, text)
    assert_error(response, "invalid argument")


def test_no_user_prompt(session):
    response = send_alert_text(session, "Federer")
    assert_error(response, "no such alert")


def test_alert_element_not_interactable(session):
    session.url = inline("<script>window.alert('Hello');</script>")

    response = send_alert_text(session, "Federer")
    assert_error(response, "element not interactable")


def test_confirm_element_not_interactable(session):
    session.url = inline("<script>window.confirm('Hello');</script>")

    response = send_alert_text(session, "Federer")
    assert_error(response, "element not interactable")


def test_send_alert_text(session):
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")

    send_response = send_alert_text(session, "Federer")
    assert_success(send_response)

    accept_response = session.transport.send("POST", "session/{session_id}/alert/accept"
                                             .format(session_id=session.session_id))
    assert_success(accept_response)
    assert session.execute_script("return window.result") == "Federer"


def test_send_alert_text_with_whitespace(session):
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>")

    send_response = send_alert_text(session, " Fed erer ")
    assert_success(send_response)

    accept_response = session.transport.send("POST", "session/{session_id}/alert/accept"
                                             .format(session_id=session.session_id))
    assert_success(accept_response)
    assert session.execute_script("return window.result") == " Fed erer "
