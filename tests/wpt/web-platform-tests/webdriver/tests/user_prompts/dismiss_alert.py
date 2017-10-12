from tests.support.asserts import assert_error, assert_success


def dismiss_alert(session):
    return session.transport.send("POST", "session/{session_id}/alert/dismiss"
                                  .format(session_id=session.session_id))


# 18.1 Dismiss Alert

def test_no_browsing_context(session, create_window):
    # 18.1 step 1
    session.window_handle = create_window()
    session.close()

    response = dismiss_alert(session)
    assert_error(response, "no such window")


def test_no_user_prompt(session):
    # 18.1 step 2
    response = dismiss_alert(session)
    assert_error(response, "no such alert")


def test_dismiss_alert(session):
    # 18.1 step 3
    session.execute_script("window.alert(\"Hello\");")
    response = dismiss_alert(session)
    assert_success(response)


def test_dismiss_confirm(session):
    # 18.1 step 3
    session.execute_script("window.result = window.confirm(\"Hello\");")
    response = dismiss_alert(session)
    assert_success(response)
    assert session.execute_script("return window.result;") is False


def test_dismiss_prompt(session):
    # 18.1 step 3
    session.execute_script("window.result = window.prompt(\"Enter Your Name: \", \"Federer\");")
    response = dismiss_alert(session)
    assert_success(response)
    assert session.execute_script("return window.result") is None
