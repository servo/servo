from tests.support.asserts import assert_error, assert_success


def delete_all_cookies(session):
    return session.transport.send(
        "DELETE", "/session/{session_id}/cookie".format(**vars(session)))


def test_null_response_value(session, url):
    response = delete_all_cookies(session)
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session, closed_window):
    response = delete_all_cookies(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = delete_all_cookies(session)
    assert_error(response, "no such window")
