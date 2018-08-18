from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success


def switch_to_window(session, handle):
    return session.transport.send(
        "POST", "session/{session_id}/window".format(**vars(session)),
        {"handle": handle})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/window".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session, create_window):
    new_handle = create_window()

    response = switch_to_window(session, new_handle)
    value = assert_success(response)
    assert value is None
