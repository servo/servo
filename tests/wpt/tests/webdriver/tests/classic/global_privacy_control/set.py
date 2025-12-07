import pytest
from tests.support.asserts import assert_error, assert_success


def set_gpc(session, value):
    return session.transport.send(
        "POST", "/session/{session_id}/privacy".format(
            session_id=session.session_id), {"gpc": value})

def get_gpc(session):
    return session.transport.send(
        "GET", "/session/{session_id}/privacy".format(
            session_id=session.session_id))

@pytest.mark.parametrize("value", [True, False])
def test_set_gpc_success(session, value):
    response = set_gpc(session, value)
    newValue = assert_success(response)
    assert "gpc" in newValue
    assert type(newValue["gpc"]) is bool
    assert newValue["gpc"] == value
    getResponse = get_gpc(session)
    value = assert_success(getResponse)
    assert "gpc" in value
    assert type(value["gpc"]) is bool
    assert newValue["gpc"] == value["gpc"]


@pytest.mark.parametrize("value", [None, 1, "hello", [], {}])
def test_set_gpc_failure(session, value):
    response = set_gpc(session, value)
    error = assert_error(response, "invalid argument")
