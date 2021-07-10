from tests.support.asserts import assert_error, assert_success


def get_window_handle(session):
    return session.transport.send(
        "GET", "session/{session_id}/window".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = get_window_handle(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = get_window_handle(session)
    assert_success(response, session.window_handle)


def test_basic(session):
    response = get_window_handle(session)
    assert_success(response, session.window_handle)
