from tests.classic.perform_actions import perform_actions
from tests.support.classic.asserts import assert_error, assert_success


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


def test_release_actions_resets_pointer_subtype(session):
    actions = [
        {
            "type": "pointer",
            "id": "foo",
            "parameters": {"pointerType": "mouse"},
            "actions": [{"type": "pointerMove", "x": 0, "y": 0}],
        }
    ]
    assert_success(perform_actions(session, actions))
    assert_success(release_actions(session))

    actions[0]["parameters"]["pointerType"] = "touch"
    assert_success(perform_actions(session, actions))
