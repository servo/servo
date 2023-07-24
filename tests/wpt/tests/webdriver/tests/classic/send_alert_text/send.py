import pytest

from webdriver.error import NoSuchAlertException
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.sync import Poll


@pytest.fixture
def page(session, inline):
    session.url = inline("""
        <script>window.result = window.prompt('Enter Your Name: ', 'Name');</script>
    """)


def send_alert_text(session, text=None):
    return session.transport.send(
        "POST", "session/{session_id}/alert/text".format(**vars(session)),
        {"text": text})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/alert/text".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session, page):
    response = send_alert_text(session, "Federer")
    value = assert_success(response)
    assert value is None


@pytest.mark.parametrize("text", [None, {}, [], 42, True])
def test_invalid_input(session, page, text):
    response = send_alert_text(session, text)
    assert_error(response, "invalid argument")


def test_no_top_browsing_context(session, closed_window):
    response = send_alert_text(session, "Federer")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = send_alert_text(session, "Federer")
    assert_error(response, "no such alert")


def test_no_user_prompt(session):
    response = send_alert_text(session, "Federer")
    assert_error(response, "no such alert")


@pytest.mark.parametrize("dialog_type", ["alert", "confirm"])
def test_alert_element_not_interactable(session, inline, dialog_type):
    session.url = inline("<script>window.{}('Hello');</script>".format(dialog_type))

    response = send_alert_text(session, "Federer")
    assert_error(response, "element not interactable")


@pytest.mark.parametrize("dialog_type", ["alert", "confirm"])
def test_chained_alert_element_not_interactable(session, inline, dialog_type):
    session.url = inline("<script>window.{}('Hello');</script>".format(dialog_type))
    session.alert.accept()

    session.url = inline("<script>window.{}('Hello');</script>".format(dialog_type))
    response = send_alert_text(session, "Federer")
    assert_error(response, "element not interactable")


@pytest.mark.parametrize("text", ["", "Federer", " Fed erer ", "Fed\terer"])
def test_send_alert_text(session, page, text):
    send_response = send_alert_text(session, text)
    assert_success(send_response)

    session.alert.accept()

    assert session.execute_script("return window.result") == text


def test_unexpected_alert(session):
    session.execute_script("setTimeout(function() { prompt('Hello'); }, 100);")
    wait = Poll(
        session,
        timeout=5,
        ignored_exceptions=NoSuchAlertException,
        message="No user prompt with text 'Hello' detected")
    wait.until(lambda s: s.alert.text == "Hello")

    response = send_alert_text(session, "Federer")
    assert_success(response)
