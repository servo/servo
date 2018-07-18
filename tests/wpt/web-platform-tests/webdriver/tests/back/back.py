from tests.support.inline import inline
from tests.support.asserts import assert_success


def back(session):
    return session.transport.send(
        "POST", "session/{session_id}/back".format(**vars(session)))


def test_null_response_value(session):
    session.url = inline("<div>")
    session.url = inline("<p>")

    response = back(session)
    value = assert_success(response)
    assert value is None
