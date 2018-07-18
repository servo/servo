from tests.support.inline import inline
from tests.support.asserts import assert_success


def navigate_to(session, url):
    return session.transport.send(
        "POST", "session/{session_id}/url".format(**vars(session)),
        {"url": url})


def test_null_response_value(session):
    response = navigate_to(session, inline("<div/>"))
    value = assert_success(response)
    assert value is None
