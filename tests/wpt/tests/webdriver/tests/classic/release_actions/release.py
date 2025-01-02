from tests.support.asserts import assert_error, assert_success


def release_actions(session):
    return session.transport.send(
        "DELETE",
        "/session/{session_id}/actions".format(**vars(session)),
    )


def test_null_response_value(session):
    response = release_actions(session)
    assert_success(response, None)


def test_no_top_browsing_context(session, closed_window):
    response = release_actions(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = release_actions(session)
    assert_error(response, "no such window")
