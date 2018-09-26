from tests.support.asserts import assert_error, assert_success


def perform_actions(session, actions):
    return session.transport.send(
        "POST",
        "/session/{session_id}/actions".format(**vars(session)),
        {"actions": actions},
    )


def test_null_response_value(session, none_chain):
    response = perform_actions(session, [none_chain.pause(0).dict])
    assert_success(response, None)


def test_no_browsing_context(session, closed_window, none_chain):
    response = perform_actions(session, [none_chain.pause(0).dict])
    assert_error(response, "no such window")
