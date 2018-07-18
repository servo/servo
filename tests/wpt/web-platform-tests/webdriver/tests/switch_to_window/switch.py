from tests.support.asserts import assert_success


def switch_to_window(session, handle):
    return session.transport.send(
        "POST", "session/{session_id}/window".format(**vars(session)),
        {"handle": handle})


def test_null_response_value(session, create_window):
    new_handle = create_window()

    response = switch_to_window(session, new_handle)
    value = assert_success(response)
    assert value is None
