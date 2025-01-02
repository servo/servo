import pytest

from webdriver.error import NoSuchAlertException

from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import wait_for_new_handle
from tests.support.sync import Poll


def accept_alert(session):
    return session.transport.send(
        "POST", "session/{session_id}/alert/accept".format(**vars(session)))


def test_null_response_value(session, inline):
    session.url = inline("<script>window.alert('Hello');</script>")

    response = accept_alert(session)
    value = assert_success(response)
    assert value is None


def test_no_top_level_browsing_context(session, closed_window):
    response = accept_alert(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = accept_alert(session)
    assert_error(response, "no such alert")


def test_no_user_prompt(session):
    response = accept_alert(session)
    assert_error(response, "no such alert")


def test_accept_alert(session, inline):
    session.url = inline("<script>window.alert('Hello');</script>")

    response = accept_alert(session)
    assert_success(response)

    with pytest.raises(NoSuchAlertException):
        session.alert.text


def test_accept_confirm(session, inline):
    session.url = inline("<script>window.result = window.confirm('Hello');</script>")

    response = accept_alert(session)
    assert_success(response)

    with pytest.raises(NoSuchAlertException):
        session.alert.text

    assert session.execute_script("return window.result") is True


def test_accept_prompt(session, inline):
    session.url = inline("""
        <script>
          window.result = window.prompt('Enter Your Name: ', 'Federer');
        </script>
        """)

    response = accept_alert(session)
    assert_success(response)

    with pytest.raises(NoSuchAlertException):
        session.alert.text

    assert session.execute_script("return window.result") == "Federer"


def test_unexpected_alert(session):
    session.execute_script("window.setTimeout(function() { window.alert('Hello'); }, 100);")
    wait = Poll(
        session,
        timeout=5,
        ignored_exceptions=NoSuchAlertException,
        message="No user prompt with text 'Hello' detected")
    wait.until(lambda s: s.alert.text == "Hello")

    response = accept_alert(session)
    assert_success(response)

    with pytest.raises(NoSuchAlertException):
        session.alert.text


def test_accept_in_popup_window(session, inline):
    orig_handles = session.handles

    session.url = inline("""
        <button onclick="window.open('about:blank', '_blank', 'width=500; height=200;resizable=yes');">open</button>
        """)
    button = session.find.css("button", all=False)
    button.click()

    session.window_handle = wait_for_new_handle(session, orig_handles)
    session.url = inline("""
        <script>window.alert("Hello")</script>
        """)

    response = accept_alert(session)
    assert_success(response)

    with pytest.raises(NoSuchAlertException):
        session.alert.text
