from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def get_alert_text(session):
    return session.transport.send(
        "GET", "session/{session_id}/alert/text".format(**vars(session)))


# 18.3 Get Alert Text

def test_no_browsing_context(session, create_window):
    # 18.3 step 1
    session.window_handle = create_window()
    session.close()

    response = get_alert_text(session)
    assert_error(response, "no such window")


def test_no_user_prompt(session):
    # 18.3 step 2
    response = get_alert_text(session)
    assert_error(response, "no such alert")


def test_get_alert_text(session):
    # 18.3 step 3
    session.url = inline("<script>window.alert('Hello');</script>")
    response = get_alert_text(session)
    assert_success(response)
    assert isinstance(response.body, dict)
    assert "value" in response.body
    alert_text = response.body["value"]
    assert isinstance(alert_text, basestring)
    assert alert_text == "Hello"


def test_get_confirm_text(session):
    # 18.3 step 3
    session.url = inline("<script>window.confirm('Hello');</script>")
    response = get_alert_text(session)
    assert_success(response)
    assert isinstance(response.body, dict)
    assert "value" in response.body
    confirm_text = response.body["value"]
    assert isinstance(confirm_text, basestring)
    assert confirm_text == "Hello"


def test_get_prompt_text(session):
    # 18.3 step 3
    session.url = inline("<script>window.prompt('Enter Your Name: ', 'Federer');</script>")
    response = get_alert_text(session)
    assert_success(response)
    assert isinstance(response.body, dict)
    assert "value" in response.body
    prompt_text = response.body["value"]
    assert isinstance(prompt_text, basestring)
    assert prompt_text == "Enter Your Name: "
