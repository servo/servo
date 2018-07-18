from tests.support.asserts import assert_success


def delete_all_cookies(session):
    return session.transport.send(
        "DELETE", "/session/{session_id}/cookie".format(**vars(session)))


def test_null_response_value(session, url):
    response = delete_all_cookies(session)
    value = assert_success(response)
    assert value is None
