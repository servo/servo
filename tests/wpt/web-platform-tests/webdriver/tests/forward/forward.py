from tests.support.inline import inline
from tests.support.asserts import assert_success


def forward(session):
    return session.transport.send(
        "POST", "session/{session_id}/forward".format(**vars(session)))


def test_null_response_value(session):
    session.url = inline("<div>")
    session.url = inline("<p>")
    session.back()

    response = forward(session)
    value = assert_success(response)
    assert value is None
