from tests.support.asserts import assert_error, assert_success


def close(session):
    return session.transport.send("DELETE", "session/%s/window" % session.session_id)


def test_no_browsing_context(session, create_window):
    new_handle = create_window()

    session.window_handle = new_handle
    session.close()
    assert new_handle not in session.handles

    response = close(session)
    assert_error(response, "no such window")


def test_close_browsing_context(session, create_window):
    handles = session.handles

    new_handle = create_window()
    session.window_handle = new_handle

    response = close(session)
    value = assert_success(response, handles)
    assert session.handles == handles
    assert new_handle not in value


def test_close_last_browsing_context(session):
    assert len(session.handles) == 1
    response = close(session)

    assert_success(response, [])

    # With no more open top-level browsing contexts, the session is closed.
    session.session_id = None
