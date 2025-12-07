from tests.support.asserts import assert_success


def get_gpc(session):
    return session.transport.send(
        "GET", "/session/{session_id}/privacy".format(
            session_id=session.session_id))


def test_get_gpc(session):
    response = get_gpc(session)
    value = assert_success(response)
    assert "gpc" in value
    assert type(value["gpc"]) is bool
