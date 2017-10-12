from tests.support.asserts import assert_error, assert_success


def accept_alert(session):
    return session.transport.send("POST", "session/{session_id}/alert/accept"
                                  .format(session_id=session.session_id))


# 18.2 Accept Alert

def test_no_browsing_context(session, create_window):
    # 18.2 step 1
    session.window_handle = create_window()
    session.close()

    response = accept_alert(session)
    assert_error(response, "no such window")


def test_no_user_prompt(session):
    # 18.2 step 2
    response = accept_alert(session)
    assert_error(response, "no such alert")


def test_accept_alert(session):
    # 18.2 step 3
    session.execute_script("window.alert(\"Hello\");")
    response = accept_alert(session)
    assert_success(response)


def test_accept_confirm(session):
    # 18.2 step 3
    session.execute_script("window.result = window.confirm(\"Hello\");")
    response = accept_alert(session)
    assert_success(response)
    assert session.execute_script("return window.result") is True


def test_accept_prompt(session):
    # 18.2 step 3
    session.execute_script("window.result = window.prompt(\"Enter Your Name: \", \"Federer\");")
    response = accept_alert(session)
    assert_success(response)
    assert session.execute_script("return window.result") == "Federer"
