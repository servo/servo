from tests.support.inline import inline
from tests.support.asserts import assert_success


def refresh(session):
    return session.transport.send(
        "POST", "session/{session_id}/refresh".format(**vars(session)))


def test_null_response_value(session):
    session.url = inline("<div>")

    response = refresh(session)
    value = assert_success(response)
    assert value is None
