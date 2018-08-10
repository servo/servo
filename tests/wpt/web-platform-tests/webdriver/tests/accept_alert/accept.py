from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def accept_alert(session):
    return session.transport.send(
        "POST", "session/{session_id}/alert/accept".format(**vars(session)))


def test_null_response_value(session, url):
    session.url = inline("<script>window.alert('Hello');</script>")

    response = accept_alert(session)
    value = assert_success(response)
    assert value is None


def test_no_browsing_context(session, closed_window):
    response = accept_alert(session)
    assert_error(response, "no such window")


def test_no_user_prompt(session):
    response = accept_alert(session)
    assert_error(response, "no such alert")


def test_accept_alert(session):
    session.url = inline("<script>window.alert('Hello');</script>")
    response = accept_alert(session)
    assert_success(response)


def test_accept_confirm(session):
    session.url = inline("<script>window.result = window.confirm('Hello');</script>")
    response = accept_alert(session)
    assert_success(response)
    assert session.execute_script("return window.result") is True


def test_accept_prompt(session):
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Federer');</script>")
    response = accept_alert(session)
    assert_success(response)
    assert session.execute_script("return window.result") == "Federer"
