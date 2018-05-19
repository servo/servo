from tests.support.asserts import assert_error, assert_success


def delete_cookie(session, name):
    return session.transport.send(
        "DELETE", "/session/{session_id}/cookie/{name}".format(
            session_id=session.session_id,
            name=name))


def test_no_browsing_context(session, create_window):
    session.window_handle = create_window()
    session.close()

    response = delete_cookie(session, "foo")
    assert_error(response, "no such window")


def test_unknown_cookie(session):
    response = delete_cookie(session, "stilton")
    assert_success(response)
