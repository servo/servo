from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def navigate_to(session, url):
    return session.transport.send(
        "POST", "session/{session_id}/url".format(**vars(session)),
        {"url": url})


def test_null_response_value(session):
    response = navigate_to(session, inline("<div/>"))
    value = assert_success(response)
    assert value is None


def test_no_browsing_context(session, closed_window):
    response = navigate_to(session, "foo")
    assert_error(response, "no such window")
