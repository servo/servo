from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def dismiss_alert(session):
    return session.transport.send(
        "POST", "session/{session_id}/alert/dismiss".format(**vars(session)))


def test_null_response_value(session, url):
    session.url = inline("<script>window.alert('Hello');</script>")

    response = dismiss_alert(session)
    value = assert_success(response)
    assert value is None


def test_no_browsing_context(session, create_window):
    session.window_handle = create_window()
    session.close()

    response = dismiss_alert(session)
    assert_error(response, "no such window")


def test_no_user_prompt(session):
    response = dismiss_alert(session)
    assert_error(response, "no such alert")


def test_dismiss_alert(session):
    session.url = inline("<script>window.alert('Hello');</script>")
    response = dismiss_alert(session)
    assert_success(response)


def test_dismiss_confirm(session):
    session.url = inline("<script>window.result = window.confirm('Hello');</script>")
    response = dismiss_alert(session)
    assert_success(response)
    assert session.execute_script("return window.result;") is False


def test_dismiss_prompt(session):
    session.url = inline("<script>window.result = window.prompt('Enter Your Name: ', 'Federer');</script>")
    response = dismiss_alert(session)
    assert_success(response)
    assert session.execute_script("return window.result") is None
