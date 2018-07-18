from tests.support.asserts import assert_success


def set_timeouts(session, timeouts):
    return session.transport.send(
        "POST", "session/{session_id}/timeouts".format(**vars(session)),
        timeouts)


def test_null_response_value(session):
    response = set_timeouts(session, {"implicit": 1000})
    value = assert_success(response)
    assert value is None

    response = set_timeouts(session, {"pageLoad": 1000})
    value = assert_success(response)
    assert value is None

    response = set_timeouts(session, {"script": 1000})
    value = assert_success(response)
    assert value is None
