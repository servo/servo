from webdriver.error import NoSuchAlertException

from tests.support.asserts import assert_error, assert_success
from tests.support.sync import Poll


def get_alert_text(session):
    return session.transport.send(
        "GET", "session/{session_id}/alert/text".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = get_alert_text(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = get_alert_text(session)
    assert_error(response, "no such alert")


def test_no_user_prompt(session):
    response = get_alert_text(session)
    assert_error(response, "no such alert")


def test_get_alert_text(session, inline):
    session.url = inline("<script>window.alert('Hello');</script>")
    response = get_alert_text(session)
    assert_success(response)
    assert isinstance(response.body, dict)
    assert "value" in response.body
    alert_text = response.body["value"]
    assert isinstance(alert_text, str)
    assert alert_text == "Hello"


def test_get_confirm_text(session, inline):
    session.url = inline("<script>window.confirm('Hello');</script>")
    response = get_alert_text(session)
    assert_success(response)
    assert isinstance(response.body, dict)
    assert "value" in response.body
    confirm_text = response.body["value"]
    assert isinstance(confirm_text, str)
    assert confirm_text == "Hello"


def test_get_prompt_text(session, inline):
    session.url = inline("<script>window.prompt('Enter Your Name: ', 'Federer');</script>")
    response = get_alert_text(session)
    assert_success(response)
    assert isinstance(response.body, dict)
    assert "value" in response.body
    prompt_text = response.body["value"]
    assert isinstance(prompt_text, str)
    assert prompt_text == "Enter Your Name: "


# TODO: Add test for beforeunload?


def test_unexpected_alert(session):
    session.execute_script("setTimeout(function() { alert('Hello'); }, 100);")
    wait = Poll(
        session,
        timeout=5,
        ignored_exceptions=NoSuchAlertException,
        message="No user prompt with text 'Hello' detected")
    wait.until(lambda s: s.alert.text == "Hello")

    response = get_alert_text(session)
    assert_success(response)
